use image::{ImageBuffer, Rgba, Rgb, DynamicImage};
use image::{imageops::FilterType, GenericImageView, GrayImage};
use std::env;
use std::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone)]
struct AsciiConfig {
    max_width: u32,
    contrast_boost: f32,
    invert: bool,
    detailed: bool,
    dither: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            max_width: 120,
            contrast_boost: 1.2,
            invert: false,
            detailed: false,
            dither: true,
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        return Ok(());
    }

    let input_path = &args[2];
    let output_path = &args[3];
    let mut config = AsciiConfig::default();
    
    for arg in &args[3..] {
        match arg.as_str() {
            "--invert" => config.invert = true,
            "--detailed" => config.detailed = true,
            "--no-dither" => config.dither = false,
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

    println!("Opening image: {}", input_path);
    let img = image::open(input_path)?;
    println!("Image dimensions: {}x{}", img.width(), img.height());

    match args[1].as_str() {
        "grayscale" => {
            println!("Converting to grayscale...");
            let grayscale_img = img.grayscale();
            println!("Saving grayscale image to: {}", output_path);
            grayscale_img.save(output_path)?;
        }
        "ascii" => {
            println!("Converting to ASCII art with config: {:?}", (config.max_width, config.contrast_boost, config.invert, config.detailed, config.dither));
            let ascii_art = to_ascii_dithered(&img, &config)?;
            println!("Saving ASCII art to: {}", output_path);
            fs::write(output_path, ascii_art)?;
        }
        "blur" | "gaussian-blur" => {
            if args.len() != 5 {
                eprintln!("Error: Blur mode requires a sigma value.");
                eprintln!("Usage: {} blur <input> <output> <sigma>", args[0]);
            }
            let sigma: f32 = args[4].parse().map_err(|_| "Invalid sigma value. Must be a number like 5.0")?;

            println!("Applying blur to the image with sigma: {}", sigma );
            let blurred_img = img.blur(sigma);
            println!("Saving the blurred image to: {}", output_path);
            blurred_img.save(output_path)?;
        }
        "box-blur" => {
            if args.len() != 5 {
                eprintln!("Error: Blur mode requires a sigma value.");
                eprintln!("Usage: {} blur <input> <output> <sigma>", args[0]);
            }
            let radius: u32 = args[4].parse().map_err(|_| "Invalid sigma value. Must be a number like 5.0")?;

            println!("Applying blur to the image with sigma: {}", radius );
            let blurred_img = box_blur(&img, radius);
            println!("Saving the blurred image to: {}", output_path);
            blurred_img.save(output_path)?;
        }
        "rotate90" => {
            println!("Rotating image 90 degree clock wise...");
            // let rotated = img.rotate90();
            let rotated = rotate90(&img);
            rotated.save(output_path)?;
        }
        "rotate180" => {
            println!("Rotating image 180 degree clock wise...");
            // let rotated = img.rotate180();
            let rotated = rotate180(&img);
            rotated.save(output_path)?;
        }
        "rotate270" => {
            println!("Rotating image 270 degree clock wise...");
            // let rotated = img.rotate270();
            let rotated = rotate270(&img);
            rotated.save(output_path)?;
        }
        "flip-horizontal" => {
            println!("Flipping image horizontally...");
            // let flipped = img.fliph();
            let flipped = flip_horizontal(&img);
            flipped.save(output_path)?;
        }
        "flip-vertical" => {
            println!("Flipping image vertically...");
            // let flipped = img.flipv();
            let flipped = flip_vertical(&img);
            flipped.save(output_path)?;
        }
        _ => {
            print_usage(&args[0]);
            return Err("Unknown mode specified.".into());
        }
    }

    println!("Operation complete.");
    Ok(())
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <mode> <input> <output> [options]", program_name);
    eprintln!("\nModes:");
    eprintln!("  grayscale <in> <out>              - Convert to a grayscale image file.");
    eprintln!("  gaussian-blur <in> <out> <sigma>  - Apply a high-quality Gaussian blur (e.g., sigma 5.0). Alias: 'blur'.");
    eprintln!("  box-blur <in> <out> <radius>      - Apply a simple, from-scratch box blur (e.g., radius 3).");
    eprintln!("  rotate90 <in> <out>               - Rotate image 90 degrees clockwise.");
    eprintln!("  rotate180 <in> <out>              - Rotate image 180 degrees.");
    eprintln!("  rotate270 <in> <out>              - Rotate image 270 degrees clockwise.");
    eprintln!("  flip-horizontal <in> <out>        - Flip image horizontally.");
    eprintln!("  flip-vertical <in> <out>          - Flip image vertically.");
    eprintln!("  ascii <in> <out> [options]    - Convert to a high-quality ASCII art text file.");
    eprintln!("\nOptions:");
    eprintln!("  --width=N                     - Set maximum width in characters (default: 120).");
    eprintln!("  --contrast=F                  - Adjust contrast. >1.0 increases, <1.0 decreases (default: 1.2).");
    eprintln!("  --detailed                    - Use a larger, more detailed character set.");
    eprintln!("  --invert                      - Invert the brightness for light-on-dark terminals.");
    eprintln!("  --no-dither                   - Disable dithering for a simpler, banded look.");
}

fn to_ascii_dithered(img: &image::DynamicImage, config: &AsciiConfig) -> Result<String> {
 
    let ascii_chars_detailed = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
    let ascii_chars_simple = "@%#*+=-:. ";
    
    let char_ramp: Vec<char> = if config.detailed {
        ascii_chars_detailed.chars().collect()
    } else {
        ascii_chars_simple.chars().collect()
    };
    let ramp_len = char_ramp.len() as f32;

    let (width, height) = img.dimensions();
    let aspect_ratio = height as f32 / width as f32;
    let new_width = config.max_width;
    let new_height = (new_width as f32 * aspect_ratio * 0.5) as u32;

    let small_img = img.resize_exact(new_width, new_height, FilterType::Lanczos3);
    let mut gray_img = small_img.to_luma8();

    if config.dither {

        let mut f32_buffer: Vec<f32> = gray_img.pixels().map(|p| p[0] as f32).collect();

        for y in 0..new_height {
            for x in 0..new_width {
                let idx = (y * new_width + x) as usize;
                let old_pixel = f32_buffer[idx];
                
                
                let new_pixel = ((old_pixel / 255.0 * (ramp_len - 1.0)).round() / (ramp_len - 1.0)) * 255.0;
                f32_buffer[idx] = new_pixel;

                let quant_error = old_pixel - new_pixel;

                
                if x + 1 < new_width {
                    f32_buffer[idx + 1] += quant_error * 7.0 / 16.0;
                }
                if x > 0 && y + 1 < new_height {
                    f32_buffer[idx + new_width as usize - 1] += quant_error * 3.0 / 16.0;
                }
                if y + 1 < new_height {
                    f32_buffer[idx + new_width as usize] += quant_error * 5.0 / 16.0;
                }
                if x + 1 < new_width && y + 1 < new_height {
                    f32_buffer[idx + new_width as usize + 1] += quant_error * 1.0 / 16.0;
                }
            }
        }
        
        let final_pixels: Vec<u8> = f32_buffer.iter().map(|&p| p.clamp(0.0, 255.0) as u8).collect();
        gray_img = GrayImage::from_raw(new_width, new_height, final_pixels).unwrap();
    }
    
    let mut ascii_art = String::with_capacity((new_width * new_height + new_height) as usize);
    for (i, pixel) in gray_img.pixels().enumerate() {
        let mut brightness = pixel[0] as f32;

        brightness = ((brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
        brightness = brightness.clamp(0.0, 255.0);

        if config.invert {
            brightness = 255.0 - brightness;
        }

        let char_index = ((brightness / 255.0) * (ramp_len - 1.0)).round() as usize;
        ascii_art.push(char_ramp[char_index]);

        if (i + 1) % new_width as usize == 0 {
            ascii_art.push('\n');
        }
    }
    Ok(ascii_art)
}

fn box_blur(img: &DynamicImage, radius: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut total_red: u32 = 0;
            let mut total_green: u32 = 0;
            let mut total_blue: u32 = 0;
            let mut pixel_count: u32 = 0;

            let y_min = y.saturating_sub(radius);
            let y_max = (y + radius).min(height -1);
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(width -1);

            for ny in y_min..=y_max {
                for nx in x_min..=x_max {
                    let pixel = img.get_pixel(nx, ny);
                    let Rgba([r, g, b, _]) = pixel;

                    total_red += r as u32;
                    total_green += g as u32;
                    total_blue += b as u32;
                    pixel_count += 1;
                }
            }

            let avg_red = (total_red / pixel_count) as u8;
            let avg_green = (total_green / pixel_count) as u8;
            let avg_blue = (total_blue / pixel_count) as u8;


            output.put_pixel(x, y, Rgb([avg_red, avg_green, avg_blue])); 
        }
    }
    output
}

fn flip_horizontal(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
            output.put_pixel(width-1-x, y, pixel);
        }
    }
    output
}

fn flip_vertical(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
            output.put_pixel(x, height-1-y, pixel);
        }
    }
    output
}

fn rotate90(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(height, width);
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
            output.put_pixel(height-1-x, y, pixel);
        }
    }
    output
}
fn rotate180(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(height, width);
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
            output.put_pixel(width-1-x, height-1-y, pixel);
        }
    }
    output
}
fn rotate270(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(height, width);
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
            output.put_pixel(x, width-1-y, pixel);
        }
    }
    output
}

