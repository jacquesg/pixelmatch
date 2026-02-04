use std::path::PathBuf;

use pixelmatch::{pixelmatch, Options};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test").join("fixtures")
}

fn read_image(name: &str) -> (Vec<u8>, u32, u32) {
    let path = fixtures_dir().join(format!("{name}.png"));
    let file = std::fs::File::open(&path).unwrap_or_else(|e| panic!("Failed to open {}: {e}", path.display()));
    let mut decoder = png::Decoder::new(file);
    // Expand all colour types to RGBA, matching pngjs behaviour
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());

    // Verify we got RGBA output
    assert_eq!(
        info.color_type,
        png::ColorType::Rgba,
        "Expected RGBA output for {name}.png, got {:?}",
        info.color_type
    );
    assert_eq!(info.bit_depth, png::BitDepth::Eight, "Expected 8-bit depth for {name}.png");

    (buf, info.width, info.height)
}

fn diff_test(img1_name: &str, img2_name: &str, diff_name: &str, options: Options, expected_mismatch: u32) {
    let (img1, width, height) = read_image(img1_name);
    let (img2, _, _) = read_image(img2_name);

    // Test with output
    let mut diff = vec![0u8; img1.len()];
    let result = pixelmatch(&img1, &img2, Some(&mut diff), width, height, &options)
        .expect("pixelmatch should not error");

    // Compare diff image to reference
    let (expected_diff, _, _) = read_image(diff_name);
    assert_eq!(
        diff, expected_diff,
        "diff image mismatch for {img1_name} vs {img2_name} (diff: {diff_name})"
    );

    assert_eq!(
        result.diff_count, expected_mismatch,
        "number of mismatched pixels for {img1_name} vs {img2_name}"
    );

    // Test without output — should produce the same count and aa_count
    let result2 = pixelmatch(&img1, &img2, None, width, height, &options)
        .expect("pixelmatch should not error");

    assert_eq!(
        result.diff_count, result2.diff_count,
        "diff_count should be the same with and without output for {img1_name} vs {img2_name}"
    );
    assert_eq!(
        result.aa_count, result2.aa_count,
        "aa_count should be the same with and without output for {img1_name} vs {img2_name}"
    );
    assert_eq!(
        result.identical, result2.identical,
        "identical should be the same with and without output for {img1_name} vs {img2_name}"
    );
}

#[test]
fn test_1a_1b_threshold_005() {
    diff_test("1a", "1b", "1diff", Options { threshold: 0.05, ..Default::default() }, 109);
}

#[test]
fn test_1a_1b_default_threshold() {
    diff_test("1a", "1b", "1diffdefaultthreshold", Options::default(), 81);
}

#[test]
fn test_1a_1b_diffmask() {
    diff_test(
        "1a",
        "1b",
        "1diffmask",
        Options { threshold: 0.05, diff_mask: true, ..Default::default() },
        109,
    );
}

#[test]
fn test_1a_1a_empty_diffmask() {
    diff_test(
        "1a",
        "1a",
        "1emptydiffmask",
        Options { threshold: 0.0, diff_mask: true, ..Default::default() },
        0,
    );
}

#[test]
fn test_2a_2b() {
    diff_test(
        "2a",
        "2b",
        "2diff",
        Options {
            threshold: 0.05,
            alpha: 0.5,
            aa_color: [0, 192, 0],
            diff_color: [255, 0, 255],
            ..Default::default()
        },
        9579,
    );
}

#[test]
fn test_3a_3b() {
    diff_test("3a", "3b", "3diff", Options { threshold: 0.05, ..Default::default() }, 112);
}

#[test]
fn test_4a_4b() {
    diff_test("4a", "4b", "4diff", Options { threshold: 0.05, ..Default::default() }, 31739);
}

#[test]
fn test_5a_5b() {
    diff_test("5a", "5b", "5diff", Options { threshold: 0.05, ..Default::default() }, 3);
}

#[test]
fn test_6a_6b() {
    diff_test("6a", "6b", "6diff", Options { threshold: 0.05, ..Default::default() }, 44);
}

#[test]
fn test_6a_6a_empty() {
    diff_test("6a", "6a", "6empty", Options { threshold: 0.0, ..Default::default() }, 0);
}

#[test]
fn test_7a_7b_diff_color_alt() {
    diff_test(
        "7a",
        "7b",
        "7diff",
        Options { diff_color_alt: Some([0, 255, 0]), ..Default::default() },
        687,
    );
}

#[test]
fn test_8a_5b() {
    diff_test("8a", "5b", "8diff", Options { threshold: 0.05, ..Default::default() }, 32896);
}

// --- Identical image tests ---

#[test]
fn test_identical_images() {
    let (img1, width, height) = read_image("1a");
    let result = pixelmatch(&img1, &img1, None, width, height, &Default::default())
        .expect("pixelmatch should not error");
    assert!(result.identical, "identical images should have identical = true");
    assert_eq!(result.diff_count, 0);
    assert_eq!(result.aa_count, 0);
}

#[test]
fn test_different_images_not_identical() {
    let (img1, width, height) = read_image("1a");
    let (img2, _, _) = read_image("1b");
    let result = pixelmatch(&img1, &img2, None, width, height, &Options { threshold: 0.05, ..Default::default() })
        .expect("pixelmatch should not error");
    assert!(!result.identical, "different images should have identical = false");
}

// --- AA count tests ---

#[test]
fn test_aa_count_with_detection() {
    let (img1, width, height) = read_image("1a");
    let (img2, _, _) = read_image("1b");

    // With AA detection (default)
    let with_aa = pixelmatch(&img1, &img2, None, width, height, &Options { threshold: 0.05, ..Default::default() })
        .expect("pixelmatch should not error");

    // Without AA detection
    let without_aa = pixelmatch(
        &img1,
        &img2,
        None,
        width,
        height,
        &Options { threshold: 0.05, detect_anti_aliasing: false, ..Default::default() },
    )
    .expect("pixelmatch should not error");

    assert!(with_aa.aa_count > 0, "aa_count should be > 0 when AA detection is enabled");
    assert_eq!(without_aa.aa_count, 0, "aa_count should be 0 when AA detection is disabled");
    assert_eq!(
        without_aa.diff_count,
        with_aa.diff_count + with_aa.aa_count,
        "without AA: diffCount should equal with AA: diffCount + aaCount"
    );
}

// --- Error handling tests ---

#[test]
fn test_image_size_mismatch() {
    let result = pixelmatch(&[0u8; 8], &[0u8; 9], None, 2, 1, &Default::default());
    assert!(result.is_err());
}

#[test]
fn test_image_data_size_mismatch() {
    let result = pixelmatch(&[0u8; 9], &[0u8; 9], None, 2, 1, &Default::default());
    assert!(result.is_err());
}

// --- Boundary dimension tests ---

#[test]
fn test_1x1_identical() {
    let img = [255u8, 0, 0, 255]; // red pixel
    let result = pixelmatch(&img, &img, None, 1, 1, &Default::default()).unwrap();
    assert_eq!(result.diff_count, 0);
    assert!(result.identical);
}

#[test]
fn test_1x1_different() {
    let img1 = [255u8, 0, 0, 255]; // red
    let img2 = [0u8, 0, 255, 255]; // blue
    let result = pixelmatch(&img1, &img2, None, 1, 1, &Options { threshold: 0.0, ..Default::default() }).unwrap();
    assert_eq!(result.diff_count, 1);
    assert!(!result.identical);
}

#[test]
fn test_1xn_image() {
    // 1x3 image — exercises edge clamping in antialiased/has_many_siblings
    let img1 = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
    let img2 = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
    let result = pixelmatch(&img1, &img2, None, 1, 3, &Default::default()).unwrap();
    assert_eq!(result.diff_count, 0);
    assert!(result.identical);
}

#[test]
fn test_nx1_image() {
    // 3x1 image
    let img1 = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
    let img2 = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
    let result = pixelmatch(&img1, &img2, None, 3, 1, &Default::default()).unwrap();
    assert_eq!(result.diff_count, 0);
    assert!(result.identical);
}

// --- Property tests ---

#[test]
fn test_color_delta_same_pixel_is_zero() {
    // colorDelta(img, img, k, k, false) == 0 for any pixel
    use pixelmatch::color_delta_public;
    let img = [128u8, 64, 32, 255, 0, 128, 255, 200];
    assert_eq!(color_delta_public(&img, &img, 0, 0, false), 0.0);
    assert_eq!(color_delta_public(&img, &img, 4, 4, false), 0.0);
}

// --- FMA canary test ---
// This test uses hand-computed values for a semi-transparent pixel pair.
// If FMA contraction is accidentally enabled, the result will differ by 1-2 ULP
// and this test will catch it.
#[test]
fn test_semi_transparent_color_delta() {
    use pixelmatch::color_delta_public;
    // Two semi-transparent pixels at byte offset 0
    let img1 = [200u8, 100, 50, 128];
    let img2 = [100u8, 200, 150, 200];

    let delta = color_delta_public(&img1, &img2, 0, 0, false);
    // Compute expected value using the JS algorithm exactly:
    // a1=128, a2=200, k=0 => k%2=0 => rb=48, gb=48+159*(floor(0/1.618..)%2)=48, bb=48+159*(floor(0/2.618..)%2)=48
    // dr = (200*128 - 100*200 - 48*(128-200)) / 255 = (25600 - 20000 - 48*(-72)) / 255 = (25600 - 20000 + 3456) / 255 = 9056/255
    // dg = (100*128 - 200*200 - 48*(-72)) / 255 = (12800 - 40000 + 3456) / 255 = -23744/255
    // db = (50*128 - 150*200 - 48*(-72)) / 255 = (6400 - 30000 + 3456) / 255 = -20144/255
    let dr = 9056.0_f64 / 255.0;
    let dg = -23744.0_f64 / 255.0;
    let db = -20144.0_f64 / 255.0;

    let y = dr * 0.29889531 + dg * 0.58662247 + db * 0.11448223;
    let i = dr * 0.59597799 - dg * 0.27417610 - db * 0.32180189;
    let q = dr * 0.21147017 - dg * 0.52261711 + db * 0.31114694;
    let expected = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;
    // y is negative here, so delta should be positive (y > 0 => -delta, else delta)
    let expected_signed = if y > 0.0 { -expected } else { expected };

    // Allow no tolerance — must be bit-exact with JS
    assert_eq!(delta, expected_signed, "FMA canary: semi-transparent colorDelta must match JS exactly");
}
