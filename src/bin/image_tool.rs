use image::{ImageBuffer, Rgba, Rgb, DynamicImage};
use image::{imageops::FilterType, GenericImageView, GrayImage};
use std::env;
use std::fs;
use rand::Rng;

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
        "sharpen" => {
            if args.len() != 5 {
                eprintln!("Error: Sharpen mode requires a strenght value.");
                eprintln!("Usage: {} sharpen <input> <output> <strength>", args[0]);
                return Ok(());
            }
            let strength: f32 = args[4].parse().map_err(|_| "Invalid strength value. Must be a number like 1.0")?;
            println!("Applying sharpen filter with strength: {}", strength);
            let sharpened_img = sharpen(&img, strength);
            sharpened_img.save(output_path)?;
        }
        "saturate" => {
            if args.len() != 5 {
                eprintln!("Error: saturate mode requires a factor");
                return Ok(());
            }
            let factor: f32 = args[4].parse()?;
            // let saturation = img.adjust_saturation(factor);
            let saturated_img = adjust_saturation(&img, factor);

            saturated_img.save(output_path)?;
        
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
        "brightness" => {
            if args.len() != 5 {
                eprintln!("Error: Brightness mode requires a value.");
                return Ok(());
            }
            let value:i32 = args[4].parse()?;
            println!("Adjusting brightness by {}", value);
            let adjusted_img = adjust_brightness(&img, value);
            adjusted_img.save(output_path)?;
        }
        "contrast" => {
            if args.len() != 5 {
                eprintln!("Error: Contrast mode requires a value.");
                return Ok(());
            }
            let factor:f32 = args[4].parse()?;
            println!("Adjusting Contrast by {}", factor);
            let adjusted_img = adjust_contrast(&img, factor);
            adjusted_img.save(output_path)?;
        }
        "invert" => {
            println!("Inverting the image colors...");
            let inverted_image = invert_colors(&img);
            inverted_image.save(output_path)?;
        }
        "sepia" => {
            println!("Applying sepia filter to the image...");
            let filtered_image = apply_sepia(&img);
            filtered_image.save(output_path)?;
        }
        "vignette" => {
            if args.len() != 5 {
                eprintln!("Error: Vignette mode requires a strength value.");
                return Ok(());
            }
            let strength: f32 = args[4].parse()?;
            println!("Applying vignette with strength: {}", strength);
            let vignetted_img = apply_vignette(&img, strength);
            vignetted_img.save(output_path)?;
        }
        "noise" => {
            if args.len() != 5 {
                eprintln!("Error: Noise filter requires a strength value.");
                return Ok(());
            }
            let strenght: u8 = args[4].parse()?;
            let noised_img = add_noise(&img, strenght);
            noised_img.save(output_path)?;
        }
        "oil" => {
            if args.len() != 6 {
                eprintln!("Error: Oil mode requires a radius and intensity levels.");
                return Ok(());
            }
            let radius: u32 = args[4].parse()?;
            let intensity: u8 = args[5].parse()?;
            println!("Applying oil painting filter with radius {} and intensity {}", radius, intensity);

            let oil_image = apply_oil_filter(&img, radius, intensity);
            oil_image.save(output_path)?;
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
    eprintln!("  gaussian-blur <in> <out> <sigma>      - Apply a high-quality Gaussian blur (e.g., sigma 5.0). Alias: 'blur'.");
    eprintln!("  box-blur <in> <out> <radius>          - Apply a simple, from-scratch box blur (e.g., radius 3).");
    eprintln!("  sharpen <in> <out> <strength>         - Sharpen the image (e.g., strength 1.0).");
    eprintln!("  rotate90 <in> <out>                   - Rotate image 90 degrees clockwise.");
    eprintln!("  rotate180 <in> <out>                  - Rotate image 180 degrees.");
    eprintln!("  rotate270 <in> <out>                  - Rotate image 270 degrees clockwise.");
    eprintln!("  flip-horizontal <in> <out>            - Flip image horizontally.");
    eprintln!("  flip-vertical <in> <out>              - Flip image vertically.");
    eprintln!("  ascii <in> <out> [options]            - Convert to a high-quality ASCII art text file.");
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

fn flip_horizontal(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    for y in 0..height{
        for x in 0..width{
            let Rgba([r,g,b,_]) = img.get_pixel(x, y);
            output.put_pixel(width-1-x, y, Rgb([r,g,b]));
        }
    }
    output
}

fn flip_vertical(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);
    for y in 0..height{
        for x in 0..width{
            let Rgba([r,g,b,_]) = img.get_pixel(x, y);
            output.put_pixel(x, height-1-y, Rgb([r,g,b]));
        }
    }
    output
}

fn rotate90(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(height, width);

    for y in 0..height{
        for x in 0..width{
            let Rgba([r, g,b, _]) = img.get_pixel(x, y);
            output.put_pixel(height-1-y, x, Rgb([r,g,b]));
        }
    }
    output
}

fn rotate180(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new( width, height);
    for y in 0..height{
        for x in 0..width{
            let Rgba([r,g, b,_]) = img.get_pixel(x, y);
            output.put_pixel(width-1-x, height-1-y, Rgb([r,g,b]));
        }
    }
    output
}

fn rotate270(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(height, width);
    for y in 0..height{
        for x in 0..width{
            let Rgba([r,g,b,_]) = img.get_pixel(x, y);
            output.put_pixel(y, width-1-x, Rgb([r,g,b]));
        }
    }
    output
}


fn sharpen(img: &DynamicImage, strenght: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);

    let kernel = [
        [0.0, -1.0, 0.0],
        [-1.0, 5.0, -1.0],
        [0.0, -1.0, 0.0],
    ];

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut sum_red = 0.0;
            let mut sum_green = 0.0;
            let mut sum_blue = 0.0;

            for kernaly in 0..3{
                for kernalx in 0..3{
                    let pixelx = x + kernalx - 1;
                    let pixely = y + kernaly - 1;

                    let Rgba([r,g,b,_]) = img.get_pixel(pixelx, pixely);
                    let weight = kernel[kernaly as usize][kernalx as usize];

                    sum_red += r as f32 * weight;
                    sum_green += g as f32 * weight;
                    sum_blue += b as f32 * weight;
                }
            }

            let original_pixel = img.get_pixel(x, y);
            let new_red = (original_pixel[0] as f32 * (1.0 - strenght) + sum_red * strenght).clamp(0.0, 255.0) as u8;
            let new_green = (original_pixel[1] as f32 * (1.0 - strenght) + sum_green * strenght).clamp(0.0, 255.0) as u8;
            let new_blue = (original_pixel[2] as f32 * (1.0 - strenght) + sum_blue * strenght).clamp(0.0, 255.0) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn adjust_brightness(img: &DynamicImage, value: i32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r,g,b,_]) = img.get_pixel(x, y);

            let new_red = (r as i32 + value).clamp(0, 255) as u8;
            let new_green = (g as i32 + value).clamp(0, 255) as u8;
            let new_blue = (b as i32 + value).clamp(0, 255) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn adjust_contrast(img: &DynamicImage, factor: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);

            let new_red = (factor * (r as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            let new_green = (factor * (g as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
            let new_blue = (factor * (b as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn rgb_to_hsl(r:u8, g:u8, b:u8) -> (f32, f32, f32) {
    let red_normal = r as f32 / 255.0;
    let green_normal = g as f32 / 255.0;
    let blue_normal = b as f32 / 255.0;


    let max = red_normal.max(green_normal.max(blue_normal));
    let min = red_normal.min(green_normal.min(blue_normal));

    let lightness = (max + min) / 2.0;
    let mut hue = 0.0;
    let mut saturation = 0.0;

    if max != min {
        let diff = max - min;
        saturation = if lightness > 0.5 {diff / (2.0 - max - min)} else {diff / (max + min)};
        hue = match max {
            x if x == red_normal => (green_normal - blue_normal) / diff + (if green_normal < blue_normal {6.0} else {0.0}),
            x if x == green_normal => (blue_normal - red_normal) / diff + 2.0,
            _=> (red_normal - green_normal) / diff + 4.0,
        };
        hue /= 6.0;
    }
    (hue * 360.0, saturation, lightness)
}

fn hsl_to_rgb(h:f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let val = (l * 255.0) as u8;
        return (val, val, val);
    }

    let hue_to_rgb = |p: f32, q: f32, mut t: f32| {
        if t < 0.0 { t += 1.0;}
        if t > 1.0 { t -= 1.0;}
        if t < 1.0 / 6.0 { return p + (q-p) * 6.0 * t;  }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0;}
        p
    };

    let q = if l < 0.5 { l * (1.0 + s)} else { l + s - l * s};
    let p = 2.0 * l - q;
    let hue_normal = h / 360.0;


    let red = hue_to_rgb(p, q, hue_normal + 1.0 / 3.0);
    let green = hue_to_rgb(p, q, hue_normal);
    let blue = hue_to_rgb(p, q, hue_normal - 1.0 / 3.0);

    ((red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8)

}

fn adjust_saturation(img: &DynamicImage, factor: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);

    for y in 0..height {
        for x in 0..width {

            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let new_s = (s * factor).clamp(0.0, 1.0);
            let (new_red, new_green, new_blue) = hsl_to_rgb(h, new_s, l);

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn invert_colors(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);
            output.put_pixel(x, y, Rgb([255 - r, 255 - g, 255 - b]));
        }
    }
    output
}

fn apply_sepia(img: &DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r,g,b,_]) = img.get_pixel(x, y);
        
            let red_filter = r as f32;
            let green_filter = g as f32;
            let blue_filter = b as f32;

            let new_red = (red_filter * 0.393) + (green_filter * 0.769) + (blue_filter * 0.189);
            let new_green = (red_filter * 0.349) + (green_filter * 0.686) + (blue_filter * 0.168);
            let new_blue = (red_filter * 0.272) + (green_filter * 0.534) + (blue_filter * 0.131);

            output.put_pixel(x, y, Rgb([
                new_red.min(255.0) as u8,
                new_green.min(255.0) as u8,
                new_blue.min(255.0) as u8,
            ]));
        }
    }
    output
}


fn apply_vignette(img: &DynamicImage, strenght: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);


    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt();

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);

            let dist_x = x as f32 - center_x;
            let dist_y = y as f32 - center_y;
            let dist = (dist_x.powi(2) + dist_y.powi(2)).sqrt();

            let normal_dist = dist / max_dist;

            let factor = 1.0 - (normal_dist.powf(2.0) * strenght);

            let new_red = (r as f32 * factor).clamp(0.0, 255.0) as u8;
            let new_green = (g as f32 * factor).clamp(0.0, 255.0) as u8;
            let new_blue = (b as f32 * factor).clamp(0.0, 255.0) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn add_noise(img: &DynamicImage, strenght: u8) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>,_>::new(width, height);

    let mut rng = rand::rng();

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, _]) = img.get_pixel(x, y);

            let noise = rng.random_range(-(strenght as i16)..=(strenght as i16));

            let new_red = (r as i16 + noise).clamp(0, 255) as u8;
            let new_green = (g as i16 + noise).clamp(0, 255) as u8;
            let new_blue = (b as i16 + noise).clamp(0, 255) as u8;

            output.put_pixel(x, y, Rgb([new_red, new_green, new_blue]));
        }
    }
    output
}

fn apply_oil_filter(img: &DynamicImage, radius: u32, intensity_levels: u8) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::<Rgb<u8>, _>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut intensity_counts = vec![0; intensity_levels as usize];

            let mut avg_red = vec![0; intensity_levels as usize];

            let mut avg_green = vec![0; intensity_levels as usize];

            let mut avg_blue = vec![0; intensity_levels as usize];

            let y_min = y.saturating_sub(radius);
            let y_max = (y + radius).min(height - 1);
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(width - 1);

            for newy in y_min..=y_max {
                for newx in x_min..=x_max {
                    let Rgba([r,g,b, _]) = img.get_pixel(newx, newy);
                    let intensity = (((r as u32 + g as u32 + b as u32) /3 * intensity_levels as u32) / 256) as usize;

                    if intensity < intensity_levels as usize {
                        intensity_counts[intensity] +=1;
                        avg_red[intensity] += r as u32;
                        avg_green[intensity] += g as u32;
                        avg_blue[intensity] += b as u32;
                    }
                    
                }
            }

            let mut max_count = 0;
            let mut max_index = 0;
            for i in 0..intensity_levels as usize {
                if intensity_counts[i] > max_count {
                    max_count = intensity_counts[i];
                    max_index = i;
                }
                
            }

            if max_count > 0 {
                let final_red = (avg_red[max_index] / max_count) as u8;
                let final_green = (avg_green[max_index] / max_count) as u8;
                let final_blue = (avg_blue[max_index] / max_count) as u8;

                output.put_pixel(x, y, Rgb([final_red, final_green, final_blue]));
            } else {
                let Rgba([r, g, b, _]) = img.get_pixel(x,y);
                output.put_pixel(x, y, Rgb([r,g,b]));
            }
            
        }
    }
    output
}