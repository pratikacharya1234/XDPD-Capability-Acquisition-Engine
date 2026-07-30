# XDPD Architecture

**Current shipped**: 0.3.0 · **This document describes**: 0.3.0
**Research basis**: live literature and market review, July 2026 (Part 0). Sources at the end.
**Rule for this document**: every claim is tagged `[PROVEN]`, `[CONTESTED]`, `[OPEN]`, or
`[OURS-UNVERIFIED]`. Parts II–III are verified against `xdpd/src/lib.rs` by reading it.
Nothing here is asserted from recollection.

---

## Part 0 — Evidence Ledger

### 0.1 What is proven, with published evidence

| Claim | Status | Evidence |
|---|---|---|
| Growing a library by MDL compression is a working learning mechanism | `[PROVEN]` | [DreamCoder](https://arxiv.org/pdf/2006.08381) — wake/sleep library learning, MDL-driven refactoring, 8 domains incl. list processing, drawing, physics; solves most held-out tasks (mean 54.1s) |
| Hierarchical structure can be induced from a discrete token stream in **linear time** | `[PROVEN]` | [Sequitur](https://ml.cms.waikato.ac.nz/publications/1997/NM-IHW-Compress97.pdf) (Nevill-Manning & Witten 1997) — digram uniqueness + rule utility, proof of linear space/time |
| Grammar-induced structure finds **variable-length** motifs *and* anomalies with no prior knowledge of length, shape, or frequency | `[PROVEN]` | [GrammarViz 2.0](https://link.springer.com/content/pdf/10.1007/978-3-662-44845-8_37.pdf), [Ensemble Grammar Induction](https://openproceedings.org/2020/conf/edbt/paper_45.pdf) — SAX + Sequitur, time-series anomalies |
| Compression distance is a real, parameter-free anomaly detector | `[PROVEN]` | [Entropy 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC8156803/) — NCD across 4 security domains (HTTP anomalies, spam, DGA, sentiment) with no domain tuning |
| Online template mining over streams is production-grade and commercially deployed | `[PROVEN]` | [Drain](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf) / [Drain3](https://github.com/logpai/Drain3) — best of 13 miners on the LogPAI benchmark, in IBM production AIOps |
| Structural repetition in agentic traffic is real and exploitable | `[PROVEN]` | [XGrammar-2](https://arxiv.org/pdf/2601.04426) — repetition-state compression + cross-grammar cache, >6× tool-calling compile speedup |
| Deterministic (non-neural) schema compilation beats verbose baselines | `[PROVEN]` | [TSCG](https://arxiv.org/pdf/2605.04107) — purely structural tool-schema compilation, no embeddings |
| Semantic-cache false hits are asymmetrically catastrophic | `[PROVEN]` | [TrueFoundry](https://www.truefoundry.com/blog/semantic-caching-llm-gateway), [MeanCache](https://arxiv.org/pdf/2403.02694) — 3 false hits vs GPTCache's 54; ">0.95 cosine similarity produces silent factual errors"; *"a cache that returns false hits is strictly worse than no cache"* |
| Agent workloads consume vastly more tokens than chat | `[PROVEN]` (range, not a point) | 5–30× ([Spheron](https://www.spheron.network/blog/agentic-ai-inference-cost-2026/)), 10–100× ([AgentMarketCap](https://agentmarketcap.ai/blog/2026/04/12/ai-agent-token-consumption-gap-enterprise-agentic-workloads)); 73% of enterprises overran AI cost projections in 2026 |
| Telemetry volume is a large, growing, budgeted line item | `[PROVEN]` | Gartner: observability spend +~20% YoY, many orgs >$800k/yr ([Grepr](https://www.grepr.ai/blog/the-hidden-cost-in-observability)); AI workloads emit **10–50× more telemetry** than traditional services |
| Demand for structural dedup of streams is validated by shipped product | `[PROVEN]` | [OpenTelemetry log deduplication processor](https://opentelemetry.io/blog/2026/log-deduplication-processor/) — collapses entries, keeps counts + first/last timestamps |
| Mechanically-derived skills beat LLM-authored ones | `[PROVEN]` | SkillsBench via [SkillOps](https://arxiv.org/pdf/2605.13716): human-authored skills **+16.2 pts** pass rate; **LLM-authored skills: no measurable gain** |

### 0.2 What is contested or outright debunked — do not build on these

| Claim | Status | Why |
|---|---|---|
| "gzip+kNN beats BERT" | `[DEBUNKED]` | Reproductions found the reported accuracy used an **oracle tie-break** / top-2 accuracy; with fair tie-breaking BERT wins ([Schutte](https://kenschutte.com/gzip-knn-paper/), [Opitz](https://arxiv.org/pdf/2307.15002)). **XDPD must never cite this.** |
| "60% semantic-cache hit rate → ~$846/mo/GPU" (from the Kimi brief) | `[CONTESTED]` | Vendor-blog arithmetic. Production data: **60–70% of real queries are genuinely unique**, and *"hit rates of 61–69% from research papers don't hold up in production"* ([DEV](https://dev.to/gauravdagde/llm-semantic-caching-the-95-hit-rate-myth-and-what-production-data-actually-shows-8ga)). Research-grade 40–65% only holds for FAQ and repeated tool calls; creative and multi-turn chat is near zero. |
| Prompt compression is free money | `[CONTESTED]` | LLMLingua reaches 20× but degrades sharply at 25–30× on GSM8K, **all variants increased hallucination**, and on optimized serving frameworks there was **no substantial end-to-end latency gain** below ~10k-token prompts ([study](https://arxiv.org/pdf/2604.02985), [empirical](https://arxiv.org/pdf/2505.00019)) |
| "Nothing like XDPD exists" (RESEARCH.md §3) | `[FALSE AS WRITTEN]` | DreamCoder does library-learning-by-compression; Sequitur does linear-time hierarchical induction; Drain does online template mining. See 1.1. |

### 0.3 What is genuinely open — where a contribution can land

| Open problem | Evidence it's open |
|---|---|
| **Skill-library health**: eviction policy, redundancy detection, staleness at ecosystem scale | [SkillOps](https://arxiv.org/pdf/2605.13716): *"no established framework for ongoing library health management, skill eviction policies, or automated redundancy detection at ecosystem scale"* |
| **Agent behavioral baselines**: what "normal" is for an agent, stably, across deployments | [Learned Capability Governance](https://arxiv.org/pdf/2604.11839): baselines *"remain contextual and potentially unstable"*; and current approaches use **learned/neural** models — foolable by adversarial input |
| **Drain's structural assumptions**: it assumes leading tokens are constant and same-event messages share length; both are documented accuracy failures | [Drain paper](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf) + [analysis](https://www.mdpi.com/2076-3417/11/24/11974) |
| **Lossless** structural reuse in an inference path (vs. lossy semantic substitution) | Every cache tier surveyed is either exact-string or embedding-similarity. Nothing sits between. |

---

## Part I — The Verdict: Three Corrections the Research Forces

### I.1 XDPD is not unprecedented. It is an unoccupied *engineering* point.

The mechanism — compile observed structure into a growing vocabulary, judge by
compression — is `[PROVEN]` in DreamCoder and Sequitur. That is good news: the thesis is
not speculative. But the current `RESEARCH.md` claim that no such system exists is false and
must be corrected before release, or the first informed reader discredits the whole project.

The defensible claim is narrower and stronger:

> DreamCoder needs a neural recognition model and offline wake/sleep cycles. Sequitur induces
> grammar but has no execution model, no generalization across values, and no persistence.
> Drain mines templates but assumes leading tokens are constant and same-event messages share
> length. **XDPD is the online, zero-dependency, deterministic intersection: a VM whose
> instruction set grows from streamed observation, with no neural component anywhere in the
> loop and no assumption about where in the stream the constants sit.**

That is a real gap, and it is an engineering gap — which is the kind you can actually ship.

### I.2 The LLM prompt-cache pitch is the **weakest** use case, not the flagship.

The Kimi brief made it the headline. The production data contradicts it: 60–70% of real
queries are unique, and published hit rates don't survive contact with production traffic.
Building the flagship on a cache-hit-rate assumption that practitioners are actively
debunking is how the project gets dismissed in one HN comment.

**Demote it.** Keep the gateway as a demo of deployment shape. Move the flagship to where
the pain is measured, budgeted, and structurally repetitive by nature: **machine-generated
telemetry** — logs, traces, agent tool-call sequences. Observability spend is rising ~20% YoY
with many orgs past $800k/yr, AI workloads emit 10–50× more telemetry, and OpenTelemetry has
already *shipped* a deduplication processor — demand validated by a competitor's roadmap.

Critically, the OTel processor collapses only **identical** entries. XDPD collapses
**structural families** — same shape, different values. That is the gap, and it is precisely
what `PatternShape` was built for.

### I.3 Losslessness is a real property but **not a moat** — measured, 2026-07-30

An earlier revision of this document claimed losslessness was the differentiator, reasoning
from published findings that a false-hit cache is *strictly worse than no cache* and that
queries differing only in time period or polarity exceed 0.95 cosine similarity and produce
silent factual errors.

**That reasoning was wrong, and the measurement is in `examples/logbench`.** The cited research
concerns **semantic caches** — embedding similarity under a tuned threshold. It does not
describe **template miners**, which are already exact and positional. Generalizing it to them
was an error.

Drain3 was tested for exact positional reconstruction on the same loghub datasets. Its `<*>`
wildcards are positionally aligned, structurally equivalent to `Fixed`/`Var` slots, and every
line rebuilds exactly. So on log template mining XDPD holds **no advantage on any measured
axis**, losslessness included.

**Update, same day — template generalization closed most of the accuracy gap, then held-out
data reopened it.** Two defects were fixed in `Learner::observe`: pairwise alignment froze
coincidental agreement and never widened, and templates decayed to death mid-ingest because
`observe` reinforced nothing. Details and mechanism in `examples/logbench/README.md`.

| evaluation | XDPD mean GA | Drain3 mean GA |
|---|---|---|
| 6 sets downloaded after parameters were frozen | 62.1% | **65.5%** |
| 6 development sets (optimistic by construction) | 86.6% | **88.9%** |

XDPD wins 2 of 12 (Proxifier +50.1, OpenSSH +0.7), ties 1, loses 9. Full per-dataset
tables and the mechanism in `examples/logbench/README.md`.

An earlier revision of this section claimed the residual gap was caused by `Template`
being fixed-length. **That was wrong and is retracted**: zero event types in Linux, BGL
or OpenSSH span multiple token counts. The fixed-length limit is real but small; the
gap's actual causes on HPC and Hadoop are undiagnosed.

### I.4 The numeric-stream direction was measured too, and also loses

Pillars 1 and 2 pointed at numeric, boundary-free streams. `examples/tsbench` tested that on
NAB labelled data against a deliberately trivial baseline — a rolling z-score, five lines of
arithmetic:

| series | XDPD F1 | z-score F1 |
|---|---|---|
| machine_temperature_system_failure | **0.856** | **0.856** |
| ec2_request_latency_system_failure | 0.545 | **1.000** |
| ec2_cpu_utilization_5f5533 | 0.704 | **0.999** |
| nyc_taxi | **0.993** | **0.993** |
| **mean** | **0.772** | **0.962** |

Won 0 of 4, tied 2 of 4. **Recall was equal on every series** — XDPD found the same anomaly
windows — so the signal is not blind, it is noisy: precision collapses because it also flags
much of the normal region. (Figures re-measured after the template-generalization fix, which
lifted the mean from 0.748; it did not change the conclusion.)

### I.5 Where that leaves the project — the strategic finding

Each of XDPD's mechanisms maps onto an existing, mature, better-tuned technique, and loses to it
on measurement:

| XDPD mechanism | Established equivalent | Result |
|---|---|---|
| Template induction by alignment | Drain3 | Loses 62.1% vs 65.5% mean GA on 6 sets measured with parameters frozen first; wins 2 of 12 datasets |
| Compression ratio as anomaly signal | rolling z-score, NCD | Loses 0.772 vs 0.962 mean F1 |
| `Seq` / arithmetic runs | delta-of-delta encoding (Gorilla, Prometheus) | Solved and shipping; untested here |
| Lossless positional reconstruction | Drain3 `<*>` templates | Ties in kind, loses on coverage |

**One measured axis now favours XDPD: it beats Drain3 on 2 of 12 loghub datasets, decisively on
Proxifier (52.6% vs 2.5%), where Drain's prefix tree collapses and XDPD's exhaustive
same-length comparison does not.** On the mean it still loses, and on numeric anomaly detection
it loses outright. The honest state is *competitive in places, ahead nowhere on average*.

What remains real:

- **Packaging.** ~1000 lines of Rust, zero dependencies, WASM- and microcontroller-viable, doing
  template mining *and* numeric structure *and* anomaly scoring in one library. Drain3 needs a
  Python runtime; a z-score needs a host to live in. For edge and embedded contexts where the
  incumbents cannot go, "adequate at several things in a few tens of KiB" is a genuine value —
  and it is a *deployment* claim, testable as one, not an accuracy claim.
- **The artifact itself.** A working non-neural learning engine with an honest benchmark trail is
  a strong research and educational contribution. That framing does not require beating anyone.

What should **not** happen: another benchmark chosen because it might finally produce a win. The
two that exist were run against fair baselines and the log one was then improved until it wins 2
of 12 datasets — that is legitimate, because the benchmark was fixed first and the improvement
was measured on datasets fetched afterwards. Picking a *third* benchmark for winnability is a
different act, and it is how the project stops being trustworthy.

The discipline that made the improvement real is worth naming, because it is repeatable: fix the
metric, run the incumbent yourself rather than quoting it, diagnose before changing anything, and
download the evaluation data only after the parameters are frozen. The one change made from a
guess rather than a measurement moved grouping accuracy *backwards*, 32.4% to 22.1%.

---

## Part II — Current Architecture (0.3.0, verified against code)

```
  DOMAIN ADAPTER (caller's job)  ->  Vec<Token = u32>
  ══════════════════ crate boundary ══════════════════
  DATA PLANE
    Learner::observe(&[Token])   (or observe_token/flush to stream)
      1. detect_pattern()  — whole-sequence Constant/Arithmetic/Repeat
      2. scan_runs()       — runs buried anywhere inside the sequence
      3. absorb_record()   — fit into an existing skeleton, widening it,
                             budget: <= 1/4 of its Fixed positions per record
         else align_template() against recent records of the same length,
              seeding a new skeleton
      freq-count by signature for (1) and (2); templates compile on first
      alignment, since alignment already required two records
    compose(&skills, target) -> (Vec<Instr>, cost)
      DP per position: naive emit (2 ops) vs Call (1 op)
      candidates narrowed by MatchKey probe, verified by PatternShape::matches()
    VM::run()  |  Load · Output · Seq · Call(name,params) · Ret
  CONTROL PLANE — the only memory
    VM.subroutines: HashMap<String, Skill>                        lib:291
    Learner.learned_signatures: HashSet<String>                   lib:690
    save/load — XDPD_SKILLS_V2, TSV, one skill per line (V1 still loads)
```

**The generalization split** — the one thing that makes this more than a lookup table:

| | values | structure |
|---|---|---|
| `Pattern` (`lib:94`) | ✅ `start`, `value`, `unit` | ✅ |
| `PatternShape` (`lib:193`) | ❌ stripped | ✅ `delta`, `len`, `unit_len`, `count` |

A `Skill` stores the shape, never the instance. The body does not exist at rest —
`to_instructions(params)` (`lib:255`) synthesizes it at call time, which is why
`Instr::Call` carries `(name, params)`. **Remember this ABI; §IV.1 depends on it.**

**Two cost models, never to be conflated:**

| Metric | Where | `Call` counts as |
|---|---|---|
| Program-level | `compose()` → `dp[n]` | 1, always |
| Execution-level | `VM::instr_count` | every instruction in the synthesized body |

`Constant`/`Arithmetic` compress at both levels (`[Seq, Ret]`, 2 instructions for any length).
`Repeat` compresses **only at program level** — its body is `unit_len * 2 * count + 1`
instructions (`lib:263`), equal to naive emission. Every README headline is program-level.

---

## Part III — Where 0.3.0 Actually Binds

Rewritten for 0.3.0. The nine constraints listed here for 0.2.1 were the plan that became
phases 1-7; most are now closed and are recorded below rather than deleted, because the list is
the project's own record of what it said and whether it delivered.

### Closed in 0.3.0

| Was | Closed by |
|---|---|
| `detect_pattern` was whole-sequence-or-nothing — a run buried in noise returned `None` | `scan_runs` finds maximal non-overlapping runs anywhere inside a sequence |
| **No variable slots** — "XDPD cannot match a real log line today" | `PatternShape::Template` with `Fixed`/`Var`/`Run` slots. It now matches real log lines at 62.1% mean grouping accuracy on held-out loghub data |
| `observe()` re-ran detection over the whole window per call | Frequency counts maintained incrementally, `VecDeque` instead of `Vec::remove(0)` |
| `strength` and `uses` were dead fields; the table grew monotonically | Decay, reinforcement on use *and* on continuing to explain incoming records, `MAX_SKILLS` cap, `forget_skill` clearing `learned_signatures` too |
| `compose()` scanned every skill at every position | `MatchKey` hash probe narrows candidates before `matches()` verifies |
| `Call` silently no-ops on an unknown skill name | Reports an error and halts execution; `last_error()` exposes it |
| `Call` recursion had no depth bound | Bounded, with a test that depth stays balanced |
| `check_anomaly` divided by zero on an empty sequence | Returns a finite value |

### Open

| # | Constraint | Consequence |
|---|---|---|
| 1 | **`Template` is fixed-length.** A record type whose messages differ in token count cannot be one template. | Bounds accuracy on datasets with variable-length events. Note this did *not* explain the Linux gap — zero event types there span multiple lengths — so it is real but smaller than once claimed. |
| 2 | **The widening budget is a single global constant** (a quarter of a skeleton's positions). Sweeping it moved Linux between 17.9% and 63.1% and OpenSSH between 51.8% and 72.5%, in opposite directions. | One number cannot be right for every log shape. Per-skeleton adaptation is the obvious next idea and is unbuilt and unmeasured. |
| 3 | **HPC (-29.7) and Hadoop (-27.5) are undiagnosed.** | The two largest remaining gaps, cause unknown. Diagnose before changing anything: the one change made in this project from a guess rather than a measurement moved accuracy backwards. |
| 4 | **`absorb_record` scans every learned template of the record's length.** | O(\|S\|) per observation. Fine at the table sizes measured; unmeasured at 10k skeletons. |
| 5 | **The gateway example does not put XDPD on the hit path.** The cache is `hash_bytes(prompt) -> String`; the learner only `observe()`s. | It demonstrates deployment shape, not mechanism. Every "structural cache" claim still rests on unbuilt work. |
| 6 | **Numeric anomaly scoring has good recall and poor precision** — 0.772 vs a rolling z-score's 0.962 mean F1 on NAB. | Usable as a cheap screen, not as a detector. Unimproved this cycle. |

Pinned invariants (55 tests in `lib.rs`): shape generalization across values, template widening
and its guard, losslessness across varying values, restart round-trip, v1 format still loading,
streaming equivalence to batch, bounded pending buffer and table, decay and relearning,
composition determinism and exactness, anomaly edge cases.

---

## Part IV — v0.3 Architecture

Design principle for this version: **adopt the proven algorithm instead of inventing a
cousin of it.** Sequitur solved linear-time hierarchical induction in 1997 with a proof.
GrammarViz proved the anomaly path on top of it. Reinventing either is unpaid work.

### IV.1 Slotted shapes — the missing primitive `[unblocks everything]`

Fixes III.2. Nothing else in this document matters until this exists.

Real streams are *template + variable*: `GET /api/v1/user/8814 200 34ms`. Today's three shapes
can't express that, which is why XDPD works on synthetic sequences and not on logs.

Add **one** variant; keep the existing three as fast paths:

```rust
pub enum Slot {
    Fixed(Token),                    // must match exactly
    Var,                             // any one token, captured into params
    Run { delta: i32, len: usize },  // arithmetic/constant run, start captured
}

pub enum PatternShape {
    Constant { len: usize },                        // existing fast path
    Arithmetic { delta: i32, len: usize },          // existing fast path
    Repeat { unit_len: usize, count: usize },       // existing fast path
    Template(Vec<Slot>),                            // new: the general case
}
```

- `matches()` returns captured `Var`/`Run`-start tokens **in slot order**.
- `to_instructions(params)`: `Fixed` → `Load`+`Output`; `Var` → `Load(params[i])`+`Output`;
  `Run` → `Seq`.

**The instruction set does not change.** `Instr::Call(String, Vec<Token>)` already carries
exactly the payload a template needs. No VM change, no new opcode, no format break beyond one
new encoded shape kind. That is not a coincidence — it's the existing shape/params split
paying off.

**Losslessness is preserved and is the whole point** (I.3): fixed positions match exactly,
variable positions are reproduced from captured params, so output is bit-identical to input.
Unlike Drain, XDPD makes no assumption that constants sit at the front or that same-event
messages share length — those are Drain's two documented accuracy failures, and slot
templates simply don't have them.

### IV.2 Sequitur-based online induction — one change retires three homegrown ones

Fixes III.1 and III.3. Supersedes what a previous draft of this document listed as three
separate features (segmenting detector, hierarchy, custom decay).

Replace the window-rescan detector with online grammar induction over the token stream:

- **Digram uniqueness** → repeated adjacent pairs become rules. This *is* segmentation; it
  falls out rather than needing a hand-rolled maximal-run scanner.
- **Rule utility** (a rule used once is dissolved) → **this is the eviction policy** XDPD
  lacks (III.4), and library-level health is a documented `[OPEN]` problem per SkillOps.
  Sequitur's answer is 29 years old, proven, and free.
- **Rules referencing rules** → hierarchy (Miller's chunk-of-chunks) without inventing a
  `MetaCall` opcode: a rule body is a slot list that can contain skill references.
- **Linear time and space, with a proof.** Directly retires the O(W·n)-per-call rescan.

Grammar rules map onto `Skill` one-to-one: rule → named skill, rule body → `Template` slots,
rule utility count → `uses` (III.4's dead field, finally load-bearing). Requires the depth cap
(III.7) and hard-fail on missing skills (III.6) before hierarchy is switched on.

### IV.3 The anomaly path, promoted to a first-class output

`check_anomaly` is currently a byproduct. The evidence says it may be the most valuable
surface in the system: NCD-style compression anomaly detection is `[PROVEN]` parameter-free
across four security domains, GrammarViz proved grammar-based anomaly discovery needs no
prior knowledge of length or shape, and *learning agent behavioral baselines is `[OPEN]`* —
with existing approaches using neural models that adversarial input can fool.

XDPD's baseline is a deterministic grammar over observed tool-call sequences. Nothing to
prompt-inject. Ship: `anomaly_score()` with the III.8 divide-by-zero fixed, per-segment
attribution (*which* span failed to compress, not just a scalar), and stable scoring under
table growth.

### IV.4 Streaming ingestion

`observe_token(t)` / `observe_chunk(&[t])` with incremental induction state; `VecDeque` for any
residual window. Sequitur is inherently online, so this stops being a bolt-on. Prerequisite
for every deployment in IV.6.

### IV.5 Shape index

Bucket skills by discriminant and `span_len`; test only plausible candidates per position.
Fixes III.5. Do this before GPU work — parallelizing a full-table scan optimizes the wrong
thing.

### IV.6 Deployment targets, reordered by evidence

| Rank | Target | Why this rank |
|---|---|---|
| **1** | **Telemetry / log / trace volume reduction** | Budgeted pain (+20% YoY, >$800k/yr), AI emits 10–50× more telemetry, OTel shipped identical-entry dedup → demand proven; XDPD's edge is *structural family* dedup with counts preserved, and no Drain-style positional assumptions |
| **2** | **Agent behavioral baseline / anomaly** | The baseline problem is `[OPEN]`; incumbents are neural and foolable; XDPD is deterministic and un-injectable |
| **3** | Agent tool-call sequence compression | XGrammar-2 proves the repetition is real and worth >6×; TSCG proves deterministic beats verbose — but TSCG is a *static* schema compiler, so learning from live traffic is open |
| **4** | Edge / embedded | Real market, but TinyML predictive maintenance is mature and well-served; win is footprint + no-accelerator, not novelty |
| **5** | LLM prompt cache (the old flagship) | 60–70% of production queries are unique; keep as demo only (I.2) |

### IV.7 Deferred, with the reason stated

- **GPU data plane** — the three kernels (signature extraction, shape match, DP) are still the
  right decomposition, but sequencing matters: IV.1 + IV.5 first, or you parallelize a scan
  that shouldn't exist.
- **WASM** — near-free once persistence is feature-gated behind `std::fs`; ship after the core
  earns it.
- **Distributed skill tables** — `XDPD_SKILLS_V1` is already a mergeable wire format. Needs
  III.6 (hard-fail) and format version negotiation first.
- **Goal-directed / Levin search over the skill table** — research, explicitly not roadmap.

---

## Part V — Ship Sequence with Acceptance Criteria

Dependency order. Every criterion is a real dataset or a real capability, not a demo number.

| # | Work | Acceptance criterion |
|---|---|---|
| 1 | Slotted shapes (IV.1) | Learns a template from real log lines with varying IDs; round-trips **bit-exact**; encoded format loads back |
| 2 | Sequitur induction (IV.2) | On [LogPAI/loghub](https://github.com/logpai/Drain3) datasets, report template accuracy **against Drain3** — publish the number even if Drain wins |
| 3 | Rule utility eviction (IV.2) | Table size bounded over 10M-token run; `learned_signatures` pruned in step; no unbounded growth |
| 4 | Streaming API (IV.4) | Sustained throughput on a live stream; no per-call window rescan |
| 5 | Shape index (IV.5) | Composition latency flat from 100 → 10k skills |
| 6 | Hard-fail + depth cap (III.6, III.7) | Missing skill is an error, not silence; recursion bounded |
| 7 | Anomaly as product (IV.3) | Precision/recall on a labeled set, plus which-span attribution |
| 8 | Telemetry integration (IV.6 #1) | Volume reduction on **captured** telemetry vs OTel dedup baseline, counts preserved |

**Release gate.** No claim ships without a named dataset and a reproducible command. The
The demo numbers (16×, 93.8%) are real but **synthetic and program-level** — they must be
labeled as such everywhere, and the v0.3 headline must come from item 2 or 8 on captured
data. If Drain3 beats XDPD on template accuracy, publish that; the differentiators (lossless
reproduction, no positional assumptions, zero deps, anomaly for free) survive losing that
benchmark. A project that publishes its losing numbers is the one people trust with the
winning ones.

---

## Part VI — Non-Goals, and Claims to Retract Before Release

**Non-goals** (so they don't get re-argued):
- **No neural network anywhere in the loop.** Not in detection, matching, or scoring. Add an
  embedding model and XDPD becomes a worse semantic cache instead of a different thing.
- **No probabilistic / soft matching in the core.** Exact matching *is* the zero-false-hit
  guarantee (I.3). Soft matching, if ever, lives in a layer above — never in the core.
- **No dependencies in the core crate.** Persistence stays hand-rolled over `std::io`.
- **No domain semantics in the engine.** `Token = u32`; adapters live in callers.
- **Not a replacement for exact/prefix/KV caching.** XDPD sits beside them, lossless.

**Retracted before the 0.3.0 release** — all five are done, listed so the record survives:
1. `RESEARCH.md` "no system satisfies all of these" → retracted; DreamCoder, Sequitur and Drain
   are cited and the claim narrowed to I.1.
2. `README.md` LLM cost-projection table → **deleted**, not defended. It rested on a contested
   hit-rate assumption that production data contradicts.
3. "16×" / "93.8%" → labelled synthetic and program-level everywhere they appear, and the real
   measured figures now lead the README instead.
4. `RESEARCH.md` "Known Limitations" → rewritten for 0.3.0; the two blockers it omitted are
   fixed, and the list now names what actually remains.
5. `xdpd/ARCHITECTURE.md` stale duplicate → **deleted**, replaced by a pointer to this file.

Two more found and fixed while preparing the release:
6. The demo attributed hand-written arithmetic progressions to Yahoo Finance and to production
   nginx. Both attributions were false; the sequences are now labelled synthetic, which is what
   they always were.
7. The Drain3 baseline was quoted from the literature rather than run. `examples/logbench/drain3_baseline.py`
   now runs it. It reproduces the quoted numbers — the figures were right, they just were not
   checkable.

---

## Appendix — VM Reference (unchanged in v0.3)

| Instruction | Effect | Program-level cost |
|---|---|---|
| `Load(t)` | `reg = t` | 1 |
| `Output` | push `reg` | 1 |
| `Seq(start, delta, len)` | emit arithmetic run | 1 |
| `Call(name, params)` | synthesize body from shape+params, run, return | 1 regardless of body |
| `Ret` | return | 1 |

`Call` saves `pc`, swaps `program` for the synthesized body, runs to `Ret`, restores both — a
skill's body never exists at rest, only during its own invocation. Slotted templates (IV.1)
need no new opcode: `params` already carries captured variables.

---

## Sources

Proven mechanism: [DreamCoder](https://arxiv.org/pdf/2006.08381) ·
[DreamCoder (Roy Soc)](https://royalsocietypublishing.org/rsta/article/381/2251/20220050/112456/DreamCoder-growing-generalizable-interpretable) ·
[Sequitur](https://ml.cms.waikato.ac.nz/publications/1997/NM-IHW-Compress97.pdf) ·
[Sequitur (JAIR)](https://dl.acm.org/doi/abs/10.5555/1622776.1622780) ·
[GrammarViz 2.0](https://link.springer.com/content/pdf/10.1007/978-3-662-44845-8_37.pdf) ·
[Ensemble Grammar Induction](https://openproceedings.org/2020/conf/edbt/paper_45.pdf)

Compression anomaly detection: [Entropy 2021 / NCD](https://pmc.ncbi.nlm.nih.gov/articles/PMC8156803/) ·
[Anomaly Detection on Compressed Data](https://arxiv.org/abs/2110.02579) ·
[Neural NCD](https://arxiv.org/html/2410.15280)

Debunked: [Bad numbers in "gzip beats BERT"](https://kenschutte.com/gzip-knn-paper/) ·
[Part 2](https://kenschutte.com/gzip-knn-paper2/) ·
[Gzip vs bag-of-words](https://arxiv.org/pdf/2307.15002)

Template mining: [Drain](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf) ·
[Drain3](https://github.com/logpai/Drain3) ·
[Log punctuation signature (Drain limits)](https://www.mdpi.com/2076-3417/11/24/11974) ·
[HELP](https://arxiv.org/html/2408.08300v1)

Caching reality: [95% hit-rate myth](https://dev.to/gauravdagde/llm-semantic-caching-the-95-hit-rate-myth-and-what-production-data-actually-shows-8ga) ·
[Text-based cache keys are wrong](https://www.truefoundry.com/blog/semantic-caching-llm-gateway) ·
[Beyond prefix caching](https://www.truefoundry.com/blog/semantic-caching-ai-gateway) ·
[MeanCache](https://arxiv.org/pdf/2403.02694) ·
[Verified semantic caching](https://arxiv.org/html/2602.13165v1) ·
[Spheron semantic cache setup](https://www.spheron.network/blog/semantic-cache-llm-inference-gpu-cloud/)

Prompt compression: [LLMLingua](https://www.microsoft.com/en-us/research/blog/llmlingua-innovating-llm-efficiency-with-prompt-compression/) ·
[Empirical study](https://arxiv.org/pdf/2505.00019) ·
[In the wild](https://arxiv.org/pdf/2604.02985)

Agent economics: [Spheron 5–30×](https://www.spheron.network/blog/agentic-ai-inference-cost-2026/) ·
[AgentMarketCap](https://agentmarketcap.ai/blog/2026/04/12/ai-agent-token-consumption-gap-enterprise-agentic-workloads) ·
[EY](https://www.ey.com/en_us/insights/ai/agentic-ai-token-costs) ·
[Modern Data Co](https://www.themoderndatacompany.com/blog/why-cheaper-ai-tokens-are-increasing-enterprise-ai-costs)

Telemetry economics: [Grepr hidden costs](https://www.grepr.ai/blog/the-hidden-cost-in-observability) ·
[OTel log dedup processor](https://opentelemetry.io/blog/2026/log-deduplication-processor/) ·
[OTel cost reduction](https://openobserve.ai/blog/opentelemetry-cost-reduction/)

Agentic structure & skills: [XGrammar-2](https://arxiv.org/pdf/2601.04426) ·
[TSCG](https://arxiv.org/pdf/2605.04107) ·
[SkillOps](https://arxiv.org/pdf/2605.13716) ·
[Agent Skill Evaluation & Evolution](https://arxiv.org/html/2606.11435v1) ·
[Learned Capability Governance](https://arxiv.org/pdf/2604.11839)

Edge: [TinyML IIoT review](https://www.mdpi.com/1424-8220/26/8/2550) ·
[Multimodal TinyML PdM](https://www.mdpi.com/1424-8220/26/14/4536)
