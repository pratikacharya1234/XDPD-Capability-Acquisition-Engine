# XDPD log template benchmark

Measures XDPD's template mining against real data with published ground truth,
and against Drain3 — the established production baseline.

**XDPD currently loses this benchmark, badly. The numbers are below anyway.**
That is the point of running it: the project's other figures are synthetic, and
a synthetic win is worth less than a measured loss you can act on.

## Run it

```sh
./fetch-data.sh                                        # loghub datasets
cargo run --release -- data/HDFS_2k.log_structured.csv
cargo run --release -- data/Apache_2k.log_structured.csv
```

Datasets are from [loghub](https://github.com/logpai/loghub), the standard
log-parsing benchmark corpus. Their `*_structured.csv` files carry a
hand-labelled `EventId` per line, which is the ground truth. Data is fetched on
demand rather than vendored.

## Metric

**Grouping Accuracy (GA)**, as used throughout the log-parsing literature: a
message counts as correct only when its predicted cluster contains *exactly* the
same set of messages as its ground-truth cluster. Partial overlap scores zero,
which is what makes GA demanding — and why XDPD's partial coverage scores so
poorly.

Both systems were measured with the identical GA implementation on the identical
`Content` field.

## Results — measured 2026-07-30, xdpd 0.2.1 + phases 1-6

| dataset | lines | true types | XDPD GA | **Drain3 GA** | XDPD templates | XDPD lines claimed | XDPD compression |
|---|---|---|---|---|---|---|---|
| HDFS_2k | 2000 | 14 | **32.4%** | **99.8%** | 6 | 753 / 2000 | 1.45x |
| Apache_2k | 2000 | 6 | **1.6%** | **100.0%** | 5 | 272 / 2000 | 1.18x |

Learn time: 7.4ms (HDFS) / 3.6ms (Apache) for 2000 lines. Drain3 parse time was
4.4ms / 4.1ms, so throughput is comparable; accuracy is not.

**Lossless reproduction: OK on both.** Every line claimed by a template was
reproduced byte-for-byte from that template plus its captured parameters. This is
the one invariant the design rests on, and the benchmark exits non-zero if it
ever fails. It held on 100% of claimed lines.

### Context for the compression figures

1.45x and 1.18x are **program-level ops on real data** — far below the 16x the
README quotes from synthetic sequences. Both are honest; they measure different
things. These are the ones to cite for real workloads.

## Why XDPD loses, with evidence

Three hypotheses were tested and **ruled out**:

| Hypothesis | Measurement | Verdict |
|---|---|---|
| The 16-record alignment window is too short | 91.6% (HDFS) / 98.3% (Apache) of same-type recurrences are ≤16 lines apart | not the cause |
| Same-length alignment is too restrictive | 13/14 and 6/6 event types have a single token count | not the cause |
| The 50%-fixed threshold rejects real templates | 0 of 12 and 0 of 6 aligned pairs fall below it | not the cause |

One real bug was found and fixed: `align_template` rejected all-`Fixed` results
as "memorized literals", which discarded event types whose messages are wholly
constant — two of six on Apache. Fixing it took Apache from 2 to 5 templates and
107 to 272 claimed lines. GA did not move, because GA needs exact cluster
equality.

**The remaining cause is convergence.** A template induced from *one pair* of
records has `Fixed` slots wherever that pair happened to agree — including
positions that vary across the event type as a whole. The template is therefore
over-specific and claims only the subset of lines that share the accident. Pairwise
alignment produces 14 distinct templates for HDFS's 14 types and 8 for Apache's
6, so the templates exist; they just never generalize to cover their whole type.

### The fix

Refine templates instead of minting them. When a new record nearly matches an
existing template, generalize that template in place — turn `Fixed` into `Var` at
the positions where they now disagree — rather than adding a second, differently
over-specific template. That is what Drain achieves with its cluster tree, and it
is the change most likely to move GA. It is a redesign of the learning path in
`Learner::observe`, so it is queued for review, not slipped in here.

Until that lands, the honest summary is: **XDPD's throughput and losslessness
hold up on real data; its template accuracy does not compete with Drain3.**
