mod aa;
mod color;

use color::{color_delta, draw_gray_pixel, draw_pixel};
use aa::antialiased;
use rayon::prelude::*;

/// Public re-export of color_delta for testing (FMA canary, property tests).
pub fn color_delta_public(img1: &[u8], img2: &[u8], k: usize, m: usize, y_only: bool) -> f64 {
    color_delta(img1, img2, k, m, y_only)
}

/// Options for pixel comparison.
#[derive(Debug, Clone)]
pub struct Options {
    /// Matching threshold (0 to 1); smaller is more sensitive. Default: 0.1
    pub threshold: f64,
    /// Whether to skip anti-aliasing detection. Default: false
    pub include_aa: bool,
    /// Opacity of original image in diff output. Default: 0.1
    pub alpha: f64,
    /// Colour of anti-aliased pixels in diff output [R, G, B]. Default: [255, 255, 0]
    pub aa_color: [u8; 3],
    /// Colour of different pixels in diff output [R, G, B]. Default: [255, 0, 0]
    pub diff_color: [u8; 3],
    /// Alternative diff colour for dark-on-light differences [R, G, B]. Default: None (uses diff_color)
    pub diff_color_alt: Option<[u8; 3]>,
    /// Draw the diff over a transparent background (a mask). Default: false
    pub diff_mask: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            threshold: 0.1,
            include_aa: false,
            alpha: 0.1,
            aa_color: [255, 255, 0],
            diff_color: [255, 0, 0],
            diff_color_alt: None,
            diff_mask: false,
        }
    }
}

/// Errors that can occur during pixel comparison.
#[derive(Debug)]
pub enum PixelmatchError {
    /// Width * height overflows usize.
    DimensionOverflow,
    /// Buffer length does not match expected width * height * 4.
    BufferLengthMismatch { expected: usize, actual: usize },
    /// img1 and img2 have different lengths.
    ImageSizeMismatch { img1_len: usize, img2_len: usize },
    /// Output buffer length does not match img1 length.
    OutputSizeMismatch { img1_len: usize, output_len: usize },
}

impl std::fmt::Display for PixelmatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionOverflow => write!(f, "Width * height overflows addressable memory"),
            Self::BufferLengthMismatch { expected, actual } => {
                write!(f, "Image data size does not match width/height. Expecting {expected}. Got {actual}")
            }
            Self::ImageSizeMismatch { img1_len, img2_len } => {
                write!(f, "Image sizes do not match. Image 1 size: {img1_len}, image 2 size: {img2_len}")
            }
            Self::OutputSizeMismatch { img1_len, output_len } => {
                write!(
                    f,
                    "Output buffer size does not match image size. Image size: {img1_len}, output size: {output_len}"
                )
            }
        }
    }
}

impl std::error::Error for PixelmatchError {}

/// Read a u32 from a byte slice without alignment requirements.
#[inline(always)]
pub(crate) fn read_u32_ne(data: &[u8], i: usize) -> u32 {
    // SAFETY: caller must ensure i + 3 < data.len()
    unsafe {
        u32::from_ne_bytes([
            *data.get_unchecked(i),
            *data.get_unchecked(i + 1),
            *data.get_unchecked(i + 2),
            *data.get_unchecked(i + 3),
        ])
    }
}

/// Process a single row, returning the diff count (no output).
#[inline]
fn process_row_no_output(
    img1: &[u8],
    img2: &[u8],
    y: usize,
    w: usize,
    h: usize,
    max_delta: f64,
    include_aa: bool,
) -> u32 {
    let mut diff: u32 = 0;
    for x in 0..w {
        let pos = (y * w + x) * 4;

        let delta = if read_u32_ne(img1, pos) == read_u32_ne(img2, pos) {
            0.0
        } else {
            color_delta(img1, img2, pos, pos, false)
        };

        if delta.abs() > max_delta {
            if include_aa {
                diff += 1;
            } else if !antialiased(img1, x, y, w, h, img1, img2)
                && !antialiased(img2, x, y, w, h, img2, img1)
            {
                diff += 1;
            }
        }
    }
    diff
}

/// Process a single row with output writing.
#[inline]
fn process_row_with_output(
    img1: &[u8],
    img2: &[u8],
    out_row: &mut [u8],
    y: usize,
    w: usize,
    h: usize,
    max_delta: f64,
    options: &Options,
    aa_r: u8, aa_g: u8, aa_b: u8,
    diff_r: u8, diff_g: u8, diff_b: u8,
    alt_r: u8, alt_g: u8, alt_b: u8,
) -> u32 {
    let mut diff: u32 = 0;
    for x in 0..w {
        let pos = (y * w + x) * 4;
        let lpos = x * 4;

        let delta = if read_u32_ne(img1, pos) == read_u32_ne(img2, pos) {
            0.0
        } else {
            color_delta(img1, img2, pos, pos, false)
        };

        if delta.abs() > max_delta {
            let is_excluded_aa = !options.include_aa
                && (antialiased(img1, x, y, w, h, img1, img2)
                    || antialiased(img2, x, y, w, h, img2, img1));

            if is_excluded_aa {
                if !options.diff_mask {
                    draw_pixel(out_row, lpos, aa_r, aa_g, aa_b);
                }
            } else {
                if delta < 0.0 {
                    draw_pixel(out_row, lpos, alt_r, alt_g, alt_b);
                } else {
                    draw_pixel(out_row, lpos, diff_r, diff_g, diff_b);
                }
                diff += 1;
            }
        } else if !options.diff_mask {
            draw_gray_pixel_local(img1, pos, options.alpha, out_row, lpos);
        }
    }
    diff
}

/// Compare two equally sized images, pixel by pixel.
///
/// Returns the number of mismatched pixels.
pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    output: Option<&mut [u8]>,
    width: u32,
    height: u32,
    options: &Options,
) -> Result<u32, PixelmatchError> {
    let len = (width as usize)
        .checked_mul(height as usize)
        .ok_or(PixelmatchError::DimensionOverflow)?;
    let expected_bytes = len
        .checked_mul(4)
        .ok_or(PixelmatchError::DimensionOverflow)?;

    if img1.len() != img2.len() {
        return Err(PixelmatchError::ImageSizeMismatch {
            img1_len: img1.len(),
            img2_len: img2.len(),
        });
    }

    if let Some(ref out) = output {
        if out.len() != img1.len() {
            return Err(PixelmatchError::OutputSizeMismatch {
                img1_len: img1.len(),
                output_len: out.len(),
            });
        }
    }

    if img1.len() != expected_bytes {
        return Err(PixelmatchError::BufferLengthMismatch {
            expected: expected_bytes,
            actual: img1.len(),
        });
    }

    let w = width as usize;
    let h = height as usize;

    // Check if images are identical (memcmp — auto-vectorised by LLVM)
    if img1 == img2 {
        if let Some(out) = output {
            if !options.diff_mask {
                for i in 0..len {
                    draw_gray_pixel(img1, i * 4, options.alpha, out);
                }
            }
        }
        return Ok(0);
    }

    let max_delta = 35215.0 * options.threshold * options.threshold;
    let [aa_r, aa_g, aa_b] = options.aa_color;
    let [diff_r, diff_g, diff_b] = options.diff_color;
    let [alt_r, alt_g, alt_b] = options.diff_color_alt.unwrap_or(options.diff_color);

    match output {
        Some(out) => {
            let row_bytes = w * 4;
            let diff: u32 = out
                .par_chunks_mut(row_bytes)
                .with_min_len(4)
                .enumerate()
                .map(|(y, out_row)| {
                    process_row_with_output(
                        img1, img2, out_row, y, w, h, max_delta, options,
                        aa_r, aa_g, aa_b, diff_r, diff_g, diff_b, alt_r, alt_g, alt_b,
                    )
                })
                .sum();
            Ok(diff)
        }
        None => {
            let diff: u32 = (0..h)
                .into_par_iter()
                .with_min_len(4)
                .map(|y| {
                    process_row_no_output(img1, img2, y, w, h, max_delta, options.include_aa)
                })
                .sum();
            Ok(diff)
        }
    }
}

/// Draw a grayscale pixel into a row-local output slice.
/// Reads from `img` at global `src_pos`, writes to `out` at local `dst_pos`.
#[inline(always)]
fn draw_gray_pixel_local(img: &[u8], src_pos: usize, alpha: f64, out: &mut [u8], dst_pos: usize) {
    unsafe {
        let r = *img.get_unchecked(src_pos) as f64;
        let g = *img.get_unchecked(src_pos + 1) as f64;
        let b = *img.get_unchecked(src_pos + 2) as f64;
        let a = *img.get_unchecked(src_pos + 3) as f64;
        let val = (255.0 + (r * 0.29889531 + g * 0.58662247 + b * 0.11448223 - 255.0) * alpha * a / 255.0) as u8;
        *out.get_unchecked_mut(dst_pos) = val;
        *out.get_unchecked_mut(dst_pos + 1) = val;
        *out.get_unchecked_mut(dst_pos + 2) = val;
        *out.get_unchecked_mut(dst_pos + 3) = 255;
    }
}

#[cfg(feature = "napi")]
mod napi_bindings;

#[cfg(feature = "wasm")]
mod wasm_bindings;
