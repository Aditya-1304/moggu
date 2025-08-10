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
  Enchancement,
  Utility,
}

impl FilterCategory {
  fn color(&self) -> Color {
    match self {
      FilterCategory::Basic => Color::Blue,
      FilterCategory::Color => Color::Red,
      FilterCategory::Geometric => Color::Green,
      FilterCategory::Artistic => Color::Magenta,
      FilterCategory::Enchancement => Color::Yellow,
      FilterCategory::Utility => Color::Cyan,
    }
  }

  fn name(&self) -> &str {
    match self {
      FilterCategory::Basic => "Basic",
      FilterCategory::Color => "Color",
      FilterCategory::Geometric => "Geometric",
      FilterCategory::Artistic => "Artistic",
      FilterCategory::Enchancement => "Enchancement",
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


fn main() {
  println!("hello world")
}