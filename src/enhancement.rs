use std::sync::{atomic::AtomicI64, Arc};
use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb, Rgba};
use rayon::{iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};
use crate::{ProgressSender, send_progress};
use std::sync::atomic::{AtomicUsize,Ordering};

pub fn brightness(
    img: &DynamicImage,
    value: i32,
    progress_tx: Option<ProgressSender>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    let total_pixels = (width * height) as usize;

    send_progress(&progress_tx, 0.0);
    let progress_counter = Arc::new(AtomicUsize::new(0));
    let progress_tx = Arc::new(progress_tx);

    let in_pixels = rgb_img.as_raw();
    let out_pixels = out_buffer.as_mut();

    in_pixels
        .par_chunks_exact(3)
        .zip(out_pixels.par_chunks_exact_mut(3))
        .for_each(|(in_pixel, out_pixel)| {

            out_pixel[0] = (in_pixel[0] as i32 + value).clamp(0, 255) as u8;
            out_pixel[1] = (in_pixel[1] as i32 + value).clamp(0, 255) as u8;
            out_pixel[2] = (in_pixel[2] as i32 + value).clamp(0, 255) as u8;

            let current = progress_counter.fetch_add(1, Ordering::Relaxed);
            if current % 50000 == 0 {
                send_progress(&progress_tx, current as f64 / total_pixels as f64);
            }
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}

/// Adjust image contrast
pub fn contrast(img: &DynamicImage, factor: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    let total_pixels = (width * height) as usize;
    
    send_progress(&progress_tx, 0.0);
    let progress_counter = Arc::new(AtomicUsize::new(0));
    let progress_tx = Arc::new(progress_tx);
    let last_update = Arc::new(AtomicI64::new(0));

    let in_pixels = rgb_img.as_raw();
    let out_pixels = out_buffer.as_mut();

    in_pixels
        .par_chunks_exact(3)
        .zip(out_pixels.par_chunks_exact_mut(3))
        .for_each(|(in_pixel, out_pixel)| {
            out_pixel[0] = (factor * (in_pixel[0] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            out_pixel[1] = (factor * (in_pixel[1] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            out_pixel[2] = (factor * (in_pixel[2] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;

            let current = progress_counter.fetch_add(1, Ordering::Relaxed);

           let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let last = last_update.load(Ordering::Relaxed);
            if now - last as u64 > 16 {
                if last_update.compare_exchange(last, now as i64, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                    send_progress(&progress_tx, current as f64 / total_pixels as f64);
                }
            }

        });
    // for y in 0..height {
    //     for x in 0..width {
    //         let Rgba([r, g, b, _]) = img.get_pixel(x, y);

    //         let new_red = (factor * (r as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
    //         let new_green = (factor * (g as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
    //         let new_blue = (factor * (b as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;

    //         output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
    //     }

    //     if y % 15 == 0 {
    //         send_progress(&progress_tx, y as f64 / height as f64);
    //     }
    // }

    send_progress(&progress_tx, 1.0);
    out_buffer
}

/// Apply box blur
pub fn box_blur(img: &DynamicImage, radius: u32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    
    let rgb_img = img.to_rgb8();

    let (width, height) = rgb_img.dimensions();

    send_progress(&progress_tx, 0.0);

    let horizontal_blurred = horizontal_box_blur(&rgb_img, radius, width, height);

    send_progress(&progress_tx, 0.5);

    let final_result = vertical_box_blur(&horizontal_blurred, radius, width, height);

    send_progress(&progress_tx, 1.0);

    ImageBuffer::from_vec(width, height, final_result).unwrap()
}


fn horizontal_box_blur(img: &ImageBuffer<Rgb<u8>, Vec<u8>>, radius: u32, width: u32, height: u32) -> Vec<u8> {
    let mut result = vec![0u8; (width * height * 3) as usize];
    let pixels = img.as_raw();

    result.par_chunks_exact_mut((width * 3) as usize).enumerate().for_each(|(y, row)| {
        let row_start = (y as u32 * width * 3) as usize;

        let mut sum_red = 0u32;
        let mut sum_green = 0u32;
        let mut sum_blue = 0u32;
        let mut window_size = 0u32;

        let x_min = 0u32.saturating_sub(radius);
        let x_max = (0 + radius).min(width - 1);

        for kx in x_min..=x_max {
            let idx = row_start + (kx * 3) as usize;
            sum_red += pixels[idx] as u32;
            sum_green += pixels[idx + 1] as u32;
            sum_blue += pixels[idx + 2] as u32;
            window_size += 1;
        }

        let out_idx = 0;
        row[out_idx] = (sum_red / window_size) as u8;
        row[out_idx + 1] = (sum_green / window_size) as u8;
        row[out_idx + 2] = (sum_blue / window_size) as u8;

        for x in 1..width {
            let new_x_min = x.saturating_sub(radius);
            let new_x_max = (x + radius).min(width - 1);
            let prev_x_min = (x - 1).saturating_sub(radius);
            let prev_x_max = ((x - 1) + radius).min(width - 1);

            if new_x_min > prev_x_min {
                let remove_idx = row_start + (prev_x_min * 3) as usize;
                sum_red -= pixels[remove_idx] as u32;
                sum_green -= pixels[remove_idx + 1] as u32;
                sum_blue -= pixels[remove_idx + 2] as u32;
                window_size -= 1;
            }

            if new_x_max > prev_x_max {
                let add_idx = row_start + (new_x_max * 3) as usize;
                sum_red += pixels[add_idx] as u32;
                sum_green += pixels[add_idx + 1] as u32;
                sum_blue += pixels[add_idx + 2] as u32;
                window_size += 1;
            }

            let out_idx = (x * 3) as usize;
            row[out_idx] = (sum_red / window_size) as u8;
            row[out_idx + 1] = (sum_green / window_size) as u8;
            row[out_idx + 2] = (sum_blue / window_size) as u8;
        }
    });

    result   
}

fn vertical_box_blur(pixels: &[u8], radius: u32, width: u32, height: u32) -> Vec<u8> {
    let mut result = vec![0u8; (width * height * 3) as usize];

    for x in 0..width {
        let mut sum_red = 0u32;
        let mut sum_green = 0u32;
        let mut sum_blue = 0u32;
        let mut window_size = 0u32;

        let y_min = 0u32.saturating_sub(radius);
        let y_max = (0 + radius).min(height - 1);

        for ky in y_min..=y_max {
            let idx = ((ky * width + x) * 3) as usize;
            sum_red += pixels[idx] as u32;
            sum_green += pixels[idx + 1] as u32;
            sum_blue += pixels[idx + 2] as u32;
            window_size += 1;
        }

        let out_idx = (x * 3) as usize;
        result[out_idx] = (sum_red / window_size) as u8;
        result[out_idx + 1] = (sum_green / window_size) as u8;
        result[out_idx + 2] = (sum_blue / window_size) as u8;

        for y in 1..height {
            let new_y_min = y.saturating_sub(radius);
            let new_y_max = (y + radius).min(height - 1);
            let prev_y_min = (y - 1).saturating_sub(radius);
            let prev_y_max = ((y - 1) + radius).min(height - 1);

            if new_y_min > prev_y_min {
                let remove_idx = ((prev_y_min * width + x) * 3) as usize;
                sum_red -= pixels[remove_idx] as u32;
                sum_green -= pixels[remove_idx + 1] as u32;
                sum_blue -= pixels[remove_idx + 2] as u32;
                window_size -= 1;
            }

            if new_y_max > prev_y_max {
                let add_idx = ((new_y_max * width + x) * 3) as usize;
                sum_red += pixels[add_idx] as u32;
                sum_green += pixels[add_idx + 1] as u32;
                sum_blue += pixels[add_idx + 2] as u32;
                window_size += 1;
            }

            let out_idx = ((y * width + x) * 3) as usize;
            result[out_idx] = (sum_red / window_size) as u8;
            result[out_idx + 1] = (sum_green / window_size) as u8;
            result[out_idx + 2] = (sum_blue / window_size) as u8;
        }
    }

    result


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


