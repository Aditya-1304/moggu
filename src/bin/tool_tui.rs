use crossterm::{
  event::{self, DisableMouseCapture}
};
use ratatui::{
  backend::Backend, style::Color, widgets::ListState
};


use std::{error::Error, io};
use tokio::runtime::Runtime;


#[derive(Debug, Clone)]
pub struct Filter {
  pub name: String,
  pub description: String,
  pub requires_param: bool,
  pub params: Vec<FilterParam>,
  pub category: FilterCategory,
  pub icon: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterCategory {
  Basic,
  Color,
  Geometric,
  Artistic,
  Enhancement,
  Utility,
}

impl FilterCategory {
  fn color(&self) -> Color {
    match self {
      FilterCategory::Basic => Color::Blue,
      FilterCategory::Color => Color::Red,
      FilterCategory::Geometric => Color::Green,
      FilterCategory::Artistic => Color::Magenta,
      FilterCategory::Enhancement => Color::Yellow,
      FilterCategory::Utility => Color::Cyan,
    }
  }

  fn name(&self) -> &str {
    match self {
      FilterCategory::Basic => "Basic",
      FilterCategory::Color => "Color",
      FilterCategory::Geometric => "Geometric",
      FilterCategory::Artistic => "Artistic",
      FilterCategory::Enhancement => "Enhancement",
      FilterCategory::Utility => "Utility",
    }
  }
}

#[derive(Debug, Clone)]
pub struct FilterParam {
  pub name: String,
  pub param_type: ParamType,
  pub default: String,
  pub description: String,
}

#[derive(Debug, Clone)]
pub enum ParamType {
  Integer { min: i32, max: i32},
  Float {min: f32, max: f32},
  Boolean,
}

#[derive(Debug, Clone)]
pub enum AppState {
  Welcome,
  FileInput,
  FilterSection,
  ParameterInput,
  Processing,
  Result,
}

#[derive(Debug)]
pub struct App {
  pub state: AppState,
  pub filters: Vec<Filter>,
  pub filter_list_state: ListState,
  pub selected_filter: Option<Filter>,
  pub input_file: String,
  pub output_file: String,
  pub current_param_index: usize,
  pub param_values: Vec<String>,
  pub input_mode: InputMode,
  pub current_input: String,
  pub message: String,
  pub show_help: bool,
  pub processing_progress: f64,
  pub selected_category: Option<FilterCategory>,
  pub runtime: Runtime,
}

#[derive(Debug)]
pub enum InputMode {
  InputFile,
  OutputFile,
  Parameter,
}

impl App {
  pub fn new() -> App {
    let filters = vec![
      Filter {
        name: "grayscale".to_string(),
        description: "Convert image to grayscale".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Basic,
        icon: "🔲"
      },
      Filter {
        name: "brigntness".to_string(),
        description: "Adjust image brightness". to_string(),
        requires_param: true,
        params: vec![FilterParam {
          name: "value".to_string(),
          param_type: ParamType::Integer { min: -100, max: 100 },
          default: "20".to_string(),
          description: "Brightness adjustment (-100 to 100)".to_string()
        }],
        category: FilterCategory::Enhancement,
        icon: ""
      },
      Filter {
        name : "contrast".to_string(),
        description: "Adjust image contrast".to_string(),
        requires_param: true,
        params: vec![FilterParam {
          name: "factor".to_string(),
          param_type: ParamType::Float { min: 0.1, max: 3.0 },
          default: "1.5".to_string(),
          description: "Contrast factor (0.1 to 3.0)".to_string(),
        }],
        category: FilterCategory::Enhancement,
        icon: "",
      },
      Filter {
        name: "gaussian-blur".to_string(),
        description: "Apply Gaussian blur".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Blur Sigma".to_string(),
            param_type: ParamType::Float { min: 0.1, max: 20.0 },
            default: "2.0".to_string(),
            description: "Blur intensity (0.1 to 20.0)".to_string(),
        }],
        category: FilterCategory::Enhancement,
        icon: "",
      },
      Filter {
        name: "box-blur".to_string(),
        description: "Apply box blur".to_string(),
        requires_param: true,
        params: vec![FilterParam {
          name: "Blur Radius".to_string(),
          param_type: ParamType::Integer { min: 1, max: 50 },
          default: "5".to_string(),
          description: "Blur radius (1 to 50)".to_string(),
        }],
        category: FilterCategory::Enhancement,
        icon: "",
      },
      Filter {
        name: "sharpen".to_string(),
        description: "Sharpen the image".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Sharpen Strength".to_string(),
            param_type: ParamType::Float { min: 0.1, max: 3.0 },
            default: "1.0".to_string(),
            description: "Sharpen strength (0.1 to 3.0)".to_string(),
        }],
        category: FilterCategory::Enhancement,
        icon: "",
      },
      Filter {
        name: "edge-detection".to_string(),
        description: "Apply Sobel edge detection".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Enhancement,
        icon: "🔍",
      },
      Filter {
        name: "thresholding".to_string(),
        description: "Apply binary thresholding".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Threshold Value".to_string(),
            param_type: ParamType::Integer { min: 0, max: 255 },
            default: "128".to_string(),
            description: "Threshold value (0 to 255)".to_string(),
        }],
        category: FilterCategory::Enhancement,
        icon: "",
      },
      Filter {
        name: "sepia".to_string(),
        description: "Apply sepia filter".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Artistic,
        icon: "",
      },
      Filter {
        name: "vignette".to_string(),
        description: "Apply vignette effect".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Vignette Strength".to_string(),
            param_type: ParamType::Float { min: 0.1, max: 1.0 },
            default: "0.5".to_string(),
            description: "Vignette strength (0.1 to 1.0)".to_string(),
        }],
        category: FilterCategory::Artistic,
        icon: "",
      },
      Filter {
        name: "noise".to_string(),
        description: "Add noise to image".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Noise Strength".to_string(),
            param_type: ParamType::Integer { min: 1, max: 100 },
            default: "20".to_string(),
            description: "Noise strength (1 to 100)".to_string(),
        }],
        category: FilterCategory::Artistic,
        icon: "",
      },
      Filter {
        name: "oil".to_string(),
        description: "Apply oil painting effect".to_string(),
        requires_param: true,
        params: vec![
            FilterParam {
                name: "Radius".to_string(),
                param_type: ParamType::Integer { min: 1, max: 10 },
                default: "4".to_string(),
                description: "Oil painting radius (1 to 10)".to_string(),
            },
            FilterParam {
                name: "Intensity Levels".to_string(),
                param_type: ParamType::Integer { min: 5, max: 50 },
                default: "20".to_string(),
                description: "Oil painting intensity levels (5 to 50)".to_string(),
            },
        ],
        category: FilterCategory::Artistic,
        icon: "",
      },
      Filter {
        name: "saturate".to_string(),
        description: "Adjust color saturation".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Saturation Factor".to_string(),
            param_type: ParamType::Float { min: 0.0, max: 3.0 },
            default: "1.5".to_string(),
            description: "Saturation factor (0.0 to 3.0)".to_string(),
        }],
        category: FilterCategory::Color,
        icon: "",
      },
      Filter {
        name: "invert".to_string(),
        description: "Invert image colors".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Color,
        icon: "",
      },
      Filter {
        name: "hue-rotate".to_string(),
        description: "Rotate hue colors".to_string(),
        requires_param: true,
        params: vec![FilterParam {
            name: "Hue Rotation".to_string(),
            param_type: ParamType::Float { min: -360.0, max: 360.0 },
            default: "90.0".to_string(),
            description: "Hue rotation in degrees (-360 to 360)".to_string(),
        }],
        category: FilterCategory::Color,
        icon: "",
      },
      Filter {
        name: "rotate90".to_string(),
        description: "Rotate image 90° clockwise".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Geometric,
        icon: "",
      },
      Filter {
        name: "rotate180".to_string(),
        description: "Rotate image 180°".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Geometric,
        icon: "",
      },
      Filter {
        name: "rotate270".to_string(),
        description: "Rotate image 270° clockwise".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Geometric,
        icon: "",
      },
      Filter {
        name: "flip-horizontal".to_string(),
        description: "Flip image horizontally".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Geometric,
        icon: "",
      },
      Filter {
        name: "flip-vertical".to_string(),
        description: "Flip image vertically".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Geometric,
        icon: "",
      },
      Filter {
        name: "crop".to_string(),
        description: "Crop image to rectangle".to_string(),
        requires_param: true,
        params: vec![
            FilterParam {
                name: "X Position".to_string(),
                param_type: ParamType::Integer { min: 0, max: 10000 },
                default: "0".to_string(),
                description: "X coordinate of crop start".to_string(),
            },
            FilterParam {
                name: "Y Position".to_string(),
                param_type: ParamType::Integer { min: 0, max: 10000 },
                default: "0".to_string(),
                description: "Y coordinate of crop start".to_string(),
            },
            FilterParam {
                name: "Width".to_string(),
                param_type: ParamType::Integer { min: 1, max: 10000 },
                default: "800".to_string(),
                description: "Width of crop area".to_string(),
            },
            FilterParam {
                name: "Height".to_string(),
                param_type: ParamType::Integer { min: 1, max: 10000 },
                default: "600".to_string(),
                description: "Height of crop area".to_string(),
            },
        ],
        category: FilterCategory::Utility,
        icon: "",
      },
      Filter {
        name: "ascii".to_string(),
        description: "Convert to ASCII art".to_string(),
        requires_param: false,
        params: vec![],
        category: FilterCategory::Utility,
        icon: "📝",
      },
    ];
  }
}
fn main() {
  println!("hello world")
}