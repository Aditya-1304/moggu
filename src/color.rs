use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb};
use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};
use crate::{ProgressSender, send_progress};

pub fn saturate(img: &DynamicImage, factor: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
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
            let (h, s, l) = rgb_to_hsl(in_pixel[0], in_pixel[1], in_pixel[2]);
            let new_saturation = (s * factor).clamp(0.0, 0.1);
            let (new_red, new_green, new_blue) = hsl_to_rgb(h, new_saturation, l);

            out_pixel[0] = new_red;
            out_pixel[1] = new_green;
            out_pixel[2] = new_blue;
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}

pub fn invert(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
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
            out_pixel[0] = 255 - in_pixel[0];
            out_pixel[1] = 255 - in_pixel[1];
            out_pixel[2] = 255 - in_pixel[2];
        });
    send_progress(&progress_tx, 1.0);
    out_buffer
}

pub fn hue_rotate(img: &DynamicImage, degrees: f32, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();
    let out_pixels = out_buffer.as_mut();

    in_pixels
        .par_chunks_exact(3)
        .zip(out_pixels.par_chunks_exact_mut(3))
        .for_each(|(in_pixel, out_pixel)| {
            let (mut h, s, l) = rgb_to_hsl(in_pixel[0], in_pixel[1], in_pixel[2]);

            h = (h + degrees) % 360.0;
            if h < 0.0 {
                h += 360.0;
            }

            let (new_red, new_green, new_blue) = hsl_to_rgb(h, s, l);
            out_pixel[0] = new_red;
            out_pixel[1] = new_green;
            out_pixel[2] = new_blue;

        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}

#[inline(always)]
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let red_normal = r as f32 * ( 1.0 / 255.0);
    let green_normal = g as f32 * ( 1.0 / 255.0);
    let blue_normal = b as f32 * ( 1.0 / 255.0);

    let max = red_normal.max(green_normal.max(blue_normal));
    let min = red_normal.min(green_normal.min(blue_normal));

    let lightness = (max + min) * 0.5;
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
            x if x == red_normal => {
                let h = (green_normal - blue_normal) / diff;
                if green_normal < blue_normal { h + 6.0 } else { h }
            },
            x if x == green_normal => (blue_normal - red_normal) / diff + 2.0,
            _ => (red_normal - green_normal) / diff + 4.0,
        };
        hue *= 60.0;
    }
    
    (hue , saturation, lightness)
}


#[inline(always)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let val = (l * 255.0) as u8;
        return (val, val, val);
    }

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c * 0.5;

    let (red, green, blue) = match h {
        h if h >= 0.0 && h < 60.0 => (c, x, 0.0),
        h if h >= 60.0 && h < 120.0 => (x, c, 0.0),
        h if h >= 120.0 && h < 180.0 => (0.0, c, x),
        h if h >= 180.0 && h < 240.0 => (0.0, x, c),
        h if h >= 240.0 && h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((red + m) * 255.0) as u8,
        ((green + m) * 255.0) as u8,
        ((blue + m) * 255.0) as u8,
    )
}