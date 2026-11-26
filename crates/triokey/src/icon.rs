//! Tray glyphs `V` / `E`, rasterised at the exact pixel size Windows asks for, so they are
//! crisp at any DPI without shipping .ico files. Every stroke is a capsule (round ends), so the
//! glyph is the distance to a few segments. E's straight edges fall on whole pixels; V's
//! diagonals and all round ends are anti-aliased.

use std::ptr::null_mut;

use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// Typing Vietnamese.
    V,
    /// Keys pass through: TrioKey off, or yielding to a CJK IME.
    E,
}

/// Glyph colours (R, G, B). DEEP for a light taskbar, LIGHT for a dark one; BLUE_DEEP is also
/// the app icon's tile.
const BLUE_DEEP: [u8; 3] = [0x35, 0x65, 0xB8];
const RED_DEEP: [u8; 3] = [0xB2, 0x3B, 0x33];
const BLUE_LIGHT: [u8; 3] = [0x8A, 0xB4, 0xF8];
const RED_LIGHT: [u8; 3] = [0xF2, 0x8B, 0x82];

/// V is red, E is blue, in the shade that reads on the taskbar.
pub fn color(g: Glyph, light_taskbar: bool) -> [u8; 3] {
    match (g, light_taskbar) {
        (Glyph::V, true) => RED_DEEP,
        (Glyph::V, false) => RED_LIGHT,
        (Glyph::E, true) => BLUE_DEEP,
        (Glyph::E, false) => BLUE_LIGHT,
    }
}

/// Whole-pixel layout at size `n`: margin, stroke, and E's left edge, width, middle bar top
/// and middle bar width.
struct Layout {
    m: f32,
    t: f32,
    e_x0: f32,
    e_w: f32,
    e_mid: f32,
    e_mid_w: f32,
}

fn layout(n: u32) -> Layout {
    let nf = n as f32;
    let m = (nf / 16.0).round().max(1.0);
    let t = (nf * 0.18).round().max(3.0);
    let h = nf - 2.0 * m;
    // same parity as n, so E sits centred on whole pixels
    let e_w = nf - 2.0 * ((nf - h * 0.75) / 2.0).round();
    // floor: an odd leftover pixel goes below the middle bar, as in most type
    let e_mid = m + ((h - t) / 2.0).floor();
    Layout { m, t, e_x0: (nf - e_w) / 2.0, e_w, e_mid, e_mid_w: e_w - (e_w * 0.15).round().max(1.0) }
}

/// `n`×`n` RGBA, straight alpha, top row first.
fn render(g: Glyph, n: u32, rgb: [u8; 3]) -> Vec<u8> {
    let nf = n as f32;
    let Layout { m, t, e_x0, e_w, e_mid, e_mid_w } = layout(n);
    let r = t / 2.0;
    // capsules of radius r around these segments
    let segs: Vec<((f32, f32), (f32, f32))> = match g {
        Glyph::E => {
            let (l, top, bottom) = (e_x0 + r, m + r, nf - m - r);
            let bar = |y: f32, w: f32| ((l, y), (e_x0 + w - r, y));
            vec![((l, top), (l, bottom)), bar(top, e_w), bar(e_mid + r, e_mid_w), bar(bottom, e_w)]
        }
        Glyph::V => {
            let (top, tip) = (m + r, (nf / 2.0, nf - m - r));
            vec![((m + r, top), tip), ((nf - m - r, top), tip)]
        }
    };
    let mut out = Vec::with_capacity((n * n * 4) as usize);
    for py in 0..n {
        for px in 0..n {
            let (x, y) = (px as f32 + 0.5, py as f32 + 0.5);
            let d = segs.iter().map(|&(a, b)| seg_dist(x, y, a, b)).fold(f32::MAX, f32::min) - r;
            // coverage from the signed distance at the pixel centre: an edge on a pixel
            // boundary gives exactly 0 or 1
            out.extend_from_slice(&rgb);
            out.push(((0.5 - d).clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    out
}

fn seg_dist(x: f32, y: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let k = (((x - a.0) * dx + (y - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    ((x - a.0 - k * dx).powi(2) + (y - a.1 - k * dy).powi(2)).sqrt()
}

/// 32-bit alpha icon; the caller owns it (`DestroyIcon`).
pub unsafe fn hicon(g: Glyph, n: i32, rgb: [u8; 3]) -> HICON {
    let n = n.max(16);
    let px = render(g, n as u32, rgb);
    let mut bi: BITMAPINFO = std::mem::zeroed();
    bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = n;
    // negative height: top-down rows, same order as `render`
    bi.bmiHeader.biHeight = -n;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    let mut bits = null_mut();
    let color = CreateDIBSection(null_mut(), &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if color.is_null() || bits.is_null() {
        return null_mut();
    }
    let dst = std::slice::from_raw_parts_mut(bits as *mut u8, px.len());
    for (d, s) in dst.as_chunks_mut::<4>().0.iter_mut().zip(px.as_chunks::<4>().0) {
        d.copy_from_slice(&[s[2], s[1], s[0], s[3]]);
    }
    // monochrome rows are WORD-aligned; with per-pixel alpha the mask is not used
    let mask_bits = vec![0u8; (n as usize).div_ceil(16) * 2 * n as usize];
    let mask = CreateBitmap(n, n, 1, 1, mask_bits.as_ptr().cast());
    let info = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
    let icon = CreateIconIndirect(&info);
    DeleteObject(color);
    DeleteObject(mask);
    icon
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [u32; 7] = [16, 20, 24, 32, 40, 48, 64];

    fn alpha(px: &[u8], n: u32, x: u32, y: u32) -> u8 {
        px[((y * n + x) * 4 + 3) as usize]
    }

    #[test]
    fn e_edges_are_on_pixels_and_v_is_symmetric() {
        for n in SIZES {
            let e = render(Glyph::E, n, BLUE_DEEP);
            let v = render(Glyph::V, n, RED_DEEP);
            assert_eq!(e.len(), (n * n * 4) as usize);
            let l = layout(n);
            let (x0, t, m, mid) = (l.e_x0 as u32, l.t as u32, l.m as u32, l.e_mid as u32);
            assert!(t >= 3 && l.e_mid_w < l.e_w, "{n}");
            // a row between the top and middle bars crosses only the stem: hard edges
            for x in 0..n {
                assert_eq!(alpha(&e, n, x, m + t), if (x0..x0 + t).contains(&x) { 255 } else { 0 }, "E row at {n}, x {x}");
            }
            // a column through the three bars, clear of their round ends: hard edges
            for y in 0..n {
                let ink = (m..m + t).contains(&y) || (mid..mid + t).contains(&y) || (n - m - t..n - m).contains(&y);
                assert_eq!(alpha(&e, n, x0 + t + 1, y), if ink { 255 } else { 0 }, "E column at {n}, y {y}");
            }
            // round ends are anti-aliased, the outer corners are cut
            assert!(e.as_chunks::<4>().0.iter().any(|p| p[3] > 0 && p[3] < 255));
            assert!(alpha(&e, n, x0, m) < 255 && alpha(&e, n, x0, n - m - 1) < 255);
            // both have ink and a transparent 1 px border; V is symmetric
            assert!(v.as_chunks::<4>().0.iter().any(|p| p[3] == 255));
            for i in 0..n {
                for p in [&e, &v] {
                    assert_eq!(alpha(p, n, i, 0) | alpha(p, n, 0, i) | alpha(p, n, i, n - 1) | alpha(p, n, n - 1, i), 0, "border at {n}");
                }
            }
            for y in 0..n {
                for x in 0..n {
                    assert_eq!(alpha(&v, n, x, y), alpha(&v, n, n - 1 - x, y), "V asymmetric at {n}");
                }
            }
        }
    }

    /// WCAG 2 contrast ratio of two sRGB colours.
    fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
        let lin = |v: u8| {
            let s = v as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        let lum = |c: [u8; 3]| 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn colours_meet_wcag_aa_on_their_taskbar() {
        for (g, light, bg) in [(Glyph::V, false, 0x20), (Glyph::E, false, 0x20), (Glyph::V, true, 0xF3), (Glyph::E, true, 0xF3)] {
            let c = contrast(color(g, light), [bg; 3]);
            println!("{g:?} light={light}: {c:.2}");
            assert!(c >= 4.5, "{g:?} light={light}: {c:.2}");
        }
    }

    /// `cargo test -p triokey -- --ignored icons_preview` writes docs/icons-preview.png.
    #[test]
    #[ignore]
    fn icons_preview() {
        const Z: u32 = 4;
        const GAP: u32 = 12;
        let w = SIZES.iter().map(|n| n * Z + GAP).sum::<u32>() + GAP;
        let row_h = 64 * Z + 2 * GAP;
        let h = row_h * 4;
        let mut img = vec![0u8; (w * h * 3) as usize];
        let rows = [(0x20u8, Glyph::V, false), (0x20, Glyph::E, false), (0xF3, Glyph::V, true), (0xF3, Glyph::E, true)];
        for (row, &(bg, g, light)) in rows.iter().enumerate() {
            let y0 = row as u32 * row_h;
            img[(y0 * w * 3) as usize..((y0 + row_h) * w * 3) as usize].fill(bg);
            let fg = color(g, light);
            let mut x0 = GAP;
            for n in SIZES {
                let px = render(g, n, fg);
                for y in 0..n * Z {
                    for x in 0..n * Z {
                        let a = alpha(&px, n, x / Z, y / Z) as u32;
                        let i = (((y0 + GAP + y) * w + x0 + x) * 3) as usize;
                        for (k, &c) in fg.iter().enumerate() {
                            img[i + k] = ((c as u32 * a + bg as u32 * (255 - a)) / 255) as u8;
                        }
                    }
                }
                x0 += n * Z + GAP;
            }
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/icons-preview.png");
        std::fs::write(path, png_rgb(w, h, &img)).unwrap();
    }

    /// Deflate bit writer: values go LSB first, Huffman codes MSB first.
    struct Bits {
        z: Vec<u8>,
        acc: u32,
        n: u32,
    }

    impl Bits {
        fn put(&mut self, v: u32, len: u32) {
            self.acc |= v << self.n;
            self.n += len;
            while self.n >= 8 {
                self.z.push(self.acc as u8);
                self.acc >>= 8;
                self.n -= 8;
            }
        }
        fn code(&mut self, c: u32, len: u32) {
            self.put((0..len).fold(0, |r, i| r | ((c >> i) & 1) << (len - 1 - i)), len);
        }
        /// A literal/length symbol in the fixed Huffman code.
        fn sym(&mut self, v: u32) {
            match v {
                0..=143 => self.code(0x30 + v, 8),
                144..=255 => self.code(0x190 + v - 144, 9),
                256..=279 => self.code(v - 256, 7),
                _ => self.code(0xC0 + v - 280, 8),
            }
        }
    }

    /// Minimal PNG: one IDAT deflated with the fixed Huffman code and only distance-3 matches
    /// (runs of one RGB colour), which is most of this picture.
    fn png_rgb(w: u32, h: u32, rgb: &[u8]) -> Vec<u8> {
        fn crc(data: &[u8]) -> u32 {
            let mut c = !0u32;
            for &b in data {
                c ^= b as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
                }
            }
            !c
        }
        fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
            out.extend((data.len() as u32).to_be_bytes());
            let body: Vec<u8> = kind.iter().chain(data).copied().collect();
            out.extend(&body);
            out.extend(crc(&body).to_be_bytes());
        }
        const BASE: [u32; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
        const EXTRA: [u32; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
        let mut raw = Vec::new();
        for row in rgb.chunks_exact(w as usize * 3) {
            raw.push(0);
            raw.extend_from_slice(row);
        }
        let mut b = Bits { z: vec![0x78, 0x01], acc: 0, n: 0 };
        b.put(0b011, 3); // final block, fixed Huffman
        let mut i = 0;
        while i < raw.len() {
            let run = if i >= 3 { (i..raw.len().min(i + 258)).take_while(|&j| raw[j] == raw[j - 3]).count() } else { 0 };
            if run >= 3 {
                let k = BASE.iter().rposition(|&l| l <= run as u32).unwrap();
                b.sym(257 + k as u32);
                b.put(run as u32 - BASE[k], EXTRA[k]);
                b.code(2, 5); // distance 3
                i += run;
            } else {
                b.sym(raw[i] as u32);
                i += 1;
            }
        }
        b.sym(256);
        b.put(0, 7); // pad to a byte
        let mut z = b.z;
        let (mut a, mut s) = (1u32, 0u32);
        for &v in &raw {
            a = (a + v as u32) % 65521;
            s = (s + a) % 65521;
        }
        z.extend(((s << 16) | a).to_be_bytes());
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend(w.to_be_bytes());
        ihdr.extend(h.to_be_bytes());
        ihdr.extend([8, 2, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &z);
        chunk(&mut out, b"IEND", &[]);
        out
    }
}
