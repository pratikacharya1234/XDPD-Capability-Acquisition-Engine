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

## Measured on real data — XDPD loses to the incumbents

Read this before the demo numbers below it.

| Benchmark | Dataset | XDPD | Baseline |
|---|---|---|---|
| [Log template mining](examples/logbench/) | loghub HDFS_2k | 32.4% GA | **Drain3 99.8%** |
| [Log template mining](examples/logbench/) | loghub Apache_2k | 1.6% GA | **Drain3 100.0%** |
| [Numeric anomaly detection](examples/tsbench/) | NAB, 4 series | 0.748 mean F1 | **rolling z-score 0.962** |

Both benchmarks are runnable and fetch their own data. No measured axis
currently favours XDPD over a specialized incumbent — see
[docs/ARCHITECTURE.md §I.5](docs/ARCHITECTURE.md) for what that does and does
not leave standing.

## Demo Results — synthetic data, not a benchmark

The numbers below are exact arithmetic on **sequences this project made up**,
counted at the program level. They show the mechanism works. They are not
measurements against real-world data and must not be read as any.

101 tokens across 3 domains. 38 observations. 3 skills learned.

| Metric | Value |
|---|---|
| Naive operations | 208 |
| Learned operations | 13 |
| Operations saved | 93.8% |
| Overall speedup | 16x |
| Anomaly detection | 6/6 correct |

Reproduce it: `cd examples && cargo run --release`.

## Generalization (synthetic)

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
| [RESEARCH.md](RESEARCH.md) | Gap analysis, research findings, hypothesis, honest limitations |
| [CHANGELOG.md](CHANGELOG.md) | Version history — what actually changed and why |
| [examples/README.md](examples/README.md) | Demo walkthrough with full output |
| [examples/logbench/](examples/logbench/) | Log template mining vs Drain3 on loghub — measured, XDPD loses |
| [examples/tsbench/](examples/tsbench/) | Anomaly detection vs rolling z-score on NAB — measured, XDPD loses |
| [examples/gateway/](examples/gateway/) | LLM inference proxy demo — runnable, not production-hardened |
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
│   ├── gateway/        # LLM inference proxy demo
│   ├── logbench/       # real-data benchmark vs Drain3 (loghub)
│   ├── tsbench/        # real-data benchmark vs z-score (NAB)
│   ├── src/main.rs     # CLI benchmark + generalization test
│   └── data/           # Sample CSV
├── AGENTS.md
├── .github/workflows/ci.yml
├── CHANGELOG.md
├── RESEARCH.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── README.md
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
