//! Embeds the application manifest (decision 0006): without one the settings window gets the
//! classic controls and Windows bitmap-stretches it on a scaled monitor.

use embed_manifest::manifest::{DpiAwareness, ExecutionLevel, MaxVersionTested, SupportedOS};
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }
    // new_manifest already depends on Common Controls 6.0; the rest is spelled out on purpose
    let manifest = new_manifest("TrioKey")
        // one GUID covers Windows 10 and 11
        .supported_os(SupportedOS::Windows10..=SupportedOS::Windows10)
        .max_version_tested(MaxVersionTested::Windows11Version22H2)
        .dpi_awareness(DpiAwareness::PerMonitorV2Only)
        // admin mode elevates on demand (admin.rs), never at start
        .requested_execution_level(ExecutionLevel::AsInvoker);
    embed_manifest(manifest).expect("embed the application manifest");
}
