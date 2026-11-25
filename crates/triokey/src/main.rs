#![windows_subsystem = "windows"]

mod admin;
mod app;
mod autostart;
mod config;
mod hook;
mod icon;
mod ime;
mod ime_watch;
mod inject;
mod keymap;
mod settings_ui;
mod tray;
mod update;

fn main() {
    // before the single-instance check: the elevated child runs while TrioKey is running
    match admin::mode(std::env::args().skip(1)) {
        admin::Mode::Normal => app::run(),
        admin::Mode::InstallAdmin => std::process::exit(admin::install() as i32),
        admin::Mode::UninstallAdmin => std::process::exit(admin::uninstall() as i32),
    }
}
