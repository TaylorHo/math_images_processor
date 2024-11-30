use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use std::path::{Path, PathBuf};

/// Configuration for image processing final sizes
#[derive(Clone)]
pub struct ImageProcessorConfig {
    pub width: u32,
    pub height: u32,
    pub border: u32,
}

/// Default configuration for image processing final sizes
impl Default for ImageProcessorConfig {
    fn default() -> Self {
        Self {
            width: 300,
            height: 100,
            border: 5,
        }
    }
}

/// The most important part of this crate.
/// Takes a `DynamicImage` from the `image` crate and returns a processed `GrayImage`, already within the desired size provided by `ImageProcessorConfig`.
pub fn preprocess_image(
    image: DynamicImage,
    config: &ImageProcessorConfig,
) -> Result<GrayImage, Box<dyn std::error::Error>> {
    let mut gray_img = image.to_luma8();

    if is_inverted(&gray_img) {
        gray_img = invert_colors(&gray_img);
    }

    let cropped_img = crop_white_borders(&enhance_contrast(&gray_img));

    let final_img = fit_into_canvas(&cropped_img, config.width, config.height, config.border);

    Ok(final_img)
}

/// Processes a single image file, specified by `input_path`, and saves the processed image to a new file, specified by `output_path`.
/// Final image sizes can be configured via `ImageProcessorConfig`.
/// For processing a directory use `process_directory`.
/// For processing a `DynamicImage` from the `image` crate, use directly `preprocess_image`.
pub fn process_image_file(
    input_path: &Path,
    output_path: &Path,
    config: &ImageProcessorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input_path)?;
    let processed_img = preprocess_image(img, config)?;
    processed_img.save(output_path)?;
    Ok(())
}

/// Processes all PNG, JPG, and JPEG files in a directory and saves the processed images in a separate directory.
/// NOTE 1: This uses asynchronous processing, so use with tokio macros in the main function.
/// NOTE 2: Asynchronous processing is almost 7x faster than synchronous processing. For synchronous processing use `process_directory_without_async`.
/// Final image sizes can be configured via `ImageProcessorConfig`.
/// For processing a single file use `process_image_file`.
/// For processing a `DynamicImage` from the `image` crate, use directly `preprocess_image`.
pub async fn process_directory(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    config: &ImageProcessorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !output_dir.exists() {
        tokio::fs::create_dir_all(output_dir).await?;
    }

    let mut entries = tokio::fs::read_dir(input_dir).await?;
    let mut tasks = vec![];

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext
                .to_str()
                .map(|s| s.to_lowercase())
                .map_or(false, |ext| matches!(ext.as_str(), "png" | "jpg" | "jpeg"))
            {
                let output_file = output_dir.join(path.file_name().unwrap());
                let cloned_config = config.clone();

                // Spawn a task for each image processing operation
                tasks.push(tokio::task::spawn(async move {
                    if let Err(e) = process_image_file(&path, &output_file, &cloned_config) {
                        eprintln!("Error processing file {:?}: {:?}", path, e);
                    }
                }));
            }
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await?;
    }

    Ok(())
}

pub fn process_directory_without_async(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    config: &ImageProcessorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext
                .to_str()
                .map(|s| s.to_lowercase())
                .map_or(false, |ext| matches!(ext.as_str(), "png" | "jpg" | "jpeg"))
            {
                let output_file = output_dir.join(path.file_name().unwrap());
                process_image_file(&path, &output_file, config)?;
            }
        }
    }

    Ok(())
}

/// Checks if the image is inverted (fonts are white on a black background)
fn is_inverted(img: &GrayImage) -> bool {
    let (mut black_count, mut white_count) = (0, 0);

    for pixel in img.pixels() {
        let Luma([l]) = *pixel;
        if l < 128 {
            black_count += 1;
        } else if l > 200 {
            white_count += 1;
        }
    }

    // If black pixels significantly outnumber white pixels, assume inverted colors
    black_count > 2 * white_count
}

/// Inverts the colors of the image. Eg. black on white becomes white on black
fn invert_colors(img: &GrayImage) -> GrayImage {
    let mut inverted = img.clone();
    for pixel in inverted.pixels_mut() {
        let Luma([l]) = *pixel;
        *pixel = Luma([255 - l]); // Invert the grayscale value
    }
    inverted
}

/// Applies contrast enhancement to the image, making it easier to read as a pixels map
fn enhance_contrast(img: &GrayImage) -> GrayImage {
    let mut enhanced = img.clone();

    // Apply a simple contrast enhancement
    for pixel in enhanced.pixels_mut() {
        let Luma([l]) = *pixel;
        *pixel = if l > 200 { Luma([255]) } else { Luma([l / 2]) };
    }

    enhanced
}

/// Crops the white borders of the image till the black font
fn crop_white_borders(img: &GrayImage) -> GrayImage {
    let (width, height) = img.dimensions();

    let mut left = width;
    let mut right = 0;
    let mut top = height;
    let mut bottom = 0;

    for y in 0..height {
        for x in 0..width {
            let Luma([l]) = img.get_pixel(x, y);
            if *l < 250 {
                // Threshold for detecting non-white
                if x < left {
                    left = x;
                }
                if x > right {
                    right = x;
                }
                if y < top {
                    top = y;
                }
                if y > bottom {
                    bottom = y;
                }
            }
        }
    }

    img.view(left, top, right - left + 1, bottom - top + 1)
        .to_image()
}

/// Fits the image into a canvas of the specified dimensions, without changing the aspect ratio
fn fit_into_canvas(img: &GrayImage, width: u32, height: u32, border: u32) -> GrayImage {
    let (img_width, img_height) = img.dimensions();

    // Calculate the effective canvas dimensions excluding borders
    let max_width = width - 2 * border;
    let max_height = height - 2 * border;

    // Compute the scaling factor to fit the image within the canvas
    let scale = f64::min(
        max_width as f64 / img_width as f64,
        max_height as f64 / img_height as f64,
    );

    // Compute the new dimensions for the image
    let new_width = (img_width as f64 * scale).round() as u32;
    let new_height = (img_height as f64 * scale).round() as u32;

    // Resize the formula image
    let resized_img = image::imageops::resize(
        img,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );

    // Create a white canvas
    let mut canvas = GrayImage::from_pixel(width, height, Luma([255]));

    // Calculate the position to center the resized image on the canvas
    let x_offset = border + (max_width - new_width) / 2;
    let y_offset = border + (max_height - new_height) / 2;

    // Overlay the resized image onto the canvas
    image::imageops::overlay(&mut canvas, &resized_img, x_offset as i64, y_offset as i64);

    canvas
}
