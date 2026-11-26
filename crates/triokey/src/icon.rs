//! Tray icons from the state glyphs `V` / `E` (`triokey_shared::glyph`), rasterised at the exact pixel size Windows
//! asks for, so they are crisp at any DPI without shipping .ico files.

use std::ptr::null_mut;

use triokey_shared::glyph::{render, Glyph};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

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
pub(crate) mod tests {
    use super::*;

    use triokey_shared::glyph::color;

    const SIZES: [u32; 7] = [16, 20, 24, 32, 40, 48, 64];

    fn alpha(px: &[u8], n: u32, x: u32, y: u32) -> u8 {
        px[((y * n + x) * 4 + 3) as usize]
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
    pub(crate) fn png_rgb(w: u32, h: u32, rgb: &[u8]) -> Vec<u8> {
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
