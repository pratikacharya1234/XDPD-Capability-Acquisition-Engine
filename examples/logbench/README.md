# XDPD log template benchmark

Measures XDPD's template mining against real data with published ground truth,
and against Drain3 — the established production baseline.

**XDPD now ties Drain3 on the two datasets it was developed against, and loses
on all four held-out ones.** Both halves of that sentence are the result.

## Run it

```sh
./fetch-data.sh                                        # loghub datasets
cargo run --release -- data/HDFS_2k.log_structured.csv

# the baseline, actually run rather than quoted:
python3 -m venv venv && ./venv/bin/pip install drain3
./venv/bin/python drain3_baseline.py data/HDFS_2k.log_structured.csv
```

Datasets are from [loghub](https://github.com/logpai/loghub), the standard
log-parsing benchmark corpus. Their `*_structured.csv` files carry a
hand-labelled `EventId` per line, which is the ground truth. Data is fetched on
demand rather than vendored.

## Metric

**Grouping Accuracy (GA)**, as used throughout the log-parsing literature: a
message counts as correct only when its predicted cluster contains *exactly* the
same set of messages as its ground-truth cluster. Partial overlap scores zero,
which is what makes GA demanding.

Both systems are measured with the same GA definition on the same `Content`
field — `grouping_accuracy` in `src/main.rs` and in `drain3_baseline.py` are
line-for-line equivalent.

## Results — measured 2026-07-30, xdpd 0.2.1 + phases 1-7

| dataset | true types | XDPD GA | **Drain3 GA** | held out? |
|---|---|---|---|---|
| HDFS_2k | 14 | 99.7% | **99.8%** | no |
| Apache_2k | 6 | **100.0%** | **100.0%** | no |
| BGL_2k | 120 | 91.2% | **96.9%** | yes |
| Zookeeper_2k | 50 | 93.3% | **96.7%** | yes |
| OpenSSH_2k | 27 | 52.1% | **71.8%** | yes |
| Linux_2k | 118 | 17.9% | **68.4%** | yes |
| **mean** | | **75.7%** | **88.9%** | |

**Read the "held out" column before the accuracy column.** HDFS and Apache are
the datasets the template-generalization work in `Learner::observe` was
developed against, and parity there is not evidence of anything. The four
datasets that were never looked at during development are, and XDPD loses all
four — narrowly on BGL and Zookeeper, heavily on Linux.

Previous measurement, before template generalization landed: **32.4% (HDFS) /
1.6% (Apache)**. The mechanism below is what moved those.

**Lossless reproduction: OK on every dataset.** Every line claimed by a template
was reproduced byte-for-byte from that template plus its captured parameters.
This is the one invariant the design rests on, and the benchmark exits non-zero
if it ever fails.

Compression on HDFS went from 1.45x to **11.90x** program-level ops, and learn
time is ~6ms per 2000 lines. Drain3 parses the same file in ~4ms, so throughput
remains comparable.

## What changed, and why it worked

Pairwise alignment marks a position `Fixed` whenever the two records being
aligned happen to agree there — including positions that vary across the event
type as a whole. Two problems followed from that, and both are now fixed:

1. **Templates never generalized.** Each pair froze a *different* set of
   coincidences, so no two aligned templates shared a signature, so frequency
   counting never saw a repeat and never compiled anything. `generalize_template`
   widens an existing skeleton on contact instead — every position that ever
   varies decays to `Var` — and templates are compiled on first alignment, since
   alignment already requires two records to exist.
2. **Templates decayed to death mid-ingest.** `observe` never reinforced
   anything, so a skill learned early sat at strength 10 and lost 1 per decay
   tick, hitting the floor about 1000 observations later. A skeleton that keeps
   explaining incoming records now counts as in use.

The second was the larger effect by far: without it HDFS sat at 33.4%, with it
99.7%.

## Where it still loses, with evidence

Linux_2k is the worst case (17.9% vs 68.4%) and shows the remaining structural
limit clearly: XDPD's `Template` is **fixed-length**, so one event type whose
messages differ in token count cannot be one template. Linux has 118 event types
across highly variable message lengths; HDFS has 14 types where 13 have a single
token count. HDFS's own residual error is the same shape — event `E4`, whose five
messages are 6, 14, and 105 tokens long, is the only type XDPD fails to claim.

Fixing that means variable-span slots — a slot that absorbs a run of *n* tokens
rather than exactly one — which is a real change to matching and to param
capture, not a threshold tweak. It is not queued here; it is written down.

There is a second, smaller limit with the same flavour. Widening only merges two
skeletons that still agree on at least half their positions, which is what stops
unrelated record types collapsing into one match-everything template. A type
carrying very little constant context — mostly variable fields — can therefore
settle into two skeletons instead of one, and GA scores both zero. The guard is
load-bearing, so this is a trade-off to tune with measurements, not a bug to
remove; `a_template_widens_to_cover_every_position_that_ever_varies` in
`xdpd/src/lib.rs` pins the current behaviour.

**Honest summary: XDPD's template mining went from unusable to competitive, and
"competitive" means it still loses to Drain3 on every dataset it was not
developed against.**
