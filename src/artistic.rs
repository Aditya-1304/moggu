use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use crate::{ProgressSender, send_progress};
use rand::Rng;

/// Apply sepia effect
pub fn sepia(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
        
            let red_filter = r as f32;
            let green_filter = g as f32;
            let blue_filter = b as f32;

            let new_red = (red_filter * 0.393) + (green_filter * 0.769) + (blue_filter * 0.189);
            let new_green = (red_filter * 0.349) + (green_filter * 0.686) + (blue_filter * 0.168);
            let new_blue = (red_filter * 0.272) + (green_filter * 0.534) + (blue_filter * 0.131);

            output.put_pixel(x, y, Rgb([
                new_red.min(255.0) as u8,
                new_green.min(255.0) as u8,
                new_blue.min(255.0) as u8,
            ]));
        }

        if y % 25 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }

    send_progress(&progress_tx, 1.0);
    output
}

/// Apply vignette effect
pub fn vignette(img: &DynamicImage, strength: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_distance = ((center_x * center_x) + (center_y * center_y)).sqrt();
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            
            let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            let vignette_factor = 1.0 - (distance / max_distance * strength).clamp(0.0, 1.0);
            
            let new_red = (r as f32 * vignette_factor) as u8;
            let new_green = (g as f32 * vignette_factor) as u8;
            let new_blue = (b as f32 * vignette_factor) as u8;
            
            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
        
        if y % 15 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Add noise to image
pub fn noise(img: &DynamicImage, strength: u8, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    let mut rng = rand::rng();
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            
            let noise_r = rng.random_range(-(strength as i32)..=strength as i32);
            let noise_g = rng.random_range(-(strength as i32)..=strength as i32);
            let noise_b = rng.random_range(-(strength as i32)..=strength as i32);
            
            let new_red = (r as i32 + noise_r).clamp(0, 255) as u8;
            let new_green = (g as i32 + noise_g).clamp(0, 255) as u8;
            let new_blue = (b as i32 + noise_b).clamp(0, 255) as u8;
            
            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
        
        if y % 20 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Apply oil painting effect
pub fn oil_painting(img: &DynamicImage, radius: u32, intensity: u32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let mut intensity_count = vec![0; intensity as usize];
            let mut avg_red = vec![0.0; intensity as usize];
            let mut avg_green = vec![0.0; intensity as usize];
            let mut avg_blue = vec![0.0; intensity as usize];

            let y_min = y.saturating_sub(radius);
            let y_max = (y + radius).min(height - 1);
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(width - 1);

            for ny in y_min..=y_max {
                for nx in x_min..=x_max {
                    let Rgba([r, g, b, _]) = img.get_pixel(nx, ny);
                    let intensity_index = ((r as u32 + g as u32 + b as u32) * intensity / (3 * 256)).min(intensity - 1) as usize;

                    intensity_count[intensity_index] += 1;
                    avg_red[intensity_index] += r as f32;
                    avg_green[intensity_index] += g as f32;
                    avg_blue[intensity_index] += b as f32;
                }
            }

            let mut max_index = 0;
            for i in 1..intensity as usize {
                if intensity_count[i] > intensity_count[max_index] {
                    max_index = i;
                }
            }

            let final_red = if intensity_count[max_index] > 0 {
                (avg_red[max_index] / intensity_count[max_index] as f32) as u8
            } else {
                0
            };
            let final_green = if intensity_count[max_index] > 0 {
                (avg_green[max_index] / intensity_count[max_index] as f32) as u8
            } else {
                0
            };
            let final_blue = if intensity_count[max_index] > 0 {
                (avg_blue[max_index] / intensity_count[max_index] as f32) as u8
            } else {
                0
            };

            output.put_pixel(x, y, Rgb([final_red, final_green, final_blue]));
        }
        
        if y % 5 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}