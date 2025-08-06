
use image::imageops::FilterType;
use image::{GenericImageView, Pixel};
use std::env;
use std::fs;
use std::io::{self, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum ConversionMode {
  Grayscale,
  Ascii,
}

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        writeln!(io::stderr(), "Usage: {} <mode> <input_file> <output_file>", args[0])?;
        writeln!(io::stderr(), "Modes: grayscale, ascii")?;
        writeln!(io::stderr(), "Example: {} grayscale input.png output.png", args[0])?;
        writeln!(io::stderr(), "Example: {} ascii input.jpg art.txt", args[0])?;
        return Ok(());
    }

    let mode = match args[1].as_str() {
      "grayscale" => ConversionMode::Grayscale,
      "ascii" => ConversionMode::Ascii,
      _=> return Err(format!("Unknown mode: {}", args[1]).into()),
    };

    let input_path = &args[2];
    let output_path = &args[3];

    println!("Opening image: {}", input_path);
    let img = image::open(input_path)?;
    let format = image::ImageFormat::from_path(input_path).ok();
    println!("Image format detected: {:?}", format);
    println!("Image dimensions: {}x{}", img.width(), img.height());
    match mode {
      ConversionMode::Grayscale => {
        println!("Converting to grayscale...");
        let grayscale_img = img.grayscale();
        println!("Saving grayscale image to: {}", output_path);
        grayscale_img.save(output_path)?;
      }
      ConversionMode::Ascii => {

        println!("Converting to ASCII art...");

        let ascii_art = to_ascii(&img);
        println!("Saving ASCII art to: {}", output_path);
        fs::write(output_path, ascii_art)?;
      }
    }
    println!("Operation complete.");
    Ok(())

}


fn to_ascii(img: &image::DynamicImage) -> String {
    const ASCII_CHARS: &[char] = &['@', '%', '#', '*', '+', '=', '-', ':', '.', ' '];

    const MAX_DIMENSION: u32 = 80;


    let (width, height) = img.dimensions();

    let (new_width, new_height) = if width > height {
      let new_width = MAX_DIMENSION;
      let new_height = (new_width as f32 * height as f32 / width as f32) as u32;
      (new_width, new_height)
    } else {
      let new_height = MAX_DIMENSION;
      let new_width = (new_height as f32 * width as f32 / height as f32) as u32;
      (new_width, new_height)
    };

    let corrected_height = (new_height as f32 * 0.5) as u32;

    let small_img = img.resize_exact(new_width, corrected_height, FilterType::Lanczos3);

    let mut ascii_art = String::new();

    for p in small_img.pixels() {
      // p is a tuple (x, y, pixel)
      let (x,_, pixel) = p;

      let luma = pixel.to_luma();
      let brightness = luma[0];

      let char_index = (brightness as f32 / 255.0 * (ASCII_CHARS.len() - 1) as f32) as usize;

      let character = ASCII_CHARS[char_index];

      ascii_art.push(character);

      if x == small_img.width() - 1 {
        ascii_art.push('\n');
      }
    }
    ascii_art
}