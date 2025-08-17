#!/bin/bash

echo " MOGGU ARTISTIC EFFECTS BENCHMARK"
echo "==================================="

IMAGE="test_images/Arcade_decay_red.png"

echo " Input: $IMAGE"
echo ""

echo " Testing sepia effect..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' sepia"

echo " Testing vignette effect..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' vignette 0.8"

echo " Testing noise effect..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' noise 25"

echo " Testing oil painting effect..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' oil_painting 3 8"

echo " Testing grayscale effect..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' grayscale"

