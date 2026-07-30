#!/bin/sh
# Fetch the loghub benchmark datasets used by this bench.
# Source: https://github.com/logpai/loghub (not vendored — fetched on demand)
set -e
mkdir -p data
BASE="https://raw.githubusercontent.com/logpai/loghub/master"
for pair in "HDFS/HDFS_2k" "Apache/Apache_2k"; do
  name=$(basename "$pair")
  echo "fetching $name ..."
  curl -sSL -o "data/${name}.log" "$BASE/${pair}.log"
  curl -sSL -o "data/${name}.log_structured.csv" "$BASE/${pair}.log_structured.csv"
done
echo "done. now: cargo run --release -- data/HDFS_2k.log_structured.csv"
