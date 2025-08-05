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
