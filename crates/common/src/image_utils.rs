//! Image analysis utilities shared across crates.

use image::{DynamicImage, GenericImageView, Pixel};

// --- Grayscale Detection Constants ---

/// Pixel sample count for distributed grayscale detection.
const GRAYSCALE_SAMPLE_COUNT: u64 = 500;

/// RGB channel deviation threshold for grayscale classification.
const GRAYSCALE_RGB_THRESHOLD: u8 = 3;

/// Minimum grayscale pixel percentage threshold.
const GRAYSCALE_THRESHOLD: f32 = 0.95;

/// Maximum dimension before downscaling for grayscale detection.
const GRAYSCALE_MAX_DIMENSION: u32 = 650;

/// Grid sampling step size for downscaled images.
const GRAYSCALE_SAMPLE_RATE: u32 = 5;

// --- Grayscale Detection Strategies ---

/// Grayscale detection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayscaleStrategy {
    /// Fast distributed sampling without downscaling. Best for encoding.
    Distributed,

    /// Grid sampling with downscaling. Best for cover detection.
    GridWithDownscale,
}

/// Detects if an image is predominantly grayscale.
#[inline]
pub fn is_grayscale(img: &DynamicImage, strategy: GrayscaleStrategy) -> bool {
    match strategy {
        GrayscaleStrategy::Distributed => detect_grayscale_distributed(img),
        GrayscaleStrategy::GridWithDownscale => detect_grayscale_grid(img),
    }
}

/// Detects if an image is predominantly grayscale with custom threshold.
#[inline]
pub fn is_grayscale_with_threshold(
    img: &DynamicImage,
    strategy: GrayscaleStrategy,
    threshold: f64,
) -> bool {
    match strategy {
        GrayscaleStrategy::Distributed => {
            detect_grayscale_distributed_with_threshold(img, threshold as f32)
        }
        GrayscaleStrategy::GridWithDownscale => {
            detect_grayscale_grid_with_threshold(img, threshold)
        }
    }
}

/// Distributed pixel sampling without downscaling.
#[inline]
fn detect_grayscale_distributed(img: &DynamicImage) -> bool {
    detect_grayscale_distributed_with_threshold(img, GRAYSCALE_THRESHOLD)
}

fn detect_grayscale_distributed_with_threshold(img: &DynamicImage, threshold: f32) -> bool {
    // Native grayscale formats - early return
    if matches!(
        img,
        DynamicImage::ImageLuma8(_) | DynamicImage::ImageLuma16(_)
    ) {
        return true;
    }

    let width = img.width() as u64;
    let height = img.height() as u64;
    let pixel_count = width * height;

    // Calculate sampling step
    let step = (pixel_count / GRAYSCALE_SAMPLE_COUNT).max(1);

    let mut grayscale_samples = 0u32;
    let mut total_sampled = 0u32;

    // Sample pixels in a cache-friendly pattern (row-major order)
    // This significantly improves performance by utilizing CPU cache better
    let mut i = 0u64;
    while i < pixel_count {
        let x = (i % width) as u32;
        let y = (i / width) as u32;

        let pixel = img.get_pixel(x, y);
        let channels = pixel.channels();

        if channels.len() >= 3 {
            let r = channels[0];
            let g = channels[1];
            let b = channels[2];

            // Branchless grayscale check using bit manipulation
            // This is faster than multiple comparisons
            let rg_diff = r.abs_diff(g);
            let gb_diff = g.abs_diff(b);
            let br_diff = b.abs_diff(r);

            if rg_diff <= GRAYSCALE_RGB_THRESHOLD
                && gb_diff <= GRAYSCALE_RGB_THRESHOLD
                && br_diff <= GRAYSCALE_RGB_THRESHOLD
            {
                grayscale_samples += 1;
            }
            total_sampled += 1;
        }

        i += step;
    }

    // Check if majority of samples are grayscale
    total_sampled > 0 && (grayscale_samples as f32 / total_sampled as f32) > threshold
}

/// Grid sampling with automatic downscaling for large images.
#[inline]
fn detect_grayscale_grid(img: &DynamicImage) -> bool {
    detect_grayscale_grid_with_threshold(img, GRAYSCALE_THRESHOLD as f64)
}

fn detect_grayscale_grid_with_threshold(img: &DynamicImage, sensibility: f64) -> bool {
    // Native grayscale formats - early return
    if matches!(
        img,
        DynamicImage::ImageLuma8(_) | DynamicImage::ImageLuma16(_)
    ) {
        return true;
    }

    // Downscale if necessary
    let img = if img.width() > GRAYSCALE_MAX_DIMENSION || img.height() > GRAYSCALE_MAX_DIMENSION {
        let scale = GRAYSCALE_MAX_DIMENSION as f32 / img.width().max(img.height()) as f32;
        let new_width = (img.width() as f32 * scale) as u32;
        let new_height = (img.height() as f32 * scale) as u32;
        img.thumbnail(new_width, new_height)
    } else {
        img.clone()
    };

    let width = img.width();
    let height = img.height();
    let total_pixels = (width * height) as f64;
    let gray_threshold = total_pixels * sensibility;

    // Pre-calculate step sizes
    let x_step = GRAYSCALE_SAMPLE_RATE;
    let y_step = GRAYSCALE_SAMPLE_RATE;

    // Estimate sample count for better memory allocation
    let estimated_samples = ((width / x_step) * (height / y_step)) as usize;
    if estimated_samples == 0 {
        return false;
    }

    // Count grayscale pixels directly without allocating sample vector
    let mut gray_pixels = 0u32;
    let mut total_samples = 0u32;

    // Iterate in cache-friendly order (row-major)
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let pixel = img.get_pixel(x, y);
            let rgb = pixel.to_rgb();
            let r = rgb[0];
            let g = rgb[1];
            let b = rgb[2];

            if r.abs_diff(g) <= GRAYSCALE_RGB_THRESHOLD
                && g.abs_diff(b) <= GRAYSCALE_RGB_THRESHOLD
                && b.abs_diff(r) <= GRAYSCALE_RGB_THRESHOLD
            {
                gray_pixels += 1;
            }
            total_samples += 1;

            x += x_step;
        }
        y += y_step;
    }

    // Extrapolate to full image
    let estimated_gray_pixels = (gray_pixels as f64 * total_pixels) / total_samples as f64;

    estimated_gray_pixels > gray_threshold
}
