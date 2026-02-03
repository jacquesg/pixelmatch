use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{Options, PixelmatchError};

#[napi(object)]
pub struct PixelmatchOptions {
    pub threshold: Option<f64>,
    pub include_aa: Option<bool>,
    pub alpha: Option<f64>,
    pub aa_color: Option<Vec<u32>>,
    pub diff_color: Option<Vec<u32>>,
    pub diff_color_alt: Option<Vec<u32>>,
    pub diff_mask: Option<bool>,
}

fn convert_options(opts: Option<PixelmatchOptions>) -> Options {
    let mut options = Options::default();
    if let Some(o) = opts {
        if let Some(t) = o.threshold {
            options.threshold = t;
        }
        if let Some(aa) = o.include_aa {
            options.include_aa = aa;
        }
        if let Some(a) = o.alpha {
            options.alpha = a;
        }
        if let Some(ref c) = o.aa_color {
            if c.len() >= 3 {
                options.aa_color = [c[0] as u8, c[1] as u8, c[2] as u8];
            }
        }
        if let Some(ref c) = o.diff_color {
            if c.len() >= 3 {
                options.diff_color = [c[0] as u8, c[1] as u8, c[2] as u8];
            }
        }
        if let Some(ref c) = o.diff_color_alt {
            if c.len() >= 3 {
                options.diff_color_alt = Some([c[0] as u8, c[1] as u8, c[2] as u8]);
            }
        }
        if let Some(m) = o.diff_mask {
            options.diff_mask = m;
        }
    }
    options
}

fn map_error(e: PixelmatchError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

/// Compare two images pixel by pixel, writing the diff to the output buffer.
/// Returns a new Buffer containing the diff image and the mismatch count.
#[napi]
pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    mut output: Buffer,
    width: u32,
    height: u32,
    options: Option<PixelmatchOptions>,
) -> Result<u32> {
    let opts = convert_options(options);
    crate::pixelmatch(img1, img2, Some(output.as_mut()), width, height, &opts).map_err(map_error)
}

/// Compare two images pixel by pixel, returning only the mismatch count (no diff output).
#[napi]
pub fn pixelmatch_count(
    img1: &[u8],
    img2: &[u8],
    width: u32,
    height: u32,
    options: Option<PixelmatchOptions>,
) -> Result<u32> {
    let opts = convert_options(options);
    crate::pixelmatch(img1, img2, None, width, height, &opts).map_err(map_error)
}
