//! The state glyphs `V` / `E` both apps draw (Windows tray, macOS menu bar), rasterised at the exact pixel size asked
//! for, so they are crisp at any scale without shipping image files. Every stroke is a capsule (round ends), so the
//! glyph is the distance to a few segments. E's straight edges fall on whole pixels; V's diagonals and all round ends
//! are anti-aliased.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// Typing Vietnamese.
    V,
    /// Keys pass through: TrioKey off, or yielding to another input method.
    E,
}

/// Glyph colours (R, G, B). DEEP for a light bar, LIGHT for a dark one; BLUE_DEEP is also the app icon's tile.
pub const BLUE_DEEP: [u8; 3] = [0x35, 0x65, 0xB8];
pub const RED_DEEP: [u8; 3] = [0xB2, 0x3B, 0x33];
pub const BLUE_LIGHT: [u8; 3] = [0x8A, 0xB4, 0xF8];
pub const RED_LIGHT: [u8; 3] = [0xF2, 0x8B, 0x82];

/// V is red, E is blue, in the shade that reads on the bar.
pub fn color(g: Glyph, light_bar: bool) -> [u8; 3] {
    match (g, light_bar) {
        (Glyph::V, true) => RED_DEEP,
        (Glyph::V, false) => RED_LIGHT,
        (Glyph::E, true) => BLUE_DEEP,
        (Glyph::E, false) => BLUE_LIGHT,
    }
}

/// Whole-pixel layout at size `n`: margin, stroke, and E's left edge, width, middle bar top and middle bar width.
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
pub fn render(g: Glyph, n: u32, rgb: [u8; 3]) -> Vec<u8> {
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
            // coverage from the signed distance at the pixel centre: an edge on a pixel boundary gives exactly 0 or 1
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

/// WCAG 2 contrast ratio of two sRGB colours (the apps' tests check their colours with it).
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows tray sizes, and the macOS menu bar's 18 pt at 1x and 2x.
    const SIZES: [u32; 9] = [16, 18, 20, 24, 32, 36, 40, 48, 64];

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

    #[test]
    fn colours_meet_wcag_aa_on_their_bar() {
        // Windows taskbars (#202020, #F3F3F3) and typical macOS menu bars (dark #2B2B2B, light #EDEDED)
        for bg in [[0x20; 3], [0x2B; 3]] {
            for g in [Glyph::V, Glyph::E] {
                assert!(contrast(color(g, false), bg) >= 4.5, "{g:?} on {bg:?}");
            }
        }
        for bg in [[0xF3; 3], [0xED; 3]] {
            for g in [Glyph::V, Glyph::E] {
                assert!(contrast(color(g, true), bg) >= 4.5, "{g:?} on {bg:?}");
            }
        }
    }
}
