use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};
use crate::{ProgressSender, send_progress};


pub fn sepia(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();
    let out_pixels = out_buffer.as_mut();

    in_pixels
        .par_chunks_exact(3)
        .zip(out_pixels.par_chunks_exact_mut(3))
        .for_each(|(in_pixel, out_pixel)| {
            let red = in_pixel[0] as f32;
            let green = in_pixel[1] as f32;
            let blue = in_pixel[2] as f32;

            let new_red = (red * 0.393) + (green * 0.769) + (blue * 0.189);
            let new_green = (red * 0.349) + (green * 0.686) + (blue * 0.168);
            let new_blue = (red * 0.272) + (green * 0.534) + (blue * 0.131);

            out_pixel[0] = new_red.min(255.0) as u8;
            out_pixel[1] = new_green.min(255.0) as u8;
            out_pixel[2] = new_blue.min(255.0) as u8;
        });

    // for y in 0..height {
    //     for x in 0..width {
    //         let Rgba([r, g, b, _]) = img.get_pixel(x, y);
        
    //         let red_filter = r as f32;
    //         let green_filter = g as f32;
    //         let blue_filter = b as f32;

    //         let new_red = (red_filter * 0.393) + (green_filter * 0.769) + (blue_filter * 0.189);
    //         let new_green = (red_filter * 0.349) + (green_filter * 0.686) + (blue_filter * 0.168);
    //         let new_blue = (red_filter * 0.272) + (green_filter * 0.534) + (blue_filter * 0.131);

    //         output.put_pixel(x, y, Rgb([
    //             new_red.min(255.0) as u8,
    //             new_green.min(255.0) as u8,
    //             new_blue.min(255.0) as u8,
    //         ]));
    //     }
    // }

    send_progress(&progress_tx, 1.0);
    out_buffer
}

/// Apply vignette effect
pub fn vignette(img: &DynamicImage, strength: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_distance = ((center_x * center_x) + (center_y * center_y)).sqrt();

    let in_pixels = rgb_img.as_raw();
    
    out_buffer
        .as_mut()
        .par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            let y_f = y as f32;

            for x in 0..width {
                let x_f = x as f32;
                let in_idx = ((y * width as usize + x as usize) * 3) as usize;
                let out_idx = ( x * 3 ) as usize;

                let distance = ((x_f - center_x).powi(2) + (y_f - center_y).powi(2)).sqrt();
                let vignette_factor = 1.0 - (distance / max_distance * strength).clamp(0.0, 1.0);

                out_row[out_idx] = (in_pixels[in_idx] as f32 * vignette_factor) as u8;
                out_row[out_idx + 1] = (in_pixels[in_idx + 1] as f32 * vignette_factor) as u8;
                out_row[out_idx + 2] = (in_pixels[in_idx + 2] as f32 * vignette_factor) as u8;
            }
        });

    // for y in 0..height {
    //     for x in 0..width {
    //         let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            
    //         let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
    //         let vignette_factor = 1.0 - (distance / max_distance * strength).clamp(0.0, 1.0);
            
    //         let new_red = (r as f32 * vignette_factor) as u8;
    //         let new_green = (g as f32 * vignette_factor) as u8;
    //         let new_blue = (b as f32 * vignette_factor) as u8;
            
    //         output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
    //     }
    // }
    
    send_progress(&progress_tx, 1.0);
    out_buffer
}

/// Add noise to image
pub fn noise(img: &DynamicImage, strength: u8, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {

    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    // let mut rng = rand::rng();
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();

    out_buffer
        .as_mut()
        .par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            use rand::{rngs::SmallRng, Rng, SeedableRng};

            let mut rng = SmallRng::seed_from_u64((y as u64).wrapping_mul(0x9e3779b97f4a7c15));

            for x in 0..width {
                let in_idx = ((y * width as usize + x as usize) * 3) as usize;
                let out_idx = (x * 3) as usize;

                let noise_red = rng.random_range(-(strength as i32)..=strength as i32);
                let noise_green = rng.random_range(-(strength as i32)..=strength as i32);
                let noise_blue = rng.random_range(-(strength as i32)..=strength as i32);

                out_row[out_idx] = (in_pixels[in_idx] as i32 + noise_red).clamp(0, 255) as u8;
                out_row[out_idx + 1] = (in_pixels[in_idx + 1 ] as i32 + noise_green).clamp(0, 255) as u8;
                out_row[out_idx + 2] = (in_pixels[in_idx + 2] as i32 + noise_blue).clamp(0, 255) as u8;
            }
        });

    // for y in 0..height {
    //     for x in 0..width {
    //         let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            
    //         let noise_r = rng.random_range(-(strength as i32)..=strength as i32);
    //         let noise_g = rng.random_range(-(strength as i32)..=strength as i32);
    //         let noise_b = rng.random_range(-(strength as i32)..=strength as i32);
            
    //         let new_red = (r as i32 + noise_r).clamp(0, 255) as u8;
    //         let new_green = (g as i32 + noise_g).clamp(0, 255) as u8;
    //         let new_blue = (b as i32 + noise_b).clamp(0, 255) as u8;
            
    //         output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
    //     }
    // }
    
    send_progress(&progress_tx, 1.0);
    out_buffer
}

/// Apply oil painting effect
pub fn oil_painting(img: &DynamicImage, radius: u32, intensity: u32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {

    let rgb_img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut out_buffer = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();

    out_buffer.as_mut()
        .par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            for x in 0..width {
                let mut intensity_count = vec![0; intensity as usize];
                let mut avg_red = vec![0.0; intensity as usize];
                let mut avg_green = vec![0.0; intensity as usize];
                let mut avg_blue = vec![0.0; intensity as usize];

                let y_min = (y as u32).saturating_sub(radius);
                let y_max = ((y as u32) + radius).min(height - 1);
                let x_min = x.saturating_sub(radius);
                let x_max = (x + radius).min(width - 1);

                for new_y in y_min..=y_max {
                    for new_x in x_min..=x_max {
                        let sample_idx = ((new_y * width + new_x) * 3) as usize;
                        let red = in_pixels[sample_idx];
                        let green = in_pixels[sample_idx + 1];
                        let blue = in_pixels[sample_idx + 2];

                        let intensity_index = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                            .min(intensity - 1)as usize;
                        
                        intensity_count[intensity_index] += 1;
                        avg_red[intensity_index] += red as f32;
                        avg_green[intensity_index] += green as f32;
                        avg_blue[intensity_index] += blue as f32;
                    } 
                }

                let max_index = intensity_count
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, count)| *count)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);


                let out_idx = (x * 3) as usize;
                if intensity_count[max_index] > 0 {
                    let count = intensity_count[max_index] as f32;
                    out_row[out_idx] = (avg_red[max_index] / count) as u8;
                    out_row[out_idx + 1] = (avg_green[max_index] / count) as u8;
                    out_row[out_idx + 2] = (avg_blue[max_index] / count) as u8;
                } else {
                    out_row[out_idx] = 0;
                    out_row[out_idx + 1] = 0;
                    out_row[out_idx + 2] = 0;
                }
            }
        });

    // for y in 0..height {
    //     for x in 0..width {
    //         let mut intensity_count = vec![0; intensity as usize];
    //         let mut avg_red = vec![0.0; intensity as usize];
    //         let mut avg_green = vec![0.0; intensity as usize];
    //         let mut avg_blue = vec![0.0; intensity as usize];

    //         let y_min = y.saturating_sub(radius);
    //         let y_max = (y + radius).min(height - 1);
    //         let x_min = x.saturating_sub(radius);
    //         let x_max = (x + radius).min(width - 1);

    //         for ny in y_min..=y_max {
    //             for nx in x_min..=x_max {
    //                 let Rgba([r, g, b, _]) = img.get_pixel(nx, ny);
    //                 let intensity_index = ((r as u32 + g as u32 + b as u32) * intensity / (3 * 256)).min(intensity - 1) as usize;

    //                 intensity_count[intensity_index] += 1;
    //                 avg_red[intensity_index] += r as f32;
    //                 avg_green[intensity_index] += g as f32;
    //                 avg_blue[intensity_index] += b as f32;
    //             }
    //         }

    //         let mut max_index = 0;
    //         for i in 1..intensity as usize {
    //             if intensity_count[i] > intensity_count[max_index] {
    //                 max_index = i;
    //             }
    //         }

    //         let final_red = if intensity_count[max_index] > 0 {
    //             (avg_red[max_index] / intensity_count[max_index] as f32) as u8
    //         } else {
    //             0
    //         };
    //         let final_green = if intensity_count[max_index] > 0 {
    //             (avg_green[max_index] / intensity_count[max_index] as f32) as u8
    //         } else {
    //             0
    //         };
    //         let final_blue = if intensity_count[max_index] > 0 {
    //             (avg_blue[max_index] / intensity_count[max_index] as f32) as u8
    //         } else {
    //             0
    //         };

    //         out_buffer.put_pixel(x, y, Rgb([final_red, final_green, final_blue]));
    //     }
        
    //     if y % 5 == 0 {
    //         send_progress(&progress_tx, y as f64 / height as f64);
    //     }
    // }
    
    send_progress(&progress_tx, 1.0);
    out_buffer
}