#!/bin/bash

OUTPUT_FILE="memtier_results.log"

echo "Starting Rust server..."
cargo run --release &

echo "Running Phase 1: SET benchmark..."
{
  echo "=========================================="
  echo "PHASE 1: SET BENCHMARK"
  echo "=========================================="
  memtier_benchmark -s 127.0.0.1 -p 6379 \
    --command="SET __key__ __data__" \
    --command-key-pattern=S \
    --key-maximum=100000 \
    --key-prefix="memtier-" \
    -c 50 -t 4 -n 2000 -d 256
  echo ""
} >>"$OUTPUT_FILE" 2>/dev/null

echo "Running Phase 2: GET benchmark..."
{
  echo "=========================================="
  echo "PHASE 2: GET BENCHMARK"
  echo "=========================================="
  memtier_benchmark -s 127.0.0.1 -p 6379 \
    --command="GET __key__" \
    --command-key-pattern=R \
    --key-maximum=100000 \
    --key-prefix="memtier-" \
    -c 50 -t 4 -n 5000
  echo ""
} >>"$OUTPUT_FILE" 2>/dev/null

echo "Benchmark completed. Results saved to $OUTPUT_FILE"
