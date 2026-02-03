use std::path::PathBuf;
use std::time::Instant;

use pixelmatch::{pixelmatch, Options};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test").join("fixtures")
}

fn read_image(name: &str) -> (Vec<u8>, u32, u32) {
    let path = fixtures_dir().join(format!("{name}.png"));
    let file = std::fs::File::open(&path).unwrap();
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    (buf, info.width, info.height)
}

fn main() {
    let data: Vec<_> = (1..=7)
        .map(|i| {
            let (a, w, h) = read_image(&format!("{i}a"));
            let (b, _, _) = read_image(&format!("{i}b"));
            (a, b, w, h)
        })
        .collect();

    let options = Options::default();

    // Warmup
    for (img1, img2, w, h) in &data {
        pixelmatch(img1, img2, None, *w, *h, &options).unwrap();
    }

    // Per-image timing
    for (idx, (img1, img2, w, h)) in data.iter().enumerate() {
        let start = Instant::now();
        let mut sum = 0u32;
        for _ in 0..100 {
            sum += pixelmatch(img1, img2, None, *w, *h, &options).unwrap();
        }
        let elapsed = start.elapsed();
        println!("  image {}: {:>8.1?}  ({}x{}, sum={})", idx + 1, elapsed, w, h, sum);
    }

    // Total timing
    let start = Instant::now();
    let mut sum: u32 = 0;
    for _ in 0..100 {
        for (img1, img2, w, h) in &data {
            sum += pixelmatch(img1, img2, None, *w, *h, &options).unwrap();
        }
    }
    let elapsed = start.elapsed();

    println!("match: {elapsed:.1?}");
    println!("{sum}");
}
