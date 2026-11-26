//! Admin mode (docs/ARCHITECTURE.md, 8. Elevated copy): a logon task with the highest run level that starts a copy of
//! triokey.exe in `%ProgramFiles%\TrioKey`, which only admins can write.
//!
//! The elevated child (`triokey.exe --install-admin` / `--uninstall-admin`) does only the parts
//! that need admin rights: the copy and the task. HKCU (Run value) and config.txt are written
//! by the unelevated parent after the child succeeds, so they land in the right user's hive.
//! The child takes no path from the environment, the config or its arguments.
//! A TrioKey that is already elevated runs the same two functions in-process (`run_elevated`).

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::null_mut;

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_CANCELLED, HWND, WAIT_OBJECT_0};
use windows_sys::Win32::Security::Authentication::Identity::{GetUserNameExW, NameSamCompatible};
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevationType, TokenElevationTypeFull, TOKEN_QUERY};
use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, WaitForSingleObject, CREATE_NO_WINDOW};
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

pub const TASK: &str = "TrioKey";
const INSTALL: &str = "--install-admin";
const UNINSTALL: &str = "--uninstall-admin";
/// Child exit codes.
pub const OK: u32 = 0;
pub const FAILED: u32 = 1;
/// Task removed, but the running admin copy keeps its folder until the next reboot.
pub const OK_AFTER_REBOOT: u32 = 2;
/// Copying the exe into `%ProgramFiles%\TrioKey` failed.
pub const COPY_FAILED: u32 = 3;
/// `schtasks /Create` failed (or its XML could not be written).
pub const TASK_FAILED: u32 = 4;
/// Task removed, but `%ProgramFiles%\TrioKey` could neither be deleted nor queued for deletion.
pub const OK_FOLDER_LEFT: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    InstallAdmin,
    UninstallAdmin,
}

/// `args` without the program name. Anything unknown starts TrioKey normally.
pub fn mode<I: IntoIterator<Item = String>>(args: I) -> Mode {
    match args.into_iter().next().as_deref() {
        Some(INSTALL) => Mode::InstallAdmin,
        Some(UNINSTALL) => Mode::UninstallAdmin,
        _ => Mode::Normal,
    }
}

fn xml_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            c => o.push(c),
        }
    }
    o
}

/// Task Scheduler XML: at logon of `user`, highest run level, no time limit, normal priority
/// (the default 7 is below normal, bad for a keyboard hook), runs on battery.
pub fn task_xml(user: &str, exe: &str) -> String {
    let (user, exe) = (xml_escape(user), xml_escape(exe));
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>TrioKey, admin mode: types Vietnamese in admin windows too.</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let mut p = null_mut();
        let hr = SHGetKnownFolderPath(id, 0, null_mut(), &mut p);
        let dir = (hr == 0 && !p.is_null()).then(|| {
            let len = (0..).take_while(|&i| *p.add(i) != 0).count();
            PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(p, len)))
        });
        CoTaskMemFree(p.cast());
        dir
    }
}

/// `%ProgramFiles%\TrioKey`, from the known-folder API (HKLM), not the user-settable env var.
pub fn admin_dir() -> Option<PathBuf> {
    known_folder(&FOLDERID_ProgramFiles).map(|d| d.join("TrioKey"))
}

/// The user's roaming AppData. Not from `%APPDATA%`, but HKCU can still redirect it: an
/// elevated TrioKey must treat everything under it as controlled by unelevated programs.
pub fn roaming_appdata() -> Option<PathBuf> {
    known_folder(&FOLDERID_RoamingAppData)
}

/// The user's profile folder (`C:\Users\<name>`), masked in the debug information.
pub fn profile_dir() -> Option<PathBuf> {
    known_folder(&FOLDERID_Profile)
}

pub fn admin_exe() -> Option<PathBuf> {
    admin_dir().map(|d| d.join("triokey.exe"))
}

/// This process is the `%ProgramFiles%\TrioKey` copy.
pub fn is_admin_copy() -> bool {
    let low = |p: &Path| p.to_string_lossy().to_lowercase();
    matches!((std::env::current_exe(), admin_exe()), (Ok(a), Some(b)) if low(&a) == low(&b))
}

/// This process has admin rights: it can do the admin part (`install` / `uninstall`) itself.
pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// This process holds the elevated half of a split (UAC) token: the admin copy, or "Run as
/// administrator". Only then do unelevated programs of the same user exist, which is what the
/// restrictions in docs/ARCHITECTURE.md, 8.3 What an elevated TrioKey must never do,
/// guard against. With UAC off, as the built-in Administrator or
/// in Windows Sandbox (probably; untested) the token is not split, every program of the user is equally privileged,
/// and TrioKey behaves like a normal copy. An admin whose token type cannot be read counts as
/// elevated.
pub fn is_elevated() -> bool {
    unsafe {
        let (mut token, mut kind, mut len) = (null_mut(), 0i32, 0u32);
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return is_admin();
        }
        let ok = GetTokenInformation(token, TokenElevationType, (&mut kind as *mut i32).cast(), 4, &mut len) != 0;
        CloseHandle(token);
        if ok {
            kind == TokenElevationTypeFull
        } else {
            is_admin()
        }
    }
}

/// `DOMAIN\user` of this process.
fn user_name() -> Option<String> {
    let mut buf = [0u16; 512];
    let mut n = buf.len() as u32;
    let ok = unsafe { GetUserNameExW(NameSamCompatible, buf.as_mut_ptr(), &mut n) };
    (ok != 0).then(|| String::from_utf16_lossy(&buf[..n as usize]))
}

/// `System32\schtasks.exe` by full path: a bare name could resolve next to a user-writable exe.
fn schtasks(args: &[&str]) -> bool {
    let mut buf = [0u16; 260];
    let n = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    if n == 0 || n >= buf.len() {
        return false;
    }
    let exe = PathBuf::from(String::from_utf16_lossy(&buf[..n])).join("schtasks.exe");
    Command::new(exe).args(args).creation_flags(CREATE_NO_WINDOW).status().is_ok_and(|s| s.success())
}

/// Elevated child: copy this exe to `%ProgramFiles%\TrioKey\triokey.exe`, register the task.
pub fn install() -> u32 {
    let (Some(dir), Some(user), Ok(src)) = (admin_dir(), user_name(), std::env::current_exe()) else { return FAILED };
    let dst = dir.join("triokey.exe");
    if std::fs::create_dir_all(&dir).is_err() {
        return FAILED;
    }
    if !src.to_string_lossy().eq_ignore_ascii_case(&dst.to_string_lossy()) && std::fs::copy(&src, &dst).is_err() {
        return COPY_FAILED;
    }
    // The running admin copy that has just turned admin mode off: `uninstall` renamed its
    // image for deletion at reboot, `src` still names the old path, and no copy was made.
    // Without this the task would point at a file that is not there.
    if !dst.is_file() {
        return COPY_FAILED;
    }
    // inside the admin-only folder: a file in %TEMP% could be swapped before schtasks reads it
    let xml_path = dir.join("task.xml");
    let utf16: Vec<u8> = [0xFEFFu16].into_iter().chain(task_xml(&user, &dst.to_string_lossy()).encode_utf16()).flat_map(u16::to_le_bytes).collect();
    if std::fs::write(&xml_path, utf16).is_err() {
        return TASK_FAILED;
    }
    let ok = schtasks(&["/Create", "/TN", TASK, "/XML", &xml_path.to_string_lossy(), "/F"]);
    let _ = std::fs::remove_file(&xml_path);
    if ok {
        OK
    } else {
        TASK_FAILED
    }
}

/// Elevated child: delete the task and `%ProgramFiles%\TrioKey`.
pub fn uninstall() -> u32 {
    let Some(dir) = admin_dir() else { return FAILED };
    if schtasks(&["/Query", "/TN", TASK]) && !schtasks(&["/Delete", "/TN", TASK, "/F"]) {
        return FAILED;
    }
    if !dir.exists() || std::fs::remove_dir_all(&dir).is_ok() {
        return OK;
    }
    // The running admin copy locks its exe: remove what is left at the next reboot. A running
    // image can be renamed, so the queued delete hits the renamed file and cannot take a new
    // triokey.exe if admin mode is turned on again before the reboot (the folder delete then fails
    // harmlessly because the folder is not empty).
    let later = |p: &Path| unsafe {
        let w: Vec<u16> = p.as_os_str().to_string_lossy().encode_utf16().chain([0]).collect();
        MoveFileExW(w.as_ptr(), std::ptr::null(), MOVEFILE_DELAY_UNTIL_REBOOT) != 0
    };
    let files = std::fs::read_dir(&dir).map(|r| r.flatten().map(|e| e.path()).collect::<Vec<_>>()).unwrap_or_default();
    let pid = std::process::id();
    let queued = files.iter().all(|f| {
        let old = f.with_file_name(format!("{}.{pid}.old", f.file_name().unwrap_or_default().to_string_lossy()));
        later(if std::fs::rename(f, &old).is_ok() { &old } else { f })
    });
    // files first: the folder can only go once it is empty. The task is gone either way, so
    // this is never FAILED: the caller must stop showing admin mode as on.
    if queued && later(&dir) {
        OK_AFTER_REBOOT
    } else {
        OK_FOLDER_LEFT
    }
}

pub enum Elevated {
    Exit(u32),
    /// The user said no at the UAC prompt.
    Cancelled,
    /// The child is still running after two minutes; what it did is unknown.
    Timeout,
    Failed,
}

/// Do the admin-only part (`on`: install, else uninstall) and wait for it: in this process when
/// it is already elevated, else in a child started with the "runas" verb (one UAC prompt).
/// Blocks the calling (tray) thread; the hook runs on its own thread meanwhile.
pub fn run_elevated(hwnd: HWND, on: bool) -> Elevated {
    // ShellExecute resolves verbs through HKCU\Software\Classes, where an unelevated program
    // can register its own command: an elevated TrioKey never calls it.
    // With an unsplit admin token (UAC off) that reason does not apply, and no prompt is needed.
    if is_admin() {
        return Elevated::Exit(if on { install() } else { uninstall() });
    }
    let Ok(exe) = std::env::current_exe() else { return Elevated::Failed };
    let w = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (verb, file, params) = (w("runas"), w(&exe.to_string_lossy()), w(if on { INSTALL } else { UNINSTALL }));
    unsafe {
        let mut sei: SHELLEXECUTEINFOW = std::mem::zeroed();
        sei.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
        sei.hwnd = hwnd;
        sei.lpVerb = verb.as_ptr();
        sei.lpFile = file.as_ptr();
        sei.lpParameters = params.as_ptr();
        sei.nShow = SW_HIDE;
        if ShellExecuteExW(&mut sei) == 0 {
            return if GetLastError() == ERROR_CANCELLED { Elevated::Cancelled } else { Elevated::Failed };
        }
        if sei.hProcess.is_null() {
            return Elevated::Failed;
        }
        if WaitForSingleObject(sei.hProcess, 120_000) != WAIT_OBJECT_0 {
            // not an exit code: GetExitCodeProcess would say 259 (still active)
            CloseHandle(sei.hProcess);
            return Elevated::Timeout;
        }
        let mut code = FAILED;
        let got = GetExitCodeProcess(sei.hProcess, &mut code);
        CloseHandle(sei.hProcess);
        if got == 0 {
            Elevated::Failed
        } else {
            Elevated::Exit(code)
        }
    }
}

/// Start the admin copy now through the task (no UAC prompt).
pub fn start_task() -> bool {
    schtasks(&["/Run", "/TN", TASK])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn elevated_implies_admin() {
        assert!(!is_elevated() || is_admin());
    }

    #[test]
    fn parses_mode() {
        assert_eq!(mode(args(&[])), Mode::Normal);
        assert_eq!(mode(args(&["--install-admin"])), Mode::InstallAdmin);
        assert_eq!(mode(args(&["--uninstall-admin"])), Mode::UninstallAdmin);
        assert_eq!(mode(args(&["--install"])), Mode::Normal);
        assert_eq!(mode(args(&["--INSTALL-ADMIN"])), Mode::Normal);
        // only the first argument counts
        assert_eq!(mode(args(&["x", "--install-admin"])), Mode::Normal);
    }

    #[test]
    fn task_xml_settings_and_escaping() {
        let x = task_xml(r#"PC\Tôm & "Cua" <a>'b'"#, r"C:\Program Files\TrioKey\triokey.exe");
        assert!(x.starts_with(r#"<?xml version="1.0" encoding="UTF-16"?>"#));
        assert_eq!(x.matches(r"<UserId>PC\Tôm &amp; &quot;Cua&quot; &lt;a&gt;&apos;b&apos;</UserId>").count(), 2);
        assert!(x.contains(r"<Command>C:\Program Files\TrioKey\triokey.exe</Command>"));
        for must in [
            "<LogonTrigger>",
            "<LogonType>InteractiveToken</LogonType>",
            "<RunLevel>HighestAvailable</RunLevel>",
            "<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>",
            "<Priority>4</Priority>",
            "<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>",
            "<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>",
        ] {
            assert!(x.contains(must), "{must}");
        }
        let x = task_xml("u", "C:\\a&b\\triokey.exe");
        assert!(x.contains(r"<Command>C:\a&amp;b\triokey.exe</Command>"));
        // no unescaped markup characters from the inputs
        assert!(!task_xml("<", ">").contains("<UserId><"));
    }
}
