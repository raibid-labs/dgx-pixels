//! Create a test image for preview testing

use image::{ImageBuffer, Rgb};

fn main() {
    let width = 512u32;
    let height = 512u32;

    // Create a new RGB image with u8 subpixels
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // Draw a simple pixel art robot
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = if x >= 200 && x <= 312 && y >= 150 && y <= 250 {
            // Head
            Rgb([0x16, 0x21, 0x3e])
        } else if (x >= 220 && x <= 240 && y >= 180 && y <= 200)
            || (x >= 272 && x <= 292 && y >= 180 && y <= 200)
        {
            // Eyes
            Rgb([0x00, 0xff, 0xff])
        } else if x >= 180 && x <= 332 && y >= 250 && y <= 380 {
            // Body
            Rgb([0xe9, 0x45, 0x60])
        } else if (x >= 140 && x <= 180 && y >= 270 && y <= 350)
            || (x >= 332 && x <= 372 && y >= 270 && y <= 350)
        {
            // Arms
            Rgb([0x16, 0x21, 0x3e])
        } else if (x >= 200 && x <= 256 && y >= 380 && y <= 480)
            || (x >= 256 && x <= 312 && y >= 380 && y <= 480)
        {
            // Legs
            Rgb([0x0f, 0x34, 0x60])
        } else {
            // Background
            Rgb([0x1a, 0x1a, 0x2e])
        };

        *pixel = color;
    }

    // Save the image
    let output_path = "outputs/test_sprite.png";
    std::fs::create_dir_all("outputs").expect("Failed to create outputs directory");
    img.save(output_path).expect("Failed to save image");

    println!("✓ Test sprite created: {}", output_path);
    println!("  Size: 512x512 pixels");

    // Also create a smaller test image
    let small_img = image::imageops::resize(&img, 256, 256, image::imageops::FilterType::Nearest);
    small_img
        .save("outputs/test_sprite_small.png")
        .expect("Failed to save small image");

    println!("✓ Small test sprite created: outputs/test_sprite_small.png");
    println!("  Size: 256x256 pixels");
}
