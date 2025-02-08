use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Rgba};

pub fn dynamic_image_to_argb8888(image: DynamicImage) -> Vec<u32> {
    // Convert the image to RGBA8 (8 bits per channel)
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let mut argb_pixels = Vec::with_capacity((width * height) as usize);

    // Iterate over each pixel in the RGBA image.
    // Each pixel is in the order [R, G, B, A]
    for pixel in rgba_image.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        let a = pixel[3] as u32;

        // Compose the pixel into ARGB_8888 format:
        // 0xAARRGGBB
        let argb = (a << 24) | (r << 16) | (g << 8) | b;
        argb_pixels.push(argb);
    }

    argb_pixels
}

/// Convert a Vec of ARGB_8888 pixels (each i32 as 0xAARRGGBB) into a DynamicImage.
/// Returns None if the number of pixels does not match width*height.
pub fn argb8888_to_dynamic_image(
    width: u32,
    height: u32,
    pixels: Vec<i32>,
) -> Option<DynamicImage> {
    if pixels.len() != (width * height) as usize {
        return None;
    }

    // Create a new vector to hold the RGBA bytes.
    // Each pixel becomes 4 bytes: red, green, blue, alpha.
    let mut raw_pixels = Vec::with_capacity((width * height * 4) as usize);

    for pixel in pixels {
        // Cast to u32 to ignore sign issues.
        let pixel = pixel as u32;
        let a = (pixel >> 24) as u8;
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;

        // The image crate expects pixels in RGBA order.
        raw_pixels.push(r);
        raw_pixels.push(g);
        raw_pixels.push(b);
        raw_pixels.push(a);
    }

    // Create an ImageBuffer from the raw RGBA pixel data.
    // from_raw returns an Option<ImageBuffer<Rgba<u8>, Vec<u8>>>
    ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, raw_pixels).map(DynamicImage::ImageRgba8)
}

pub fn resize_image_to_max_edge(img: DynamicImage, max_edge: u32) -> DynamicImage {
    // Get the original dimensions of the image
    let (width, height) = img.dimensions();

    // Calculate the scaling factor
    let scale = if width > height {
        max_edge as f32 / width as f32
    } else {
        max_edge as f32 / height as f32
    };

    // If we would be upsampling, do not bother.
    if scale >= 1.0 {
        return img;
    }

    // New dimensions
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    // Resize the image
    img.resize_exact(new_width, new_height, FilterType::Lanczos3)
}
