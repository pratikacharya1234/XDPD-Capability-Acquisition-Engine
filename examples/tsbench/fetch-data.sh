#!/bin/sh
# Fetch the NAB series and flatten its labels for this bench.
# Source: https://github.com/numenta/NAB (not vendored — fetched on demand)
set -e
mkdir -p data
B="https://raw.githubusercontent.com/numenta/NAB/master"
for f in realKnownCause/machine_temperature_system_failure.csv \
         realKnownCause/ec2_request_latency_system_failure.csv \
         realKnownCause/nyc_taxi.csv \
         realAWSCloudwatch/ec2_cpu_utilization_5f5533.csv; do
  echo "fetching $(basename "$f") ..."
  curl -sSL -o "data/$(basename "$f")" "$B/data/$f"
done
curl -sSL -o data/combined_windows.json "$B/labels/combined_windows.json"
python3 - <<'PY'
import json
d = json.load(open('data/combined_windows.json'))
want = {"machine_temperature_system_failure.csv","ec2_request_latency_system_failure.csv",
        "ec2_cpu_utilization_5f5533.csv","nyc_taxi.csv"}
rows = [(k.split('/')[-1], w[0], w[1]) for k, v in d.items() for w in v
        if k.split('/')[-1] in want]
open('data/labels.csv','w').write('file,start,end\n' + '\n'.join(','.join(r) for r in rows) + '\n')
print(f"{len(rows)} labelled windows")
PY
echo "done. now: cargo run --release"
