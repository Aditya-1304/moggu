use image::{DynamicImage,GenericImageView, ImageBuffer, Rgb};
use crate::{ProgressSender, send_progress};

/// Convert image to grayscale
pub fn grayscale(img: &DynamicImage, progress_tx: Option<ProgressSender>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    send_progress(&progress_tx, 0.0);
    
    let gray_img = img.grayscale();
    let (width, height) = gray_img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let pixel = gray_img.get_pixel(x, y);
            let gray_value = pixel[0];
            output.put_pixel(x, y, Rgb([gray_value, gray_value, gray_value]));
        }
        
        if y % 20 == 0 {
            send_progress(&progress_tx, y as f64 / height as f64);
        }
    }
    
    send_progress(&progress_tx, 1.0);
    output
}