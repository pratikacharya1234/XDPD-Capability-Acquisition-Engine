# XDPD: Capability Acquisition Engine

A learning mechanism that grows its own instruction set from observed patterns.
Zero dependencies. Pure Rust. CPU only.

[![Crates.io](https://img.shields.io/crates/v/xdpd)](https://crates.io/crates/xdpd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![docs.rs](https://img.shields.io/docsrs/xdpd)](https://docs.rs/xdpd)

## Overview

XDPD detects invariants in token streams and compiles them into executable subroutines. The subroutine table **is** the memory. One `Call` instruction executes an entire learned subroutine — atomic regardless of body size.

No neural networks. No weights. No gradient descent. No GPU.

## Measured against established baselines

Both benchmarks download public datasets with published ground truth and run the
incumbent tool for comparison rather than quoting its published numbers.

| Benchmark | XDPD | Baseline |
|---|---|---|
| Log template mining, 6 loghub sets, parameters frozen before download | 62.1% mean grouping accuracy | **Drain3 65.5%** |
| Log template mining, 6 development sets | 86.6% | **Drain3 88.9%** |
| Numeric anomaly detection, NAB, 4 series | 0.772 mean F1 | **rolling z-score 0.962** |

**XDPD wins 2 of 12 log datasets and loses the mean.** It wins Proxifier
decisively (52.6% vs Drain3's 2.5%), where Drain's prefix tree collapses. On
numeric anomaly detection it loses outright to five lines of arithmetic.

Every line a template claims reproduces byte-for-byte — losslessness held on all
twelve datasets. Learn time is ~1.3ms per 2000 log lines.

Reproduce: `examples/logbench` and `examples/tsbench` in the
[repository](https://github.com/pratikacharya1234/XDPD-Capability-Acquisition-Engine).

## Quick Start

```bash
cargo add xdpd
```

```rust
use xdpd::{Learner, Token};

let mut learner = Learner::new();
learner.observe(&vec![0, 2, 4, 6, 8]);

let (output, ops) = learner.generate(&vec![0, 2, 4, 6, 8], true);
```

## API

```rust
let mut learner = Learner::new();

// Observe token sequences — returns names of newly learned skills
learner.observe(&vec![0, 1, 2, 3, 4]);

// Generate output with or without learned skills
let (output, program_ops) = learner.generate(&target, true);

// Compression ratio as an anomaly signal: familiar structure compresses,
// unfamiliar structure does not. Higher is more familiar.
// Measured on NAB it has good recall and poor precision — it flags the real
// anomalies and much of the normal region too, scoring 0.772 mean F1 against
// a rolling z-score's 0.962. Treat it as a cheap screen, not a detector.
let ratio = learner.check_anomaly(&sequence);

// Access learned skills
for skill in learner.skills() {
    println!("{}: {} ops", skill.name, skill.instruction_count());
}

// Standalone pattern detection
use xdpd::detect_pattern;
let pattern = detect_pattern(&vec![0, 2, 4, 6, 8]);

// DP composition over any skill table
use xdpd::compose;
let (program, cost) = compose(&skill_table, &target_sequence);

// Persist the subroutine table across process restarts
learner.save_to_file("skills.tsv")?;
let mut restarted = Learner::new();
restarted.load_from_file("skills.tsv")?;
```

## Generalization

Skills match on *structure*, not the exact values they were compiled from.
Learning from `[0, 2, 4, 6, 8]` also recognizes `[100, 102, 104, 106, 108]`
and any other unseen delta=2, length=5 sequence — one skill, not one per
instance:

```rust
let mut learner = Learner::new();
for _ in 0..3 {
    learner.observe(&vec![0, 2, 4, 6, 8]); // trains on this instance only
}
assert_eq!(learner.skill_count(), 1);

// Never observed, same shape — still compresses to a single Call.
let (output, ops) = learner.generate(&vec![9000, 9002, 9004, 9006, 9008], true);
assert_eq!(ops, 1);
```

## Architecture

```
Observation -> Pattern Detection -> Subroutine Compilation -> DP Composition -> Execution
                                      |
                              subroutine table (the memory)
```

## Key Properties

| Property | Implementation |
|---|---|
| No separate memory | Subroutine table IS the memory |
| No neural networks | No weights, no gradients, no GPU |
| Atomic execution | Call = 1 instruction, any body size |
| Structural forgetting | Observation window eviction |
| Compositional | DP over skills for minimal program |
| Generalizes across values | Skills store a value-free `PatternShape`, not a frozen instance |
| Learns templates from records | Skeletons widen as records arrive; a record may turn at most a quarter of a skeleton's fixed positions variable |
| Survives restarts | Plain-text save/load, zero extra dependencies |
| Zero dependencies | Pure Rust standard library |

## Examples

| Directory | What it is |
|---|---|
| [`examples/logbench`](../examples/logbench/) | Template mining vs Drain3 on 12 loghub datasets. Real data, published ground truth, baseline actually run. |
| [`examples/tsbench`](../examples/tsbench/) | Anomaly detection vs a rolling z-score on NAB. Real labelled data. |
| [`examples/`](../examples/) | Mechanism walkthrough on **synthetic** hand-written sequences. Illustrates how it works; proves nothing about accuracy. |

```bash
cd ../examples/logbench && ./fetch-data.sh && cargo run --release -- data/HDFS_2k.log_structured.csv
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
