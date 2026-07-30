#!/bin/sh
# Fetch the loghub benchmark datasets used by this bench.
# Source: https://github.com/logpai/loghub (not vendored — fetched on demand)
set -e
mkdir -p data
BASE="https://raw.githubusercontent.com/logpai/loghub/master"
# The first six were used while developing and tuning the template logic. The
# last six were downloaded only after every parameter was frozen, and are the
# only honest evidence of how this generalizes. Keep that split.
for pair in "HDFS/HDFS_2k" "Apache/Apache_2k" \
            "BGL/BGL_2k" "Zookeeper/Zookeeper_2k" \
            "Hadoop/Hadoop_2k" "Spark/Spark_2k" "HealthApp/HealthApp_2k" \
            "Proxifier/Proxifier_2k" "Mac/Mac_2k" "HPC/HPC_2k" \
            "Linux/Linux_2k" "OpenSSH/OpenSSH_2k"; do
  name=$(basename "$pair")
  echo "fetching $name ..."
  curl -sSL -o "data/${name}.log_structured.csv" "$BASE/${pair}.log_structured.csv"
done
echo "done. now: cargo run --release -- data/HDFS_2k.log_structured.csv"
