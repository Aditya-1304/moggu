
use moggu::*;  
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        return Ok(());
    }

    let input_path = &args[2];
    let output_path = &args[3];
    let mut config = AsciiConfig::default();

    for arg in &args[4..] {
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
            let result = basic::grayscale(&img, None);
            println!("Saving grayscale image to: {}", output_path);
            result.save(output_path)?;
        }
        "ascii" => {
            println!("Converting to ASCII art...");
            let ascii_art = utility::to_ascii_dithered(&img, &config, None)?;
            println!("Saving ASCII art to: {}", output_path);
            std::fs::write(output_path, ascii_art)?;
        }
        "brightness" => {
            if args.len() != 5 {
                eprintln!("Error: Brightness mode requires a value.");
                return Ok(());
            }
            let value: i32 = args[4].parse()?;
            println!("Adjusting brightness by {}", value);
            let adjusted_img = enhancement::brightness(&img, value, None);
            adjusted_img.save(output_path)?;
        }
        "contrast" => {
            if args.len() != 5 {
                eprintln!("Error: Contrast mode requires a value.");
                return Ok(());
            }
            let factor: f32 = args[4].parse()?;
            println!("Adjusting contrast by {}", factor);
            let adjusted_img = enhancement::contrast(&img, factor, None);
            adjusted_img.save(output_path)?;
        }
        "gaussian-blur" | "blur" => {
            if args.len() != 5 {
                eprintln!("Error: Blur mode requires a sigma value.");
                eprintln!("Usage: {} blur <input> <output> <sigma>", args[0]);
                return Ok(());
            }
            let sigma: f32 = args[4].parse().map_err(|_| "Invalid sigma value. Must be a number like 5.0")?;
            println!("Applying Gaussian blur with sigma: {}", sigma);
            let blurred_img = enhancement::gaussian_blur(&img, sigma, None);
            blurred_img.save(output_path)?;
        }
        "box-blur" => {
            if args.len() != 5 {
                eprintln!("Error: Box blur mode requires a radius value.");
                eprintln!("Usage: {} box-blur <input> <output> <radius>", args[0]);
                return Ok(());
            }
            let radius: u32 = args[4].parse().map_err(|_| "Invalid radius value. Must be a number like 5")?;
            println!("Applying box blur with radius: {}", radius);
            let blurred_img = enhancement::box_blur(&img, radius, None);
            blurred_img.save(output_path)?;
        }
        "sharpen" => {
            if args.len() != 5 {
                eprintln!("Error: Sharpen mode requires a strength value.");
                eprintln!("Usage: {} sharpen <input> <output> <strength>", args[0]);
                return Ok(());
            }
            let strength: f32 = args[4].parse().map_err(|_| "Invalid strength value. Must be a number like 1.0")?;
            println!("Applying sharpen filter with strength: {}", strength);
            let sharpened_img = enhancement::sharpen(&img, strength, None);
            sharpened_img.save(output_path)?;
        }
        "saturate" => {
            if args.len() != 5 {
                eprintln!("Error: Saturate mode requires a factor");
                return Ok(());
            }
            let factor: f32 = args[4].parse()?;
            println!("Adjusting saturation by factor: {}", factor);
            let saturated_img = color::saturate(&img, factor, None);
            saturated_img.save(output_path)?;
        }
        "rotate90" => {
            println!("Rotating image 90 degrees clockwise...");
            let rotated = geometric::rotate90(&img, None);
            rotated.save(output_path)?;
        }
        "rotate180" => {
            println!("Rotating image 180 degrees...");
            let rotated = geometric::rotate180(&img, None);
            rotated.save(output_path)?;
        }
        "rotate270" => {
            println!("Rotating image 270 degrees clockwise...");
            let rotated = geometric::rotate270(&img, None);
            rotated.save(output_path)?;
        }
        "flip-horizontal" => {
            println!("Flipping image horizontally...");
            let flipped = geometric::flip_horizontal(&img, None);
            flipped.save(output_path)?;
        }
        "flip-vertical" => {
            println!("Flipping image vertically...");
            let flipped = geometric::flip_vertical(&img, None);
            flipped.save(output_path)?;
        }
        "invert" => {
            println!("Inverting the image colors...");
            let inverted_image = color::invert(&img, None);
            inverted_image.save(output_path)?;
        }
        "sepia" => {
            println!("Applying sepia filter to the image...");
            let filtered_image = artistic::sepia(&img, None);
            filtered_image.save(output_path)?;
        }
        "vignette" => {
            if args.len() != 5 {
                eprintln!("Error: Vignette mode requires a strength value.");
                return Ok(());
            }
            let strength: f32 = args[4].parse()?;
            println!("Applying vignette with strength: {}", strength);
            let vignetted_img = artistic::vignette(&img, strength, None);
            vignetted_img.save(output_path)?;
        }
        "noise" => {
            if args.len() != 5 {
                eprintln!("Error: Noise filter requires a strength value.");
                return Ok(());
            }
            let strength: u8 = args[4].parse()?;
            println!("Adding noise with strength: {}", strength);
            let noised_img = artistic::noise(&img, strength, None);
            noised_img.save(output_path)?;
        }
        "oil" => {
            if args.len() != 6 {
                eprintln!("Error: Oil mode requires a radius and intensity levels.");
                return Ok(());
            }
            let radius: u32 = args[4].parse()?;
            let intensity: u32 = args[5].parse()?;
            println!("Applying oil painting filter with radius {} and intensity {}", radius, intensity);
            let oil_image = artistic::oil_painting(&img, radius, intensity, None);
            oil_image.save(output_path)?;
        }
        "edge-detection" => {
            println!("Applying Sobel edge detection...");
            let edge_image = enhancement::edge_detection(&img, None);
            edge_image.save(output_path)?;
        }
        "crop" => {
            if args.len() != 8 {
                eprintln!("Error: Crop mode requires 4 values: x, y, width and height.");
                eprintln!("Usage: {} crop <in> <out> <x> <y> <width> <height>", args[0]);
                return Ok(());
            }
            let x: u32 = args[4].parse()?;
            let y: u32 = args[5].parse()?;
            let width: u32 = args[6].parse()?;
            let height: u32 = args[7].parse()?;
            println!("Cropping image to {}x{} at ({}, {})", width, height, x, y);
            let crop_image = utility::crop(&img, x, y, width, height, None);
            crop_image.save(output_path)?;
        }
        "hue-rotate" => {
            if args.len() != 5 {
                eprintln!("Error: Hue rotate mode requires degrees value.");
                return Ok(());
            }
            let degrees: f32 = args[4].parse()?;
            println!("Rotating hue by {} degrees", degrees);
            let new_img = color::hue_rotate(&img, degrees, None);
            new_img.save(output_path)?;
        }
        "thresholding" => {
            if args.len() != 5 {
                eprintln!("Error: Thresholding requires you to give threshold.");
                return Ok(());
            }
            let threshold: u8 = args[4].parse()?;
            println!("Applying thresholding with threshold: {}", threshold);
            let threshed_image = enhancement::thresholding(&img, threshold, None);
            threshed_image.save(output_path)?;
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
    eprintln!("Usage: {} <mode> <input> <output> [options/value]", program_name);
    eprintln!("\nModes:");
    eprintln!("  grayscale <in> <out>                  - Convert to a grayscale image file.");
    eprintln!("  brightness <in> <out> <value>         - Adjust brightness (e.g., 20 or -20).");
    eprintln!("  contrast <in> <out> <factor>          - Adjust contrast (e.g., 1.5).");
    eprintln!("  saturate <in> <out> <factor>          - Adjust color saturation (e.g., 1.5 or -1.0).");
    eprintln!("  invert <in> <out>                     - Invert the image colors.");
    eprintln!("  sepia <in> <out>                      - Apply a sepia (antique) filter.");
    eprintln!("  vignette <in> <out> <strength>        - Apply a vignette effect (e.g., strength 0.8).");
    eprintln!("  noise <in> <out> <strength>           - Add random noise/grain to the image (e.g., strength 20).");
    eprintln!("  oil <in> <out> <radius> <intensity>   - Apply an oil painting effect (e.g., radius 4, intensity 20).");
    eprintln!("  edge-detection <in> <out>             - Apply Sobel edge detection.");
    eprintln!("  crop <in> <out> <x> <y> <w> <h>       - Crop the image to a rectangle.");
    eprintln!("  gaussian-blur <in> <out> <sigma>      - Apply a high-quality Gaussian blur (e.g., sigma 5.0). Alias: 'blur'.");
    eprintln!("  box-blur <in> <out> <radius>          - Apply a simple, from-scratch box blur (e.g., radius 3).");
    eprintln!("  sharpen <in> <out> <strength>         - Sharpen the image (e.g., strength 1.0).");
    eprintln!("  rotate90 <in> <out>                   - Rotate image 90 degrees clockwise.");
    eprintln!("  rotate180 <in> <out>                  - Rotate image 180 degrees.");
    eprintln!("  rotate270 <in> <out>                  - Rotate image 270 degrees clockwise.");
    eprintln!("  flip-horizontal <in> <out>            - Flip image horizontally.");
    eprintln!("  flip-vertical <in> <out>              - Flip image vertically.");
    eprintln!("  hue-rotate <in> <out> <degrees>       - Rotate hue colors (e.g., 90).");
    eprintln!("  thresholding <in> <out> <threshold>   - Apply binary thresholding (e.g., 128).");
    eprintln!("  ascii <in> <out> [options]            - Convert to a high-quality ASCII art text file.");
    eprintln!("\nOptions for ASCII:");
    eprintln!("  --width=N                     - Set maximum width in characters (default: 120).");
    eprintln!("  --contrast=F                  - Adjust contrast. >1.0 increases, <1.0 decreases (default: 1.2).");
    eprintln!("  --detailed                    - Use a larger, more detailed character set.");
    eprintln!("  --invert                      - Invert the brightness for light-on-dark terminals.");
    eprintln!("  --no-dither                   - Disable dithering for a simpler, banded look.");
}

