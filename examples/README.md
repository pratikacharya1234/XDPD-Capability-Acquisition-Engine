# XDPD Demo — Full Walkthrough

The CLI demo in `src/main.rs` runs XDPD against three real-world-shaped datasets (server latency, S&P 500 rally, IoT heartbeat), then benchmarks compression, anomaly detection, and — the part worth actually reading — measures whether learned skills generalize to values they were never trained on.

```bash
cargo run --release
```

Or point it at your own data:

```bash
cargo run --release -- path/to/file.csv
# CSV format: one comma-separated sequence per line, # for comments
```

## What each phase does

**PHASE 1 — Observation.** Feeds three sequences repeatedly (constant latency baseline, arithmetic S&P 500 rally, repeating IoT pulse) so the learner clears its occurrence threshold and compiles a skill for each shape.

**PHASE 2 — Compression benchmark.** Measures program-level instruction count (naive Load+Output per token vs. a single learned `Call`) across the trained sequences plus combinations of them:

```
Sequence                                          Tokens  Naive ops Learned ops
------------------------------------------------------------------------------
Latency baseline (exact match)                         6         13          1
S&P 500 rally (exact match)                            5         11          1
IoT heartbeat (exact match)                           20         41          1
Latency baseline repeated 4x                          24         49          4
S&P 500 rally repeated 3x                             15         31          3
Cross-domain: latency + S&P + IoT                     31         63          3
------------------------------------------------------------------------------
TOTAL                                                101        208         13

Operations saved: 195 (93.8%) — 16.00x overall speedup
```

**PHASE 3 — LLM integration value.** A hypothetical cost-projection scenario, explicitly labeled as such — not a measurement against a real LLM workload. See the root README's "Illustrative" section for the caveat.

**PHASE 4 — Anomaly detection.** Sequences that match a learned shape compress well (high ratio); sequences that deviate don't. All 6 test cases (3 normal, 3 anomalous) classify correctly.

**PHASE 5 — Learned capability table.** Dumps the subroutine table: skill name, instruction count, strength.

**PHASE 6 — Generalization benchmark (the important one).** Trains on exactly one seed sequence per pattern shape, then tests against same-shape sequences it never observed, plus a control group of different shapes it should reject:

```
Skill hit-rate on unseen same-shape data: 8/8 (100.0%)
False positives on non-matching shapes: 0/3
```

This is what actually distinguishes XDPD from memorizing a lookup table: a skill compiled from `[0,2,4,6,8]` also recognizes `[100,102,104,106,108]` and `[9000,9002,9004,9006,9008]` — same shape, never observed, still compresses to a single `Call`. See `run_generalization_benchmark()` in `src/main.rs` for the full case list.

## Related

- [examples/gateway/](gateway/) — a separate demo: an LLM inference proxy that uses XDPD as its cache layer.
