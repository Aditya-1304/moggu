
use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb};
use crate::{ProgressSender, send_progress};
use rayon::prelude::*;

pub fn grayscale(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    send_progress(&progress_tx, 0.0);
    
    let gray_img = img.grayscale();
    let (width, height) = gray_img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);

    let pixels: Vec<_> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut row_pixels = Vec::with_capacity(width as usize);

            for x in 0..width {
                let pixel = gray_img.get_pixel(x, y);
                let gray_value = pixel[0];
                row_pixels.push((x,y, Rgb([gray_value, gray_value, gray_value])));
            }
            row_pixels
        })
        .collect();

    for row in pixels {
        for (x, y, pixel) in row {
            output.put_pixel(x, y, pixel);
        }
    }
    send_progress(&progress_tx, 1.0);
    output

}