use std::env;
use std::fs;
use std::io::{self, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct PpmHeader {
    format: String,
    width: u32,
    height: u32,
    max_value: u32,
    header_len: usize,
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        writeln!(io::stderr(), "Usage: {} <input.ppm> <output.ppm>", args[0])?;

        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Reading image from: {}", input_path);

    let image_bytes = fs::read(input_path)?;

    let header = parse_ppm_header(&image_bytes)?;
    println!("Parsed header: {:?}", header);

    if header.format != "P6" {
        return Err("Only P6 (binary) PPM format is supported.".into());
    }

    let pixel_data = &image_bytes[header.header_len..];
    println!("Converting to grayscale...");

    let grayscale_data = to_grayscale(pixel_data);
    println!("Conversion complete.");

    let mut output_bytes = Vec::new();
    output_bytes.extend_from_slice(&image_bytes[..header.header_len]);
    output_bytes.extend_from_slice(&grayscale_data);

    fs::write(output_path, output_bytes)?;

    println!("Successfully saved grayscale image to: {}", output_path);

    Ok(())

}


fn parse_ppm_header(bytes: &[u8]) -> Result<PpmHeader> {
    let mut iter = bytes.iter();

    let mut get_next_token = || -> Result<String> {
        let token_bytes: Vec<u8> = iter
            .by_ref()
            .skip_while(|&&b| b.is_ascii_whitespace())
            .take_while(|&&b| !b.is_ascii_whitespace())
            .cloned()
            .collect();
        String::from_utf8(token_bytes).map_err(|e| e.into())
    };

    let format = get_next_token()?;
    let width_str = get_next_token()?;
    let height_str = get_next_token()?;
    let max_val_str = get_next_token()?;

    let header_len = format.len() + width_str.len() + height_str.len() + max_val_str.len() + 4;

    Ok(PpmHeader {
        format,
        width: width_str.parse()?,
        height: height_str.parse()?,
        max_value: max_val_str.parse()?,
        header_len,
    })
}

