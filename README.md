# XDPD: Capability Acquisition Engine

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

| LLM Model | Per 1M Tokens | Monthly Saved (50K q/day) |
|---|---|---|
| GPT-5.5 Instant | $0.15 / $0.60 | $8.44 |
| Claude Haiku 4.5 | $1.00 / $5.00 | $56.25 |
| Claude Sonnet 5 | $3.00 / $15.00 | $168.75 |
| GPT-5.6 Sol | $2.50 / $10.00 | $112.50 |
| Claude Opus 4.8 | $5.00 / $25.00 | $281.25 |

## Quick Start

```rust
use xdpd::{Learner, Token};

let mut learner = Learner::new();
learner.observe(&vec![0, 2, 4, 6, 8]);
let (output, ops) = learner.generate(&vec![0, 2, 4, 6, 8], true);
println!("{:?} in {} ops", output, ops);
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

MIT
