# XDPD numeric-stream anomaly benchmark

Tests whether XDPD's compression-ratio anomaly signal beats a rolling z-score on
real labelled data.

**It does not. Mean F1 0.772 vs 0.962, winning no series.** Recorded here
because a measured loss you can act on is worth more than an unmeasured claim.

## Run it

```sh
./fetch-data.sh
cargo run --release
```

Data is the [Numenta Anomaly Benchmark](https://github.com/numenta/NAB), whose
`combined_windows` labels give hand-marked anomaly windows per series. Fetched on
demand, not vendored.

## Why this benchmark exists

The log benchmark (`../logbench`) showed XDPD loses template mining to Drain3 on
every axis, losslessness included. Two properties survived as *possibly* distinct
(`docs/ARCHITECTURE.md` §I.3): boundary-free stream coverage, and algebraic
primitives — `Seq` encodes an arithmetic progression in one instruction where
grammar induction spends a rule per step.

The bar was set deliberately low: a **rolling z-score**, five lines of
arithmetic. If XDPD cannot beat that, the numeric-stream direction is not worth
pursuing.

## Method

- Discretization: z-normalize, quantize to 16 bands (the step SAX performs).
- Learning: 8-token windows over the leading 15% only — NAB's probationary period.
- XDPD score: `1 / check_anomaly(window)`. Familiar structure compresses; unfamiliar
  structure does not.
- Baseline: rolling z-score on raw values, 64-point window.
- Scoring: a labelled window counts as detected if any flagged point falls inside
  it; flags outside every window are false positives. **The F1-maximising
  threshold is chosen per detector**, so neither gets a tuning advantage — an
  oracle choice, but symmetrically generous.

## Results — measured 2026-07-30, xdpd 0.2.1 + phases 1-7

| series | windows | XDPD F1 | z-score F1 | windows found |
|---|---|---|---|---|
| machine_temperature_system_failure | 4 | **0.856** | **0.856** | 3/4 vs 3/4 |
| ec2_request_latency_system_failure | 3 | 0.545 | **1.000** | 3/3 vs 3/3 |
| ec2_cpu_utilization_5f5533 | 2 | 0.704 | **0.999** | 2/2 vs 2/2 |
| nyc_taxi | 5 | **0.993** | **0.993** | 5/5 vs 5/5 |
| **mean** | | **0.772** | **0.962** | won 0 of 4 |

Re-measured after the template-generalization fix in `Learner::observe` (see
`../logbench/README.md`), which lifted the mean from 0.748. The conclusion is unchanged.

## What the numbers actually say

**Recall is fine. Precision is the problem.** XDPD found the same anomaly windows
as the z-score on every series (3/4, 3/3, 2/2, 5/5). It loses because it also
flags large stretches of the normal region — the compression ratio is a noisy
signal, not a blind one.

## What this benchmark does *not* settle

Stated so nobody over-reads it, in either direction:

- **The algebraic-primitive claim is still untested.** NAB is noisy sensor data
  with no clean arithmetic ramps, so `Seq` never gets the chance to pay off.
  Counters, sequence numbers, and byte totals would test it — but note that
  delta-of-delta encoding already solves exactly that, and ships in Gorilla and
  Prometheus. That space looks occupied too.
- **The discretization is crude**: global z-normalize plus equal-width bands, not
  proper SAX with PAA and Gaussian breakpoints. A better adapter might narrow the
  gap. This is not much of a defence, though: the z-score needed no adapter at
  all, which is itself part of why it wins.
- **Only four series.** Enough to kill a claim, not enough to establish one.
