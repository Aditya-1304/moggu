#!/bin/bash

echo "====================================="

IMAGE="test_images/Arcade_decay_red.png"


echo "📸 Input: $IMAGE (17.5M pixels)"
echo ""

hyperfine --warmup 3 --runs 10 --ignore-failure \
    --command-name "🔆 Brightness" "./target/release/benchmark '$IMAGE' brightness 50" \
    --command-name "🎨 Contrast" "./target/release/benchmark '$IMAGE' contrast 1.5" \
    --command-name "🌫️ Box Blur" "./target/release/benchmark '$IMAGE' box_blur 20" \
    --command-name "🌀 Gaussian Blur" "./target/release/benchmark '$IMAGE' gaussian_blur 10.0" \
    --command-name "✨ Sharpen" "./target/release/benchmark '$IMAGE' sharpen 1.2" \
    --command-name "🔍 Edge Detection" "./target/release/benchmark '$IMAGE' edge_detection" \
    --command-name "⚫ Threshold" "./target/release/benchmark '$IMAGE' thresholding 128" \
    --command-name "🌟 Sepia" "./target/release/benchmark '$IMAGE' sepia" \
    --command-name "🖤 Vignette" "./target/release/benchmark '$IMAGE' vignette 0.8" \
    --command-name "📡 Noise" "./target/release/benchmark '$IMAGE' noise 25" \
    --command-name "🎨 Oil Painting" "./target/release/benchmark '$IMAGE' oil_painting 3 8" \
    --command-name "🌈 Saturate" "./target/release/benchmark '$IMAGE' saturate 1.5" \
    --command-name "🔄 Invert" "./target/release/benchmark '$IMAGE' invert" \
    --command-name "🎭 Hue Rotate" "./target/release/benchmark '$IMAGE' hue_rotate 90" \
    --command-name "🔄 Rotate 90°" "./target/release/benchmark '$IMAGE' rotate90" \
    --command-name "🔄 Rotate 180°" "./target/release/benchmark '$IMAGE' rotate180" \
    --command-name "🔄 Rotate 270°" "./target/release/benchmark '$IMAGE' rotate270" \
    --command-name "↔️ Flip H" "./target/release/benchmark '$IMAGE' flip_horizontal" \
    --command-name "↕️ Flip V" "./target/release/benchmark '$IMAGE' flip_vertical" \
    --command-name "✂️ Crop" "./target/release/benchmark '$IMAGE' crop 100 100 1000 1000" \
    --command-name "⚫ Grayscale" "./target/release/benchmark '$IMAGE' to_grayscale"

echo ""
echo "BENCHMARK COMPLETE!"
echo "==================="
