#!/bin/bash
# filepath: test_utility.sh

echo "🛠️ MOGGU UTILITY FUNCTIONS BENCHMARK"
echo "===================================="

IMAGE="test_images/Arcade_decay_red.png"

echo "📸 Input: $IMAGE"
echo ""

echo "✂️ Testing crop..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' crop 100 100 1000 1000"

echo "🎨 Testing ASCII conversion..."
hyperfine --warmup 1 "./target/release/benchmark '$IMAGE' to_ascii"
