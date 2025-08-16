#!/bin/bash

echo "⚡ MOGGU INDIVIDUAL FUNCTION PERFORMANCE TEST"
echo "============================================="

IMAGE="test_images/Arcade_decay_red.png"

echo "📸 Input: $IMAGE (17.5M pixels)"
echo ""

echo " Testing Brightness..."
./target/release/benchmark "$IMAGE" brightness 50

echo ""
echo " Testing Contrast..."
./target/release/benchmark "$IMAGE" contrast 1.5

echo ""
echo " Testing Box Blur..."
./target/release/benchmark "$IMAGE" box_blur 20

echo ""
echo " Testing Gaussian Blur..."
./target/release/benchmark "$IMAGE" gaussian_blur 10.0

echo ""
echo " Testing Sharpen..."
./target/release/benchmark "$IMAGE" sharpen 1.2

echo ""
echo " Testing Edge Detection..."
./target/release/benchmark "$IMAGE" edge_detection

echo ""
echo " Testing Thresholding..."
./target/release/benchmark "$IMAGE" thresholding 128

echo ""
echo " Testing Sepia..."
./target/release/benchmark "$IMAGE" sepia

echo ""
echo " Testing Vignette..."
./target/release/benchmark "$IMAGE" vignette 0.8

echo ""
echo " Testing Noise..."
./target/release/benchmark "$IMAGE" noise 25

echo ""

echo " Testing Saturate..."
./target/release/benchmark "$IMAGE" saturate 1.5

echo ""
echo " Testing Invert..."
./target/release/benchmark "$IMAGE" invert

echo ""
echo " Testing Hue Rotate..."
./target/release/benchmark "$IMAGE" hue_rotate 90

echo ""

echo " Testing Rotate 90°..."
./target/release/benchmark "$IMAGE" rotate90

echo ""
echo " Testing Rotate 180°..."
./target/release/benchmark "$IMAGE" rotate180

echo ""
echo "↔ Testing Flip Horizontal..."
./target/release/benchmark "$IMAGE" flip_horizontal

echo ""
echo "↕ Testing Flip Vertical..."
./target/release/benchmark "$IMAGE" flip_vertical

echo ""

echo " Testing Crop..."
./target/release/benchmark "$IMAGE" crop 100 100 1000 1000

echo ""
echo " Testing Grayscale..."
./target/release/benchmark "$IMAGE" to_grayscale

echo ""
