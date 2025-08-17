use std::sync::mpsc;

// Re-export all filter modules
pub mod basic;
pub mod color;
pub mod geometric;
pub mod artistic;
pub mod enhancement;
pub mod utility;

// Common types and utilities
pub type ProgressSender = mpsc::Sender<f64>;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[derive(Clone)]
pub struct AsciiConfig {
    pub max_width: u32,
    pub contrast_boost: f32,
    pub invert: bool,
    pub detailed: bool,
    pub dither: bool,
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

// Helper function to send progress updates
pub fn send_progress(tx: &Option<ProgressSender>, progress: f64) {
    if let Some(sender) = tx {
        let _ = sender.send(progress);
    }
}

// Main filter processor function
pub fn process_filter(
    filter_name: &str,
    input_file: &str,
    output_file: &str,
    param_values: &[String],
    progress_tx: Option<ProgressSender>,
) -> Result<()> {
    
    let img = image::open(input_file)?;
    
    match filter_name {

        "grayscale" => {
            let result = basic::grayscale(&img, progress_tx);
            result.save(output_file)?;
        }

        "brightness" => {
            let value: i32 = param_values[0].parse()?;
            let result = enhancement::brightness(&img, value, progress_tx);
            result.save(output_file)?;
        }
        "contrast" => {
            let factor: f32 = param_values[0].parse()?;
            let result = enhancement::contrast(&img, factor, progress_tx);
            result.save(output_file)?;
        }
        "gaussian-blur" => {
            let sigma: f32 = param_values[0].parse()?;
            let result = enhancement::gaussian_blur(&img, sigma, progress_tx);
            result.save(output_file)?;
        }
        "box-blur" => {
            let radius: u32 = param_values[0].parse()?;
            let result = enhancement::box_blur(&img, radius, progress_tx);
            result.save(output_file)?;
        }
        "sharpen" => {
            let strength: f32 = param_values[0].parse()?;
            let result = enhancement::sharpen(&img, strength, progress_tx);
            result.save(output_file)?;
        }
        "edge-detection" => {
            let result = enhancement::edge_detection(&img, progress_tx);
            result.save(output_file)?;
        }
        "thresholding" => {
            let threshold: u8 = param_values[0].parse()?;
            let result = enhancement::thresholding(&img, threshold, progress_tx);
            result.save(output_file)?;
        }

        "saturate" => {
            let factor: f32 = param_values[0].parse()?;
            let result = color::saturate(&img, factor, progress_tx);
            result.save(output_file)?;
        }
        "invert" => {
            let result = color::invert(&img, progress_tx);
            result.save(output_file)?;
        }
        "hue-rotate" => {
            let degrees: f32 = param_values[0].parse()?;
            let result = color::hue_rotate(&img, degrees, progress_tx);
            result.save(output_file)?;
        }

        "rotate90" => {
            let result = geometric::rotate90(&img, progress_tx);
            result.save(output_file)?;
        }
        "rotate180" => {
            let result = geometric::rotate180(&img, progress_tx);
            result.save(output_file)?;
        }
        "rotate270" => {
            let result = geometric::rotate270(&img, progress_tx);
            result.save(output_file)?;
        }
        "flip-horizontal" => {
            let result = geometric::flip_horizontal(&img, progress_tx);
            result.save(output_file)?;
        }
        "flip-vertical" => {
            let result = geometric::flip_vertical(&img, progress_tx);
            result.save(output_file)?;
        }

        "sepia" => {
            let result = artistic::sepia(&img, progress_tx);
            result.save(output_file)?;
        }
        "vignette" => {
            let strength: f32 = param_values[0].parse()?;
            let result = artistic::vignette(&img, strength, progress_tx);
            result.save(output_file)?;
        }
        "noise" => {
            let strength: u8 = param_values[0].parse()?;
            let result = artistic::noise(&img, strength, progress_tx);
            result.save(output_file)?;
        }
        "oil" => {
            let radius: u32 = param_values[0].parse()?;
            let intensity: u32 = param_values[1].parse()?;
            let result = artistic::oil_painting(&img, radius, intensity, progress_tx);
            result.save(output_file)?;
        }

        "crop" => {
            let x: u32 = param_values[0].parse()?;
            let y: u32 = param_values[1].parse()?;
            let width: u32 = param_values[2].parse()?;
            let height: u32 = param_values[3].parse()?;
            let result = utility::crop(&img, x, y, width, height, progress_tx);
            result.save(output_file)?;
        }
        "ascii" => {
            let config = AsciiConfig::default();
            let result = utility::to_ascii_dithered(&img, &config, progress_tx)?;
            std::fs::write(output_file, result)?;
        }
        
        _ => {
            return Err(format!("Unknown filter: {}", filter_name).into());
        }
    }
    
    Ok(())
}