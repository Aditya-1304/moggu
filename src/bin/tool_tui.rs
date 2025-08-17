use moggu::*;
use std::sync::mpsc;
use std::thread;

use base64::Engine;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use rfd::AsyncFileDialog;
use std::{ io, process::Command};
use tokio::runtime::Runtime;

use image::{ImageReader, DynamicImage};

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

  pub fn next_parameter(&mut self) {
    if let Some(filter) = &self.selected_filter {
      if self.current_param_index < filter.params.len() - 1 {
          self.param_values[self.current_param_index] = self.current_input.clone();
          self.current_param_index += 1;
          self.current_input = self.param_values[self.current_param_index].clone();
      } else {
          self.param_values[self.current_param_index] = self.current_input.clone();
          self.process_image();
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
    if let Some(ref reciever) = self.progress_receiver {
        match reciever.try_recv() {
          Ok(progress) => {
            self.processing_progress = progress;
            if progress >= 1.0 {
              self.message = if let Some(filter) = &self.selected_filter {
                format!(" ^_^ Successfully applied {} filter!\n\n Output saved to: {}\n\n Press 'v' to view image or 'r' to process another", filter.name, self.output_file)
              } else {
                "Processing Completed !!".to_string()
              };
              // self.image_preview = self.generate_ascii_preview(&self.output_file);
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
            // println!("\nImage displayed using chafa");
            return true;
        } else {
            // println!("Chafa failed: {}", String::from_utf8_lossy(&output.stderr));
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
                    // println!("\n Image displayed using viu");
                    return true;
                } else {
                    // println!("Viu failed: {}", String::from_utf8_lossy(&output.stderr));
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
                  KeyCode::Char('v') => {
                    if !app.output_file.is_empty() && app.message.contains(" ") {

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
                          _ => {

                          } 
                        }
                      }

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