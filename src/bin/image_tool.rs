
use image::{GenericImageView, Pixel};
use std::env;
use std::fs;
use std::io::{self, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// #[derive(Debug)]
// struct PpmHeader {
//     format: String,
//     width: u32,
//     _height: u32,
//     _max_val: u32,
//     header_len: usize,Rgb
// }

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

    // println!("Reading image from: {}", input_path);

    // let image_bytes = fs::read(input_path)?;

    // let header = parse_ppm_header(&image_bytes)?;
    // println!("Parsed header: {:?}", header);

    // if header.format != "P6" {
    //     return Err("Only P6 (binary) PPM format is supported.".into());
    // }

    // let pixel_data = &image_bytes[header.header_len..];

    println!("Opening image: {}", input_path);
    let img = image::open(input_path)?;
    let format = image::ImageFormat::from_path(input_path).ok();
    println!("Image format detected: {:?}", format);
    println!("Image dimensions: {}x{}", img.width(), img.height());
    match mode {
      ConversionMode::Grayscale => {
        // println!("Converting to grayscale...");
        // let grayscale_data = to_grayscale(pixel_data);
        // println!("Conversation complete.");

        // let mut output_bytes = Vec::new();
        // output_bytes.extend_from_slice(&image_bytes[..header.header_len]);
        // output_bytes.extend_from_slice(&grayscale_data);

        // fs::write(output_path, output_bytes)?;

        println!("Converting to grayscale...");
        let grayscale_img = img.grayscale();
        println!("Saving grayscale image to: {}", output_path);
        grayscale_img.save(output_path)?;
      }
      ConversionMode::Ascii => {
        // println!("Converting to ASCII art...");
        // let ascii_art = to_ascii(pixel_data, header.width);
        // println!("Conversion complete.");

        // fs::write(output_path, ascii_art)?;
        // println!("Successfully saved ASCII art to: {}", output_path);

        println!("Converting to ASCII art...");

        let ascii_art = to_ascii(&img);
        println!("Saving ASCII art to: {}", output_path);
        fs::write(output_path, ascii_art)?;
      }
    }
    println!("Operation complete.");
    Ok(())

}


// fn parse_ppm_header(bytes: &[u8]) -> Result<PpmHeader> {
//     let mut iter = bytes.iter();

//     let mut get_next_token = || -> Result<String> {
//         let token_bytes: Vec<u8> = iter
//             .by_ref()
//             .skip_while(|&&b| b.is_ascii_whitespace())
//             .take_while(|&&b| !b.is_ascii_whitespace())
//             .cloned()
//             .collect();
//         String::from_utf8(token_bytes).map_err(|e| e.into())
//     };

//     let format = get_next_token()?;
//     let width_str = get_next_token()?;
//     let height_str = get_next_token()?;
//     let max_val_str = get_next_token()?;

//     let header_len = format.len() + width_str.len() + height_str.len() + max_val_str.len() + 4;

//     Ok(PpmHeader {
//         format,
//         width: width_str.parse()?,
//         height: height_str.parse()?,
//         max_value: max_val_str.parse()?,
//         header_len,
//     })
// }
// fn parse_ppm_header(bytes: &[u8]) -> Result<PpmHeader> {
//     let mut parts = bytes.split(|b| b.is_ascii_whitespace()).filter(|s| !s.is_empty());

//     let format_slice = parts.next().ok_or("Parsing error: Missing format")?;
//     let width_slice = parts.next().ok_or("Parsing error: Missing width")?;
//     let height_slice = parts.next().ok_or("Parsing error: Missing height")?;
//     let max_val_slice = parts.next().ok_or("Parsing error: Missing max value")?;

//     // --- BUG FIX ---
//     // Perform pointer arithmetic on the ORIGINAL slices from the iterator.
//     // This correctly calculates the position of the end of the 4th token.
//     let header_end_pos = max_val_slice.as_ptr() as usize - bytes.as_ptr() as usize + max_val_slice.len();
    
//     // Find the single whitespace character that follows the max_val token to get the exact header length.
//     let header_len = bytes[header_end_pos..].iter().position(|b| b.is_ascii_whitespace()).unwrap_or(0) + header_end_pos + 1;

//     // Now it is safe to convert the slices to owned Strings for parsing.
//     let format = String::from_utf8(format_slice.to_vec())?;
//     let width_str = String::from_utf8(width_slice.to_vec())?;
//     let height_str = String::from_utf8(height_slice.to_vec())?;
//     let max_val_str = String::from_utf8(max_val_slice.to_vec())?;

//     Ok(PpmHeader {
//         format,
//         width: width_str.parse()?,
//         _height: height_str.parse()?,
//         _max_val: max_val_str.parse()?,
//         header_len,
//     })
// }


// fn to_grayscale(data: &[u8]) -> Vec<u8> {
//     let mut grayscale_data = Vec::with_capacity(data.len());

//     for pixel in data.chunks_exact(3) {
//         let r = pixel[0] as f32;
//         let g = pixel[1] as f32;
//         let b = pixel[2] as f32;

//         let gray_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

//         grayscale_data.push(gray_val);
//         grayscale_data.push(gray_val);
//         grayscale_data.push(gray_val);

//     }

//     grayscale_data
// }


fn to_ascii(img: &image::DynamicImage) -> String {
    const ASCII_CHARS: &[char] = &['@', '%', '#', '*', '+', '=', '-', ':', '.', ' '];

    let mut ascii_art = String::new();
    let (width, height) = img.dimensions();

    let y_step = 2;
    let x_step = 1;

    // for pixel in data.chunks_exact(3) {
    //   let r = pixel[0] as f32;
    //   let g = pixel[1] as f32;
    //   let b = pixel[2] as f32;

    //   let brightness = 0.299 * r + 0.587 * g + 0.114 * b;

    //   let char_index = (brightness / 255.0 * (ASCII_CHARS.len() - 1) as f32) as usize;

    //   ascii_art.push(ASCII_CHARS[char_index]);

    //   column += 1;

    //   if column >= width {
    //     ascii_art.push('\n');
    //     column = 0;
    //   }
    // }
    for y in (0..height).step_by(y_step) {
      for x in (0..width).step_by(x_step) {
        let pixel = img.get_pixel(x, y);
        let luma = pixel.to_luma();
        let brightness = luma[0];

        let char_index = (brightness as f32 / 255.0 * (ASCII_CHARS.len() - 1) as f32) as usize;

        let character = ASCII_CHARS[char_index];
        ascii_art.push(character);
        ascii_art.push(character);
      }
      ascii_art.push('\n');
    }

    ascii_art
}