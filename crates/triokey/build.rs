//! Embeds the application manifest (decision 0006) and the app icon (assets/brand/triokey.ico).
//! Without the manifest the settings window gets the classic controls and Windows
//! bitmap-stretches it on a scaled monitor.
//!
//! The GNU toolchain has no resource compiler here (no windres), so both go into one COFF
//! object with a `.rsrc` section written below, the way embed-manifest does it for the
//! manifest alone. Two objects with their own `.rsrc` would not merge into one resource tree.

use std::path::PathBuf;

use embed_manifest::manifest::{DpiAwareness, ExecutionLevel, MaxVersionTested, SupportedOS};
use embed_manifest::new_manifest;

const ICON: &str = "../../assets/brand/triokey.ico";
const RT_ICON: u32 = 3;
const RT_GROUP_ICON: u32 = 14;
const RT_MANIFEST: u32 = 24;
const LANG_EN_US: u32 = 1033;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={ICON}");
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
    let ico = std::fs::read(ICON).expect("read the app icon");
    let mut resources = icon_resources(&ico);
    resources.push((RT_MANIFEST, 1, manifest.to_string().into_bytes()));

    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    assert!(env == "gnu" && arch == "x86_64", "build.rs writes resources for x86_64-pc-windows-gnu only, not {arch}-{env}");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("resources.o");
    std::fs::write(&out, coff(&resources)).expect("write resources.o");
    println!("cargo:rustc-link-arg-bins={}", out.display());
}

/// Splits an .ico file into RT_ICON images (ids 1..) and the RT_GROUP_ICON (id 1) that
/// lists them: the group entry is the file's 16-byte entry with the offset replaced by a u16 id.
fn icon_resources(ico: &[u8]) -> Vec<(u32, u32, Vec<u8>)> {
    let u16_at = |i: usize| u16::from_le_bytes([ico[i], ico[i + 1]]) as usize;
    let u32_at = |i: usize| u32::from_le_bytes(ico[i..i + 4].try_into().unwrap()) as usize;
    assert!(ico.len() >= 6 && u16_at(2) == 1, "{ICON} is not an icon file");
    let count = u16_at(4);
    let mut group = ico[..6].to_vec();
    let mut out = Vec::new();
    for i in 0..count {
        let e = 6 + 16 * i;
        let (size, offset) = (u32_at(e + 8), u32_at(e + 12));
        let id = i as u32 + 1;
        out.push((RT_ICON, id, ico[offset..offset + size].to_vec()));
        group.extend_from_slice(&ico[e..e + 12]);
        group.extend_from_slice(&(id as u16).to_le_bytes());
    }
    out.push((RT_GROUP_ICON, 1, group));
    out
}

/// One x86-64 COFF object holding a `.rsrc` section: type -> id -> language directories,
/// then the data entries, then the data. Each data entry's address is section-relative and
/// gets an IMAGE_REL_AMD64_ADDR32NB relocation against the section symbol.
fn coff(resources: &[(u32, u32, Vec<u8>)]) -> Vec<u8> {
    // the directory tables below need types, and ids within a type, strictly ascending
    assert!(resources.windows(2).all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)), "resources out of order");
    let mut types: Vec<u32> = resources.iter().map(|r| r.0).collect();
    types.dedup();
    let ids_of = |t: u32| resources.iter().filter(move |r| r.0 == t);
    let table = |n: usize| {
        let mut b = vec![0u8; 16];
        b[14..16].copy_from_slice(&(n as u16).to_le_bytes());
        b
    };
    let entry = |id: u32, offset: usize, dir: bool| {
        let mut b = id.to_le_bytes().to_vec();
        b.extend_from_slice(&(offset as u32 | if dir { 0x8000_0000 } else { 0 }).to_le_bytes());
        b
    };

    // layout: root, one id table per type, one language table per resource, data entries, data
    let root_len = 16 + 8 * types.len();
    let type_lens: Vec<usize> = types.iter().map(|&t| 16 + 8 * ids_of(t).count()).collect();
    let lang_start = root_len + type_lens.iter().sum::<usize>();
    let entries_start = lang_start + resources.len() * 24;
    let data_start = entries_start + resources.len() * 16;

    let mut sec = table(types.len());
    let mut at = root_len;
    for (&t, len) in types.iter().zip(&type_lens) {
        sec.extend(entry(t, at, true));
        at += len;
    }
    let mut k = 0; // resource index in tree order (resources are grouped by type)
    for &t in &types {
        sec.extend(table(ids_of(t).count()));
        for r in ids_of(t) {
            sec.extend(entry(r.1, lang_start + 24 * k, true));
            k += 1;
        }
    }
    for k in 0..resources.len() {
        sec.extend(table(1));
        sec.extend(entry(LANG_EN_US, entries_start + 16 * k, false));
    }
    let mut relocs = Vec::new();
    let mut data_at = data_start;
    for r in resources {
        relocs.push(sec.len() as u32);
        sec.extend((data_at as u32).to_le_bytes());
        sec.extend((r.2.len() as u32).to_le_bytes());
        sec.extend([0u8; 8]);
        data_at = (data_at + r.2.len()).next_multiple_of(8);
    }
    assert_eq!(sec.len(), data_start);
    for r in resources {
        sec.extend(&r.2);
        sec.resize(sec.len().next_multiple_of(8), 0);
    }

    const HEADERS: u32 = 20 + 40;
    let size = sec.len() as u32;
    let symtab = HEADERS + size + 10 * relocs.len() as u32;
    let mut o = Vec::new();
    o.extend(0x8664u16.to_le_bytes()); // machine
    o.extend(1u16.to_le_bytes()); // sections
    o.extend(0u32.to_le_bytes()); // timestamp: 0 keeps builds reproducible
    o.extend(symtab.to_le_bytes());
    o.extend(2u32.to_le_bytes()); // symbols: the section symbol and its aux record
    o.extend([0u8; 4]); // optional header size, characteristics
    o.extend(b".rsrc\0\0\0");
    o.extend([0u8; 8]); // virtual size, virtual address
    o.extend(size.to_le_bytes());
    o.extend(HEADERS.to_le_bytes()); // raw data
    o.extend((HEADERS + size).to_le_bytes()); // relocations
    o.extend([0u8; 4]); // line numbers
    o.extend((relocs.len() as u16).to_le_bytes());
    o.extend([0u8; 2]);
    o.extend(0xC000_0040u32.to_le_bytes()); // initialized data, read, write
    o.extend(&sec);
    for r in &relocs {
        o.extend(r.to_le_bytes());
        o.extend(0u32.to_le_bytes()); // symbol 0: .rsrc
        o.extend(3u16.to_le_bytes()); // IMAGE_REL_AMD64_ADDR32NB
    }
    o.extend(b".rsrc\0\0\0");
    o.extend(0u32.to_le_bytes()); // value
    o.extend(1u16.to_le_bytes()); // section number
    o.extend([0, 0, 3, 1]); // type, storage class static, one aux record
    o.extend(size.to_le_bytes());
    o.extend((relocs.len() as u16).to_le_bytes());
    o.extend([0u8; 12]);
    o.extend(4u32.to_le_bytes()); // empty string table
    o
}
