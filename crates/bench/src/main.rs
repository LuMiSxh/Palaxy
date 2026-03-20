//! Palaxy performance benchmark.
//!
//! Generates synthetic test images and measures encode/decode/pipeline performance.
//! Run with: `cargo run -p bench --release`

use common::prelude::*;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use packager::{
    process_and_write_streaming, process_images_to_memory,
    process_images_to_memory_with_options, Cbz, Generator,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;

// --- Size tiers matching constants.rs thresholds ---

struct SizeTier {
    name: &'static str,
    width: u32,
    height: u32,
}

const TIERS: &[SizeTier] = &[
    SizeTier { name: "tiny", width: 500, height: 600 },
    SizeTier { name: "small", width: 800, height: 1200 },
    SizeTier { name: "medium", width: 1400, height: 2100 },
    SizeTier { name: "large", width: 2000, height: 3000 },
    SizeTier { name: "huge", width: 2500, height: 4000 },
];

const ITERATIONS: usize = 3;
const PIPELINE_IMAGE_COUNT: usize = 200;

fn main() {
    println!("Palaxy Performance Benchmark");
    println!("============================\n");

    let num_threads = rayon::current_num_threads();
    println!("Rayon threads: {}", num_threads);
    println!("Iterations per test: {}\n", ITERATIONS);

    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Generate test images
    println!("Generating synthetic test images...");
    let test_images = generate_all_test_images(&tmp);
    println!("Done.\n");

    // Per-image encode benchmarks
    run_encode_benchmarks(&test_images);

    // Full pipeline benchmark (200 medium images -> CBZ)
    println!("\n");
    run_pipeline_benchmark(&tmp, &test_images);
}

struct TestImage {
    tier_name: &'static str,
    variant: &'static str,
    path: PathBuf,
    input_size: u64,
    _width: u32,
    _height: u32,
}

fn generate_all_test_images(tmp: &TempDir) -> Vec<TestImage> {
    let mut images = Vec::new();

    for tier in TIERS {
        // Grayscale variant
        let gray_path = tmp.path().join(format!("{}_{}.jpg", tier.name, "gray"));
        let gray_img = generate_grayscale_image(tier.width, tier.height);
        gray_img
            .save(&gray_path)
            .expect("Failed to save gray image");
        let gray_size = std::fs::metadata(&gray_path).unwrap().len();
        images.push(TestImage {
            tier_name: tier.name,
            variant: "gray",
            path: gray_path,
            input_size: gray_size,
            _width: tier.width,
            _height: tier.height,
        });

        // Color variant
        let color_path = tmp.path().join(format!("{}_{}.jpg", tier.name, "color"));
        let color_img = generate_color_image(tier.width, tier.height);
        color_img
            .save(&color_path)
            .expect("Failed to save color image");
        let color_size = std::fs::metadata(&color_path).unwrap().len();
        images.push(TestImage {
            tier_name: tier.name,
            variant: "color",
            path: color_path,
            input_size: color_size,
            _width: tier.width,
            _height: tier.height,
        });
    }

    images
}

/// Generates a grayscale manga-like image with panels and lines.
fn generate_grayscale_image(width: u32, height: u32) -> DynamicImage {
    let mut img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));

    // Simple deterministic seed based on dimensions
    let mut seed: u64 = (width as u64) * 31 + (height as u64) * 17;

    // Draw random dark rectangles (simulating manga panels)
    for _ in 0..12 {
        seed = lcg(seed);
        let x1 = (seed % width as u64) as u32;
        seed = lcg(seed);
        let y1 = (seed % height as u64) as u32;
        seed = lcg(seed);
        let w = (seed % (width as u64 / 3)).max(20) as u32;
        seed = lcg(seed);
        let h = (seed % (height as u64 / 3)).max(20) as u32;
        let gray = ((seed % 80) + 10) as u8;

        for y in y1..y1.saturating_add(h).min(height) {
            for x in x1..x1.saturating_add(w).min(width) {
                img.put_pixel(x, y, Rgb([gray, gray, gray]));
            }
        }
    }

    // Draw horizontal lines (simulating text/screentone)
    for i in 0..30 {
        seed = lcg(seed);
        let y = (seed % height as u64) as u32;
        let thickness = ((seed % 3) + 1) as u32;
        for dy in 0..thickness {
            let yy = y.saturating_add(dy).min(height - 1);
            for x in 0..width {
                if (x + i) % 3 != 0 {
                    img.put_pixel(x, yy, Rgb([30, 30, 30]));
                }
            }
        }
    }

    DynamicImage::ImageRgb8(img)
}

/// Generates a color image with dense colored content (comparable complexity to grayscale).
/// Fills the entire image with varied color regions, gradients, and detail patterns
/// to produce a realistic encoding workload.
fn generate_color_image(width: u32, height: u32) -> DynamicImage {
    let mut img: RgbImage = ImageBuffer::new(width, height);

    let mut seed: u64 = (width as u64) * 53 + (height as u64) * 37;

    // Fill background with a color gradient (not uniform)
    for y in 0..height {
        for x in 0..width {
            let r = ((x * 200 / width) + 30) as u8;
            let g = ((y * 180 / height) + 40) as u8;
            let b = (((x + y) * 150 / (width + height)) + 50) as u8;
            img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    // Overlay many colored rectangles (dense coverage)
    for _ in 0..40 {
        seed = lcg(seed);
        let x1 = (seed % width as u64) as u32;
        seed = lcg(seed);
        let y1 = (seed % height as u64) as u32;
        seed = lcg(seed);
        let w = (seed % (width as u64 / 3)).max(20) as u32;
        seed = lcg(seed);
        let h = (seed % (height as u64 / 3)).max(20) as u32;
        seed = lcg(seed);
        let r = (seed % 256) as u8;
        seed = lcg(seed);
        let g = (seed % 256) as u8;
        seed = lcg(seed);
        let b = (seed % 256) as u8;

        for y in y1..y1.saturating_add(h).min(height) {
            for x in x1..x1.saturating_add(w).min(width) {
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
    }

    // Add colored lines (simulating detail/texture)
    for i in 0..30 {
        seed = lcg(seed);
        let y = (seed % height as u64) as u32;
        seed = lcg(seed);
        let r = (seed % 200) as u8;
        seed = lcg(seed);
        let g = (seed % 200) as u8;
        seed = lcg(seed);
        let b = (seed % 200) as u8;
        let thickness = ((seed % 3) + 1) as u32;
        for dy in 0..thickness {
            let yy = y.saturating_add(dy).min(height - 1);
            for x in 0..width {
                if (x + i) % 3 != 0 {
                    img.put_pixel(x, yy, Rgb([r, g, b]));
                }
            }
        }
    }

    DynamicImage::ImageRgb8(img)
}

/// Simple LCG for deterministic pseudo-random numbers.
#[inline]
fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

fn run_encode_benchmarks(test_images: &[TestImage]) {
    // AVIF benchmarks
    println!("=== AVIF Encode Benchmarks ===");
    print_header();

    for img in test_images {
        let results = bench_format(&img.path, ImageOutputFormat::Avif, ITERATIONS);
        print_row(img, &results);
    }

    println!();

    // WebP benchmarks
    println!("=== WebP Encode Benchmarks ===");
    print_header();

    for img in test_images {
        let results = bench_format(&img.path, ImageOutputFormat::WebP, ITERATIONS);
        print_row(img, &results);
    }
}

struct BenchResult {
    times: Vec<Duration>,
    output_size: usize,
}

impl BenchResult {
    fn mean_ms(&self) -> f64 {
        let total: f64 = self.times.iter().map(|t| t.as_secs_f64() * 1000.0).sum();
        total / self.times.len() as f64
    }

    fn min_ms(&self) -> f64 {
        self.times
            .iter()
            .map(|t| t.as_secs_f64() * 1000.0)
            .fold(f64::MAX, f64::min)
    }

    fn max_ms(&self) -> f64 {
        self.times
            .iter()
            .map(|t| t.as_secs_f64() * 1000.0)
            .fold(0.0f64, f64::max)
    }
}

fn bench_format_with_downscale(
    path: &Path,
    format: ImageOutputFormat,
    iterations: usize,
    max_dimension: Option<u32>,
) -> BenchResult {
    let paths = [path.to_path_buf()];
    let mut times = Vec::with_capacity(iterations);
    let mut output_size = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let pages =
            process_images_to_memory_with_options(&paths, format, None::<&fn()>, None, max_dimension)
                .expect("Encode failed");
        let elapsed = start.elapsed();
        output_size = pages[0].data.len();
        times.push(elapsed);
    }

    BenchResult { times, output_size }
}

fn bench_format(path: &Path, format: ImageOutputFormat, iterations: usize) -> BenchResult {
    let paths = [path.to_path_buf()];
    let mut times = Vec::with_capacity(iterations);
    let mut output_size = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let pages = process_images_to_memory(&paths, format, None::<&fn()>, None)
            .expect("Encode failed");
        let elapsed = start.elapsed();
        output_size = pages[0].data.len();
        times.push(elapsed);
    }

    BenchResult { times, output_size }
}

fn print_header() {
    println!(
        "{:<10}| {:<7}| {:>10} | {:>10} | {:>10} | {:>10} | {:>10}",
        "Tier", "Type", "Avg (ms)", "Min (ms)", "Max (ms)", "Out (KB)", "Ratio"
    );
    println!("{}", "-".repeat(82));
}

fn print_row(img: &TestImage, result: &BenchResult) {
    let ratio = img.input_size as f64 / result.output_size as f64;
    println!(
        "{:<10}| {:<7}| {:>10.1} | {:>10.1} | {:>10.1} | {:>10.1} | {:>9.1}x",
        img.tier_name,
        img.variant,
        result.mean_ms(),
        result.min_ms(),
        result.max_ms(),
        result.output_size as f64 / 1024.0,
        ratio,
    );
}

fn run_pipeline_benchmark(tmp: &TempDir, test_images: &[TestImage]) {
    println!("=== Pipeline Benchmark ({} medium images -> AVIF -> CBZ) ===", PIPELINE_IMAGE_COUNT);

    // Generate 200 medium-tier grayscale images
    let pipeline_dir = tmp.path().join("pipeline");
    std::fs::create_dir_all(&pipeline_dir).expect("Failed to create pipeline dir");

    println!("Generating {} test images...", PIPELINE_IMAGE_COUNT);
    let mut paths = Vec::with_capacity(PIPELINE_IMAGE_COUNT);
    let medium = &TIERS[2]; // medium tier

    for i in 0..PIPELINE_IMAGE_COUNT {
        let path = pipeline_dir.join(format!("page_{:03}.jpg", i + 1));
        // Alternate gray/color for variety
        let img = if i % 5 == 0 {
            generate_color_image(medium.width, medium.height)
        } else {
            generate_grayscale_image(medium.width, medium.height)
        };
        img.save(&path).expect("Failed to save pipeline image");
        paths.push(path);
    }
    println!("Done.\n");

    // Measure full pipeline: encode + CBZ write
    let output_dir = tmp.path().join("output");
    std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    let start = Instant::now();

    // Encode all images
    let pages = process_images_to_memory(&paths, ImageOutputFormat::Avif, None::<&fn()>, None)
        .expect("Pipeline encode failed");

    let encode_elapsed = start.elapsed();

    // Write to CBZ
    let write_start = Instant::now();
    let output_str = output_dir.to_str().unwrap();
    let mut cbz = Cbz::new(output_str, "benchmark").expect("Failed to create CBZ");
    for page in &pages {
        cbz.add_page_from_memory(&page.data, &page.extension)
            .expect("Failed to add page");
    }
    cbz.set_metadata("Benchmark", 1).expect("Failed to set metadata");
    cbz.save().expect("Failed to save CBZ");

    let write_elapsed = write_start.elapsed();
    let total_elapsed = start.elapsed();

    let cbz_path = output_dir.join("benchmark.cbz");
    let cbz_size = std::fs::metadata(&cbz_path).map(|m| m.len()).unwrap_or(0);

    let images_per_sec = PIPELINE_IMAGE_COUNT as f64 / total_elapsed.as_secs_f64();
    let total_input_bytes: u64 = paths.iter().map(|p| std::fs::metadata(p).unwrap().len()).sum();

    println!("Encode:     {:>8.2}s", encode_elapsed.as_secs_f64());
    println!("CBZ Write:  {:>8.2}s", write_elapsed.as_secs_f64());
    println!("Total:      {:>8.2}s", total_elapsed.as_secs_f64());
    println!("Throughput: {:>8.1} images/sec", images_per_sec);
    println!(
        "Input:      {:>8.1} MB | Output: {:.1} MB | Ratio: {:.1}x",
        total_input_bytes as f64 / (1024.0 * 1024.0),
        cbz_size as f64 / (1024.0 * 1024.0),
        total_input_bytes as f64 / cbz_size as f64,
    );

    // Now test streaming pipeline
    println!("\n=== Streaming Pipeline ({} medium images -> AVIF -> CBZ) ===", PIPELINE_IMAGE_COUNT);

    let output_dir_stream = tmp.path().join("output_stream");
    std::fs::create_dir_all(&output_dir_stream).expect("Failed to create output dir");

    let start = Instant::now();

    let output_str = output_dir_stream.to_str().unwrap();
    let mut cbz = Cbz::new(output_str, "benchmark_stream").expect("Failed to create CBZ");

    process_and_write_streaming(
        &paths,
        ImageOutputFormat::Avif,
        &mut cbz,
        None,
        8,
    )
    .expect("Streaming pipeline failed");

    cbz.set_metadata("Benchmark", 1).expect("Failed to set metadata");
    cbz.save().expect("Failed to save CBZ");

    let stream_elapsed = start.elapsed();

    let cbz_stream_path = output_dir_stream.join("benchmark_stream.cbz");
    let cbz_stream_size = std::fs::metadata(&cbz_stream_path).map(|m| m.len()).unwrap_or(0);

    let stream_per_sec = PIPELINE_IMAGE_COUNT as f64 / stream_elapsed.as_secs_f64();

    println!("Total:      {:>8.2}s", stream_elapsed.as_secs_f64());
    println!("Throughput: {:>8.1} images/sec", stream_per_sec);
    println!(
        "Input:      {:>8.1} MB | Output: {:.1} MB | Ratio: {:.1}x",
        total_input_bytes as f64 / (1024.0 * 1024.0),
        cbz_stream_size as f64 / (1024.0 * 1024.0),
        total_input_bytes as f64 / cbz_stream_size as f64,
    );
    println!(
        "vs In-Memory: {:.1}x speedup",
        total_elapsed.as_secs_f64() / stream_elapsed.as_secs_f64(),
    );

    // Downscaling benchmark (large/huge images downscaled to max 1800px)
    println!("\n=== Downscaling Benchmark (large/huge -> max 1800px, AVIF) ===");
    println!(
        "{:<10}| {:<7}| {:>10} | {:>10} | {:>10} | {:>10}",
        "Tier", "Type", "No-DS (KB)", "DS (KB)", "Savings", "DS Time"
    );
    println!("{}", "-".repeat(68));

    for max_dim in [1800u32] {
        for img in test_images.iter().filter(|i| i.tier_name == "large" || i.tier_name == "huge") {
            // Without downscaling
            let no_ds = bench_format(&img.path, ImageOutputFormat::Avif, 1);
            // With downscaling
            let ds = bench_format_with_downscale(&img.path, ImageOutputFormat::Avif, 1, Some(max_dim));
            let savings = 1.0 - (ds.output_size as f64 / no_ds.output_size as f64);
            println!(
                "{:<10}| {:<7}| {:>10.1} | {:>10.1} | {:>9.0}% | {:>8.1}ms",
                img.tier_name,
                img.variant,
                no_ds.output_size as f64 / 1024.0,
                ds.output_size as f64 / 1024.0,
                savings * 100.0,
                ds.mean_ms(),
            );
        }
    }
}
