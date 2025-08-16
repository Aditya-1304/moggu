use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::{iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};
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

    send_progress(&progress_tx, 1.0);
    out_buffer
}



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
    
    send_progress(&progress_tx, 1.0);
    out_buffer
}



pub fn noise(img: &DynamicImage, strength: u8, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {

    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
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
    
    send_progress(&progress_tx, 1.0);
    out_buffer
}



pub fn oil_painting(img: &DynamicImage, radius: u32, intensity: u32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    send_progress(&progress_tx, 0.0);

    let horizontal_result = horizontal_oil_pass(&rgb_img, radius, intensity, width, height);
    
    send_progress(&progress_tx, 0.5);
    
    let final_result = vertical_oil_pass(&horizontal_result, radius, intensity, width, height);
    
    send_progress(&progress_tx, 1.0);
    
    ImageBuffer::from_vec(width, height, final_result).unwrap()
}



fn horizontal_oil_pass(img: &ImageBuffer<Rgb<u8>, Vec<u8>>, radius: u32, intensity: u32, width: u32, height: u32) -> Vec<u8> {
    let mut result = vec![0u8; (width * height * 3) as usize];
    let pixels = img.as_raw();
    
    result.par_chunks_exact_mut((width * 3) as usize).enumerate().for_each(|(y, row)| {
        let row_start = (y as u32 * width * 3) as usize;
        
        let mut intensity_count = vec![0u32; intensity as usize];
        let mut avg_red = vec![0.0f32; intensity as usize];
        let mut avg_green = vec![0.0f32; intensity as usize];
        let mut avg_blue = vec![0.0f32; intensity as usize];
        
        let x_min = 0u32.saturating_sub(radius);
        let x_max = (0 + radius).min(width - 1);
        
        for kx in x_min..=x_max {
            let idx = row_start + (kx * 3) as usize;
            let red = pixels[idx];
            let green = pixels[idx + 1];
            let blue = pixels[idx + 2];
            
            let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                .min(intensity - 1) as usize;
            
            intensity_count[intensity_idx] += 1;
            avg_red[intensity_idx] += red as f32;
            avg_green[intensity_idx] += green as f32;
            avg_blue[intensity_idx] += blue as f32;
        }
        
        let max_idx = find_dominant_intensity(&intensity_count);
        set_oil_pixel(row, 0, max_idx, &intensity_count, &avg_red, &avg_green, &avg_blue);
        
        for x in 1..width {
            let new_x_min = x.saturating_sub(radius);
            let new_x_max = (x + radius).min(width - 1);
            let prev_x_min = (x - 1).saturating_sub(radius);
            let prev_x_max = ((x - 1) + radius).min(width - 1);
            
            if new_x_min > prev_x_min {
                let remove_idx = row_start + (prev_x_min * 3) as usize;
                let red = pixels[remove_idx];
                let green = pixels[remove_idx + 1]; 
                let blue = pixels[remove_idx + 2];
                
                let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                    .min(intensity - 1) as usize;
                
                intensity_count[intensity_idx] -= 1;
                avg_red[intensity_idx] -= red as f32;
                avg_green[intensity_idx] -= green as f32;
                avg_blue[intensity_idx] -= blue as f32;
            }
            
            if new_x_max > prev_x_max {
                let add_idx = row_start + (new_x_max * 3) as usize;
                let red = pixels[add_idx];
                let green = pixels[add_idx + 1];
                let blue = pixels[add_idx + 2];
                
                let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                    .min(intensity - 1) as usize;
                
                intensity_count[intensity_idx] += 1;
                avg_red[intensity_idx] += red as f32;
                avg_green[intensity_idx] += green as f32;
                avg_blue[intensity_idx] += blue as f32;
            }
            
            let max_idx = find_dominant_intensity(&intensity_count);
            set_oil_pixel(row, x, max_idx, &intensity_count, &avg_red, &avg_green, &avg_blue);
        }
    });
    
    result
}



fn vertical_oil_pass(pixels: &[u8], radius: u32, intensity: u32, width: u32, height: u32) -> Vec<u8> {
    let result = vec![0u8; (width * height * 3) as usize];
    
    (0..width).into_par_iter().for_each(|x| {

        let mut intensity_count = vec![0u32; intensity as usize];
        let mut avg_red = vec![0.0f32; intensity as usize];
        let mut avg_green = vec![0.0f32; intensity as usize];
        let mut avg_blue = vec![0.0f32; intensity as usize];
        
        let y_min = 0u32.saturating_sub(radius);
        let y_max = (0 + radius).min(height - 1);
        
        for ky in y_min..=y_max {
            let idx = ((ky * width + x) * 3) as usize;
            let red = pixels[idx];
            let green = pixels[idx + 1];
            let blue = pixels[idx + 2];
            
            let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                .min(intensity - 1) as usize;
            
            intensity_count[intensity_idx] += 1;
            avg_red[intensity_idx] += red as f32;
            avg_green[intensity_idx] += green as f32;
            avg_blue[intensity_idx] += blue as f32;
        }
        
        let out_idx = (x * 3) as usize;
        let max_idx = find_dominant_intensity(&intensity_count);
        unsafe {
            let result_ptr = result.as_ptr() as *mut u8;
            set_oil_pixel_unsafe(result_ptr, out_idx, max_idx, &intensity_count, &avg_red, &avg_green, &avg_blue);
        }
        
        for y in 1..height {
            let new_y_min = y.saturating_sub(radius);
            let new_y_max = (y + radius).min(height - 1);
            let prev_y_min = (y - 1).saturating_sub(radius);
            let prev_y_max = ((y - 1) + radius).min(height - 1);
            
            if new_y_min > prev_y_min {
                let remove_idx = ((prev_y_min * width + x) * 3) as usize;
                let red = pixels[remove_idx];
                let green = pixels[remove_idx + 1];
                let blue = pixels[remove_idx + 2];
                
                let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                    .min(intensity - 1) as usize;
                
                intensity_count[intensity_idx] -= 1;
                avg_red[intensity_idx] -= red as f32;
                avg_green[intensity_idx] -= green as f32;
                avg_blue[intensity_idx] -= blue as f32;
            }
            
            if new_y_max > prev_y_max {
                let add_idx = ((new_y_max * width + x) * 3) as usize;
                let red = pixels[add_idx];
                let green = pixels[add_idx + 1];
                let blue = pixels[add_idx + 2];
                
                let intensity_idx = ((red as u32 + green as u32 + blue as u32) * intensity / (3 * 256))
                    .min(intensity - 1) as usize;
                
                intensity_count[intensity_idx] += 1;
                avg_red[intensity_idx] += red as f32;
                avg_green[intensity_idx] += green as f32;
                avg_blue[intensity_idx] += blue as f32;
            }
            
            let out_idx = ((y * width + x) * 3) as usize;
            let max_idx = find_dominant_intensity(&intensity_count);
            unsafe {
                let result_ptr = result.as_ptr() as *mut u8;
                set_oil_pixel_unsafe(result_ptr, out_idx, max_idx, &intensity_count, &avg_red, &avg_green, &avg_blue);
            }
        }
    });
    
    result
}

#[inline(always)]
fn find_dominant_intensity(intensity_count: &[u32]) -> usize {
    intensity_count
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

#[inline(always)]
fn set_oil_pixel(row: &mut [u8], x: u32, max_idx: usize, intensity_count: &[u32], avg_red: &[f32], avg_green: &[f32], avg_blue: &[f32]) {
    let out_idx = (x * 3) as usize;
    if intensity_count[max_idx] > 0 {
        let count = intensity_count[max_idx] as f32;
        row[out_idx] = (avg_red[max_idx] / count) as u8;
        row[out_idx + 1] = (avg_green[max_idx] / count) as u8;
        row[out_idx + 2] = (avg_blue[max_idx] / count) as u8;
    } else {
        row[out_idx] = 0;
        row[out_idx + 1] = 0;
        row[out_idx + 2] = 0;
    }
}

#[inline(always)]
unsafe fn set_oil_pixel_unsafe(result_ptr: *mut u8, out_idx: usize, max_idx: usize, intensity_count: &[u32], avg_red: &[f32], avg_green: &[f32], avg_blue: &[f32]) {
    if intensity_count[max_idx] > 0 {
        let count = intensity_count[max_idx] as f32;
        unsafe {
            *result_ptr.add(out_idx) = (avg_red[max_idx] / count) as u8;
            *result_ptr.add(out_idx + 1) = (avg_green[max_idx] / count) as u8;
            *result_ptr.add(out_idx + 2) = (avg_blue[max_idx] / count) as u8;
        }
    } else {
        unsafe {
            *result_ptr.add(out_idx) = 0;
            *result_ptr.add(out_idx + 1) = 0;
            *result_ptr.add(out_idx + 2) = 0;
        }
    }
}