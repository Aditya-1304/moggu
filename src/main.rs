use moggu::*;
use std::sync::mpsc;
use std::thread;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap
    },
    Frame, Terminal,
};
use rfd::AsyncFileDialog;
use std::{ io, process::Command};
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
      FilterCategory::Basic => Color::Rgb(100, 149, 237),      
      FilterCategory::Color => Color::Rgb(186, 85, 211),       
      FilterCategory::Geometric => Color::Rgb(138, 169, 238),  
      FilterCategory::Artistic => Color::Rgb(255, 105, 180),   
      FilterCategory::Enhancement => Color::Rgb(138, 43, 226), 
      FilterCategory::Utility => Color::Rgb(30, 144, 255),     
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
  FilterSelection,
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
  pub image_preview: Option<String>,
  pub has_image_support: bool,
  pub progress_receiver: Option<mpsc::Receiver<f64>>,
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
        icon: ""
      },
      Filter {
        name: "brightness".to_string(),
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
        icon: "",
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
      // Filter {
      //   name: "ascii".to_string(),
      //   description: "Convert to ASCII art".to_string(),
      //   requires_param: false,
      //   params: vec![],
      //   category: FilterCategory::Utility,
      //   icon: "",
      // },
    ];

    let mut app = App {
      state: AppState::Welcome,
      filters,
      filter_list_state: ListState::default(),
      selected_filter: None,
      input_file: String::new(),
      output_file: String::new(),
      current_param_index: 0,
      param_values: vec![],
      input_mode: InputMode::InputFile,
      current_input: String::new(),
      message: String::new(),
      show_help: false,
      processing_progress: 0.0,
      selected_category: None,
      runtime: Runtime::new().unwrap(),
      image_preview: None,
      has_image_support: Self::detect_image_support(),
      progress_receiver: None,
    };

    app.filter_list_state.select(Some(0));
    app
  }

  fn detect_image_support() -> bool {
    if Command::new("chafa").arg("--version").output().is_ok() {
      return true;
    }
    if Command::new("viu").arg("--version").output().is_ok() {
      return true;
    }
    false
  }

  pub fn next_filter(&mut self) {
    let filtered_filters = self.get_filtered_filters();
    let i = match self.filter_list_state.selected() {
      Some(i) => {
        if i >= filtered_filters.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.filter_list_state.select(Some(i));
  }

  pub fn previous_filter(&mut self) {
        let filtered_filters = self.get_filtered_filters();
        let i = match self.filter_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    filtered_filters.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.filter_list_state.select(Some(i));
    }
  

  pub fn get_filtered_filters(&self) -> Vec<&Filter> {
    if let Some(category) = &self.selected_category {
      self.filters
        .iter()
        .filter(|f| &f.category == category)
        .collect()
    } else {
        self.filters.iter().collect()
    }
  }

  pub fn select_current_filter(&mut self) {
    let filtered_filters = self.get_filtered_filters();
    if let Some(i) = self.filter_list_state.selected() {
      if i  < filtered_filters.len() {
        let selected_filters = filtered_filters[i].clone();
        let requires_params = selected_filters.requires_param;
        let params = selected_filters.params.clone();

        self.selected_filter = Some(selected_filters);

        if requires_params {
          self.param_values = params.iter().map(|p| p.default.clone()).collect();
          self.current_param_index = 0;
          self.state = AppState::ParameterInput;
          self.current_input = self.param_values[0].clone();
        } else {
          self.process_image();
        }
      }
    }
  }
  fn validate_parameter(&self, value: &str, param: &FilterParam) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &param.param_type {
            ParamType::Integer { min, max } => {
                match value.parse::<i32>() {
                    Ok(val) if val >= *min && val <= *max => Ok(()),
                    Ok(_) => Err(format!("Value must be between {} and {}", min, max).into()),
                    Err(_) => Err("Invalid integer value".into()),
                }
            }
            ParamType::Float { min, max } => {
                match value.parse::<f32>() {
                    Ok(val) if val >= *min && val <= *max => Ok(()),
                    Ok(_) => Err(format!("Value must be between {:.1} and {:.1}", min, max).into()),
                    Err(_) => Err("Invalid decimal value".into()),
                }
            }
            ParamType::Boolean => {
                match value.to_lowercase().as_str() {
                    "true" | "false" => Ok(()),
                    _ => Err("Value must be 'true' or 'false'".into()),
                }
            }
        }
  }

  pub fn next_parameter(&mut self) {
        if let Some(filter) = &self.selected_filter {
            if self.current_param_index < filter.params.len() {
                let param = &filter.params[self.current_param_index];
                
                if let Err(error) = self.validate_parameter(&self.current_input, param) {
                    self.message = format!("Parameter Error: {}", error);
                    return;
                }
                
                self.param_values[self.current_param_index] = self.current_input.clone();
                
                if self.current_param_index < filter.params.len() - 1 {
                    self.current_param_index += 1;
                    self.current_input = self.param_values[self.current_param_index].clone();
                } else {
                    self.message.clear();
                    self.process_image();
                }
            }
        }
  }

  pub fn previous_parameter(&mut self) {
    if self.current_param_index > 0 {
      self.param_values[self.current_param_index] = self.current_input.clone();
      self.current_param_index -= 1;
      self.current_input = self.param_values[self.current_param_index].clone();
    }
  }

  pub async fn open_file_dialog(&mut self, for_output: bool) {
    let title = if for_output {
      "Save processed image as..."
    } else {
      "Select an image to process"
    };

    let file = if for_output {
      AsyncFileDialog::new()
        .set_title(title)
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "tiff", "gif"])
        .save_file()
        .await
    } else {
      AsyncFileDialog::new()
        .set_title(title)
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "tiff", "gif"])
        .pick_file()
        .await
    };

    if let Some(file) = file {
      let path = file.path().to_string_lossy().to_string();
      if for_output {
          self.output_file = path;
          self.state = AppState::FilterSelection;
      } else {
          self.input_file = path;
          self.input_mode = InputMode::OutputFile;
      }
    }
  }

  pub fn process_image(&mut self) {
    self.state = AppState::Processing;
    self.processing_progress = 0.0;

    if let Some(filter) = &self.selected_filter.clone() {
      let input_file = self.input_file.clone();
      let output_file = self.output_file.clone();
      let param_values = self.param_values.clone();
      let filter_name = filter.name.clone();

      let (progress_tx, progress_rx) = mpsc::channel();

      let _handle = thread::spawn(move || {
        let _ = process_filter(&filter_name, &input_file, &output_file, &param_values, Some(progress_tx)); 
      });

      self.progress_receiver = Some(progress_rx);
    }
  }

  

  pub fn update_progress(&mut self) -> bool {
    if let Some(ref receiver) = self.progress_receiver {
        match receiver.try_recv() {
            Ok(progress) => {
                self.processing_progress = progress;
                if progress >= 1.0 {
                    self.message = if let Some(filter) = &self.selected_filter {
                        format!("^_^ Successfully applied {} filter!\n\nOutput saved to: {}\n\nPress 'v' to view image or 'r' to process another", filter.name, self.output_file)
                    } else {
                        "Processing Completed!".to_string()
                    };
                    self.state = AppState::Result;
                    self.progress_receiver = None;
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    } else {
        false
    }
}


  fn display_inline_image(&self, image_path: &str) {
    println!(" Displaying image: {}", image_path);
    println!();

    if self.try_chafa(image_path) {
      return;
    }
    if self.try_viu(image_path) {
      return;
    }

    self.show_installation_help(image_path);
  }

  fn try_chafa(&self, image_path: &str) -> bool {
    if Command::new("chafa").arg("--version").output().is_ok() {
      let (width, height) = self.get_terminal_size();

      if let Ok(output) = Command::new("chafa")
        .arg("--size").arg(format!("{}x{}", width.min(120), height.min(40)))
        .arg("--colors=256")
        .arg("--symbols=block")
        .arg("--stretch")
        .arg("--animate=off")
        .arg(image_path)
        .output()
    {
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            return true;
        } else {
            println!("");
        }
      }
    }
    false
  }

  fn try_viu(&self, image_path: &str) -> bool {
        if Command::new("viu").arg("--version").output().is_ok() {
            println!("Using viu for image display...");
            
            if let Ok(output) = Command::new("viu")
                .arg("-w").arg("100")
                .arg("-h").arg("35")
                .arg("--static")
                .arg(image_path)
                .output()
            {
                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                    return true;
                } else {
                    println!("");
                }
            }
        }
        false
    }

  fn get_terminal_size(&self) -> (u16, u16) {
    if let Ok((width, height)) = crossterm::terminal::size() {
      (width, height)
    } else {
      (80, 24)
    }
  }

  fn show_installation_help(&self, image_path: &str) {
    println!(" No supported image display method found.");
    println!();
    println!(" For Arch Linux, install chafa (recommended):");
    println!("   sudo pacman -S chafa");
    println!();
    println!(" For other Linux distributions, macOS, or Windows:");
    println!("   Check the README.md file for detailed installation instructions");
    println!("   for your specific operating system and package manager.");
    println!();
    println!(" Alternative options:");
    println!("   • viu (Rust):     cargo install viu");
    println!("   • catimg:         sudo pacman -S catimg");
    println!();
    println!(" Image saved to: {}", image_path);
    
    if let Ok(metadata) = std::fs::metadata(image_path) {
        println!(" File size: {:.2} KB", metadata.len() as f64 / 1024.0);
    }
    
    println!();
    println!(" After installing chafa, restart the application for best results!");
  }

  pub fn reset(&mut self) {
    self.state = AppState::Welcome;
    self.input_mode = InputMode::InputFile;
    self.selected_filter = None;
    self.current_param_index = 0;
    self.param_values.clear();
    self.current_input.clear();
    self.message.clear();
    self.processing_progress = 0.0;
    self.selected_category = None;
    self.progress_receiver = None;
  }

  pub fn cycle_category(&mut self) {
    let categories = vec![
      None,
      Some(FilterCategory::Basic),
      Some(FilterCategory::Color),
      Some(FilterCategory::Geometric),
      Some(FilterCategory::Artistic),
      Some(FilterCategory::Enhancement),
      Some(FilterCategory::Utility),
    ];

    let current_index = categories.iter().position(|c| c == &self.selected_category).unwrap_or(0);
    let next_index = (current_index + 1) % categories.len();
    self.selected_category = categories[next_index].clone();
    self.filter_list_state.select(Some(0));
  }

}


fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let app = App::new();
  let res = run_app(&mut terminal, app);

  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{err:?}");
  }

  Ok(())
}


fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
  loop {
      terminal.draw(|f| ui(f, &mut app))?;

      if matches!(app.state, AppState::Processing) {
        app.update_progress();

        if event::poll(std::time::Duration::from_millis(20))? {
          if let Event::Key(key) = event::read()? {
              if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
              }
          }
        }
        continue;
      }

      if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
          match app.state {
              AppState::Welcome => {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Enter | KeyCode::Char(' ') => app.state = AppState::FileInput,
                    _ => {}
                }
              }

              AppState::FileInput => {
                match key.code {
                  KeyCode::Char('q') => return Ok(()),
                  KeyCode::Char('h') => app.show_help = !app.show_help,
                  KeyCode::Char('f') => {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        app.open_file_dialog(false).await;
                    });
                  }

                  KeyCode::Char('o') => {
                    if !app.input_file.is_empty() {
                      let rt = Runtime::new().unwrap();
                      rt.block_on(async {
                          app.open_file_dialog(true).await;
                      });
                    }
                  }

                  KeyCode::Tab => {
                      match app.input_mode {
                        InputMode::InputFile => {
                          if !app.current_input.is_empty() {
                              app.input_file = app.current_input.clone();
                              app.current_input.clear();
                              app.input_mode = InputMode::OutputFile;
                          }
                        }

                        InputMode::OutputFile => {
                          if !app.current_input.is_empty() {
                              app.output_file = app.current_input.clone();
                              app.current_input.clear();
                              app.state = AppState::FilterSelection;
                          }
                        }

                        _ => {}
                      }
                  }
                  KeyCode::Enter => {
                    match app.input_mode {
                      InputMode::InputFile => {
                        if !app.current_input.is_empty() {
                    
                          if !std::path::Path::new(&app.current_input).exists() {
                            app.message = "Error: Input file does not exist".to_string();
                            continue;
                          }
                          app.input_file = app.current_input.clone();
                          app.current_input.clear();
                          app.input_mode = InputMode::OutputFile;
                          app.message.clear(); 
                        }
                      }
                      InputMode::OutputFile => {
                        if !app.current_input.is_empty() {
                          
                          if let Some(parent) = std::path::Path::new(&app.current_input).parent() {
                            if !parent.exists() {
                              app.message = "Error: Output directory does not exist".to_string();
                              continue;
                            }
                          }
                          app.output_file = app.current_input.clone();
                          app.current_input.clear();
                          app.state = AppState::FilterSelection;
                          app.message.clear(); 
                        }
                      }
                      _ => {}
                    }
                  }
                  KeyCode::Char(c) => {
                      app.current_input.push(c);
                  }
                  KeyCode::Backspace => {
                      app.current_input.pop();
                  }
                  KeyCode::Esc => app.state = AppState::Welcome,
                  _ => {}
                }
              }

              AppState::FilterSelection => {
                match key.code {
                  KeyCode::Char('q') => return Ok(()),
                  KeyCode::Char('h') => app.show_help = !app.show_help,
                  KeyCode::Char('c') => app.cycle_category(),
                  KeyCode::Down | KeyCode::Char('j') => app.next_filter(),
                  KeyCode::Up | KeyCode::Char('k') => app.previous_filter(),
                  KeyCode::Enter => app.select_current_filter(),
                  KeyCode::Esc => app.state = AppState::FileInput,
                  _ => {}
                }
              }

              AppState::ParameterInput => {
                match key.code {
                  KeyCode::Char('q') => return Ok(()),
                  KeyCode::Char('h') => app.show_help = !app.show_help,
                  KeyCode::Enter | KeyCode::Tab => app.next_parameter(),
                  KeyCode::Up => app.previous_parameter(),
                  KeyCode::Char(c) => {
                      app.current_input.push(c);
                  }
                  KeyCode::Backspace => {
                      app.current_input.pop();
                  }
                  KeyCode::Esc => {
                      app.state = AppState::FilterSelection;
                  }
                  _ => {}
                }
              }

              AppState::Processing => {
                if key.code == KeyCode::Char('q') {
                  return Ok(());
                }
              }

              AppState::Result => {
                match key.code {
                  KeyCode::Char('q') => return Ok(()),
                  KeyCode::Char('r') => app.reset(),
                  KeyCode::Char('c') => {
                    if app.has_image_support {
                      print!("\x1b[2J\x1b[H"); 
                      println!("Images cleared.");
                      std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                  }
                  KeyCode::Char('v') => {
                    if !app.output_file.is_empty() && app.message.contains("Successfully") {

                      let current_state = app.state.clone();
                      disable_raw_mode()?;

                      execute!(
                        io::stdout(),
                        crossterm::terminal::LeaveAlternateScreen,
                        DisableMouseCapture
                      )?;

                      while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                        let _ = crossterm::event::read()?;
                      }

                      print!("\x1b[2J\x1b[H");
                      println!(" Image Preview - {}", app.output_file);
                      println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                      println!();

                      app.display_inline_image(&app.output_file);

                      println!();
                      println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                      println!("Press Enter to return to the app...");

                      loop {
                        match crossterm::event::read() {
                          Ok(Event::Key(key)) => {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                              break;
                            }
                          }
                          Ok(_) => continue, 
                          Err(_) => break,
                          
                        }
                      }

                      enable_raw_mode()?;
                      execute!(
                        io::stdout(),
                        EnterAlternateScreen,
                        EnableMouseCapture
                      )?;

                      while crossterm::event::poll(std::time::Duration::from_millis(50))? {
                        let _ = crossterm::event::read()?;
                      }

                      app.state = current_state;

                      terminal.clear()?;

                      while crossterm::event::poll(std::time::Duration::from_millis(10))? {
                        let _ = crossterm::event::read()?;
                      }
                      continue;
                    }
                  }

                  KeyCode::Enter | KeyCode::Esc => app.reset(),

                  _ => {}
                }
              }
          }
        }
      }
  }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    match app.state {
        AppState::Welcome => render_welcome(f, app, size),
        AppState::FileInput => render_file_input(f, app, size),
        AppState::FilterSelection => render_filter_selection(f, app, size),
        AppState::ParameterInput => render_parameter_input(f, app, size),
        AppState::Processing => render_processing(f, app, size),
        AppState::Result => render_result(f, app, size),
    }

    if app.show_help {
        render_help_popup(f, app, size);
    }
}

fn render_welcome(f: &mut Frame, _app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints([
      Constraint::Length(8),
      Constraint::Length(6),
      Constraint::Min(0),
    ])
    .split(area);

  let title_text = vec![
    Line::from(""),
    Line::from(vec![
      Span::styled("", Style::default().fg(Color::Rgb(147, 112, 219))),
      Span::styled("Moggu", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
      Span::styled(" - Image Processing Tool", Style::default().fg(Color::Rgb(176, 196, 222))),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::raw("    "),
      Span::styled(" Transform your images with powerful filters ", Style::default().fg(Color::Rgb(123, 104, 238))),
    ]),
    Line::from(""),
  ];

  let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .block(
        Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        .title(" Welcome ")
        .title_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        .add_modifier(Modifier::BOLD)
      );
  f.render_widget(title, chunks[0]);

  let features_text = vec![
    Line::from(vec![
        Span::styled(" ", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::raw("22+ Professional Filters"),
    ]),
    Line::from(vec![
        Span::styled(" ", Style::default().fg(Color::Rgb(138, 43, 226))),
        Span::raw("Easy File Selection with Built-in File Browser"),
    ]),
    Line::from(vec![
        Span::styled(" ", Style::default().fg(Color::Rgb(147, 112, 219))),
        Span::raw("Real-time Parameter Adjustment"),
    ]),
    Line::from(vec![
        Span::styled(" ", Style::default().fg(Color::Rgb(123, 104, 238))),
        Span::raw("Categorized Filters for Easy Navigation"),
    ]),
  ];

  let features = Paragraph::new(features_text)
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
            .title(" Features ")
            .title_style(Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD))
    );
  f.render_widget(features, chunks[1]);
  
  

 let controls_text = vec![
    Line::from(""),
    Line::from(vec![
        Span::styled(" Press ", Style::default().fg(Color::Rgb(176, 196, 222))),
        Span::styled("Enter", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(Color::Rgb(176, 196, 222))),
        Span::styled("Space", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::styled(" to get started!", Style::default().fg(Color::Rgb(176, 196, 222))),
    ]),
    Line::from(""),
    Line::from(vec![
        Span::styled(" Press ", Style::default().fg(Color::Rgb(176, 196, 222))),
        Span::styled("'h'", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
        Span::styled(" for help  |  ", Style::default().fg(Color::Rgb(176, 196, 222))),
        Span::styled("'q'", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::styled(" to quit", Style::default().fg(Color::Rgb(176, 196, 222))),
    ]),
    Line::from(""),
  ];

  let controls = Paragraph::new(controls_text)
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(123, 104, 238)))
        .title(" Controls ")
        .title_style(Style::default().fg(Color::Rgb(123, 104, 238)).add_modifier(Modifier::BOLD))
    );
  f.render_widget(controls, chunks[2]);


}


fn render_file_input(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let input_title = match app.input_mode {
        InputMode::InputFile => " Input File Path (Currently Typing)",
        _ => " Input File Path",
    };

    let input_style = if matches!(app.input_mode, InputMode::InputFile) {
        Style::default().fg(Color::Rgb(147, 112, 219))
    } else {
        Style::default().fg(Color::Rgb(100, 149, 237))
    };

    let input_value = if matches!(app.input_mode, InputMode::InputFile) {
        &app.current_input
    } else {
        &app.input_file
    };

    let input = Paragraph::new(input_value.as_str())
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(input_title)
                .border_style(if matches!(app.input_mode, InputMode::InputFile) {
                    Style::default().fg(Color::Rgb(147, 112, 219))
                } else {
                    Style::default().fg(Color::Rgb(100, 149, 237))
                })
        );
    f.render_widget(input, chunks[0]);

    let output_title = match app.input_mode {
        InputMode::OutputFile => " Output File Path (Currently Typing)",
        _ => " Output File Path",
    };

    let output_style = if matches!(app.input_mode, InputMode::OutputFile) {
        Style::default().fg(Color::Rgb(147, 112, 219))
    } else if !app.output_file.is_empty() {
        Style::default().fg(Color::Rgb(100, 149, 237))
    } else {
        Style::default().fg(Color::Rgb(105, 105, 105))
    };

    let output_value = if matches!(app.input_mode, InputMode::OutputFile) {
        &app.current_input
    } else {
        &app.output_file
    };

    let output = Paragraph::new(output_value.as_str())
        .style(output_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(output_title)
                .border_style(if matches!(app.input_mode, InputMode::OutputFile) {
                    Style::default().fg(Color::Rgb(147, 112, 219))
                } else if !app.output_file.is_empty() {
                    Style::default().fg(Color::Rgb(100, 149, 237))
                } else {
                    Style::default().fg(Color::Rgb(105, 105, 105))
                })
        );
    f.render_widget(output, chunks[1]);

    let picker_text = vec![
        Line::from(vec![
            Span::styled("📂 Quick File Selection:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  'f'", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
            Span::styled(" - Open file browser to select input image", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  'o'", Style::default().fg(Color::Rgb(123, 104, 238)).add_modifier(Modifier::BOLD)),
            Span::styled(" - Open file browser to choose output location", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
    ];

    let picker = Paragraph::new(picker_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" File Browser ")
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        );
    f.render_widget(picker, chunks[2]);

    if !app.message.is_empty() {
        let error_color = if app.message.starts_with("Error:") {
            Color::Rgb(220, 20, 60)  
        } else {
            Color::Rgb(100, 149, 237)  
        };
        
        let error = Paragraph::new(app.message.as_str())
            .style(Style::default().fg(error_color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Message ")
                    .border_style(Style::default().fg(error_color))
            )
            .wrap(Wrap { trim: true });
        f.render_widget(error, chunks[3]);
    }

    let help_text = vec![
        Line::from(vec![
            Span::styled("⌨️  Manual Input:", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Type file paths directly", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Tab/Enter - Move to next field", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Backspace - Delete characters", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🧭 Navigation:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Esc - Go back to welcome screen", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("'h' - Show help  •  'q' - Quit", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Controls ")
                .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
        );
    f.render_widget(help, chunks[4]);
}

fn render_filter_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let category_text = if let Some(cat) = &app.selected_category {
        format!(" Category: {} (Press 'c' to cycle)", cat.name())
    } else {
        " Category: All Filters (Press 'c' to cycle)".to_string()
    };

    let category_color = app.selected_category
        .as_ref()
        .map(|c| c.color())
        .unwrap_or(Color::Rgb(176, 196, 222));

    let category = Paragraph::new(category_text)
        .style(Style::default().fg(category_color).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(category_color))
        );
    f.render_widget(category, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let all_filters = &app.filters;
    let selected_category = &app.selected_category;
    let selected_index = app.filter_list_state.selected();
    

    let filtered_filters: Vec<&Filter> = if let Some(category) = selected_category {
        all_filters.iter().filter(|f| &f.category == category).collect()
    } else {
        all_filters.iter().collect()
    };
    
    let filters_count = filtered_filters.len();
    
    let filters: Vec<ListItem> = filtered_filters
        .iter()
        .enumerate()
        .map(|(i, filter)| {
            let content = format!("{} {} - {}", filter.icon, filter.name, filter.description);
            if Some(i) == selected_index {
                ListItem::new(content).style(
                    Style::default()
                        .fg(Color::Rgb(25, 25, 112))  // Dark background text
                        .bg(filter.category.color())
                        .add_modifier(Modifier::BOLD)
                )
            } else {
                ListItem::new(content).style(Style::default().fg(filter.category.color()))
            }
        })
        .collect();

    let filters_list = List::new(filters)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("  Filters ({}) ", filters_count))
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .highlight_style(Style::default().bg(Color::Rgb(47, 79, 79)));

    f.render_stateful_widget(filters_list, main_chunks[0], &mut app.filter_list_state);

    let info_text = if let Some(i) = selected_index {
        if i < filtered_filters.len() {
            let filter = filtered_filters[i];
            let mut lines = vec![
                Line::from(vec![
                    Span::raw(filter.icon),
                    Span::raw(" "),
                    Span::styled(&filter.name, Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Category: ", Style::default().fg(Color::Rgb(147, 112, 219))),
                    Span::styled(filter.category.name(), Style::default().fg(filter.category.color()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description:", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&filter.description, Style::default().fg(Color::Rgb(176, 196, 222))),
                ]),
                Line::from(""),
            ];

            if filter.requires_param {
                lines.push(Line::from(vec![
                    Span::styled("Parameters:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                ]));
                for param in &filter.params {
                    lines.push(Line::from(vec![
                        Span::raw("  🔹 "),
                        Span::styled(&param.name, Style::default().fg(Color::Rgb(147, 112, 219))),
                        Span::styled(": ", Style::default().fg(Color::Rgb(176, 196, 222))),
                        Span::styled(&param.description, Style::default().fg(Color::Rgb(176, 196, 222))),
                    ]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" No parameters required", Style::default().fg(Color::Rgb(100, 149, 237))),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("🔹 "),
                Span::styled("Press Enter to select", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]));
            lines.push(Line::from(vec![
                Span::raw("🔹 "),
                Span::styled("Use ↑/↓ or j/k to navigate", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]));
            lines.push(Line::from(vec![
                Span::raw("🔹 "),
                Span::styled("Press 'c' to cycle categories", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]));
            lines.push(Line::from(vec![
                Span::raw("🔹 "),
                Span::styled("Press Esc to go back", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]));

            lines
        } else {
            vec![Line::from("No filter selected")]
        }
    } else {
        vec![Line::from("No filter selected")]
    };

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("   Filter Details ")
                .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
        )
        .wrap(Wrap { trim: true });
    f.render_widget(info, main_chunks[1]);
}

fn render_parameter_input(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    if let Some(filter) = &app.selected_filter {
        let param = &filter.params[app.current_param_index];
        
        let title = format!(" {} - Parameter {} of {}", filter.name, app.current_param_index + 1, filter.params.len());
        let input = Paragraph::new(app.current_input.as_str())
            .style(Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title.as_str())
                    .border_style(Style::default().fg(Color::Rgb(147, 112, 219)))
            );
        f.render_widget(input, chunks[0]);

        let param_info = vec![
            Line::from(vec![
                Span::styled("Parameter: ", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                Span::styled(&param.name, Style::default().fg(Color::Rgb(176, 196, 222)).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Rgb(100, 149, 237))),
                Span::styled(&param.description, Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Default Value: ", Style::default().fg(Color::Rgb(123, 104, 238))),
                Span::styled(&param.default, Style::default().fg(Color::Rgb(123, 104, 238)).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from({
                let range_text = match &param.param_type {
                    ParamType::Integer { min, max } => format!("{} to {}", min, max),
                    ParamType::Float { min, max } => format!("{:.1} to {:.1}", min, max),
                    ParamType::Boolean => "true or false".to_string(),
                };
                vec![
                    Span::styled("Range: ", Style::default().fg(Color::Rgb(147, 112, 219))),
                    Span::styled(range_text, Style::default().fg(Color::Rgb(176, 196, 222))),
                ]
            }),
        ];

        let info = Paragraph::new(param_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Parameter Details ")
                    .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
            );
        f.render_widget(info, chunks[1]);

        let progress = (app.current_param_index as f64 + 1.0) / filter.params.len() as f64;
        let progress_text = format!("Progress: {}/{} parameters", app.current_param_index + 1, filter.params.len());
        
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("  Progress ")
                    .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
            )
            .gauge_style(Style::default().fg(Color::Rgb(138, 43, 226)).bg(Color::Black))
            .percent((progress * 100.0) as u16)
            .label(progress_text);
        f.render_widget(gauge, chunks[2]);

        if !app.message.is_empty() && app.message.contains("Parameter Error") {
            let error = Paragraph::new(app.message.as_str())
                .style(Style::default().fg(Color::Rgb(220, 20, 60)).add_modifier(Modifier::BOLD))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Error ")
                        .border_style(Style::default().fg(Color::Rgb(220, 20, 60)))
                )
                .wrap(Wrap { trim: true });
            f.render_widget(error, chunks[3]);
        }

        let help_text = vec![
            Line::from(vec![
                Span::styled("⌨  Controls:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Type parameter value", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Tab/Enter - Next parameter or process", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("↑ - Previous parameter", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Backspace - Delete characters", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Esc - Back to filter selection", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("'q' - Quit application", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Help ")
                    .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
            );
        f.render_widget(help, chunks[4]);
    }
}

fn render_processing(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let processing_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Processing your image...", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    let processing = Paragraph::new(processing_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("  Processing ")
                .border_style(Style::default().fg(Color::Rgb(147, 112, 219)))
        );
    f.render_widget(processing, chunks[0]);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .gauge_style(Style::default().fg(Color::Rgb(138, 43, 226)).bg(Color::Black))
        .percent((app.processing_progress * 100.0) as u16)
        .label(format!("{:.0}%", app.processing_progress * 100.0));
    f.render_widget(gauge, chunks[1]);

    let info_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Please wait while your image is being processed...", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("This may take a few seconds depending on the filter and image size.", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
    ];

    let info = Paragraph::new(info_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("   Information ")
                .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
        );
    f.render_widget(info, chunks[2]);
}

fn render_result(f: &mut Frame, app: &App, area: Rect) {
    if app.has_image_support {
        render_result_with_external_image(f, app, area);
    } else {
        render_result_standard(f, app, area);
    }
}


fn render_result_with_external_image(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(10),   
            Constraint::Length(6),    
            Constraint::Length(8),    
        ])
        .split(area);

    let message_color = if app.message.contains("Successfully") {
        Color::Rgb(100, 149, 237)
    } else {
        Color::Rgb(220, 20, 60)
    };

    let border_color = if app.message.contains("Successfully") {
        Color::Rgb(100, 149, 237)
    } else {
        Color::Rgb(220, 20, 60)
    };

    let result = Paragraph::new(app.message.as_str())
        .style(Style::default().fg(message_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("  Result ")
                .border_style(Style::default().fg(border_color))
        )
        .wrap(Wrap { trim: true });
    f.render_widget(result, chunks[0]);

    let preview_notice = vec![
        Line::from(vec![
            Span::styled("  Image Preview Controls", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 'v' to display the processed image alongside this interface.", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("Press 'c' to clear any displayed images.", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("The image will appear without disrupting the TUI!", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
    ];

    let notice = Paragraph::new(preview_notice)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("  Image Display ")
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        );
    f.render_widget(notice, chunks[1]);

    let help_text = vec![
        Line::from(vec![
            Span::styled(" Controls:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  'v'", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
            Span::styled(" - View processed image (inline display)", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  'c'", Style::default().fg(Color::Rgb(123, 104, 238)).add_modifier(Modifier::BOLD)),
            Span::styled(" - Clear displayed images", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  'r'", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
            Span::styled(" - Process another image", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  'q'", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
            Span::styled(" - Quit application", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Controls ")
                .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
        );
    f.render_widget(help, chunks[2]);
}

fn render_result_standard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = if app.image_preview.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(8), 
                Constraint::Min(10),   
                Constraint::Length(7), 
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(0), Constraint::Length(7)])
            .split(area)
    };

    let message_color = if app.message.contains("Successfully") {
        Color::Rgb(100, 149, 237)
    } else {
        Color::Rgb(220, 20, 60)
    };

    let border_color = if app.message.contains("Successfully") {
        Color::Rgb(100, 149, 237)
    } else {
        Color::Rgb(220, 20, 60)
    };

    let result = Paragraph::new(app.message.as_str())
        .style(Style::default().fg(message_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("  Result ")
                .border_style(Style::default().fg(border_color))
        )
        .wrap(Wrap { trim: true });
    f.render_widget(result, chunks[0]);

    if let Some(preview) = &app.image_preview {
        let preview_widget = Paragraph::new(preview.as_str())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("   ASCII Preview (Install chafa/viu for real image preview) ")
                    .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
            );
        f.render_widget(preview_widget, chunks[1]);

        let help_text = vec![
            Line::from(vec![
                Span::styled(" What's Next?", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  'r'", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Process another image", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Start over", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::styled("  'q'", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Quit application", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Controls ")
                    .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
            );
        f.render_widget(help, chunks[2]);
    } else {
        let help_text = vec![
            Line::from(vec![
                Span::styled(" What's Next?", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  'r'", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Process another image", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Start over", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
            Line::from(vec![
                Span::styled("  'q'", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                Span::styled(" - Quit application", Style::default().fg(Color::Rgb(176, 196, 222))),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Controls ")
                    .border_style(Style::default().fg(Color::Rgb(100, 149, 237)))
            );
        f.render_widget(help, chunks[1]);
    }
}


fn render_help_popup(f: &mut Frame, _app: &App, area: Rect) {
    let popup_area = centered_rect(85, 85, area);
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(vec![
            Span::styled(" Moggu - Image Processing Tool", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" File Input:", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • Type file paths manually or use file browser", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'f' - Open file browser for input image", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'o' - Open file browser for output location", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Tab/Enter - Move between input fields", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Filter Selection:", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • ↑/↓ or j/k - Navigate through filters", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'c' - Cycle through filter categories", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Enter - Select current filter", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Filters are color-coded by category", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Parameter Input:", Style::default().fg(Color::Rgb(123, 104, 238)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • Type parameter values", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Tab/Enter - Next parameter or start processing", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • ↑ - Previous parameter", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Default values are pre-filled", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Image Preview:", Style::default().fg(Color::Rgb(186, 85, 211)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • 'v' - Display processed image inline", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'c' - Clear displayed images", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Images appear alongside the TUI interface", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⌨  Global Controls:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • 'q' - Quit application", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'h' - Toggle this help", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • Esc - Go back to previous screen", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(vec![
            Span::styled("  • 'r' - Reset and start over", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Filter Categories:", Style::default().fg(Color::Rgb(147, 112, 219)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Basic", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::raw(" • "),
            Span::styled("Color", Style::default().fg(Color::Rgb(186, 85, 211))),
            Span::raw(" • "),
            Span::styled("Geometric", Style::default().fg(Color::Rgb(138, 169, 238))),
            Span::raw(" • "),
            Span::styled("Artistic", Style::default().fg(Color::Rgb(255, 105, 180))),
            Span::raw(" • "),
            Span::styled("Enhancement", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw(" • "),
            Span::styled("Utility", Style::default().fg(Color::Rgb(30, 144, 255))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" The image preview works best in Kitty terminal with graphics support", Style::default().fg(Color::Rgb(176, 196, 222))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 'h' again to close this help", Style::default().fg(Color::Rgb(105, 105, 105)).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("  Help & Documentation ")
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .wrap(Wrap { trim: true });
    f.render_widget(help, popup_area);
}


fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}