use std::sync::{Arc};
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


    send_progress(&progress_tx, 0.0);
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
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}


pub fn contrast(img: &DynamicImage, factor: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let progress_tx = Arc::new(progress_tx);


    let in_pixels = rgb_img.as_raw();
    let out_pixels = out_buffer.as_mut();

    in_pixels
        .par_chunks_exact(3)
        .zip(out_pixels.par_chunks_exact_mut(3))
        .for_each(|(in_pixel, out_pixel)| {
            out_pixel[0] = (factor * (in_pixel[0] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            out_pixel[1] = (factor * (in_pixel[1] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            out_pixel[2] = (factor * (in_pixel[2] as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
        });

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

pub fn gaussian_blur(img: &DynamicImage, sigma: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let radius = (sigma * 2.0) as u32;
    box_blur(img, radius, progress_tx)
}

// pub fn sharpen(img: &DynamicImage, strength: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
//     let (width, height) = img.dimensions();
//     let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
//     send_progress(&progress_tx, 0.0);

//     let kernel = [
//         [0.0, -1.0, 0.0],
//         [-1.0, 5.0, -1.0],
//         [0.0, -1.0, 0.0],
//     ];

//     for y in 1..height - 1 {
//         for x in 1..width - 1 {
//             let mut sum_red = 0.0;
//             let mut sum_green = 0.0;
//             let mut sum_blue = 0.0;

//             for kernel_y in 0..3 {
//                 for kernel_x in 0..3 {
//                     let pixel_x = x + kernel_x - 1;
//                     let pixel_y = y + kernel_y - 1;

//                     let Rgba([r, g, b, _]) = img.get_pixel(pixel_x, pixel_y);
//                     let weight = kernel[kernel_y as usize][kernel_x as usize];

//                     sum_red += r as f32 * weight;
//                     sum_green += g as f32 * weight;
//                     sum_blue += b as f32 * weight;
//                 }
//             }

//             let original_pixel = img.get_pixel(x, y);
//             let new_red = (original_pixel[0] as f32 * (1.0 - strength) + sum_red * strength).clamp(0.0, 255.0) as u8;
//             let new_green = (original_pixel[1] as f32 * (1.0 - strength) + sum_green * strength).clamp(0.0, 255.0) as u8;
//             let new_blue = (original_pixel[2] as f32 * (1.0 - strength) + sum_blue * strength).clamp(0.0, 255.0) as u8;

//             output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
//         }
        
//         if y % 10 == 0 {
//             send_progress(&progress_tx, y as f64 / height as f64);
//         }
//     }
    
//     send_progress(&progress_tx, 1.0);
//     output
// }
pub fn sharpen(img: &DynamicImage, strenght: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    
    let rgb_img = img.to_rgb8();

    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);

    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();
    
    out_buffer.as_mut().par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            if y == 0 || y == height as usize - 1 {
                let src_start = (y * width as usize * 3) as usize;
                out_row.copy_from_slice(&in_pixels[src_start..src_start + (width * 3) as usize]);
                return;
            }
            
            let row_stride = (width * 3) as usize;
            
            for x in 0..width {
                if x == 0 || x == width - 1 {
                
        
                    let dst_idx = (x * 3) as usize;
                    out_row[dst_idx] = in_pixels[(y * width as usize + x as usize) * 3];
                    out_row[dst_idx + 1] = in_pixels[(y * width as usize + x as usize) * 3 + 1];
                    out_row[dst_idx + 2] = in_pixels[(y * width as usize + x as usize) * 3 + 2];
                    continue;
                }
                
                let center_idx = ((y * width as usize + x as usize) * 3) as usize;

                let center_red = in_pixels[center_idx] as f32;
                let center_green = in_pixels[center_idx + 1] as f32;
                let center_blue = in_pixels[center_idx + 2] as f32;

                let top_red = in_pixels[center_idx - row_stride] as f32;
                let top_green = in_pixels[center_idx - row_stride + 1] as f32;
                let top_blue = in_pixels[center_idx - row_stride + 2] as f32;

                let left_red = in_pixels[center_idx - 3] as f32;
                let left_green = in_pixels[center_idx - 2] as f32;
                let left_blue = in_pixels[center_idx - 1] as f32;

                let right_red = in_pixels[center_idx + 3] as f32;
                let right_green = in_pixels[center_idx + 4] as f32;
                let right_blue = in_pixels[center_idx + 5] as f32;

                let bottom_red = in_pixels[center_idx + row_stride] as f32;
                let bottom_green = in_pixels[center_idx + row_stride + 1] as f32;
                let bottom_blue = in_pixels[center_idx + row_stride + 2] as f32;

                let sharpened_red = center_red * 5.0 - (top_red + left_red + right_red + bottom_red);
                let sharpened_green = center_green * 5.0 - (top_green + left_green + right_green + bottom_green);
                let sharpened_blue = center_blue * 5.0 - (top_blue + left_blue + right_blue + bottom_blue);

                let final_red = center_red + strenght * (sharpened_red - center_red);
                let final_green = center_green + strenght * (sharpened_green - center_green);
                let final_blue = center_blue + strenght * (sharpened_blue - center_blue);

                let out_idx = (x * 3) as usize;
                out_row[out_idx] = final_red.clamp(0.0, 255.0) as u8;
                out_row[out_idx + 1] = final_green.clamp(0.0, 255.0) as u8;
                out_row[out_idx + 2] = final_blue.clamp(0.0, 255.0) as u8;
            }
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}

// pub fn edge_detection(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
//     let (width, height) = img.dimensions();
//     let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
//     let gray_img = img.grayscale();
    
//     send_progress(&progress_tx, 0.0);

//     let sobel_x = [
//         [-1, 0, 1],
//         [-2, 0, 2],
//         [-1, 0, 1],
//     ];

//     let sobel_y = [
//         [-1, -2, -1],
//         [0, 0, 0],
//         [1, 2, 1],
//     ];

//     for y in 1..height - 1 {
//         for x in 1..width - 1 {
//             let mut gx = 0.0;
//             let mut gy = 0.0;

//             for i in 0..3 {
//                 for j in 0..3 {
//                     let pixel_x = x + j - 1;
//                     let pixel_y = y + i - 1;
//                     let pixel_value = gray_img.get_pixel(pixel_x, pixel_y)[0] as f32;

//                     gx += pixel_value * sobel_x[i as usize][j as usize] as f32;
//                     gy += pixel_value * sobel_y[i as usize][j as usize] as f32;
//                 }
//             }

//             let magnitude = (gx * gx + gy * gy).sqrt().clamp(0.0, 255.0) as u8;
//             output.put_pixel(x, y, Rgb([magnitude, magnitude, magnitude]));
//         }
        
//         if y % 15 == 0 {
//             send_progress(&progress_tx, y as f64 / height as f64);
//         }
//     }
    
//     send_progress(&progress_tx, 1.0);
//     output
// }

pub fn edge_detection(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    let mut out_buffer = ImageBuffer::new(width, height);

    let in_pixels = rgb_img.as_raw();

    out_buffer.as_mut().par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .skip(1)
        .take(height as usize - 2)
        .for_each(|(y, out_row)| {
            let row_stride = (width * 3) as usize;
            let y = y + 1; // Adjust for skip(1)

            for x in 1..width - 1 {
                let center_idx = ((y * width as usize + x as usize) * 3) as usize;

                let get_gray = |idx: usize| {
                    0.299 * in_pixels[idx] as f32 + 
                    0.587 * in_pixels[idx + 1] as f32 + 
                    0.114 * in_pixels[idx + 2] as f32
                };

                let gx = -get_gray(center_idx - row_stride - 3) + 
                          get_gray(center_idx - row_stride + 3) -
                          2.0 * get_gray(center_idx - 3) +
                          2.0 * get_gray(center_idx + 3) - 
                          get_gray(center_idx + row_stride - 3) +
                          get_gray(center_idx + row_stride + 3);

                
                let gy = -get_gray(center_idx - row_stride - 3) -    
                         2.0 * get_gray(center_idx - row_stride) -   
                          get_gray(center_idx - row_stride + 3) +    
                          get_gray(center_idx + row_stride - 3) +    
                         2.0 * get_gray(center_idx + row_stride) +   
                          get_gray(center_idx + row_stride + 3);    

                let magnitude = (gx * gx + gy * gy).sqrt().clamp(0.0, 255.0) as u8;

                let out_idx = (x * 3) as usize;
                out_row[out_idx] = magnitude;
                out_row[out_idx + 1] = magnitude;
                out_row[out_idx + 2] = magnitude;
            }
        });

    send_progress(&progress_tx, 1.0);
    out_buffer

}


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


