# Changelog

## 0.3.0

Behaviour change: template learning no longer goes through `min_occurrences`.
Templates compile on first alignment, because alignment already requires two
records to exist. The config field still governs `Constant`/`Arithmetic`/
`Repeat` shapes.

**Template mining went from unusable to competitive on real logs.** Measured on
12 loghub datasets against Drain3, run rather than quoted: 62.1% vs 65.5% mean
grouping accuracy on six sets downloaded *after* every parameter was frozen, and
86.6% vs 88.9% on the six used during development. XDPD wins 2 of 12 — decisively
on Proxifier (52.6% vs 2.5%) — and loses the mean. HDFS went 32.4% -> 99.7%,
Apache 1.6% -> 100%. See `examples/logbench`.

- Skeletons now widen as records arrive. Pairwise alignment marked a position
  fixed wherever two records happened to agree, including positions that vary
  across the record type; every pair froze a different set of accidents, so no
  two templates shared a signature, frequency counting never saw a repeat, and
  the largest event types compiled nothing at all.
- A record joins a skeleton in exactly one place, under one rule: it may turn at
  most a quarter of that skeleton's fixed positions variable. Previously records
  could also merge in via template-to-template comparison, which does not
  converge — agreement shrinks as templates widen — and let a lone record of a
  different type dissolve a skeleton with hundreds of members.
- Templates no longer decay to death mid-ingest. `observe` reinforced nothing,
  so a skeleton learned early hit the strength floor about 1000 observations
  later while records of its own type were still arriving. This was the single
  largest accuracy effect.
- Alignment candidates are bucketed by record length, so two records of the same
  shape can meet however far apart they arrive. Bounded by `RECENT_TOTAL`.
- Compression on HDFS 1.45x -> 11.90x program-level ops. Learn time ~1.3ms per
  2000 lines. Losslessness held on all twelve datasets.
- Removed fabricated provenance from the demo: sequences attributed to Yahoo
  Finance and to production nginx were hand-written arithmetic progressions.
  They are now labelled synthetic, which is what they always were.
- `examples/logbench/drain3_baseline.py` runs the Drain3 baseline instead of
  citing its published figures. It reproduces them.
- Numeric anomaly detection re-measured at 0.772 mean F1 against a rolling
  z-score's 0.962. Still loses.

## 0.2.1

- Dual-licensed under MIT OR Apache-2.0 (was MIT-only). Apache-2.0 adds an
  explicit patent grant with patent retaliation that plain MIT lacks.
- Fixed a missing root-level `LICENSE` file — the license lived only inside
  `xdpd/`, so GitHub's license detector reported the repo as unlicensed.

## 0.2.0

Breaking change (`Skill` fields, `Instr::Call` signature).

- Skills now generalize across values instead of memorizing one instance.
  A skill compiled from `[0,2,4,6,8]` also matches `[100,102,104,106,108]`
  and any other unseen sequence with the same shape (delta=2, len=5).
  Implemented via a value-free `PatternShape` that skills match against
  and re-derive output from at call time, instead of a frozen output
  sequence baked in at compile time.
- Subroutine table now persists across process restarts —
  `Learner::save_to_file` / `load_from_file`, plain-text format, zero
  extra dependencies.
- Fixed `Pattern::signature()` for `Constant` patterns, which still baked
  in the concrete value unlike `Arithmetic`/`Repeat` — caused redundant
  duplicate skills for different constant values of the same length.
- Fixed the crate source being excluded from GitHub via `.gitignore`
  while simultaneously published to crates.io — the tarball already ships
  the full source, so the exclusion gave no real protection while
  blocking code review and discoverability.
- Fixed `examples/` and `examples/gateway/` depending on the published
  crates.io version instead of the local source, which meant the demo
  and benchmark had silently been running the old, unfixed crate.

## 0.1.0

Initial release: pattern detection (constant, arithmetic, repeat),
subroutine compilation, DP-based composition, anomaly detection.

Known issues fixed in 0.2.0: MSRV-only `is_multiple_of()` call (required
Rust 1.87+ despite no declared `rust-version`), dead `repository` link in
`Cargo.toml`.
