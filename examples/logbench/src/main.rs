// XDPD log template benchmark — real data, real numbers.
//
// Measures XDPD's template mining against loghub ground truth, using the same
// Grouping Accuracy metric the log-parsing literature uses for Drain and its
// successors. The point of this benchmark is to replace the project's synthetic
// headline numbers with measured ones, whatever they turn out to be.
//
//   cargo run --release -- data/HDFS_2k.log_structured.csv
//   cargo run --release -- data/Apache_2k.log_structured.csv
//
// Datasets are from https://github.com/logpai/loghub (the standard benchmark
// corpus). Their *_structured.csv files carry the hand-labelled EventId per
// line, which is the ground truth compared against here.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;

use xdpd::{Learner, LearnerConfig, PatternShape, Slot, Token};

// ---------------------------------------------------------------------------
// Tokenization — the domain adapter
// ---------------------------------------------------------------------------
//
// XDPD is domain-agnostic: it consumes u32 tokens and knows nothing about text.
// Turning a log line into tokens is the caller's job, and this is the whole of
// it — split on whitespace, hash each word. Note this produces tokens spread
// across the full u32 range, which is exactly the case that used to overflow
// the delta arithmetic.

fn fnv1a(word: &str) -> Token {
    let mut h: u32 = 0x811c_9dc5;
    for b in word.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn tokenize(line: &str) -> Vec<Token> {
    line.split_whitespace().map(fnv1a).collect()
}

// ---------------------------------------------------------------------------
// Minimal CSV reader — quoted fields may contain commas
// ---------------------------------------------------------------------------

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => fields.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    fields.push(cur);
    fields
}

struct Record {
    content: String,
    event_id: String,
}

fn load_dataset(path: &str) -> Result<Vec<Record>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
    let mut lines = text.lines();
    let header = split_csv_line(lines.next().ok_or("empty file")?);
    let col = |name: &str| -> Result<usize, String> {
        header
            .iter()
            .position(|h| h.trim() == name)
            .ok_or_else(|| format!("no `{}` column; found {:?}", name, header))
    };
    let (ci, ei) = (col("Content")?, col("EventId")?);

    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv_line(line);
        if f.len() <= ci.max(ei) {
            continue;
        }
        out.push(Record {
            content: f[ci].clone(),
            event_id: f[ei].clone(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Cluster assignment
// ---------------------------------------------------------------------------

fn fixed_count(shape: &PatternShape) -> usize {
    match shape {
        PatternShape::Template(slots) => {
            slots.iter().filter(|s| matches!(s, Slot::Fixed(_))).count()
        }
        _ => 0,
    }
}

/// Which learned template claims this line. When several match, the most
/// specific one wins (most fixed positions), with the name as a tie-break so
/// the assignment is deterministic.
fn assign<'a>(templates: &'a [(String, PatternShape)], tokens: &[Token]) -> Option<&'a str> {
    templates
        .iter()
        .filter(|(_, shape)| shape.matches(tokens).is_some())
        .max_by(|a, b| {
            fixed_count(&a.1)
                .cmp(&fixed_count(&b.1))
                .then_with(|| b.0.cmp(&a.0))
        })
        .map(|(name, _)| name.as_str())
}

/// Grouping Accuracy, as used throughout the log-parsing literature: a message
/// counts as correct only when its predicted cluster holds exactly the same set
/// of messages as its ground-truth cluster. Partial overlap scores zero, which
/// is what makes GA a demanding metric.
fn grouping_accuracy(predicted: &[Option<&str>], truth: &[String]) -> (f64, usize) {
    let mut pred_groups: HashMap<&str, HashSet<usize>> = HashMap::new();
    let mut true_groups: HashMap<&str, HashSet<usize>> = HashMap::new();
    for (i, p) in predicted.iter().enumerate() {
        // Unassigned lines each form their own singleton cluster; that is the
        // honest reading, not a free pass.
        match p {
            Some(name) => {
                pred_groups.entry(name).or_default().insert(i);
            }
            None => {
                pred_groups.entry("<unassigned>").or_default();
            }
        }
    }
    for (i, t) in truth.iter().enumerate() {
        true_groups.entry(t.as_str()).or_default().insert(i);
    }

    let true_sets: HashSet<Vec<usize>> = true_groups
        .values()
        .map(|s| {
            let mut v: Vec<usize> = s.iter().copied().collect();
            v.sort_unstable();
            v
        })
        .collect();

    let mut correct = 0usize;
    for (name, set) in &pred_groups {
        if *name == "<unassigned>" {
            continue;
        }
        let mut v: Vec<usize> = set.iter().copied().collect();
        v.sort_unstable();
        if true_sets.contains(&v) {
            correct += set.len();
        }
    }
    (correct as f64 / truth.len() as f64, pred_groups.len())
}

// ---------------------------------------------------------------------------

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "data/HDFS_2k.log_structured.csv".to_string());

    let records = match load_dataset(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("usage: cargo run --release -- <loghub *_structured.csv>");
            std::process::exit(1);
        }
    };

    println!("XDPD log template benchmark  (xdpd {})", xdpd::VERSION);
    println!("dataset: {}", path);
    println!("lines:   {}", records.len());

    let tokenized: Vec<Vec<Token>> = records.iter().map(|r| tokenize(&r.content)).collect();
    let truth: Vec<String> = records.iter().map(|r| r.event_id.clone()).collect();
    let distinct_true = truth.iter().collect::<HashSet<_>>().len();
    println!("ground-truth event types: {}", distinct_true);

    // --- learn ------------------------------------------------------------
    let mut learner = Learner::with_config(LearnerConfig {
        min_occurrences: 2,
        window_size: 5000,
    });
    let start = std::time::Instant::now();
    for tokens in &tokenized {
        learner.observe(tokens);
    }
    let learn_time = start.elapsed();

    let templates: Vec<(String, PatternShape)> = learner
        .skills()
        .into_iter()
        .filter(|s| matches!(s.shape, PatternShape::Template(_)))
        .map(|s| (s.name.clone(), s.shape.clone()))
        .collect();

    println!();
    println!("learn time:        {:?}", learn_time);
    println!("skills learned:    {}", learner.skill_count());
    println!("  of which templates: {}", templates.len());

    // --- lossless check ---------------------------------------------------
    // The guarantee that matters most: whenever a template claims a line, the
    // line must be reproducible from it byte for byte.
    let mut claimed = 0usize;
    let mut lossless_failures = 0usize;
    for tokens in &tokenized {
        if let Some(name) = assign(&templates, tokens) {
            claimed += 1;
            let shape = &templates.iter().find(|(n, _)| n == name).unwrap().1;
            let params = shape.matches(tokens).unwrap();
            let mut vm = xdpd::VM::new();
            vm.add_skill(xdpd::Skill::new("t".into(), shape.clone()));
            vm.load_program(vec![
                xdpd::Instr::Call("t".into(), params),
                xdpd::Instr::Ret,
            ]);
            vm.run();
            if vm.output() != tokens.as_slice() {
                lossless_failures += 1;
            }
        }
    }

    // --- accuracy ---------------------------------------------------------
    let predicted: Vec<Option<&str>> = tokenized
        .iter()
        .map(|t| assign(&templates, t))
        .collect();
    let (ga, pred_clusters) = grouping_accuracy(&predicted, &truth);

    // --- compression ------------------------------------------------------
    let mut naive_ops = 0u64;
    let mut learned_ops = 0u64;
    for tokens in &tokenized {
        let (_, n) = xdpd::compose(&HashMap::new(), tokens);
        naive_ops += n;
        let (_, l) = xdpd::compose(learner.vm().subroutines(), tokens);
        learned_ops += l;
    }

    println!();
    println!("--- results (program-level ops, measured on real data) ---");
    println!("lines claimed by a template: {} / {}", claimed, tokenized.len());
    println!("predicted clusters:          {}", pred_clusters);
    println!("grouping accuracy (GA):      {:.1}%", ga * 100.0);
    println!("naive ops:                   {}", naive_ops);
    println!("learned ops:                 {}", learned_ops);
    if learned_ops > 0 {
        println!(
            "compression:                 {:.2}x  ({:.1}% fewer ops)",
            naive_ops as f64 / learned_ops as f64,
            (1.0 - learned_ops as f64 / naive_ops as f64) * 100.0
        );
    }
    println!(
        "lossless reproduction:       {}",
        if lossless_failures == 0 {
            "OK — every claimed line reproduced exactly".to_string()
        } else {
            format!("FAILED on {} lines", lossless_failures)
        }
    );

    if lossless_failures > 0 {
        // Losslessness is the one property the whole design rests on. A failure
        // here is not a weak score, it is a broken invariant.
        std::process::exit(1);
    }
}
