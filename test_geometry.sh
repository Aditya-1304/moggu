#!/bin/bash

echo "📐 MOGGU GEOMETRIC TRANSFORMATIONS BENCHMARK"
echo "==========================================="

IMAGE="test_images/Arcade_decay_red.png"

echo "📸 Input: $IMAGE"
echo ""

echo "🔄 Testing 90° rotation..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' rotate90"

echo "🔄 Testing 180° rotation..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' rotate180"

echo "🔄 Testing 270° rotation..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' rotate270"

echo "↔️ Testing horizontal flip..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' flip_horizontal"

echo "↕️ Testing vertical flip..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' flip_vertical"
