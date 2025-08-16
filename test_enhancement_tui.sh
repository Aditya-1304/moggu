#!/bin/bash
# filepath: test_image_tui_batch.sh

echo "🎨 MOGGU IMAGE TUI BATCH PERFORMANCE TEST"
echo "========================================"

# Build if needed
if [ ! -f "./target/release/image_tui" ]; then
    echo "🏗️  Building image_tui..."
    cargo build --release --bin image_tui
fi

IMAGE="test_images/Arcade_decay_red.png"
OUTPUT_DIR="benchmark_outputs"
mkdir -p "$OUTPUT_DIR"

echo "📸 Input: $IMAGE"
echo "📁 Output: $OUTPUT_DIR/"
echo ""

# Comprehensive test function
comprehensive_test() {
    echo "🔥 COMPREHENSIVE TUI BENCHMARK"
    echo "============================="
    
    # Brightness tests
    echo "📸 Brightness Tests..."
    hyperfine --export-markdown "$OUTPUT_DIR/brightness_results.md" \
        --parameter-list brightness_val '-50,-20,0,20,50,80' \
        './target/release/image_tui "'$IMAGE'" brightness {brightness_val}'
    
    # Contrast tests  
    echo "🎨 Contrast Tests..."
    hyperfine --export-markdown "$OUTPUT_DIR/contrast_results.md" \
        --parameter-list contrast_val '0.5,0.8,1.0,1.5,2.0,2.5' \
        './target/release/image_tui "'$IMAGE'" contrast {contrast_val}'
    
    # Blur tests
    echo "🌫️ Blur Tests..."
    hyperfine --export-markdown "$OUTPUT_DIR/blur_results.md" \
        --parameter-list blur_radius '5,10,15,20,25,30' \
        './target/release/image_tui "'$IMAGE'" box_blur {blur_radius}'
    
    # Sharpen tests
    echo "✨ Sharpen Tests..."
    hyperfine --export-markdown "$OUTPUT_DIR/sharpen_results.md" \
        --parameter-list sharpen_val '0.2,0.5,0.8,1.0,1.2,1.5' \
        './target/release/image_tui "'$IMAGE'" sharpen {sharpen_val}'
    
    # Single-parameter tests
    echo "🔍 Single Parameter Tests..."
    hyperfine --export-markdown "$OUTPUT_DIR/single_param_results.md" \
        --command-name "Edge Detection" './target/release/image_tui "'$IMAGE'" edge_detection' \
        --command-name "Threshold 128" './target/release/image_tui "'$IMAGE'" thresholding 128' \
        --command-name "Gaussian Blur" './target/release/image_tui "'$IMAGE'" gaussian_blur 3.0'
}

# Quick test function
quick_test() {
    echo "⚡ QUICK TUI BENCHMARK"
    echo "===================="
    
    hyperfine --warmup 1 --runs 3 \
        --command-name "Brightness" './target/release/image_tui "'$IMAGE'" brightness 50' \
        --command-name "Contrast" './target/release/image_tui "'$IMAGE'" contrast 1.5' \
        --command-name "Box Blur" './target/release/image_tui "'$IMAGE'" box_blur 20' \
        --command-name "Sharpen" './target/release/image_tui "'$IMAGE'" sharpen 1.0' \
        --command-name "Edge Detection" './target/release/image_tui "'$IMAGE'" edge_detection' \
        --command-name "Thresholding" './target/release/image_tui "'$IMAGE'" thresholding 128'
}

# Run based on argument
case "${1:-quick}" in
    "comprehensive"|"comp")
        comprehensive_test
        ;;
    "quick"|*)
        quick_test
        ;;
esac

echo ""
echo "🏆 TUI BENCHMARK COMPLETE!"
echo "📊 Check $OUTPUT_DIR/ for detailed results"