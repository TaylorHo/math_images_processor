# Image Preprocessing for Mathematical Formulas

> By [TaylorHo](https://github.com/TaylorHo/).

This Rust crate processes images of mathematical formulas by:

- Converting the image to grayscale.
- Enhancing contrast (making the formula more distinct).
- Cropping out any large white borders around the formula.
- Resizing the image to fit within a fixed canvas (300x100 by default) with a 5px border.

It supports processing single image files, entire directories of images, and a DynamicImage from the [image crate](https://crates.io/crates/image).

## Why this crate exists?

This crate can quickly standardize images and make them visually better for Neural Networks, used for Machine Learning. In these cases, the more standardized the images are, the better.

It's a quick way to have images ready for training.

## Asynchronous

This crate uses tokio to process directories of images in an asynchronous way. **This is 6x to 7x times faster than the synchronous way** (tested with 36 images).
So, use it with the async macro from tokio in your main function, like below:

```rust
#[tokio::main] // Add this line
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   /// ...
}
```

If you don't want to use async processing and don't want to configure tokio, you can process directories of images using the "no async" method (instead of `process_directory` use `process_directory_without_async`). If you want to process a single image file or `DynamicImage`, use `process_image_file` or `preprocess_image` (respectively, these methods are sync, so there's no need to use tokio).

## Usage

1. **Add the crate to your project**:

   In your `Cargo.toml`:
   ```toml
   [dependencies]
   math_images_processor = "^0.1"
   ```

   Or run:
   ```bash
   cargo add math_images_processor
   ```

2. **Command-line usage** (after cloned locally):

   To process a single image:
   ```bash
   cargo run -- <path_to_image_file>
   ```

   To process all images in a directory:
   ```bash
   cargo run -- <path_to_directory>
   ```

   Processed images will be saved in the `processed-formulas` folder.

## Contributing

Feel free to fork this project, open issues, or submit pull requests.

### Steps to contribute:

1. Fork the repository.
2. Clone your fork locally.
3. Make your changes and commit them.
4. Push your changes and create a pull request.

We welcome contributions! Make sure your code follows the style and includes tests when necessary.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
