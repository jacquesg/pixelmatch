use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{Options, PixelmatchError};

#[napi(object)]
pub struct PixelmatchOptions {
    pub threshold: Option<f64>,
    pub detect_anti_aliasing: Option<bool>,
    pub alpha: Option<f64>,
    pub aa_color: Option<Vec<u32>>,
    pub diff_color: Option<Vec<u32>>,
    pub diff_color_alt: Option<Vec<u32>>,
    pub diff_mask: Option<bool>,
}

#[napi(object)]
pub struct NapiMatchResult {
    pub diff_count: u32,
    pub aa_count: u32,
    pub identical: bool,
}

fn convert_options(opts: Option<PixelmatchOptions>) -> Options {
    let mut options = Options::default();
    if let Some(o) = opts {
        if let Some(t) = o.threshold {
            options.threshold = t;
        }
        if let Some(aa) = o.detect_anti_aliasing {
            options.detect_anti_aliasing = aa;
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
/// Returns a NapiMatchResult with diff_count, aa_count, and identical fields.
#[napi]
pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    mut output: Buffer,
    width: u32,
    height: u32,
    options: Option<PixelmatchOptions>,
) -> Result<NapiMatchResult> {
    let opts = convert_options(options);
    let result = crate::pixelmatch(img1, img2, Some(output.as_mut()), width, height, &opts).map_err(map_error)?;
    Ok(NapiMatchResult {
        diff_count: result.diff_count,
        aa_count: result.aa_count,
        identical: result.identical,
    })
}

/// Compare two images pixel by pixel, returning only the match result (no diff output).
#[napi]
pub fn pixelmatch_count(
    img1: &[u8],
    img2: &[u8],
    width: u32,
    height: u32,
    options: Option<PixelmatchOptions>,
) -> Result<NapiMatchResult> {
    let opts = convert_options(options);
    let result = crate::pixelmatch(img1, img2, None, width, height, &opts).map_err(map_error)?;
    Ok(NapiMatchResult {
        diff_count: result.diff_count,
        aa_count: result.aa_count,
        identical: result.identical,
    })
}
