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
    edge_enhance: bool,
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
            edge_enhance: false,
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
        "ascii-color-term" => ConversionMode::ColorAsciiTerminal,
        "ascii-sharp" => ConversionMode::Ascii,
        _ => return Err(format!("Unknown mode: {}", args[1]).into()),
    };

    let input_path = &args[2];
    let output_path = &args[3];
    let mut config = AsciiConfig::default();

    if args[1] == "ascii-sharp" {
      config.supersample = 2;
      config.edge_enhance = true;
      config.detailed = true;
    }

    if args.len() > 4 {
        for i in 4..args.len() {
            match args[i].as_str() {
                "--invert" => config.invert = true,
                "--detailed" => config.detailed = true,
                "--no-aspect" => config.preserve_aspect = false,
                "--edge-enhance" => config.edge_enhance = true,
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
                s if s.starts_with("--supersample=") => {
                  if let Ok(sample) = s.trim_start_matches("--supersample=").parse::<u32>() {
                    config.supersample = sample.clamp(1, 4);
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
            let ascii_art = to_ascii_enhanced(&img, &config)?;
            println!("Saving ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::DetailedAscii => {
            println!("Converting to detailed ASCII art...");
            config.detailed = true;
            let ascii_art = to_ascii_enhanced(&img, &config)?;
            println!("Saving detailed ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::ColorAscii => {
            println!("Converting to colored ASCII art...");
            let ascii_art = to_color_ascii_enhanced(&img, &config)?;
            println!("Saving colored ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        ConversionMode::ColorAsciiTerminal => {
            println!("Converting to colored ASCII art for terminal display...");
            let ascii_art = to_color_ascii_enhanced(&img, &config)?;
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
    eprintln!("  ascii-sharp       - Convert to sharp ASCII art (supersampled)");
    eprintln!("  ascii-color       - Convert to colored ASCII art (for file)");
    eprintln!("  ascii-color-term  - Convert to colored ASCII art (for terminal)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --width=N         - Set maximum width (default: 120)");
    eprintln!("  --contrast=F      - Set contrast boost (default: 1.2)");
    eprintln!("  --supersample=N   - Supersample factor (1-4, default: 1)");
    eprintln!("  --invert          - Invert brightness values");
    eprintln!("  --detailed        - Use detailed character set");
    eprintln!("  --edge-enhance    - Enhance edges for sharpness");
    eprintln!("  --no-aspect       - Don't preserve aspect ratio");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} ascii-sharp input.jpg art.txt --width=80", program_name);
    eprintln!("  {} ascii input.jpg art.txt --supersample=2 --edge-enhance", program_name);
}

fn to_ascii_enhanced(img: &image::DynamicImage, config: &AsciiConfig) -> Result<String> {
    // Ultra-high quality character set optimized for sharpness
    let ascii_chars = if config.detailed {
        &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█',
          '▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟', '░', '▒', '▓', '█',
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
    let char_aspect_ratio = 0.43; // Optimized for text editors
    
    let (target_width, target_height) = if config.preserve_aspect {
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

    // Apply supersampling - render at higher resolution then downsample
    let sample_factor = config.supersample;
    let (render_width, render_height) = (
        target_width * sample_factor, 
        target_height * sample_factor
    );

    println!("Supersampling: {}x ({}x{} -> {}x{})", 
             sample_factor, render_width, render_height, target_width, target_height);

    // Use highest quality filter for supersampling
    let filter = FilterType::Lanczos3;
    let resized_img = img.resize_exact(render_width, render_height, filter);
    
    // Apply edge enhancement if requested
    let processed_img = if config.edge_enhance {
        apply_edge_enhancement(&resized_img, 0.3)?
    } else {
        resized_img.to_luma8()
    };

    let mut ascii_art = String::with_capacity((target_width * target_height + target_height) as usize);

    // Downsample with averaging for anti-aliasing
    for target_y in 0..target_height {
        for target_x in 0..target_width {
            let mut total_brightness = 0.0;
            let mut pixel_count = 0;

            // Sample all pixels in the supersample block
            for sample_y in 0..sample_factor {
                for sample_x in 0..sample_factor {
                    let src_x = target_x * sample_factor + sample_x;
                    let src_y = target_y * sample_factor + sample_y;
                    
                    if src_x < render_width && src_y < render_height {
                        let pixel = processed_img.get_pixel(src_x, src_y);
                        total_brightness += pixel[0] as f32;
                        pixel_count += 1;
                    }
                }
            }

            let avg_brightness = total_brightness / pixel_count as f32;
            
            // Apply contrast boost
            let mut brightness = ((avg_brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
            brightness = brightness.clamp(0.0, 255.0);

            // Invert if requested
            if config.invert {
                brightness = 255.0 - brightness;
            }

            // Apply improved gamma correction for better contrast
            let gamma_corrected = (brightness / 255.0).powf(0.75) * 255.0;

            let char_index = ((gamma_corrected / 255.0) * (ascii_chars.len() - 1) as f32) as usize;
            let char_index = char_index.min(ascii_chars.len() - 1);

            ascii_art.push(ascii_chars[char_index]);
        }
        ascii_art.push('\n');
    }

    Ok(ascii_art)
}

fn to_color_ascii_enhanced(img: &image::DynamicImage, config: &AsciiConfig) -> Result<String> {
    // Same enhanced character set
    let ascii_chars = if config.detailed {
        &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█',
          '▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟', '░', '▒', '▓', '█',
          '@', '&', '%', '$', '#', 'W', 'M', 'B', 'R', 'N', 'H', 'Q', 'K', 'A', 'G', 
          'O', 'C', 'V', 'U', 'Y', 'X', 'Z', 'b', 'd', 'h', 'k', 'p', 'q', 'w', 'm', 
          '*', 'o', 'a', 'z', 'c', 'v', 'u', 'n', 'x', 'r', 'j', 'f', 't', '/', '\\', 
          '|', '(', ')', '1', '{', '}', '[', ']', '?', '-', '_', '+', '~', '<', '>', 
          'i', '!', 'l', 'I', ';', ':', ',', '"', '^', '`', '\'', '.', ' '][..]
    } else {
        &['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.', ' '][..]
    };
    
    let (width, height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;
    let char_aspect_ratio = 0.43;
    
    let (target_width, target_height) = if config.preserve_aspect {
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

    let sample_factor = config.supersample;
    let (render_width, render_height) = (
        target_width * sample_factor, 
        target_height * sample_factor
    );

    println!("Color supersampling: {}x ({}x{} -> {}x{})", 
             sample_factor, render_width, render_height, target_width, target_height);

    let resized_img = img.resize_exact(render_width, render_height, FilterType::Lanczos3);
    let mut ascii_art = String::with_capacity((target_width * target_height * 25 + target_height) as usize);

    for target_y in 0..target_height {
        for target_x in 0..target_width {
            let mut total_r = 0.0;
            let mut total_g = 0.0;
            let mut total_b = 0.0;
            let mut total_brightness = 0.0;
            let mut pixel_count = 0;

            // Sample all pixels in the supersample block
            for sample_y in 0..sample_factor {
                for sample_x in 0..sample_factor {
                    let src_x = target_x * sample_factor + sample_x;
                    let src_y = target_y * sample_factor + sample_y;
                    
                    if src_x < render_width && src_y < render_height {
                        let pixel = resized_img.get_pixel(src_x, src_y);
                        let Rgba([r, g, b, _]) = pixel;
                        
                        total_r += r as f32;
                        total_g += g as f32;
                        total_b += b as f32;
                        total_brightness += (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32);
                        pixel_count += 1;
                    }
                }
            }

            let avg_r = (total_r / pixel_count as f32) as u8;
            let avg_g = (total_g / pixel_count as f32) as u8;
            let avg_b = (total_b / pixel_count as f32) as u8;
            let avg_brightness = total_brightness / pixel_count as f32;
            
            // Apply contrast boost to brightness
            let adjusted_brightness = ((avg_brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
            let clamped_brightness = adjusted_brightness.clamp(0.0, 255.0);
            
            let final_brightness = if config.invert {
                255.0 - clamped_brightness
            } else {
                clamped_brightness
            };

            let char_index = ((final_brightness / 255.0) * (ascii_chars.len() - 1) as f32) as usize;
            let char_index = char_index.min(ascii_chars.len() - 1);
            let chosen_char = ascii_chars[char_index];

            ascii_art.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                avg_r, avg_g, avg_b, chosen_char
            ));
        }
        ascii_art.push('\n');
    }

    Ok(ascii_art)
}

fn apply_edge_enhancement(img: &image::DynamicImage, strength: f32) -> Result<image::GrayImage> {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();
    let mut enhanced = image::GrayImage::new(width, height);

    // Apply unsharp mask for edge enhancement
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let center = gray.get_pixel(x, y)[0] as f32;
            
            // Calculate Laplacian for edge detection
            let laplacian = 
                -1.0 * gray.get_pixel(x - 1, y - 1)[0] as f32 +
                -1.0 * gray.get_pixel(x, y - 1)[0] as f32 +
                -1.0 * gray.get_pixel(x + 1, y - 1)[0] as f32 +
                -1.0 * gray.get_pixel(x - 1, y)[0] as f32 +
                 8.0 * center +
                -1.0 * gray.get_pixel(x + 1, y)[0] as f32 +
                -1.0 * gray.get_pixel(x - 1, y + 1)[0] as f32 +
                -1.0 * gray.get_pixel(x, y + 1)[0] as f32 +
                -1.0 * gray.get_pixel(x + 1, y + 1)[0] as f32;

            let enhanced_value = (center + strength * laplacian).clamp(0.0, 255.0) as u8;
            enhanced.put_pixel(x, y, image::Luma([enhanced_value]));
        }
    }

    Ok(enhanced)
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