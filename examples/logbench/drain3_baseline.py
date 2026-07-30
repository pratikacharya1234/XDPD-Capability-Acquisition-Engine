#!/usr/bin/env python3
"""Drain3 baseline for the XDPD log benchmark.

Runs Drain3 over the same `Content` field and scores it with the same Grouping
Accuracy definition as `src/main.rs`, so the two numbers are comparable. The
baseline used to be quoted from the literature; quoting a number you have not
run is exactly the habit this benchmark exists to break.

    python3 -m venv venv && ./venv/bin/pip install drain3
    ./venv/bin/python drain3_baseline.py data/HDFS_2k.log_structured.csv
"""

import csv
import sys
from collections import defaultdict

from drain3 import TemplateMiner


def grouping_accuracy(predicted, truth):
    """A message is correct only when its predicted cluster holds exactly the
    same set of messages as its ground-truth cluster. Identical to the Rust
    implementation in src/main.rs."""
    pred_groups = defaultdict(set)
    true_groups = defaultdict(set)
    for i, p in enumerate(predicted):
        pred_groups[p].add(i)
    for i, t in enumerate(truth):
        true_groups[t].add(i)

    true_sets = {frozenset(v) for v in true_groups.values()}
    correct = sum(len(s) for s in pred_groups.values() if frozenset(s) in true_sets)
    return correct / len(truth), len(pred_groups)


def main(path):
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    contents = [r["Content"] for r in rows]
    truth = [r["EventId"] for r in rows]

    miner = TemplateMiner()
    for line in contents:
        miner.add_log_message(line)

    # Score in a second pass so every line is judged against the final template
    # set, matching how the Rust benchmark assigns clusters after learning.
    predicted = []
    for line in contents:
        cluster = miner.match(line)
        predicted.append(cluster.cluster_id if cluster else -1)

    ga, clusters = grouping_accuracy(predicted, truth)
    unmatched = sum(1 for p in predicted if p == -1)

    print(f"Drain3 baseline: {path}")
    print(f"lines:                    {len(contents)}")
    print(f"ground-truth event types: {len(set(truth))}")
    print(f"templates mined:          {len(miner.drain.clusters)}")
    print(f"predicted clusters:       {clusters}")
    print(f"unmatched lines:          {unmatched}")
    print(f"grouping accuracy (GA):   {ga * 100:.1f}%")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "data/HDFS_2k.log_structured.csv")
