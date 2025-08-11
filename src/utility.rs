use image::{DynamicImage,GenericImageView, GrayImage};
use crate::{ProgressSender, send_progress, AsciiConfig, Result};

/// Crop image to specified rectangle
pub fn crop(img: &DynamicImage, x: u32, y: u32, width: u32, height: u32, progress_tx: Option<ProgressSender>) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    use image::{ImageBuffer, Rgb, Rgba};
    
    send_progress(&progress_tx, 0.0);
    
    let (img_width, img_height) = img.dimensions();
    let crop_width = width.min(img_width.saturating_sub(x));
    let crop_height = height.min(img_height.saturating_sub(y));
    
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(crop_width, crop_height);
    
    for crop_y in 0..crop_height {
        for crop_x in 0..crop_width {
            let source_x = x + crop_x;
            let source_y = y + crop_y;
            
            if source_x < img_width && source_y < img_height {
                let Rgba([r, g, b, _]) = img.get_pixel(source_x, source_y);
                output.put_pixel(crop_x, crop_y, Rgb([r, g, b]));
            }
        }
        
        if crop_y % 20 == 0 {
            send_progress(&progress_tx, crop_y as f64 / crop_height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Convert image to ASCII art with dithering
pub fn to_ascii_dithered(img: &DynamicImage, config: &AsciiConfig, progress_tx: Option<ProgressSender>) -> Result<String> {
    let ascii_chars_detailed = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
    let ascii_chars_simple = "@%#*+=-:. ";
    
    let char_ramp: Vec<char> = if config.detailed {
        ascii_chars_detailed.chars().collect()
    } else {
        ascii_chars_simple.chars().collect()
    };
    let ramp_len = char_ramp.len() as f32;

    let (width, height) = img.dimensions();
    let aspect_ratio = height as f32 / width as f32;
    let new_width = config.max_width;
    let new_height = (new_width as f32 * aspect_ratio * 0.5) as u32;

    send_progress(&progress_tx, 0.1);

    let small_img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);
    let mut gray_img = small_img.to_luma8();

    send_progress(&progress_tx, 0.3);

    if config.dither {
        let mut f32_buffer: Vec<f32> = gray_img.pixels().map(|p| p[0] as f32).collect();

        for y in 0..new_height {
            for x in 0..new_width {
                let idx = (y * new_width + x) as usize;
                let old_pixel = f32_buffer[idx];
            
                let new_pixel = ((old_pixel / 255.0 * (ramp_len - 1.0)).round() / (ramp_len - 1.0)) * 255.0;
                f32_buffer[idx] = new_pixel;

                let quant_error = old_pixel - new_pixel;
    
                if x + 1 < new_width {
                    f32_buffer[idx + 1] += quant_error * 7.0 / 16.0;
                }
                if x > 0 && y + 1 < new_height {
                    f32_buffer[idx + new_width as usize - 1] += quant_error * 3.0 / 16.0;
                }
                if y + 1 < new_height {
                    f32_buffer[idx + new_width as usize] += quant_error * 5.0 / 16.0;
                }
                if x + 1 < new_width && y + 1 < new_height {
                    f32_buffer[idx + new_width as usize + 1] += quant_error * 1.0 / 16.0;
                }
            }
            
            if y % 10 == 0 {
                let progress = 0.3 + (y as f64 / new_height as f64) * 0.5;
                send_progress(&progress_tx, progress);
            }
        }
        
        let final_pixels: Vec<u8> = f32_buffer.iter().map(|&p| p.clamp(0.0, 255.0) as u8).collect();
        gray_img = GrayImage::from_raw(new_width, new_height, final_pixels).unwrap();
    }

    send_progress(&progress_tx, 0.8);
    
    let mut ascii_art = String::with_capacity((new_width * new_height + new_height) as usize);
    for (i, pixel) in gray_img.pixels().enumerate() {
        let mut brightness = pixel[0] as f32;

        brightness = ((brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
        brightness = brightness.clamp(0.0, 255.0);

        if config.invert {
            brightness = 255.0 - brightness;
        }

        let char_index = ((brightness / 255.0) * (ramp_len - 1.0)).round() as usize;
        ascii_art.push(char_ramp[char_index]);

        if (i + 1) % new_width as usize == 0 {
            ascii_art.push('\n');
        }
    }

    send_progress(&progress_tx, 1.0);
    Ok(ascii_art)
}