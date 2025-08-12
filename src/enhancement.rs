use std::sync::{Arc, Mutex};

use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb, Rgba};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::{ProgressSender, send_progress};


pub fn brightness(img: &DynamicImage, value: i32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let progress_counter = Arc::new(Mutex::new(0u32));
    let progress_tx = Arc::new(progress_tx);

    let pixels: Vec<_> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut row_pixels = Vec::with_capacity(width as usize);

            for x in 0..width {
                let Rgba([r, g, b, _]) = img.get_pixel(x, y);

                let new_red = (r as i32 + value).clamp(0, 255) as u8;
                let new_green = (g as i32 + value).clamp(0, 255) as u8;
                let new_blue = (b as i32 + value).clamp(0, 255) as u8;

                row_pixels.push((x, y, Rgb([new_red, new_green, new_blue])));
            }

            if y % 20 == 0 {
                if let Ok(mut counter) = progress_counter.lock() {
                    *counter += 20;
                    send_progress(&progress_tx, *counter as f64 / height as f64);
                }
            }
            row_pixels
        })
        .collect();


        for row in pixels {
            for (x, y, pixel) in row {
                output.put_pixel(x, y, pixel);
            }
        }
    // for y in 0..height {
    //     for x in 0..width {
    //         let Rgba([r, g, b, _]) = img.get_pixel(x, y);

    //         let new_red = (r as i32 + value).clamp(0, 255) as u8;
    //         let new_green = (g as i32 + value).clamp(0, 255) as u8;
    //         let new_blue = (b as i32 + value).clamp(0, 255) as u8;

    //         output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
    //     }

    //     if y % 20 == 0 {
    //         send_progress(&progress_tx, y as f64 / height as f64);
    //     }
    // }

    send_progress(&progress_tx, 1.0);
    output
}

/// Adjust image contrast
pub fn contrast(img: &DynamicImage, factor: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);

            let new_red = (factor * (r as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            let new_green = (factor * (g as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            let new_blue = (factor * (b as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }

        if y % 15 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }

    send_progress(&progress_tx, 1.0);
    output
}

/// Apply box blur
pub fn box_blur(img: &DynamicImage, radius: u32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let mut total_red: u32 = 0;
            let mut total_green: u32 = 0;
            let mut total_blue: u32 = 0;
            let mut pixel_count: u32 = 0;

            let y_min = y.saturating_sub(radius);
            let y_max = (y + radius).min(height - 1);
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(width - 1);

            for ny in y_min..=y_max {
                for nx in x_min..=x_max {
                    let pixel = img.get_pixel(nx, ny);
                    let Rgba([r, g, b, _]) = pixel;

                    total_red += r as u32;
                    total_green += g as u32;
                    total_blue += b as u32;
                    pixel_count += 1;
                }
            }

            let avg_red = (total_red / pixel_count) as u8;
            let avg_green = (total_green / pixel_count) as u8;
            let avg_blue = (total_blue / pixel_count) as u8;

            output.put_pixel(x, y, Rgb([avg_red, avg_green, avg_blue]));
        }

        if y % 10 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }

    send_progress(&progress_tx, 1.0);
    output
}

/// Apply Gaussian blur (simplified implementation)
pub fn gaussian_blur(img: &DynamicImage, sigma: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // For simplicity, we'll use a box blur approximation
    let radius = (sigma * 2.0) as u32;
    box_blur(img, radius, progress_tx)
}

/// Sharpen the image
pub fn sharpen(img: &DynamicImage, strength: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let kernel = [
        [0.0, -1.0, 0.0],
        [-1.0, 5.0, -1.0],
        [0.0, -1.0, 0.0],
    ];

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut sum_red = 0.0;
            let mut sum_green = 0.0;
            let mut sum_blue = 0.0;

            for kernel_y in 0..3 {
                for kernel_x in 0..3 {
                    let pixel_x = x + kernel_x - 1;
                    let pixel_y = y + kernel_y - 1;

                    let Rgba([r, g, b, _]) = img.get_pixel(pixel_x, pixel_y);
                    let weight = kernel[kernel_y as usize][kernel_x as usize];

                    sum_red += r as f32 * weight;
                    sum_green += g as f32 * weight;
                    sum_blue += b as f32 * weight;
                }
            }

            let original_pixel = img.get_pixel(x, y);
            let new_red = (original_pixel[0] as f32 * (1.0 - strength) + sum_red * strength).clamp(0.0, 255.0) as u8;
            let new_green = (original_pixel[1] as f32 * (1.0 - strength) + sum_green * strength).clamp(0.0, 255.0) as u8;
            let new_blue = (original_pixel[2] as f32 * (1.0 - strength) + sum_blue * strength).clamp(0.0, 255.0) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
        
        if y % 10 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Apply edge detection using Sobel operator
pub fn edge_detection(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    let gray_img = img.grayscale();
    
    send_progress(&progress_tx, 0.0);

    let sobel_x = [
        [-1, 0, 1],
        [-2, 0, 2],
        [-1, 0, 1],
    ];

    let sobel_y = [
        [-1, -2, -1],
        [0, 0, 0],
        [1, 2, 1],
    ];

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut gx = 0.0;
            let mut gy = 0.0;

            for i in 0..3 {
                for j in 0..3 {
                    let pixel_x = x + j - 1;
                    let pixel_y = y + i - 1;
                    let pixel_value = gray_img.get_pixel(pixel_x, pixel_y)[0] as f32;

                    gx += pixel_value * sobel_x[i as usize][j as usize] as f32;
                    gy += pixel_value * sobel_y[i as usize][j as usize] as f32;
                }
            }

            let magnitude = (gx * gx + gy * gy).sqrt().clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgb([magnitude, magnitude, magnitude]));
        }
        
        if y % 15 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Apply binary thresholding
pub fn thresholding(img: &DynamicImage, threshold: u8, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    let gray_img = img.grayscale();
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let pixel_value = gray_img.get_pixel(x, y)[0];
            let binary_value = if pixel_value > threshold { 255 } else { 0 };
            output.put_pixel(x, y, Rgb([binary_value, binary_value, binary_value]));
        }
        
        if y % 20 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}