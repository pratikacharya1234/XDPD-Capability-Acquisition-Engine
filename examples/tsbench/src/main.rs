// XDPD numeric-stream anomaly benchmark — real labelled data.
//
// Tests the two properties that survived the log benchmark as *possibly*
// distinct (docs/ARCHITECTURE.md §I.3): boundary-free coverage of a continuous
// stream, and algebraic primitives that encode a progression in one instruction.
//
// The question is narrow and falsifiable: can XDPD's compression ratio flag real
// labelled anomalies better than a rolling z-score? A z-score is five lines of
// arithmetic. If XDPD cannot beat it, the numeric-stream pitch is dead and it is
// better to learn that here than from a customer.
//
//   ./fetch-data.sh
//   cargo run --release
//
// Data: Numenta Anomaly Benchmark (https://github.com/numenta/NAB), whose
// combined_windows labels give hand-marked anomaly windows per series.

use std::collections::HashMap;
use std::fs;

use xdpd::{Learner, LearnerConfig, Token};

const SERIES: &[&str] = &[
    "machine_temperature_system_failure.csv",
    "ec2_request_latency_system_failure.csv",
    "ec2_cpu_utilization_5f5533.csv",
    "nyc_taxi.csv",
];

/// NAB's probationary period: the leading fraction a detector may learn from
/// before its output counts.
const PROBATION: f64 = 0.15;
/// Tokens per observation fed to the learner, and the scoring window width.
const WINDOW: usize = 8;
/// Discretization levels. SAX-style: z-normalize, then quantize.
const LEVELS: i32 = 16;

struct Series {
    name: String,
    values: Vec<f64>,
    /// True where the point falls inside a labelled anomaly window.
    labels: Vec<bool>,
}

fn parse_series(dir: &str, name: &str, windows: &[(String, String)]) -> Option<Series> {
    let text = fs::read_to_string(format!("{}/{}", dir, name)).ok()?;
    let mut values = Vec::new();
    let mut stamps = Vec::new();
    for line in text.lines().skip(1) {
        let mut f = line.split(',');
        let ts = f.next()?.trim().to_string();
        let v: f64 = f.next()?.trim().parse().ok()?;
        stamps.push(ts);
        values.push(v);
    }
    // Labels are timestamp ranges; NAB writes them with a .000000 suffix that
    // the data files omit, so compare on the shared prefix.
    let labels = stamps
        .iter()
        .map(|ts| {
            windows.iter().any(|(a, b)| {
                let (a, b) = (&a[..19.min(a.len())], &b[..19.min(b.len())]);
                ts.as_str() >= a && ts.as_str() <= b
            })
        })
        .collect();
    Some(Series {
        name: name.to_string(),
        values,
        labels,
    })
}

fn load_labels(dir: &str) -> HashMap<String, Vec<(String, String)>> {
    let mut out: HashMap<String, Vec<(String, String)>> = HashMap::new();
    if let Ok(text) = fs::read_to_string(format!("{}/labels.csv", dir)) {
        for line in text.lines().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() >= 3 {
                out.entry(f[0].to_string())
                    .or_default()
                    .push((f[1].to_string(), f[2].to_string()));
            }
        }
    }
    out
}

/// The domain adapter: z-normalize, then quantize to `LEVELS` bands. This is the
/// discretization step SAX performs, and it is what lets an integer-token engine
/// see a float series at all.
fn discretize(values: &[f64]) -> Vec<Token> {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let sd = var.sqrt().max(1e-9);
    values
        .iter()
        .map(|v| {
            let z = (v - mean) / sd;
            // Map roughly [-3, 3] onto 0..LEVELS.
            let band = (((z + 3.0) / 6.0) * LEVELS as f64).round() as i32;
            band.clamp(0, LEVELS - 1) as Token
        })
        .collect()
}

/// Scores from XDPD: for each position, how well the surrounding window
/// compresses against what the learner knows. Familiar structure compresses;
/// unfamiliar structure does not. Higher score = more anomalous.
fn xdpd_scores(tokens: &[Token], probation: usize) -> Vec<f64> {
    let mut learner = Learner::with_config(LearnerConfig {
        min_occurrences: 2,
        window_size: 2000,
    });
    // Learn only from the probationary prefix, as a detector would.
    let mut i = 0;
    while i + WINDOW <= probation {
        learner.observe(&tokens[i..i + WINDOW]);
        i += 1;
    }

    let mut scores = vec![0.0; tokens.len()];
    for pos in 0..tokens.len() {
        let start = pos.saturating_sub(WINDOW / 2);
        let end = (start + WINDOW).min(tokens.len());
        if end - start < 2 {
            continue;
        }
        let ratio = learner.check_anomaly(&tokens[start..end]);
        // check_anomaly returns naive/learned: high means it compressed well.
        // Invert so that "did not compress" reads as "anomalous".
        scores[pos] = 1.0 / ratio.max(1e-9);
    }
    scores
}

/// Baseline: rolling z-score on the raw values. Five lines of arithmetic, and
/// the bar any new detector has to clear to be worth its complexity.
fn zscore_scores(values: &[f64], win: usize) -> Vec<f64> {
    let mut scores = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(win);
        let slice = &values[start..=i];
        if slice.len() < 3 {
            continue;
        }
        let n = slice.len() as f64;
        let mean = slice.iter().sum::<f64>() / n;
        let sd = (slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n)
            .sqrt()
            .max(1e-9);
        scores[i] = ((values[i] - mean) / sd).abs();
    }
    scores
}

/// Window-detection scoring, the standard reading for NAB-style labels: a
/// labelled window counts as detected if any flagged point lands inside it. A
/// flag outside every window is a false positive.
struct Eval {
    detected: usize,
    windows: usize,
    false_positives: usize,
    negatives: usize,
}

impl Eval {
    fn f1(&self) -> f64 {
        let recall = if self.windows == 0 {
            0.0
        } else {
            self.detected as f64 / self.windows as f64
        };
        // Precision proxy: how much of the normal region stayed unflagged.
        let fpr = if self.negatives == 0 {
            0.0
        } else {
            self.false_positives as f64 / self.negatives as f64
        };
        let precision = 1.0 - fpr;
        if recall + precision <= 0.0 {
            0.0
        } else {
            2.0 * recall * precision / (recall + precision)
        }
    }
}

fn evaluate(scores: &[f64], labels: &[bool], threshold: f64, probation: usize) -> Eval {
    // Contiguous runs of `true` are the labelled windows.
    let mut windows: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < labels.len() {
        if labels[i] {
            let start = i;
            while i < labels.len() && labels[i] {
                i += 1;
            }
            windows.push((start, i - 1));
        } else {
            i += 1;
        }
    }

    let flagged: Vec<bool> = scores
        .iter()
        .enumerate()
        .map(|(i, s)| i >= probation && *s >= threshold)
        .collect();

    let detected = windows
        .iter()
        .filter(|(a, b)| (*a..=*b).any(|i| flagged[i]))
        .count();
    let mut false_positives = 0;
    let mut negatives = 0;
    for i in probation..labels.len() {
        if !labels[i] {
            negatives += 1;
            if flagged[i] {
                false_positives += 1;
            }
        }
    }
    Eval {
        detected,
        windows: windows.len(),
        false_positives,
        negatives,
    }
}

/// Pick the threshold that maximises F1 — applied identically to both detectors,
/// so neither gets a tuning advantage. This is generous to both (it is an oracle
/// choice), and that is fine as long as it is generous symmetrically.
fn best(scores: &[f64], labels: &[bool], probation: usize) -> (f64, Eval) {
    let mut candidates: Vec<f64> = scores[probation..].to_vec();
    candidates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    candidates.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let step = (candidates.len() / 200).max(1);

    let mut best_f1 = -1.0;
    let mut out = (
        0.0,
        Eval {
            detected: 0,
            windows: 0,
            false_positives: 0,
            negatives: 0,
        },
    );
    for t in candidates.iter().step_by(step) {
        let e = evaluate(scores, labels, *t, probation);
        if e.f1() > best_f1 {
            best_f1 = e.f1();
            out = (*t, e);
        }
    }
    out
}

fn main() {
    let dir = "data";
    let labels = load_labels(dir);
    if labels.is_empty() {
        eprintln!("no labels.csv in {}/ — run ./fetch-data.sh first", dir);
        std::process::exit(1);
    }

    println!("XDPD numeric-stream anomaly benchmark  (xdpd {})", xdpd::VERSION);
    println!(
        "discretization: z-normalize -> {} bands   window: {}   probation: {:.0}%",
        LEVELS,
        WINDOW,
        PROBATION * 100.0
    );
    println!();
    println!(
        "{:<42} {:>7} {:>9} {:>9}",
        "series", "windows", "XDPD F1", "z-score F1"
    );
    println!("{}", "-".repeat(72));

    let mut xdpd_wins = 0;
    let mut z_wins = 0;
    let mut sum_x = 0.0;
    let mut sum_z = 0.0;
    let mut count = 0;

    for name in SERIES {
        let windows = labels.get(*name).cloned().unwrap_or_default();
        let series = match parse_series(dir, name, &windows) {
            Some(s) => s,
            None => {
                println!("{:<42} (missing)", name);
                continue;
            }
        };
        let probation = (series.values.len() as f64 * PROBATION) as usize;
        let tokens = discretize(&series.values);

        let xs = xdpd_scores(&tokens, probation);
        let zs = zscore_scores(&series.values, WINDOW * 8);

        let (_, xe) = best(&xs, &series.labels, probation);
        let (_, ze) = best(&zs, &series.labels, probation);

        let short: String = series.name.trim_end_matches(".csv").chars().take(40).collect();
        println!(
            "{:<42} {:>7} {:>8.3} {:>9.3}   [{}/{} vs {}/{} windows found]",
            short,
            xe.windows,
            xe.f1(),
            ze.f1(),
            xe.detected,
            xe.windows,
            ze.detected,
            ze.windows
        );
        if xe.f1() > ze.f1() {
            xdpd_wins += 1;
        } else if ze.f1() > xe.f1() {
            z_wins += 1;
        }
        sum_x += xe.f1();
        sum_z += ze.f1();
        count += 1;
    }

    println!("{}", "-".repeat(72));
    if count > 0 {
        println!(
            "mean F1:  XDPD {:.3}   z-score {:.3}",
            sum_x / count as f64,
            sum_z / count as f64
        );
        println!("series won: XDPD {}   z-score {}", xdpd_wins, z_wins);
        println!();
        if sum_x > sum_z {
            println!("XDPD ahead of the z-score baseline on mean F1.");
        } else {
            println!("XDPD does NOT beat a rolling z-score. Reporting it as measured.");
        }
    }
}
