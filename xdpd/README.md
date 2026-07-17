# XDPD: Capability Acquisition Engine

A learning mechanism that grows its own instruction set from observed patterns.
Zero dependencies. Pure Rust. CPU only.

[![Crates.io](https://img.shields.io/crates/v/xdpd)](https://crates.io/crates/xdpd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![docs.rs](https://img.shields.io/docsrs/xdpd)](https://docs.rs/xdpd)

## Overview

XDPD detects invariants in token streams and compiles them into executable subroutines. The subroutine table **is** the memory. One `Call` instruction executes an entire learned subroutine — atomic regardless of body size.

No neural networks. No weights. No gradient descent. No GPU.

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

// Check if a sequence is anomalous (low compression = unknown pattern)
let ratio = learner.check_anomaly(&sequence);
// ratio >= 2.0 -> normal, ratio < 1.3 -> anomalous

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
| Survives restarts | Plain-text save/load, zero extra dependencies |
| Zero dependencies | Pure Rust standard library |

## Example

See the [examples directory](../examples/) for a full real-world demonstration:
- S&P 500 pattern learning from historical data
- Server latency baseline detection
- IoT sensor heartbeat recognition
- Anomaly detection on market crashes and latency spikes
- LLM cost reduction analysis (GPT-4o, Claude)

Run it:
```bash
cd ../examples && cargo run --release
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
