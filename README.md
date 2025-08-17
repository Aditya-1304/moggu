# 🎨 Moggu - Ultra-Fast Image Processing Tool

A lightning-fast, terminal-based image processing application built in Rust that delivers professional-grade filters in under 200ms, even for large images.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-Interface-blueviolet?style=for-the-badge)
![Performance](https://img.shields.io/badge/Performance-<200ms-brightgreen?style=for-the-badge)

## ⚡ Key Features

### 🚀 **Blazing Fast Performance**
- **Sub-200ms processing** for most operations, even on heavy files (10MB+)
- Multi-threaded processing with optimized algorithms
- Memory-efficient operations using Rust's zero-cost abstractions
- Real-time progress tracking

### 🎯 **22+ Professional Filters**
- **Basic Filters**: Grayscale conversion
- **Color Manipulation**: Brightness, contrast, saturation, hue rotation, inversion, sepia
- **Blur & Sharpening**: Gaussian blur, box blur, sharpen, edge detection
- **Artistic Effects**: Oil painting, vignette, noise, thresholding
- **Geometric Operations**: Rotation (90°/180°/270°), horizontal/vertical flip, crop
- **Utility Tools**: ASCII art conversion

### 💻 **Intuitive Terminal UI**
- Beautiful, responsive interface with color-coded categories
- Built-in file browser with async file dialogs
- Smart parameter validation with range checking
- Interactive help system with keyboard shortcuts
- Category filtering for easy navigation

### 🖼️ **Advanced Image Preview**
- **Inline image display** directly in terminal
- Support for `chafa` and `viu` rendering engines
- Preview processed images without leaving the application
- Terminal-friendly image viewing

## 🛠️ Installation

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For best image preview experience (optional)
sudo pacman -S chafa          # Arch Linux
# or
cargo install viu             # Cross-platform
```

### Build & Run
```bash
git clone https://github.com/yourusername/moggu
cd moggu
cargo build --release
cargo run
```

## 🎮 Usage

### Quick Start
1. **Launch**: `cargo run`
2. **Select Files**: Use built-in browser (`f` for input, `o` for output) or type paths
3. **Choose Filter**: Navigate with `↑/↓` or `j/k`, cycle categories with `c`
4. **Set Parameters**: Enter values or use defaults
5. **Process**: Automatic processing with real-time progress
6. **Preview**: Press `v` to view results inline

### Keyboard Controls

#### Global
- `q` - Quit application
- `h` - Toggle help
- `Esc` - Go back
- `r` - Reset to start

#### File Input
- `f` - Open file browser (input)
- `o` - Open file browser (output)
- `Tab`/`Enter` - Next field

#### Filter Selection
- `↑/↓` or `j/k` - Navigate filters
- `c` - Cycle categories
- `Enter` - Select filter

#### Parameter Input
- `Tab`/`Enter` - Next parameter
- `↑` - Previous parameter
- `Backspace` - Delete

#### Results
- `v` - View processed image
- `c` - Clear displayed images
- `r` - Process another image

## 🎨 Filter Categories

### 🔵 Basic (1 filter)
- **Grayscale**: Convert to grayscale

### 🟣 Color (4 filters)  
- **Saturate**: Adjust color saturation (0.0-3.0)
- **Invert**: Invert all colors
- **Hue Rotate**: Shift hue spectrum (-360° to 360°)
- **Sepia**: Apply warm sepia tone

### 🟦 Enhancement (7 filters)
- **Brightness**: Adjust luminosity (-100 to 100)
- **Contrast**: Modify contrast (0.1-3.0)
- **Gaussian Blur**: Smooth blur (0.1-20.0 sigma)
- **Box Blur**: Fast blur (1-50 radius)
- **Sharpen**: Enhance details (0.1-3.0)
- **Edge Detection**: Sobel edge detection
- **Thresholding**: Binary threshold (0-255)

### 🟪 Artistic (4 filters)
- **Vignette**: Dark edge effect (0.1-1.0)
- **Noise**: Add random noise (1-100)
- **Oil Painting**: Artistic oil effect (radius 1-10, levels 5-50)

### 🟨 Geometric (6 filters)
- **Rotate**: 90°, 180°, 270° rotation
- **Flip**: Horizontal/vertical mirroring
- **Crop**: Custom rectangle cropping

### 🟩 Utility (2 filters)
- **ASCII Art**: Convert to text representation
- Advanced cropping with position control

## 🔧 Technical Details

### Architecture
- **Language**: Rust 2024 Edition
- **UI Framework**: Ratatui with Crossterm
- **Image Processing**: Custom algorithms + image crate
- **Async Runtime**: Tokio for file operations
- **Concurrency**: Multi-threaded processing

### Optimization Features
- **SIMD Instructions**: Vectorized operations
- **Memory Pooling**: Reduced allocations
- **Lazy Loading**: On-demand processing
- **Pipeline Architecture**: Streaming transforms
- **Zero-Copy Operations**: Direct memory access

### Supported Formats
- **Input**: PNG, JPEG, BMP, TIFF, GIF
- **Output**: PNG, JPEG, BMP, TIFF

## 🖥️ Terminal Compatibility

### Recommended Terminals
- **Kitty** (Best image preview support)
- **iTerm2** (macOS)
- **Alacritty**
- **GNOME Terminal**
- **Windows Terminal**

### Image Preview Support
- **chafa** (Recommended) - Rich color display
- **viu** - Fast Unicode display
- **Fallback** - File path display

## 🤝 Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Commit changes: `git commit -am 'Add feature'`
4. Push branch: `git push origin feature-name`
5. Submit pull request

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

**⚡ Built for speed. Designed for productivity. Perfect for