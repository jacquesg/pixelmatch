use crate::color::color_delta;
use crate::read_u32_ne;

/// Check if a pixel is likely a part of anti-aliasing;
/// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009.
///
/// DEVIATION FROM UPSTREAM (mapbox/pixelmatch):
/// Two changes to improve detection for thin strokes, text, and small features:
///
/// 1. Two-pass approach: the original algorithm only checks has_many_siblings on the
///    last neighbour found with the min/max delta, missing tied candidates. We now find
///    min/max in pass 1, then check ALL matching neighbours in pass 2.
///
/// 2. Relaxed sibling check: changed from requiring has_many_siblings in both images
///    (a AND b) to either image (a OR b). For 1px-wide strokes, the stroke-side
///    extreme never has 3+ identical siblings because the feature is too narrow.
pub fn antialiased(
    img: &[u8],
    x1: usize,
    y1: usize,
    width: usize,
    height: usize,
    img_a: &[u8],
    img_b: &[u8],
) -> bool {
    let x0 = x1.saturating_sub(1);
    let y0 = y1.saturating_sub(1);
    let x2 = (x1 + 1).min(width - 1);
    let y2 = (y1 + 1).min(height - 1);
    let pos = (y1 * width + x1) * 4;
    let mut zeroes: i32 = if x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2 { 1 } else { 0 };
    let mut min: f64 = 0.0;
    let mut max: f64 = 0.0;

    // Cache deltas and coordinates from pass 1 to avoid recomputing in pass 2.
    // Max 8 neighbours (3×3 grid minus centre).
    let mut deltas: [f64; 8] = [0.0; 8];
    let mut coords: [(usize, usize); 8] = [(0, 0); 8];
    let mut n: usize = 0;

    // Pass 1: find min/max brightness deltas and count equal neighbours
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }
            let delta = color_delta(img, img, pos, (y * width + x) * 4, true);
            deltas[n] = delta;
            coords[n] = (x, y);
            n += 1;

            if delta == 0.0 {
                zeroes += 1;
                if zeroes > 2 {
                    return false;
                }
            } else if delta < min {
                min = delta;
            } else if delta > max {
                max = delta;
            }
        }
    }

    // If there are no both darker and brighter pixels among siblings, it's not anti-aliasing
    if min == 0.0 || max == 0.0 {
        return false;
    }

    // Pass 2: check cached deltas for min/max matches and test flat-region siblings
    // in either image (relaxed from both images — see deviation note above)
    for i in 0..n {
        let delta = deltas[i];
        if delta == min || delta == max {
            let (x, y) = coords[i];
            if has_many_siblings(img_a, x, y, width, height)
                || has_many_siblings(img_b, x, y, width, height)
            {
                return true;
            }
        }
    }
    false
}

/// Check if a pixel has 3+ adjacent pixels of the same colour.
/// Uses unchecked u32 reads from byte buffer for fast comparison.
#[inline]
fn has_many_siblings(img: &[u8], x1: usize, y1: usize, width: usize, height: usize) -> bool {
    let x0 = x1.saturating_sub(1);
    let y0 = y1.saturating_sub(1);
    let x2 = (x1 + 1).min(width - 1);
    let y2 = (y1 + 1).min(height - 1);
    let val = read_u32_ne(img, (y1 * width + x1) * 4);
    let mut zeroes: i32 = if x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2 { 1 } else { 0 };

    // Go through 8 adjacent pixels
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }
            if val == read_u32_ne(img, (y * width + x) * 4) {
                zeroes += 1;
            }
            if zeroes > 2 {
                return true;
            }
        }
    }
    false
}
