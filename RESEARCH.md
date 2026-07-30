# XDPD: Research Brief

This document has been through **two research passes**. They disagree, and the disagreement
is the most useful thing in here.

| Pass | Date | Method | Status |
|---|---|---|---|
| **1 — Origin** | July 2026 | Web research via Kimi (kimi.moonshot.cn). Raw log not preserved in this repo. | Motivating survey. **Never independently verified.** Its central gap claim is now known to be wrong (§4). |
| **2 — Verification** | 2026-07-29 | Live literature + market review: 12 targeted searches, 3 papers fetched. Every source linked. | Current basis of record. Supersedes Pass 1 on every point of conflict. |

**How to read the tags.** Nothing in this document is asserted from recollection:

- `[PROVEN]` — published, with a citation, and reproduced or benchmarked by others
- `[CONTESTED]` — published but disputed, or vendor arithmetic rather than measurement
- `[DEBUNKED]` — actively refuted; must never be cited by this project
- `[OPEN]` — a named unsolved problem in the literature
- `[UNVERIFIED-RELAY]` — Pass 1 finding, not re-checked in Pass 2. Treat as a lead, not a fact.
- `[OURS-VERIFIED]` — verified by reading `xdpd/src/lib.rs` or running its tests

---

## 1. Research Motivation

Build a learning mechanism fundamentally different from current AI:

- No neural networks — no weights, no gradient descent, no GPU
- No storage of raw experiences — no database, no vector store, no RAG
- Structural self-modification — the system changes ITSELF after experience
- CPU-only, lightweight
- General-purpose — not domain-specific

Original question: *does anything like this already exist?* Pass 1 answered "no."
Pass 2 answered "**yes, in pieces, and two of those pieces are directly load-bearing prior
art we should adopt rather than reinvent.**" That correction is §4.

---

## 2. Pass 1 Findings (Origin Survey)

Preserved as the project's intellectual lineage. All `[UNVERIFIED-RELAY]` unless Pass 2
re-checked them — flagged inline where it did.

| # | Finding | What XDPD took from it |
|---|---|---|
| A | **Compression as intelligence** — Hutter's AIXI; PAQ8 reportedly beat RNNs on text prediction by up to 6%; "compressors are predictors, RNNs are imitators" | Compression as the learning criterion. Online adaptation, aggressive forgetting via counter halving, CPU-only. → **Pass 2 confirmed the principle via DreamCoder/MDL `[PROVEN]`, but see the gzip `[DEBUNKED]` warning in §3.2** |
| B | **Self-modifying code** — Schmidhuber's Gödel Machine (2007, theoretical); Darwin Gödel Machine (2025) improved SWE-bench 20%→50% by editing its own Python | The self-modification loop. Critical caveat DGM itself has: **the intelligence driving the changes is an external LLM (Claude 3.5 Sonnet).** XDPD has no external brain. |
| C | **Structural memory without storage** — Structurally Dynamic Cellular Automata (2025): memory regenerated from graph topology, no long-term storage of inputs/outputs; uses de-inforcement | The thesis that the subroutine table can *be* the memory rather than index it |
| D | **Biological learning without brains** — *Physarum polycephalum* habituates, solves mazes, anticipates periodic events, transfers memory by contact; memory is the tube network itself | Memory as structure, not a separate subsystem |
| E | **Cognitive architectures** — SOAR chunking compiles a whole reasoning trace into one production rule; ACT-R production compilation merges two rules into one, removing the retrieval between them | The core mechanism: compile experience into a rule, change the rule set. This is XDPD's `Call`. |
| F | **Emergent Models (2025)** — replace NNs with iterative fixed rules over large state spaces; theoretical | Direction only |
| G | **Sparse Distributed Memory** (Kanerva) — high-dimensional binary vectors, Hamming distance, not a NN | Considered and not adopted: approximate matching would forfeit the zero-false-hit guarantee (§5.3) |
| H | **Structural plasticity** — SMGrNN (2025) grows/prunes topology, but weights still trained by backprop | Partial match only; XDPD has no weights at all |
| I | **Active inference / FEP** (Friston) — unifies perception/action/learning as prediction-error minimization; pymdp explodes combinatorially, deep AIF reintroduces NNs. "Theory without engineering." | Confirms an engineering vacuum exists in non-neural learning |
| J | **Hyperdimensional computing** — no backprop, but caps out (MNIST yes, CIFAR-10 no) and needs manual per-domain encoding | Confirms the encoding-design trap XDPD avoids by making `Token = u32` the caller's problem |

---

## 3. Pass 2 — Verification (2026-07-29)

### 3.1 What is proven, with evidence

| Claim | Status | Evidence |
|---|---|---|
| Growing a library by MDL compression is a **working** learning mechanism | `[PROVEN]` | [DreamCoder](https://arxiv.org/pdf/2006.08381) — wake/sleep library learning with an automatic MDL-driven refactoring algorithm that extracts shared sub-expressions into new library entries; 8 domains (list processing, drawing, physics, recursive programming); solves the most held-out tasks, mean 54.1s / median 15.0s |
| Hierarchical structure can be induced from a discrete token stream in **linear time, with a proof** | `[PROVEN]` | [Sequitur](https://ml.cms.waikato.ac.nz/publications/1997/NM-IHW-Compress97.pdf) (Nevill-Manning & Witten, 1997) — two constraints, *digram uniqueness* and *rule utility*; `S→abcab` becomes `S→AcA, A→ab` |
| Grammar-induced structure finds **variable-length** motifs *and* anomalies with no prior knowledge of length, shape, or minimum frequency | `[PROVEN]` | [GrammarViz 2.0](https://link.springer.com/content/pdf/10.1007/978-3-662-44845-8_37.pdf), [Ensemble Grammar Induction](https://openproceedings.org/2020/conf/edbt/paper_45.pdf) — SAX discretization + Sequitur, applied to time-series anomaly discovery |
| Compression distance is a real, **parameter-free, feature-free** anomaly detector | `[PROVEN]` | [Entropy 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC8156803/) — NCD features across 4 cybersecurity domains (HTTP anomalies, spam, DGA tracking, sentiment) with no domain-specific tuning |
| Online template mining over streams is production-grade and commercially deployed | `[PROVEN]` | [Drain](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf) / [Drain3](https://github.com/logpai/Drain3) — best of 13 miners on the LogPAI benchmark; in IBM production AIOps |
| Structural repetition in **agentic** traffic is real and worth exploiting | `[PROVEN]` | [XGrammar-2](https://arxiv.org/pdf/2601.04426) — repetition-state compression + cross-grammar substructure cache, >6× tool-calling compilation speedup, near-zero added latency |
| Deterministic, non-neural structural compilation beats verbose baselines | `[PROVEN]` | [TSCG](https://arxiv.org/pdf/2605.04107) — tool-schema compilation with no embeddings and no learned representations |
| Semantic-cache false hits are **asymmetrically catastrophic** | `[PROVEN]` | [TrueFoundry](https://www.truefoundry.com/blog/semantic-caching-llm-gateway): queries differing only in time period or polarity exceed 0.95 cosine similarity → *silent factual errors*; *"a cache that returns false hits is strictly worse than no cache"*; [MeanCache](https://arxiv.org/pdf/2403.02694) reports 3 false hits vs GPTCache's 54 |
| Agentic workloads consume far more tokens than chat (range, not a point) | `[PROVEN]` | 5–30× ([Spheron](https://www.spheron.network/blog/agentic-ai-inference-cost-2026/)); 10–100× ([AgentMarketCap](https://agentmarketcap.ai/blog/2026/04/12/ai-agent-token-consumption-gap-enterprise-agentic-workloads)); 73% of enterprises overran 2026 AI cost projections; ~$1.20 per orchestrated interaction vs $0.04 in 2023 |
| Telemetry volume is a large, growing, **already-budgeted** line item | `[PROVEN]` | Gartner via [Grepr](https://www.grepr.ai/blog/the-hidden-cost-in-observability): observability spend +~20% YoY, many orgs >$800k/yr; **AI workloads emit 10–50× more telemetry** than traditional services |
| Demand for structural stream dedup is validated by shipped product | `[PROVEN]` | [OpenTelemetry log deduplication processor](https://opentelemetry.io/blog/2026/log-deduplication-processor/) — collapses entries into one, preserving count and first/last timestamps. **Note: identical entries only.** |
| **Mechanically-derived** skills beat LLM-authored ones | `[PROVEN]` | SkillsBench via [SkillOps](https://arxiv.org/pdf/2605.13716): human-authored skills **+16.2 pts** pass rate; **LLM-authored skills show no measurable gain** |

### 3.2 What is contested or debunked — never build on these

| Claim | Status | Why |
|---|---|---|
| **"gzip+kNN beats BERT"** | `[DEBUNKED]` | Reproductions found the accuracy was computed with an **oracle tie-break** — effectively top-2 accuracy, not kNN(k=2). Authors confirmed it was intentional ("maximum possible accuracy for a stochastic classifier"). With fair tie-breaking, BERT wins; plain bag-of-words distance beats gzip by +1.4 pts. [Schutte](https://kenschutte.com/gzip-knn-paper/), [part 2](https://kenschutte.com/gzip-knn-paper2/), [Opitz](https://arxiv.org/pdf/2307.15002). **This project must never cite it** — it is the single easiest way to lose credibility in this exact niche. |
| "60% semantic-cache hit rate → ~$846/mo/GPU" | `[CONTESTED]` | Vendor arithmetic, not measurement. Production data: **60–70% of real queries are genuinely unique**, and *"hit rates of 61–69% from research papers don't hold up in production"* ([DEV](https://dev.to/gauravdagde/llm-semantic-caching-the-95-hit-rate-myth-and-what-production-data-actually-shows-8ga)). The 40–65% figures hold only for FAQ bots and repeated tool calls; creative generation and multi-turn chat are near zero. |
| Prompt compression is close to free | `[CONTESTED]` | LLMLingua reaches up to 20×, but: sharp degradation at 25–30× on GSM8K; **all methods increased hallucination** via information loss; on optimized serving frameworks there was **no substantial end-to-end latency gain** except on prompts >10k tokens; only LLMLingua-2 was practical. [Empirical study](https://arxiv.org/pdf/2505.00019), [in the wild](https://arxiv.org/pdf/2604.02985) |
| PAQ8 beat RNNs by 6% (Pass 1, Finding A) | `[UNVERIFIED-RELAY]` | Not re-checked in Pass 2. The underlying *principle* is independently `[PROVEN]` by DreamCoder/MDL, so the project does not need this number. Do not quote it. |

### 3.3 What is genuinely open — where XDPD can contribute

| Open problem | Evidence |
|---|---|
| **Skill-library health** — eviction policy, redundancy detection, staleness at ecosystem scale | [SkillOps](https://arxiv.org/pdf/2605.13716): *"no established framework for ongoing library health management, skill eviction policies, or automated redundancy detection at ecosystem scale"* |
| **Agent behavioral baselines** — defining "normal" for an agent, stably, across deployments | [Learned Capability Governance](https://arxiv.org/pdf/2604.11839): baselines *"remain contextual and potentially unstable."* Current approaches use **learned/neural** models → foolable by adversarial input |
| **Drain's positional assumptions** — assumes leading tokens are constant, and that same-event messages share length; both are documented accuracy failures | [Drain](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf), [analysis](https://www.mdpi.com/2076-3417/11/24/11974) |
| **Lossless** structural reuse inside an inference path | Every surveyed cache tier is exact-string or embedding-similarity. Nothing occupies the middle. |
| Voyager-style skill libraries: flat key-value stores with textual retrieval, no continuous maintenance | [Agent Skill Evaluation & Evolution](https://arxiv.org/html/2606.11435v1), [SkillOps](https://arxiv.org/pdf/2605.13716) |

---

## 4. Corrections to Pass 1

Stated plainly, because a research brief that hides its own errors is worthless.

### 4.1 RETRACTED: "No system satisfies all of these" / "XDPD is the first implementation"

**This was false.** Pass 1 concluded, after 30+ searches, that nothing combined
learning-without-storage, structural self-modification, no-NN, CPU-only, and general-purpose.
Pass 2 found three systems that occupy most of that space:

- **DreamCoder** does library-learning-by-MDL-compression — the same core mechanism XDPD
  implements. It is `[PROVEN]` across 8 domains. It needs a neural recognition model and
  offline wake/sleep cycles; XDPD does not. That is a difference in *engineering*, not in kind.
- **Sequitur** does linear-time hierarchical grammar induction from a token stream — which is
  exactly the segmentation and hierarchy XDPD's roadmap listed as future work. It has been
  solved since 1997, with a proof.
- **Drain** does online template mining in production, benchmarked as best of 13.

**Why the error happened, and why it matters:** Pass 1 searched for the *philosophy*
("learning without storage", "non-neural AI", "self-modifying") and found a vacuum. It never
searched for the *mechanism* under the names practitioners actually use — grammar induction,
library learning, template mining. Those fields are mature. The lesson generalizes: search
for what your thing **does**, not for what it **means**.

### 4.2 REVISED: the gap claim

Pass 1's claim is replaced with this narrower, defensible one:

> DreamCoder needs a neural recognition model and offline wake/sleep cycles. Sequitur induces
> grammar but has no execution model, no generalization across values, and no persistence.
> Drain mines templates but assumes constants sit at the front of a message and that
> same-event messages share length. **XDPD is the online, zero-dependency, deterministic
> intersection: a VM whose instruction set grows from streamed observation, with no neural
> component anywhere in the loop, and no assumption about where in the stream the constants
> sit.**

That is an engineering gap. Engineering gaps are the shippable kind.

### 4.3 DEMOTED: the LLM prompt-cache use case

Pass 1 made it the flagship. The production evidence (§3.2) contradicts the hit-rate
assumption it rests on. It stays as a demo of deployment shape; the flagship moves to
machine-generated telemetry, where the repetition is structural by nature and the budget
already exists. Full reasoning and re-ranked targets: `docs/ARCHITECTURE.md` §I.2, §IV.6.

### 4.4 REMOVE: the LLM cost-projection table in README

Already labeled hypothetical, but built on the contested hit-rate assumption. Delete it
rather than defend it.

---

## 5. The Hypothesis (as it now stands)

### 5.1 Core idea (unchanged, and now with proven lineage)

Start with a small primitive set. Observe token streams. On detecting an invariant, compile it
into a subroutine that becomes a permanent first-class operation. Discard the raw
observations. The subroutine table is the only memory.

What changes inside the system: which subroutines exist, the structures they match, and their
strength scores. Not weights, not stored text, not vectors.

### 5.2 Sources combined

| Source | What XDPD takes | Status |
|---|---|---|
| Sequitur | Digram uniqueness → segmentation; **rule utility → the eviction policy** | `[PROVEN]`, to adopt in 0.3 |
| DreamCoder | MDL-driven extraction of shared substructure into library entries | `[PROVEN]` |
| GrammarViz | Grammar structure → anomaly discovery without prior knowledge | `[PROVEN]` |
| NCD / compression distance | Compression ratio as a parameter-free anomaly signal | `[PROVEN]` |
| SOAR chunking | Compile the trace into one rule; change the rule set | `[UNVERIFIED-RELAY]`, well-established |
| Miller's chunking (1956) | Primitives → chunks → meta-chunks | `[UNVERIFIED-RELAY]` |
| PAQ | Online adaptation, aggressive forgetting, CPU-only | `[UNVERIFIED-RELAY]` |
| Slime mold | Memory as structure, no separate store | `[UNVERIFIED-RELAY]` |
| Structurally Dynamic CA | Topology encodes the past | `[UNVERIFIED-RELAY]` |
| MDL (Rissanen) | Compression as the learning criterion | `[PROVEN]` via DreamCoder |

### 5.3 RETRACTED by measurement: "losslessness is the moat"

Pass 2 concluded that losslessness was the differentiator, reasoning from `[PROVEN]` findings
that a false-hit cache is strictly worse than no cache and that threshold tuning has no
universally correct value.

**Pass 3 measured it and the conclusion does not hold.** Those findings concern **semantic
caches** — embedding similarity under a tuned threshold. **Template miners are already exact and
positional**, so the argument never applied to them. Drain3, tested for exact positional
reconstruction on loghub HDFS_2k and Apache_2k (`examples/logbench`):

| | XDPD | Drain3 |
|---|---|---|
| Lossless reconstruction | 100% of the 38% / 14% of lines it claims | **100% of 100% of lines** |
| Grouping accuracy | 32.4% / 1.6% | **99.8% / 100%** |

Losslessness is still a real property of XDPD. It is simply not scarce, and it must not be sold
as scarce.

**What this does not change:** approximate matching remains a permanent non-goal of the core,
and §2 Finding G (Sparse Distributed Memory, Hamming-distance matching) remains rejected.
Exactness is still the right design — it is just table stakes in this space rather than an
advantage.

**What remains genuinely distinct, and is unmeasured** (so: unclaimable until measured):
boundary-free stream coverage via DP composition, and algebraic primitives — `Seq` encodes an
arithmetic progression in one instruction, where grammar induction and template mining capture
repetition rather than progression and spend a rule per step on a ramp. The honest baseline for
both is GrammarViz / SAX + Sequitur (§3.1), not Drain3.

---

## 6. What Our Own Code Proves

`[OURS-VERIFIED]` — each pinned by a test in `xdpd/src/lib.rs`, not asserted in prose.

| # | Question | Answer | Test |
|---|---|---|---|
| 1 | Learns arithmetic patterns? | Yes — `Seq` | `detect_arithmetic_pattern` |
| 2 | Learns constant patterns? | Yes — `Seq(start, 0, len)` | `detect_constant_pattern` |
| 3 | Learns repeat patterns? | Yes — Load/Output body | `detect_repeat_pattern` |
| 4 | Composes learned skills? | Yes — DP over the skill table | `compose_uses_skills_when_available` |
| 5 | Detects anomalies? | Yes — known compresses more than unknown | `anomaly_detection_low_ratio_for_unknown` |
| 6 | Subroutine table holds all state? | Yes — no raw data retained after observation | by construction |
| 7 | CPU-only, zero dependencies? | Yes — pure Rust stdlib | `Cargo.toml` has no `[dependencies]` |
| 8 | Generalizes across values, not memorizing one instance? | Yes — skills store a value-free `PatternShape`; one compiled from `[0,2,4,6,8]` also matches unseen `[100,102,…]` | `skill_generalizes_across_values`, `constant_skill_dedupes_across_values` |
| 9 | Survives a process restart, still generalizing? | Yes — plain-text table, zero extra deps | `skills_round_trip_through_file_survives_restart` |

**Measured generalization** (`examples/src/main.rs` PHASE 6): 8/8 hit rate on unseen
same-shape sequences, 0/3 false positives on non-matching shapes.

**Scope of the headline numbers.** The 16× / 93.8% figures are real but **synthetic and
program-level** (1 `Call` = 1 instruction). They are not a measurement on captured
real-world data, and every published use of them must say so.

---

## 7. Known Limitations

Ordered by severity. Items 1 and 2 were missing from the previous version of this document
and are the two that actually block real-world use.

1. **No variable slots.** `Constant`/`Arithmetic`/`Repeat` cannot express "same message,
   different ID". **XDPD cannot match a real log line or a real tool call today.** Fix
   specified in `docs/ARCHITECTURE.md` §IV.1.
2. **Whole-sequence-or-nothing detection.** `detect_pattern` returns a pattern only if the
   entire slice is one invariant, so callers must pre-segment the stream — meaning the
   measured hit rate is partly a property of the caller's slicing. Note `compose()` already
   segments its target; only *learning* doesn't. Sequitur solves this (§IV.2).
3. **No skill decay or GC.** The table grows monotonically, and `learned_signatures` is a
   second unbounded structure. `strength` and `uses` are **dead fields** — initialized and
   serialized, but never updated. Sequitur's rule utility is the fix.
4. **Repeat patterns compress only at program level.** Execution-level cost equals naive
   emission.
5. **`Call` silently no-ops on an unknown skill name** — emits nothing, reports success.
   Unreachable today; a correctness hazard as soon as tables are synced or loaded mismatched.
6. **No `Call` recursion depth bound.** Flat shapes make depth 1 today; hierarchy makes this a
   stack-overflow surface.
7. **`compose()` is O(n·|S|)** — a full skill-table scan per position.
8. **`check_anomaly` returns `inf`** on an empty sequence (divides by a zero cost).
9. **Single persistence format version, no migration path.**
10. **The gateway example does not put XDPD on the hit path** — its cache is an exact
    `hash_bytes(prompt)` map; the learner only observes. It demonstrates deployment shape,
    not mechanism.

---

## 8. Honest Assessment

Pass 1 concluded that XDPD "occupies genuinely open territory" because nothing like it
existed. That conclusion was reached by searching for a philosophy instead of a mechanism, and
it is wrong. The mechanism has been proven repeatedly since 1997.

**This is better news than the original claim.** An unprecedented mechanism has to prove it
works at all. A proven mechanism with an unoccupied engineering point only has to be built
well. XDPD's actual position:

- The learning principle is `[PROVEN]` — DreamCoder, Sequitur, GrammarViz, Drain, NCD.
- The specific combination — online, deterministic, zero-dependency, no neural component,
  no positional assumptions, lossless by construction, with anomaly detection falling out of
  the compressor for free — is not occupied by any of them.
- The core runs, generalizes across values, and survives restart, `[OURS-VERIFIED]`.
- It cannot yet match a real log line (§7.1). That is the next commit, not a research question.
- Two named `[OPEN]` problems in the 2026 literature — skill-library health and stable agent
  behavioral baselines — sit exactly where XDPD's roadmap already pointed.

The honest summary: **the mechanism is proven, the engineering point is open, the
implementation is early, and the headline numbers are synthetic until item 2 and item 8 of
`docs/ARCHITECTURE.md` §V land on captured data.** If Drain3 beats XDPD on template accuracy
when that benchmark runs, that number gets published too.

---

## Sources

Proven mechanism: [DreamCoder](https://arxiv.org/pdf/2006.08381) ·
[DreamCoder (Royal Society)](https://royalsocietypublishing.org/rsta/article/381/2251/20220050/112456/DreamCoder-growing-generalizable-interpretable) ·
[Sequitur](https://ml.cms.waikato.ac.nz/publications/1997/NM-IHW-Compress97.pdf) ·
[Sequitur (JAIR)](https://dl.acm.org/doi/abs/10.5555/1622776.1622780) ·
[GrammarViz 2.0](https://link.springer.com/content/pdf/10.1007/978-3-662-44845-8_37.pdf) ·
[Ensemble Grammar Induction](https://openproceedings.org/2020/conf/edbt/paper_45.pdf)

Compression-based anomaly detection: [Entropy 2021 / NCD](https://pmc.ncbi.nlm.nih.gov/articles/PMC8156803/) ·
[Anomaly Detection on Compressed Data](https://arxiv.org/abs/2110.02579) ·
[Neural NCD](https://arxiv.org/html/2410.15280)

Debunked — do not cite: [Bad numbers in "gzip beats BERT"](https://kenschutte.com/gzip-knn-paper/) ·
[Part 2](https://kenschutte.com/gzip-knn-paper2/) ·
[Gzip vs bag-of-words](https://arxiv.org/pdf/2307.15002)

Template mining: [Drain](https://jiemingzhu.github.io/pub/pjhe_icws2017.pdf) ·
[Drain3](https://github.com/logpai/Drain3) ·
[Drain limitations](https://www.mdpi.com/2076-3417/11/24/11974) ·
[HELP](https://arxiv.org/html/2408.08300v1)

Caching reality: [The 95% hit-rate myth](https://dev.to/gauravdagde/llm-semantic-caching-the-95-hit-rate-myth-and-what-production-data-actually-shows-8ga) ·
[Text-based cache keys are the wrong default](https://www.truefoundry.com/blog/semantic-caching-llm-gateway) ·
[Beyond prefix caching](https://www.truefoundry.com/blog/semantic-caching-ai-gateway) ·
[MeanCache](https://arxiv.org/pdf/2403.02694) ·
[Verified semantic caching](https://arxiv.org/html/2602.13165v1) ·
[Spheron cache setup](https://www.spheron.network/blog/semantic-cache-llm-inference-gpu-cloud/)

Prompt compression: [LLMLingua (MSR)](https://www.microsoft.com/en-us/research/blog/llmlingua-innovating-llm-efficiency-with-prompt-compression/) ·
[Empirical study](https://arxiv.org/pdf/2505.00019) ·
[In the wild](https://arxiv.org/pdf/2604.02985) ·
[LongLLMLingua](https://arxiv.org/pdf/2310.06839)

Agent economics: [Spheron 5–30×](https://www.spheron.network/blog/agentic-ai-inference-cost-2026/) ·
[AgentMarketCap](https://agentmarketcap.ai/blog/2026/04/12/ai-agent-token-consumption-gap-enterprise-agentic-workloads) ·
[EY](https://www.ey.com/en_us/insights/ai/agentic-ai-token-costs) ·
[Modern Data Company](https://www.themoderndatacompany.com/blog/why-cheaper-ai-tokens-are-increasing-enterprise-ai-costs)

Telemetry economics: [Grepr — hidden costs](https://www.grepr.ai/blog/the-hidden-cost-in-observability) ·
[OTel log dedup processor](https://opentelemetry.io/blog/2026/log-deduplication-processor/) ·
[OTel cost reduction](https://openobserve.ai/blog/opentelemetry-cost-reduction/)

Agentic structure & skill libraries: [XGrammar-2](https://arxiv.org/pdf/2601.04426) ·
[TSCG](https://arxiv.org/pdf/2605.04107) ·
[SkillOps](https://arxiv.org/pdf/2605.13716) ·
[Agent Skill Evaluation & Evolution](https://arxiv.org/html/2606.11435v1) ·
[Learned Capability Governance](https://arxiv.org/pdf/2604.11839)

Edge / embedded: [TinyML IIoT systematic review](https://www.mdpi.com/1424-8220/26/8/2550) ·
[Multimodal TinyML predictive maintenance](https://www.mdpi.com/1424-8220/26/14/4536)

---

*Pass 1 (July 2026) was a motivating survey conducted via Kimi; its raw chat log is not
preserved in this repo. Pass 2 (2026-07-29) is a live literature and market review with every
source linked above. Where they conflict, Pass 2 governs. Code claims marked `[OURS-VERIFIED]` were
checked against `xdpd/src/lib.rs` directly.*
