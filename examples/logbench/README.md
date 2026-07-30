# XDPD log template benchmark

Measures XDPD's template mining against real data with published ground truth,
and against Drain3 — the established production baseline.

**XDPD does not win. On six datasets downloaded only after every parameter was
frozen, it scores 62.1% mean grouping accuracy against Drain3's 65.5%** — close,
two outright wins, still behind. It started this work at 32.4% and 1.6%.

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

## Results — measured 2026-07-30, xdpd 0.3.0

### Clean evaluation: downloaded after the parameters were frozen

These six were never looked at during development. They are the number that
counts.

| dataset | XDPD GA | Drain3 GA | delta |
|---|---|---|---|
| Spark_2k | 91.3% | **92.2%** | -0.9 |
| Mac_2k | 67.8% | **71.5%** | -3.7 |
| Hadoop_2k | 67.8% | **95.3%** | -27.5 |
| HealthApp_2k | 48.8% | **57.6%** | -8.8 |
| HPC_2k | 44.4% | **74.1%** | -29.7 |
| Proxifier_2k | **52.6%** | 2.5% | **+50.1** |
| **mean** | **62.1%** | **65.5%** | **-3.4** |

### Development sets: used while building and tuning

Reported for completeness. These numbers are optimistic by construction and
must not be quoted on their own.

| dataset | XDPD GA before | XDPD GA now | Drain3 GA |
|---|---|---|---|
| HDFS_2k | 32.4% | 99.7% | **99.8%** |
| Apache_2k | 1.6% | **100.0%** | **100.0%** |
| BGL_2k | — | 91.0% | **96.9%** |
| Zookeeper_2k | — | 93.2% | **96.7%** |
| OpenSSH_2k | — | **72.5%** | 71.8% |
| Linux_2k | — | 63.1% | **68.4%** |
| **mean** | | **86.6%** | **88.9%** |

Across all twelve: XDPD wins 2, ties 1, loses 9.

**Where XDPD wins, and why it is interesting.** Proxifier is the case Drain3
collapses on (2.5%) and XDPD does not. Proxifier's messages are highly uniform
in structure with variability concentrated in a few fields — Drain's prefix tree
puts nearly everything in one leaf and merges it. XDPD has no prefix tree; it
compares each record against every skeleton of the same length. That is slower
and usually worse, and on this shape of data it is much better.

**Lossless reproduction: OK on all twelve.** Every line claimed by a template
reproduces byte-for-byte from that template plus its captured parameters. The
benchmark exits non-zero if it ever fails.

Compression on HDFS went 1.45x -> **11.90x** program-level ops. Learn time is
~1.3ms per 2000 lines against Drain3's ~4ms.

## What changed, and why it worked

Three defects, found by measuring rather than guessing — the first guess made
things *worse* (32.4% -> 22.1%).

1. **Templates decayed to death mid-ingest.** `observe` reinforced nothing, so a
   skill learned early sat at strength 10, lost 1 per decay tick and hit the
   floor about 1000 observations later — while records of its own type were
   still arriving. The table was forgetting exactly what it was looking at. A
   skeleton that keeps explaining incoming records now counts as in use. This
   was the single largest effect: HDFS 33.4% -> 99.7%.
2. **Alignment froze coincidence and never widened.** Aligning two records marks
   a position `Fixed` wherever those two agree, including positions that vary
   across the type as a whole. Every pair froze a *different* set of accidents,
   so no two templates shared a signature, frequency counting never saw a repeat
   and the big event types compiled nothing at all.
3. **Records joined skeletons through two different doors.** Judging a record
   against a *template* converges; judging two templates against each other does
   not, because their agreement shrinks as they widen — so a skeleton that had
   generalized enough to be useful stopped accepting anything and its record type
   shattered. Worse, the two paths interacted: a record the strict path rejected
   got paired with a neighbour by alignment and merged back in through the side
   door. There is now exactly one way in, under one rule.

The rule that door enforces: a record may turn at most **a quarter** of a
skeleton's positions variable. Another instance of the same type disagrees with
its skeleton in a few places; a record that contradicts many at once is a
different type sharing a prefix. `Received disconnect from IP: 11: Bye Bye
[preauth]` and `Received disconnect from IP: 11: disconnected by user` are the
real case — same length, same first five tokens, different events. Similarity
alone cannot separate them; how much the skeleton must give up to accept the
record can.

That quarter, and the 0.4 similarity floor, are the only two tunables. 0.4 is
Drain's own published default. The quarter was chosen by sweeping against the
six development sets — which is exactly why the six clean sets were downloaded
afterwards and the parameters frozen first.

## Where it still loses

HPC (-29.7) and Hadoop (-27.5) are the worst cases and neither has been
diagnosed yet. Do that before touching anything else; two of the three fixes
above only worked because they came after a measurement, and the one guess made
without one moved the number backwards.

The known structural limit is that `Template` is **fixed-length**, so an event
type whose messages differ in token count cannot be a single template. This did
*not* explain the earlier Linux failure — zero event types in Linux, BGL or
OpenSSH span multiple lengths — but it is real: HDFS's only residual error is
the event whose five messages run 6, 14 and 105 tokens. Fixing it means
variable-span slots, a real change to matching and param capture rather than a
threshold tweak. Written down, not queued.

**Honest summary: template mining went from unusable to within 3.4 points of
Drain3 on clean data, winning two of twelve datasets. That is a large
improvement and it is not a win.**
