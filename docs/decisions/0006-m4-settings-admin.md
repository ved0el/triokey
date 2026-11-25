# 0006: M4 — settings window, autostart, admin mode, tray icons

Date: 2026-11-25. Status: accepted, not yet run on a real machine (see HANDS_ON_TEST.md M4).

## Settings and live apply

- `app::SETTINGS` is now a `RwLock<Settings>` (`Settings` is `Copy`); the hook and the IME
  watcher read a copy per key / per pass, so hotkey and `yield_cjk` changes apply at once.
- `Engine` has no config setter, and the engine crate is off limits for M4. The hook thread
  keeps the `Lexicon` and rebuilds the engine on `WM_APP_RECONFIGURE` (posted by
  `app::update`). The word being typed at that moment is dropped.
- Each change rewrites `config.txt` from the tray thread, never from the hook (spec 5.8).
  Unknown keys in a hand-edited file are lost on the first change.
- A `config.txt` that exists but cannot be read (saved as UTF-16, locked) is never
  overwritten: defaults apply for this run, `app::update` skips the write, and the settings
  window says so each time it opens. A UTF-8 BOM is skipped by `config::parse`.
- The `Bật gõ tiếng Việt` box shows and sets the live on/off state and also saves `enabled`.
  The tray click and the hotkey still do not write the file; the tray's `WM_APP_STATE`
  handler updates the box when the window is open.
- Hotkey presets: Ctrl+Shift (default), Alt+Z, Ctrl+Space, `Không dùng` (`hotkey = none`).
  These are exactly what `hook::hotkey_step` already handles; Ctrl+Alt+V was left out
  because AltGr layouts send Ctrl+Alt.
- Alt+Z and Ctrl+Space swallow the key. `HotState::swallowed` remembers it until its release:
  auto-repeats and the release are swallowed without toggling again, even if the modifier is
  released first; any other key-down clears it, so a lost release cannot eat a later key.
  Shift held means another shortcut (Ctrl+Shift+Space), not the toggle. After a swallowed
  Alt+Z the hook sends one tap of the unassigned key `0xE8`, tagged like all our input:
  otherwise the app sees Alt go down and up with nothing between and opens its menu bar.
- The window is plain Win32 (`BUTTON`/`COMBOBOX`/`STATIC`) on the tray thread with
  `IsDialogMessageW` for Tab. Because it is not a dialog, `WM_ACTIVATE`/`WM_SETFOCUS` keep
  and restore the focused control themselves.

## Manifest, visual styles, DPI

- `crates/triokey/build.rs` embeds an application manifest with the build dependency
  `embed-manifest` (pure Rust, writes the resource object itself: no `windres`, `rc` or
  `mt`, works on the gnu and msvc toolchains). Content: Common Controls 6.0, `dpiAwareness`
  `permonitorv2`, `requestedExecutionLevel asInvoker` (admin mode elevates on demand),
  `supportedOS` Windows 10 (the same GUID covers Windows 11). The crate's other defaults
  stay: UTF-8 active code page (TrioKey calls only `W` functions), long paths.
- `SetProcessDPIAware()` is gone. The settings window is created empty at the primary
  monitor's origin, reads `GetDpiForWindow`, then sizes itself (`AdjustWindowRectExForDpi`)
  and builds its controls with the message font from
  `SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS)`. On `WM_DPICHANGED` it destroys the
  controls and builds them again for the new DPI (all state is in `app::settings()`, so this
  is less code than moving each control), at the position Windows suggests.
- Layout follows the Windows dialog metrics for Segoe UI 9 pt at 96 DPI: 11 px margins,
  17 px check boxes, 23 px button and combo box, 7 px between related rows, 11 px between
  groups. `InitCommonControlsEx(ICC_STANDARD_CLASSES)` runs before the first control.
- Not done: the tray icon and the window's caption icon still use the system-DPI
  `SM_CXSMICON`; dark mode for the window.

## Start with Windows

`HKCU\...\Run` value `TrioKey` = `"<current exe>"`. The checkbox shows the registry, not the
config, and TrioKey never writes Run without a click, even though `run_at_startup` defaults to
`true` (AT6). `run_at_startup` only remembers the choice while admin mode replaces Run.

## Admin mode (spec 5.9)

- The elevated child (`triokey.exe --install-admin` / `--uninstall-admin`, parsed in `main`
  before the single-instance check) does only the admin-only part: copy itself to
  `%ProgramFiles%\TrioKey\triokey.exe` and `schtasks /Create /XML` (or delete both). It exits
  0 / 1 / 2 (2 = folder removed at next reboot); 3 / 4 / 5 are the copy, task and
  folder-left cases named in `admin.rs`.
- Deviation from the brief: the **unelevated parent** removes/restores the Run value and
  writes config.txt after the child returns 0/2, so these land in the right user's hive.
- No paths from the environment in the child: `SHGetKnownFolderPath(FOLDERID_ProgramFiles)`,
  `GetSystemDirectoryW()\schtasks.exe`. The task XML is written inside the new admin-only
  folder (not `%TEMP%`, where it could be swapped before schtasks reads it), UTF-16 with BOM,
  then deleted.
- Task XML (`admin::task_xml`, unit tested): logon trigger and principal for this user
  (`GetUserNameExW(NameSamCompatible)`, XML-escaped), `InteractiveToken`, `HighestAvailable`,
  `ExecutionTimeLimit PT0S` (default 72 h would kill TrioKey), battery flags off, `Priority 4`
  (default 7 is below normal; bad for AT7), `IgnoreNew`.
- Admin mode assumes the user consent-elevates as themselves. Over-the-shoulder elevation
  with another account would register the task for that account.
- "Start now" runs `schtasks /Run /TN TrioKey`, then this copy quits; the new copy retries the
  single-instance check for 3 s, so it starts once this one is gone.
- Turning admin mode off from the running admin copy: the child cannot delete its own image,
  so the folder is removed at the next reboot (`MOVEFILE_DELAY_UNTIL_REBOOT`), and the Run
  value is not restored (only the user knows where the normal copy lives); a message says so.
  From the normal copy the folder goes at once and Run is restored if it was on.
- UAC "No" (`ERROR_CANCELLED`): nothing changes, no message.
- The child still running after two minutes is `Elevated::Timeout` with its own message, not
  "exit code 259". Uninstall returns 5 (`OK_FOLDER_LEFT`) when the task is gone but the
  folder could be neither deleted nor queued: the parent still clears `run_as_admin` and
  restores Run, and the message asks for a manual delete. A failed Run restore is reported.

## What an elevated TrioKey must not do (review of M4)

"Elevated" is `admin::is_elevated()` (the token), not `is_admin_copy()` (the path): a normal
copy started with "Run as administrator" counts too. It is true only for the elevated half of
a split UAC token (`TokenElevationTypeFull`). With UAC switched off, as the built-in
Administrator (and probably in Windows Sandbox, untested) the token is not split: no unelevated program of the same
user exists, the rules below would protect nothing, and one of them (no file creation) would
keep `config.txt` from ever being saved. There TrioKey behaves like a normal copy. The first
version of this check used `IsUserAnAdmin`, which is also true for an unsplit token; that
call now only decides whether `admin::run_elevated` can do the admin part in-process
(`admin::is_admin`). Unelevated programs of the same user control HKCU and `%APPDATA%`, so:

- **No `ShellExecute`.** Verbs (`open`, `runas`) are resolved through
  `HKCU\Software\Classes`, where such a program can register its own command and have it
  run with our token. The earlier claim here that opening a fixed path is safe was wrong:
  the handler is what gets hijacked, not the path. When elevated, `Mở thư mục cấu hình…`
  shows the path in a message box and starts nothing (an elevated `explorer.exe` by full
  path was rejected: it cannot be shown safe from here), and `admin::run_elevated` calls
  `install()` / `uninstall()` in-process instead of starting a `runas` child.
- **No file creation under `%APPDATA%`.** `app::write_config` is the only write. Elevated,
  it refuses unless `%APPDATA%\triokey` is an existing plain directory and `config.txt` an
  existing plain file (no reparse point on either: junction, symlink). The file is opened
  with `FILE_FLAG_OPEN_REPARSE_POINT` and without truncating, and checked on the handle
  (`GetFileInformationByHandle`): regular file, no reparse point, exactly one name. A hard
  link named `config.txt` to another file is therefore refused too, and the file that is
  checked is the file that is written. So a junction to an admin-only folder gets nothing created or truncated, and the
  default `config.txt` is not written at startup. The folder comes from
  `SHGetKnownFolderPath(FOLDERID_RoamingAppData)` instead of the `%APPDATA%` variable; HKCU
  can still redirect it, which is why the rules above are the real defence.
  Left open: a swap of the folder between the check and the open (needs handle-relative
  opens; not worth it for one fixed-text file). The elevated copy writes only when the user
  changes a setting: the updater's `last_update_check` stays in memory there (0007), so
  nothing reaches this code on a timer. Consequence: an admin copy with no
  `config.txt` runs on defaults and saves nothing; normally the normal copy wrote the file
  when admin mode was turned on.
- A failed `config.txt` write is never silent: `app::take_save_failed` makes the settings
  window say so when it opens and after the change that was not saved.
- `install()` checks that `%ProgramFiles%\TrioKey\triokey.exe` exists before it creates
  the task. In-process, the admin copy that has just turned admin mode off would otherwise
  register a task for its own renamed (deleted at reboot) image; it now reports the copy as
  failed, and admin mode has to be turned on again from the normal copy.
- Elevated TrioKey still reads the data files (`config.txt`, `force_*.txt`) through that
  path; it parses them as data and runs nothing named in them.
- Not changed: the elevated child copies `current_exe()` from a folder the user can write.
  A program that can write there can replace the exe before the UAC prompt anyway, so
  hardening the copy step gains nothing; the README says to turn admin mode on from a
  folder other programs cannot write.

## Rename from the working name (EVJ)

All system names are now `TrioKey` / `triokey` (Run value, task, `%ProgramFiles%\TrioKey`,
`%APPDATA%\triokey`, mutex, window classes). `app::migrate` runs once at startup, before
`load_settings`: when `%APPDATA%\triokey` does not exist and `%APPDATA%\evj` does, it copies
`config.txt`, `force_vi.txt`, `force_en.txt` (regular files only) and leaves the old folder.
It is skipped when elevated, and it resets `run_as_admin` because no `TrioKey` task exists
yet. Old-name leftovers (Run value `EVJ`, task `EVJ`, `%ProgramFiles%\EVJ`) are never touched;
HANDS_ON_TEST lists how to remove them. The new mutex name does not see a running old build.
The injected-input marker keeps its value `0x45564A31`.

## Single instance

`FindWindowW` on the tray class is replaced by a named mutex `Local\triokey-single-instance`
(per session, both copies see it). When the elevated copy creates it, the normal copy's
`CreateMutexW` fails with access denied; any failure or `ERROR_ALREADY_EXISTS` counts as
"running". A second start therefore waits up to 3 s before it quits silently. Any program
in the session can hold that name and keep TrioKey from starting; nothing is gained by it,
and a same-user program cannot be stopped from doing so, so it is left as is.

## Tray icons and menu

- Two states: `V` (typing Vietnamese) and `E` (keys pass through: off, or yielding to an IME);
  the tooltip says which, and names the IME's language (`IME Nhật` / `Trung` / `Hàn`, from
  the foreground layout at the watcher's last read; `IME CJK` before the first read).
  Monochrome glyph on transparent, white on a dark taskbar,
  near-black on a light one (`SystemUsesLightTheme`, refreshed on
  `WM_SETTINGCHANGE "ImmersiveColorSet"`).
- Every icon update goes through `tray::show`: `NIM_MODIFY`, then `NIM_ADD` if that fails
  (modify first, because an add that timed out may have gone through and a second add
  fails), and a 2 s timer retries while both fail (no taskbar yet at logon, Explorer
  restarting). `ChangeWindowMessageFilterEx` lets `TaskbarCreated` through to the elevated
  copy, which UIPI would otherwise drop. `NIM_DELETE` is sent in `WM_DESTROY`, while the
  window still exists.
- The Run check box is on only when the Run value starts this exe (`autostart::points_at`);
  a value left by a moved copy shows as off.
- No .ico assets: `icon::render` rasterises the glyphs at the exact `SM_CXSMICON` size into
  a 32-bit alpha icon. E's stems and bars are snapped to whole pixels (a test asserts there
  are no partial pixels); V's diagonals are 4×4 supersampled. This is less code than a
  generator plus committed .ico files and is crisp at any size, not only 16–48.
  `cargo test -p triokey -- --ignored icons_preview` rewrites `docs/icons-preview.png`.
- The settings window uses V in accent blue (readable on a light caption and a dark taskbar).
- Menu: `Tiếng Việt` (checked) · `Cài đặt…` · separator · `Thoát`. The p99 line moved into
  the settings window, split into "gửi" (keys that called `SendInput`, time includes it) and
  "không gửi". A dark popup menu would need the undocumented uxtheme ordinal 135
  (`SetPreferredAppMode`); not done.
