//! Tray glyphs `V` / `E`, rasterised at the exact pixel size Windows asks for, so they are
//! crisp at any DPI without shipping .ico files. Stems and bars are snapped to whole pixels;
//! only the diagonals of `V` are anti-aliased.

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

/// Glyph colours (R, G, B): white for a dark taskbar, near-black for a light one.
pub const ON_DARK: [u8; 3] = [0xFF, 0xFF, 0xFF];
pub const ON_LIGHT: [u8; 3] = [0x1C, 0x1C, 0x1C];

/// `n`×`n` RGBA, straight alpha, top row first.
pub fn render(g: Glyph, n: u32, rgb: [u8; 3]) -> Vec<u8> {
    let nf = n as f32;
    let t = (nf * 0.12).round().max(2.0);
    let m = (nf * 0.125).round();
    let (top, bottom) = (m, nf - m);
    let h = bottom - top;
    let inside: Box<dyn Fn(f32, f32) -> bool> = match g {
        Glyph::E => {
            let w = (h * 0.72).round();
            let x0 = ((nf - w) / 2.0).floor();
            let mid = ((top + bottom - t) / 2.0).round();
            let mid_w = (w * 0.85).round();
            Box::new(move |x, y| {
                let in_x = |a: f32, b: f32| x >= a && x < b;
                let in_y = |a: f32, b: f32| y >= a && y < b;
                (in_x(x0, x0 + t) && in_y(top, bottom))
                    || (in_x(x0, x0 + w) && (in_y(top, top + t) || in_y(bottom - t, bottom)))
                    || (in_x(x0, x0 + mid_w) && in_y(mid, mid + t))
            })
        }
        Glyph::V => {
            let w = h;
            let x0 = (nf - w) / 2.0;
            let r = t * 0.55;
            let (l, tip, rt) = ((x0 + r, top), (nf / 2.0, bottom - r), (x0 + w - r, top));
            Box::new(move |x, y| y >= top && y < bottom && (seg_dist(x, y, l, tip) <= r || seg_dist(x, y, rt, tip) <= r))
        }
    };
    const S: u32 = 4;
    let mut out = Vec::with_capacity((n * n * 4) as usize);
    for py in 0..n {
        for px in 0..n {
            let mut hits = 0;
            for sy in 0..S {
                for sx in 0..S {
                    let x = px as f32 + (sx as f32 + 0.5) / S as f32;
                    let y = py as f32 + (sy as f32 + 0.5) / S as f32;
                    hits += inside(x, y) as u32;
                }
            }
            out.extend_from_slice(&rgb);
            out.push((hits * 255 / (S * S)) as u8);
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

    const SIZES: [u32; 6] = [16, 20, 24, 32, 40, 48];

    fn alpha(px: &[u8], n: u32, x: u32, y: u32) -> u8 {
        px[((y * n + x) * 4 + 3) as usize]
    }

    #[test]
    fn e_is_pixel_aligned_and_v_is_not_e() {
        for n in SIZES {
            let e = render(Glyph::E, n, ON_DARK);
            let v = render(Glyph::V, n, ON_DARK);
            assert_eq!(e.len(), (n * n * 4) as usize);
            // E has only fully on / fully off pixels
            assert!(e.as_chunks::<4>().0.iter().all(|p| p[3] == 0 || p[3] == 255), "E blurry at {n}");
            // both have ink, transparent corners, and differ
            assert!(e.as_chunks::<4>().0.iter().any(|p| p[3] == 255) && v.as_chunks::<4>().0.iter().any(|p| p[3] > 200));
            assert_eq!(alpha(&v, n, 0, 0), 0);
            assert_eq!(alpha(&e, n, n - 1, n - 1), 0);
            assert_ne!(e, v);
            // V is symmetric
            for y in 0..n {
                for x in 0..n {
                    assert_eq!(alpha(&v, n, x, y), alpha(&v, n, n - 1 - x, y), "V asymmetric at {n}");
                }
            }
        }
    }

    /// `cargo test -p triokey -- --ignored icons_preview` writes docs/icons-preview.png.
    #[test]
    #[ignore]
    fn icons_preview() {
        // all colours are neutral grey, so an 8-bit greyscale PNG is enough
        const Z: u32 = 4;
        const GAP: u32 = 12;
        let w = SIZES.iter().map(|n| n * Z + GAP).sum::<u32>() + GAP;
        let row_h = 48 * Z + 2 * GAP;
        let h = row_h * 4;
        let themes = [(0x20u8, ON_DARK), (0xF3, ON_LIGHT)];
        let mut img = vec![0u8; (w * h) as usize];
        for (row, (bg, fg, g)) in themes.iter().flat_map(|&(bg, fg)| [(bg, fg, Glyph::V), (bg, fg, Glyph::E)]).enumerate() {
            let y0 = row as u32 * row_h;
            img[(y0 * w) as usize..((y0 + row_h) * w) as usize].fill(bg);
            let mut x0 = GAP;
            for n in SIZES {
                let px = render(g, n, fg);
                for y in 0..n * Z {
                    for x in 0..n * Z {
                        let a = alpha(&px, n, x / Z, y / Z) as u32;
                        let v = (fg[0] as u32 * a + bg as u32 * (255 - a)) / 255;
                        img[((y0 + GAP + y) * w + x0 + x) as usize] = v as u8;
                    }
                }
                x0 += n * Z + GAP;
            }
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/icons-preview.png");
        std::fs::write(path, png_grey(w, h, &img)).unwrap();
    }

    /// Minimal PNG: one IDAT of stored (uncompressed) deflate blocks.
    fn png_grey(w: u32, h: u32, grey: &[u8]) -> Vec<u8> {
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
        let mut raw = Vec::new();
        for row in grey.chunks_exact(w as usize) {
            raw.push(0);
            raw.extend_from_slice(row);
        }
        let mut z = vec![0x78, 0x01];
        let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
        for (i, b) in blocks.iter().enumerate() {
            z.push((i + 1 == blocks.len()) as u8);
            z.extend((b.len() as u16).to_le_bytes());
            z.extend((!(b.len() as u16)).to_le_bytes());
            z.extend_from_slice(b);
        }
        let (mut a, mut s) = (1u32, 0u32);
        for &b in &raw {
            a = (a + b as u32) % 65521;
            s = (s + a) % 65521;
        }
        z.extend(((s << 16) | a).to_be_bytes());
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend(w.to_be_bytes());
        ihdr.extend(h.to_be_bytes());
        ihdr.extend([8, 0, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &z);
        chunk(&mut out, b"IEND", &[]);
        out
    }
}
