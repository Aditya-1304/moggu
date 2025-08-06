use image::imageops::FilterType;
use image::{GenericImageView, Pixel, Rgba};
use std::env;
use std::fs;
use std::io::{self, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
enum ConversionMode {
    Grayscale,
    Ascii,
    DetailedAscii,
    ColorAscii,
    ColorAsciiTerminal,
}

#[derive(Clone)]
struct AsciiConfig {
    max_width: u32,
    contrast_boost: f32,
    invert: bool,
    detailed: bool,
    preserve_aspect: bool,
    supersample: u32,
    edge_enchance: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            max_width: 120,
            contrast_boost: 1.2,
            invert: false,
            detailed: false,
            preserve_aspect: true,
            supersample: 1,
            edge_enchance: false,
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        print_usage(&args[0]);
        return Ok(());
    }

    let mode = match args[1].as_str() {
        "grayscale" => ConversionMode::Grayscale,
        "ascii" => ConversionMode::Ascii,
        "ascii-detailed" => ConversionMode::DetailedAscii,
        "ascii-color" => ConversionMode::ColorAscii,
        "ascii-color-term" => ConversionMode::ColorAsciiTerminal, // New mode
        _ => return Err(format!("Unknown mode: {}", args[1]).into()),
    };

    let input_path = &args[2];
    let output_path = &args[3];
    let mut config = AsciiConfig::default();
    if args.len() > 4 {
        for i in 4..args.len() {
            match args[i].as_str() {
                "--invert" => config.invert = true,
                "--detailed" => config.detailed = true,
                "--no-aspect" => config.preserve_aspect = false,
                s if s.starts_with("--width=") => {
                    if let Ok(width) = s.trim_start_matches("--width=").parse::<u32>() {
                        config.max_width = width;
                    }
                }
                s if s.starts_with("--contrast=") => {
                    if let Ok(contrast) = s.trim_start_matches("--contrast=").parse::<f32>() {
                        config.contrast_boost = contrast;
                    }
                }
                _ => {}
            }
        }
    }

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
            let ascii_art = to_ascii(&img, &config)?;
            println!("Saving ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::DetailedAscii => {
            println!("Converting to detailed ASCII art...");
            config.detailed = true;
            let ascii_art = to_ascii(&img, &config)?;
            println!("Saving detailed ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::ColorAscii => {
            println!("Converting to colored ASCII art...");
            let ascii_art = to_color_ascii(&img, &config)?;
            println!("Saving colored ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::ColorAsciiTerminal => {
            println!("Converting to colored ASCII art for terminal display...");
            let ascii_art = to_color_ascii(&img, &config)?;
            println!("Displaying colored ASCII art:");
            println!("{}", ascii_art);
            println!("Also saving to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
    }

    println!("Operation complete.");
    Ok(())
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <mode> <input_file> <output_file> [options]", program_name);
    eprintln!("Modes:");
    eprintln!("  grayscale         - Convert to grayscale image");
    eprintln!("  ascii             - Convert to ASCII art");
    eprintln!("  ascii-detailed    - Convert to detailed ASCII art");
    eprintln!("  ascii-color       - Convert to colored ASCII art (for file)");
    eprintln!("  ascii-color-term  - Convert to colored ASCII art (for terminal)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --width=N     - Set maximum width (default: 120)");
    eprintln!("  --contrast=F  - Set contrast boost (default: 1.2)");
    eprintln!("  --invert      - Invert brightness values");
    eprintln!("  --detailed    - Use detailed character set");
    eprintln!("  --no-aspect   - Don't preserve aspect ratio");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} grayscale input.png output.png", program_name);
    eprintln!("  {} ascii input.jpg art.txt --width=80", program_name);
    eprintln!("  {} ascii-color-term input.png output.txt --width=60", program_name);
}

fn to_ascii(img: &image::DynamicImage, config: &AsciiConfig) -> Result<String> {
    // Different character sets for different quality levels
    let ascii_chars = if config.detailed {
        &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█', 
          '@', '&', '%', '$', '#', 'W', 'M', 'B', 'R', 'N', 'H', 'Q', 'K', 'A', 'G', 
          'O', 'C', 'V', 'U', 'Y', 'X', 'Z', 'b', 'd', 'h', 'k', 'p', 'q', 'w', 'm', 
          '*', 'o', 'a', 'h', 'k', 'b', 'd', 'p', 'q', 'w', 'm', 'Z', 'O', '0', 'Q', 
          'L', 'C', 'J', 'U', 'Y', 'X', 'z', 'c', 'v', 'u', 'n', 'x', 'r', 'j', 'f', 
          't', '/', '\\', '|', '(', ')', '1', '{', '}', '[', ']', '?', '-', '_', '+', 
          '~', '<', '>', 'i', '!', 'l', 'I', ';', ':', ',', '"', '^', '`', '\'', '.', ' '][..]
    } else {
        &['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.', ' '][..]
    };

    let (width, height) = img.dimensions();
    
    let aspect_ratio = width as f32 / height as f32;
    let char_aspect_ratio = 0.5;
    
    let (new_width, new_height) = if config.preserve_aspect {
        if width > config.max_width {
            let new_width = config.max_width;
            let new_height = ((new_width as f32 / aspect_ratio) * char_aspect_ratio) as u32;
            (new_width, new_height)
        } else {
            let new_width = width;
            let new_height = ((height as f32) * char_aspect_ratio) as u32;
            (new_width, new_height)
        }
    } else {
        (config.max_width, config.max_width / 2)
    };

    println!("Resizing to: {}x{}", new_width, new_height);

    let filter = if new_width < 50 || new_height < 50 {
        FilterType::CatmullRom
    } else {
        FilterType::Lanczos3
    };

    let resized_img = img.resize_exact(new_width, new_height, filter);
    let gray_img = resized_img.grayscale();

    let mut ascii_art = String::with_capacity((new_width * new_height + new_height) as usize);

    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = gray_img.get_pixel(x, y);
            let mut brightness = pixel[0] as f32;

            // Apply contrast boost
            brightness = ((brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
            brightness = brightness.clamp(0.0, 255.0);

            // Invert if requested
            if config.invert {
                brightness = 255.0 - brightness;
            }

            // Apply gamma correction for better visual perception
            let gamma_corrected = (brightness / 255.0).powf(0.8) * 255.0;

            let char_index = ((gamma_corrected / 255.0) * (ascii_chars.len() - 1) as f32) as usize;
            let char_index = char_index.min(ascii_chars.len() - 1);

            ascii_art.push(ascii_chars[char_index]);
        }
        ascii_art.push('\n');
    }

    Ok(ascii_art)
}
fn to_color_ascii(img: &image::DynamicImage, config: &AsciiConfig) -> Result<String> {
    // Better character set ordered by visual density
    let ascii_chars = if config.detailed {
        &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏', '░', '▒', '▓', '█',
          '@', '&', '%', '$', '#', 'W', 'M', 'B', 'R', 'N', 'H', 'Q', 
          'K', 'A', 'G', 'O', 'C', 'V', 'U', 'Y', 'X', 'Z', 'b', 'd', 
          'h', 'k', 'p', 'q', 'w', 'm', '*', 'o', 'a', 'h', 'k', 'b', 
          'd', 'p', 'q', 'w', 'm', 'Z', 'O', '0', 'Q', 'L', 'C', 'J', 
          'U', 'Y', 'X', 'z', 'c', 'v', 'u', 'n', 'x', 'r', 'j', 'f', 
          't', '/', '\\', '|', '(', ')', '1', '{', '}', '[', ']', '?', 
          '-', '_', '+', '~', '<', '>', 'i', '!', 'l', 'I', ';', ':', 
          ',', '"', '^', '`', '\'', '.', ' '][..]
    } else {
        &['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.', ' '][..]
    };
    
    let (width, height) = img.dimensions();
    
    // Fixed aspect ratio calculation
    let aspect_ratio = width as f32 / height as f32;
    let char_aspect_ratio = 0.45; // Better ratio for most terminals
    
    let (new_width, new_height) = if config.preserve_aspect {
        if width > config.max_width {
            let new_width = config.max_width;
            let new_height = ((new_width as f32 / aspect_ratio) * char_aspect_ratio) as u32;
            (new_width, new_height)
        } else {
            let new_width = width.min(config.max_width);
            let new_height = ((height as f32) * char_aspect_ratio) as u32;
            (new_width, new_height)
        }
    } else {
        (config.max_width, (config.max_width as f32 * 0.5) as u32)
    };

    println!("Resizing to: {}x{}", new_width, new_height);

    let resized_img = img.resize_exact(new_width, new_height, FilterType::Lanczos3);
    let mut ascii_art = String::with_capacity((new_width * new_height * 25 + new_height) as usize);

    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = resized_img.get_pixel(x, y);
            let Rgba([r, g, b, _]) = pixel;

            // Calculate brightness using proper luminance formula
            let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32);
            
            // Apply contrast boost
            let adjusted_brightness = ((brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
            let clamped_brightness = adjusted_brightness.clamp(0.0, 255.0);
            
            // Invert if requested
            let final_brightness = if config.invert {
                255.0 - clamped_brightness
            } else {
                clamped_brightness
            };

            // Select character based on brightness
            let char_index = ((final_brightness / 255.0) * (ascii_chars.len() - 1) as f32) as usize;
            let char_index = char_index.min(ascii_chars.len() - 1);
            let chosen_char = ascii_chars[char_index];

            // Create proper ANSI color escape sequence
            ascii_art.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                r, g, b, chosen_char
            ));
        }
        ascii_art.push('\n');
    }

    Ok(ascii_art)
}

// Add edge detection for better detail preservation
fn detect_edges(img: &image::DynamicImage) -> image::GrayImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();
    let mut edges = image::GrayImage::new(width, height);

    // Simple Sobel edge detection
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let gx = 
                -1 * gray.get_pixel(x - 1, y - 1)[0] as i32 +
                 1 * gray.get_pixel(x + 1, y - 1)[0] as i32 +
                -2 * gray.get_pixel(x - 1, y)[0] as i32 +
                 2 * gray.get_pixel(x + 1, y)[0] as i32 +
                -1 * gray.get_pixel(x - 1, y + 1)[0] as i32 +
                 1 * gray.get_pixel(x + 1, y + 1)[0] as i32;

            let gy = 
                -1 * gray.get_pixel(x - 1, y - 1)[0] as i32 +
                -2 * gray.get_pixel(x, y - 1)[0] as i32 +
                -1 * gray.get_pixel(x + 1, y - 1)[0] as i32 +
                 1 * gray.get_pixel(x - 1, y + 1)[0] as i32 +
                 2 * gray.get_pixel(x, y + 1)[0] as i32 +
                 1 * gray.get_pixel(x + 1, y + 1)[0] as i32;

            let magnitude = ((gx * gx + gy * gy) as f32).sqrt() as u8;
            edges.put_pixel(x, y, image::Luma([magnitude]));
        }
    }

    edges
}