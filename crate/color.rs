/// Read RGBA channels from a byte slice at the given byte offset.
///
/// # Safety
/// Caller must ensure `off + 3 < data.len()`.
#[inline(always)]
unsafe fn rgba_at_unchecked(data: &[u8], off: usize) -> (f64, f64, f64, f64) {
    (
        *data.get_unchecked(off) as f64,
        *data.get_unchecked(off + 1) as f64,
        *data.get_unchecked(off + 2) as f64,
        *data.get_unchecked(off + 3) as f64,
    )
}

/// Calculate colour difference according to the paper "Measuring perceived colour difference
/// using YIQ NTSC transmission colour space in mobile applications" by Y. Kotsarenko and F. Ramos.
///
/// `k` and `m` are byte offsets into the image data (multiples of 4).
/// Caller must ensure `k + 3 < img1.len()` and `m + 3 < img2.len()`.
#[inline]
pub fn color_delta(img1: &[u8], img2: &[u8], k: usize, m: usize, y_only: bool) -> f64 {
    debug_assert!(k + 3 < img1.len(), "k out of bounds");
    debug_assert!(m + 3 < img2.len(), "m out of bounds");

    // SAFETY: pixelmatch() validates buffer sizes before calling this function.
    // k and m are always `(y * width + x) * 4` where x < width and y < height,
    // so k + 3 and m + 3 are always within bounds.
    unsafe { color_delta_inner(img1, img2, k, m, y_only) }
}

#[inline(always)]
unsafe fn color_delta_inner(img1: &[u8], img2: &[u8], k: usize, m: usize, y_only: bool) -> f64 {
    let (r1, g1, b1, a1) = rgba_at_unchecked(img1, k);
    let (r2, g2, b2, a2) = rgba_at_unchecked(img2, m);

    let mut dr = r1 - r2;
    let mut dg = g1 - g2;
    let mut db = b1 - b2;
    let da = a1 - a2;

    if dr == 0.0 && dg == 0.0 && db == 0.0 && da == 0.0 {
        return 0.0;
    }

    if a1 < 255.0 || a2 < 255.0 {
        // Blend pixels with background.
        // The background pattern uses k (byte offset) to create a checkerboard-like dither.
        let rb = 48.0 + 159.0 * ((k % 2) as f64);
        let gb = 48.0 + 159.0 * (((k as f64 / 1.618033988749895_f64) as i64 % 2) as f64);
        let bb = 48.0 + 159.0 * (((k as f64 / 2.618033988749895_f64) as i64 % 2) as f64);
        dr = (r1 * a1 - r2 * a2 - rb * da) / 255.0;
        dg = (g1 * a1 - g2 * a2 - gb * da) / 255.0;
        db = (b1 * a1 - b2 * a2 - bb * da) / 255.0;
    }

    let y = dr * 0.29889531 + dg * 0.58662247 + db * 0.11448223;

    if y_only {
        return y; // brightness difference only
    }

    let i = dr * 0.59597799 - dg * 0.27417610 - db * 0.32180189;
    let q = dr * 0.21147017 - dg * 0.52261711 + db * 0.31114694;

    let delta = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;

    // Encode whether the pixel lightens or darkens in the sign
    if y > 0.0 { -delta } else { delta }
}

/// Draw a pixel with the given colour at the specified byte offset.
#[inline(always)]
pub fn draw_pixel(output: &mut [u8], pos: usize, r: u8, g: u8, b: u8) {
    // SAFETY: pixelmatch() validates buffer sizes; pos is always within bounds.
    unsafe {
        *output.get_unchecked_mut(pos) = r;
        *output.get_unchecked_mut(pos + 1) = g;
        *output.get_unchecked_mut(pos + 2) = b;
        *output.get_unchecked_mut(pos + 3) = 255;
    }
}

/// Draw a grayscale pixel blended with white at the specified byte offset.
#[inline(always)]
pub fn draw_gray_pixel(img: &[u8], i: usize, alpha: f64, output: &mut [u8]) {
    // SAFETY: pixelmatch() validates buffer sizes; i is always within bounds.
    unsafe {
        let (r, g, b, a) = rgba_at_unchecked(img, i);
        let val = 255.0 + (r * 0.29889531 + g * 0.58662247 + b * 0.11448223 - 255.0) * alpha * a / 255.0;
        let val_u8 = val as u8;
        *output.get_unchecked_mut(i) = val_u8;
        *output.get_unchecked_mut(i + 1) = val_u8;
        *output.get_unchecked_mut(i + 2) = val_u8;
        *output.get_unchecked_mut(i + 3) = 255;
    }
}
