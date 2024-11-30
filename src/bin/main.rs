use math_images_processor::{process_directory, process_image_file, ImageProcessorConfig};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_or_folder_path>", args[0]);
        return Ok(());
    }

    let input_path = PathBuf::from(&args[1]);
    let output_dir = env::current_dir()?.join("processed-formulas");

    // Ensure the output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    // Default configuration
    let config = ImageProcessorConfig::default();

    // Check if the input path is a file or a directory
    if input_path.is_file() {
        println!("Processing single file: {}", input_path.display());
        let output_path = output_dir.join(input_path.file_name().unwrap());
        process_image_file(&input_path, &output_path, &config)?;
    } else if input_path.is_dir() {
        println!("Processing directory: {}", input_path.display());
        process_directory(&input_path, &output_dir, &config)?;
    } else {
        eprintln!("Error: Provided path is neither a file nor a directory.");
        return Ok(());
    }

    println!(
        "Processing completed. Output saved in {}",
        output_dir.display()
    );
    Ok(())
}
