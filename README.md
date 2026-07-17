# XDPD: Capability Acquisition Engine

[![Crates.io](https://img.shields.io/crates/v/xdpd)](https://crates.io/crates/xdpd)
[![docs.rs](https://img.shields.io/docsrs/xdpd)](https://docs.rs/xdpd)
[![CI](https://github.com/pratikacharya1234/XDPD-Capability-Acquisition-Engine/actions/workflows/ci.yml/badge.svg)](https://github.com/pratikacharya1234/XDPD-Capability-Acquisition-Engine/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](xdpd/LICENSE-MIT)

Learning by growing an instruction set, not storing data.
No neural networks. No separate storage. No GPU. Zero dependencies.

```
cargo add xdpd
```

## What It Does

```
Observation Stream -> Pattern Detection -> Subroutine Compilation -> DP Composition -> Execution
                                     |
                             subroutine table (the memory)
```

Starts with 5 primitives: `Load`, `Output`, `Seq`, `Call`, `Ret`. All other capabilities are learned.

## Proven Results (Live Demo)

101 tokens across 3 domains. 38 observations. 3 skills learned.

| Metric | Value |
|---|---|
| Naive operations | 208 |
| Learned operations | 13 |
| Operations saved | 93.8% |
| Overall speedup | 16x |
| Anomaly detection | 6/6 correct |

Reproduce it: `cd examples && cargo run --release`.

## Generalization (Measured)

Skills store a value-free structural template, not a frozen copy of one
instance — `[0,2,4,6,8]` and `[100,102,104,106,108]` both compile to the
*same* skill because they share a shape (delta=2, len=5), not because they
share values. The demo trains on exactly one seed sequence per pattern
shape, then tests against same-shape sequences it never observed, plus a
control group of different shapes it should reject:

| Test | Result |
|---|---|
| Hit-rate on unseen same-shape values | 8/8 (100%) |
| False positives on non-matching shapes | 0/3 |

```rust
let mut learner = Learner::new();
for _ in 0..3 {
    learner.observe(&vec![0, 2, 4, 6, 8]); // trains on this instance only
}
assert_eq!(learner.skill_count(), 1);

// Never observed, same shape — still compresses to a single Call.
let (_, ops) = learner.generate(&vec![9000, 9002, 9004, 9006, 9008], true);
assert_eq!(ops, 1);
```

Full table (including the control group and how each case is constructed)
is in PHASE 6 of `examples/src/main.rs` — `cd examples && cargo run --release`.

### Illustrative: LLM cost projection (not measured)

The numbers below are a hypothetical scenario computed from three assumed
inputs — 120 tokens per matched reasoning pattern, 50,000 queries/day, a
25% repeat rate — not a measurement against a real LLM workload. They
model what the compression ratio above *could* be worth if XDPD sat in
front of an LLM and those assumptions held; treat them as a starting point
for your own numbers, not a benchmark.

| LLM Model | Per 1M Tokens | Monthly Saved (hypothetical) |
|---|---|---|
| GPT-5.5 Instant | $0.15 / $0.60 | $6.75 |
| Claude Haiku 4.5 | $1.00 / $5.00 | $45.00 |
| GPT-5.6 Sol | $2.50 / $10.00 | $112.50 |
| Claude Sonnet 5 | $3.00 / $15.00 | $135.00 |
| Claude Opus 4.8 | $5.00 / $25.00 | $225.00 |

## Quick Start

```rust
use xdpd::{Learner, Token};

let mut learner = Learner::new();
learner.observe(&vec![0, 2, 4, 6, 8]);
let (output, ops) = learner.generate(&vec![0, 2, 4, 6, 8], true);
println!("{:?} in {} ops", output, ops);

// The subroutine table is the only memory — persist it across restarts:
learner.save_to_file("skills.tsv")?;
```

## Run the Demo

```bash
cd examples && cargo run --release
```

## Run the LLM Proxy

```bash
cd examples/gateway && cargo run --release
# Then: curl -X POST http://localhost:8080/v1/chat/completions \
#   -H "Content-Type: application/json" \
#   -d '{"model":"gpt-5.6-sol","messages":[{"role":"user","content":"hello"}]}'
```

## Documentation

| Document | Description |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Design rationale, data flow, VM internals, design decisions |
| [RESEARCH.md](RESEARCH.md) | Gap analysis, 10 research findings, hypothesis, honest limitations |
| [examples/README.md](examples/README.md) | Demo walkthrough with full output |
| [examples/gateway/](examples/gateway/) | LLM inference proxy — real running service |
| [docs.rs/xdpd](https://docs.rs/xdpd) | API documentation |
| [crates.io/crates/xdpd](https://crates.io/crates/xdpd) | Library package |

## Project Structure

```
XDPD/
├── xdpd/               # the crate — published on crates.io as `xdpd`
│   ├── src/lib.rs
│   └── README.md       # crate-level README (shown on crates.io/docs.rs)
├── docs/
│   └── ARCHITECTURE.md
├── examples/
│   ├── gateway/        # LLM inference proxy
│   ├── src/main.rs     # CLI benchmark
│   └── data/           # Sample CSV
├── RESEARCH.md
└── README.md
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
