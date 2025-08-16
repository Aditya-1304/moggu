use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSliceMut};
use crate::{ProgressSender, send_progress};

pub fn rotate90(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(height, width);
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();

    out_buffer.as_mut()
        .par_chunks_exact_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(new_y, out_row)| {
            for new_x in 0..height {
                let old_x = new_y as u32;
                let old_y = height -1 - new_x;

                let in_idx = ((old_y * width + old_x) * 3) as usize;
                let out_idx = (new_x * 3) as usize;

                out_row[out_idx] = in_pixels[in_idx];
                out_row[out_idx + 1] = in_pixels[in_idx + 1];
                out_row[out_idx + 2] = in_pixels[in_idx + 2];
            }
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}



pub fn rotate180(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);

    let in_pixels = rgb_img.as_raw();

    out_buffer.as_mut()
        .par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {

            let old_y = height - 1 - y as u32;

            for x in 0..width {
                let old_x = width - 1 - x;

                let in_idx = ((old_y * width + old_x) * 3) as usize;
                let out_idx = (x * 3) as usize;

                out_row[out_idx] = in_pixels[in_idx];
                out_row[out_idx + 1] = in_pixels[in_idx + 1];
                out_row[out_idx + 2] = in_pixels[in_idx + 2];
            }

        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}



pub fn rotate270(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut out_buffer = ImageBuffer::new(height, width);
    send_progress(&progress_tx, 0.0);
    
    let in_pixels = rgb_img.as_raw();

    out_buffer.as_mut().par_chunks_exact_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(new_y, out_row)| {
            
            for new_x in 0..height {

                let old_x = height - 1 - new_y as u32;
                let old_y = new_x;
                
                if old_x < height && old_y < width {
                    let in_idx = ((old_y * width + old_x) * 3) as usize;
                    let out_idx = (new_x * 3) as usize;
                    
                    out_row[out_idx] = in_pixels[in_idx];
                    out_row[out_idx + 1] = in_pixels[in_idx + 1];
                    out_row[out_idx + 2] = in_pixels[in_idx + 2];
                }
            }
        });

    send_progress(&progress_tx, 1.0);
    out_buffer
}



pub fn flip_horizontal(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut output = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);
    
    let in_pixels = rgb_img.as_raw();

    output.as_mut().par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            
            for x in 0..width {
                let old_x = width - 1 - x;
                
                let in_idx = ((y as u32 * width + old_x) * 3) as usize;
                let out_idx = (x * 3) as usize;
                
                out_row[out_idx] = in_pixels[in_idx];
                out_row[out_idx + 1] = in_pixels[in_idx + 1];
                out_row[out_idx + 2] = in_pixels[in_idx + 2];
            }
        });

    send_progress(&progress_tx, 1.0);
    output
}



pub fn flip_vertical(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let mut output = ImageBuffer::new(width, height);
    
    send_progress(&progress_tx, 0.0);
    
    let in_pixels = rgb_img.as_raw();

    output.as_mut().par_chunks_exact_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, out_row)| {
            let old_y = height - 1 - y as u32;
            let src_start = (old_y * width * 3) as usize;
            out_row.copy_from_slice(&in_pixels[src_start..src_start + (width * 3) as usize]);
        });

    send_progress(&progress_tx, 1.0);
    output
}