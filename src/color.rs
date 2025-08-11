use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb, Rgba};
use crate::{ProgressSender, send_progress};

/// Adjust color saturation
pub fn saturate(img: &DynamicImage, factor: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let new_s = (s * factor).clamp(0.0, 1.0);
            let (new_red, new_green, new_blue) = hsl_to_rgb(h, new_s, l);

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
        
        if y % 15 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Invert image colors
pub fn invert(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            output.put_pixel(x, y, Rgb([255 - r, 255 - g, 255 - b]));
        }
        
        if y % 20 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

/// Rotate hue
pub fn hue_rotate(img: &DynamicImage, degrees: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            let (mut h, s, l) = rgb_to_hsl(r, g, b);
            
            h = (h + degrees) % 360.0;
            if h < 0.0 {
                h += 360.0;
            }
            
            let (new_red, new_green, new_blue) = hsl_to_rgb(h, s, l);
            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
        
        if y % 15 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}

// Helper functions for color space conversion
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let red_normal = r as f32 / 255.0;
    let green_normal = g as f32 / 255.0;
    let blue_normal = b as f32 / 255.0;

    let max = red_normal.max(green_normal.max(blue_normal));
    let min = red_normal.min(green_normal.min(blue_normal));

    let lightness = (max + min) / 2.0;
    let mut hue = 0.0;
    let mut saturation = 0.0;

    if max != min {
        let diff = max - min;
        saturation = if lightness > 0.5 {
            diff / (2.0 - max - min)
        } else {
            diff / (max + min)
        };
        
        hue = match max {
            x if x == red_normal => (green_normal - blue_normal) / diff + (if green_normal < blue_normal { 6.0 } else { 0.0 }),
            x if x == green_normal => (blue_normal - red_normal) / diff + 2.0,
            _ => (red_normal - green_normal) / diff + 4.0,
        };
        hue /= 6.0;
    }
    
    (hue * 360.0, saturation, lightness)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let val = (l * 255.0) as u8;
        return (val, val, val);
    }

    let hue_to_rgb = |p: f32, q: f32, mut t: f32| {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue_normal = h / 360.0;

    let red = hue_to_rgb(p, q, hue_normal + 1.0 / 3.0);
    let green = hue_to_rgb(p, q, hue_normal);
    let blue = hue_to_rgb(p, q, hue_normal - 1.0 / 3.0);

    ((red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8)
}