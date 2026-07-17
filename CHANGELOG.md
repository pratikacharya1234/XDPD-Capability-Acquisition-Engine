# Changelog

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
