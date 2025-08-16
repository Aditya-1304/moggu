#!/bin/bash

echo "🎨 MOGGU COLOR EFFECTS BENCHMARK"
echo "==============================="

IMAGE="test_images/Arcade_decay_red.png"

echo "📸 Input: $IMAGE"
echo ""

echo "🌈 Testing saturation..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' saturate 1.5"

echo "🔄 Testing color invert..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' invert"

echo "🎭 Testing hue rotation..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' hue_rotate 90"
