use wasm_bindgen::prelude::*;

use crate::Options;

/// Result of a WASM pixel comparison.
#[wasm_bindgen]
pub struct WasmMatchResult {
    diff_count: u32,
    aa_count: u32,
    identical: bool,
}

#[wasm_bindgen]
impl WasmMatchResult {
    #[wasm_bindgen(getter)]
    pub fn diff_count(&self) -> u32 {
        self.diff_count
    }

    #[wasm_bindgen(getter)]
    pub fn aa_count(&self) -> u32 {
        self.aa_count
    }

    #[wasm_bindgen(getter)]
    pub fn identical(&self) -> bool {
        self.identical
    }
}

/// Compare two images pixel by pixel (WASM entry point).
///
/// Uses flattened parameters instead of an options object — the thin JS wrapper
/// in `wasm/index.js` destructures the ergonomic options-object API into these.
#[wasm_bindgen]
pub fn pixelmatch_wasm(
    img1: &[u8],
    img2: &[u8],
    output: &mut [u8],
    width: u32,
    height: u32,
    threshold: f64,
    detect_anti_aliasing: bool,
    alpha: f64,
    aa_r: u8,
    aa_g: u8,
    aa_b: u8,
    diff_r: u8,
    diff_g: u8,
    diff_b: u8,
    has_alt: bool,
    alt_r: u8,
    alt_g: u8,
    alt_b: u8,
    diff_mask: bool,
) -> Result<WasmMatchResult, JsError> {
    let options = Options {
        threshold,
        detect_anti_aliasing,
        alpha,
        aa_color: [aa_r, aa_g, aa_b],
        diff_color: [diff_r, diff_g, diff_b],
        diff_color_alt: if has_alt { Some([alt_r, alt_g, alt_b]) } else { None },
        diff_mask,
    };
    let result = crate::pixelmatch(img1, img2, Some(output), width, height, &options)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(WasmMatchResult {
        diff_count: result.diff_count,
        aa_count: result.aa_count,
        identical: result.identical,
    })
}

/// Compare two images pixel by pixel without diff output (WASM entry point).
#[wasm_bindgen]
pub fn pixelmatch_wasm_count(
    img1: &[u8],
    img2: &[u8],
    width: u32,
    height: u32,
    threshold: f64,
    detect_anti_aliasing: bool,
    alpha: f64,
    aa_r: u8,
    aa_g: u8,
    aa_b: u8,
    diff_r: u8,
    diff_g: u8,
    diff_b: u8,
    has_alt: bool,
    alt_r: u8,
    alt_g: u8,
    alt_b: u8,
    diff_mask: bool,
) -> Result<WasmMatchResult, JsError> {
    let options = Options {
        threshold,
        detect_anti_aliasing,
        alpha,
        aa_color: [aa_r, aa_g, aa_b],
        diff_color: [diff_r, diff_g, diff_b],
        diff_color_alt: if has_alt { Some([alt_r, alt_g, alt_b]) } else { None },
        diff_mask,
    };
    let result = crate::pixelmatch(img1, img2, None, width, height, &options)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(WasmMatchResult {
        diff_count: result.diff_count,
        aa_count: result.aa_count,
        identical: result.identical,
    })
}
