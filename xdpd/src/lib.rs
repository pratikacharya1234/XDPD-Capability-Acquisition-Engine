// XDPD: Capability Acquisition Engine
//
// A learning mechanism fundamentally different from neural networks.
// Instead of storing observations or adjusting weights, it grows
// its own instruction set by detecting invariants in token streams
// and compiling them into permanent subroutines.
//
// The subroutine table is the only memory. Raw observations are
// discarded after pattern detection. One Call instruction executes
// an entire learned subroutine atomically, regardless of its body size.
//
// Zero dependencies. Pure Rust. CPU only. No GPU required.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::collections::HashMap;
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Token type
// ---------------------------------------------------------------------------

/// Unified token type for all discrete observations, actions, and symbols.
pub type Token = u32;

/// Difference between two tokens, or `None` when it cannot be represented as
/// the `i32` delta a pattern stores.
///
/// Tokens span the entire `u32` range, so their true difference can exceed
/// `i32`. Computing it as `b as i32 - a as i32` reinterprets both values as
/// signed and then overflows the subtraction — a debug-build panic on ordinary
/// data such as hashed identifiers or wide numeric ids. Every delta comparison
/// in this crate goes through here.
fn token_delta(from: Token, to: Token) -> Option<i32> {
    i32::try_from(to as i64 - from as i64).ok()
}

/// Advance a token by a delta, wrapping rather than panicking.
///
/// `i32` addition on a value reinterpreted from `u32` overflows in debug builds
/// even when the wrapped bit pattern is exactly what the sequence needs.
fn token_step(v: Token, delta: i32) -> Token {
    v.wrapping_add(delta as Token)
}

// ---------------------------------------------------------------------------
// Instruction set
// ---------------------------------------------------------------------------

/// The system starts with exactly these primitives. All other capabilities
/// are learned by compiling observed patterns into subroutines.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Instr {
    /// Load an immediate value into the register.
    Load(Token),
    /// Emit the register value to the output buffer.
    Output,
    /// Generate an arithmetic sequence: start, delta, length.
    Seq(Token, i32, usize),
    /// Execute a learned subroutine by name, binding it to these parameters
    /// (e.g. the arithmetic start value, or the repeat unit's contents).
    /// Cost: 1 instruction regardless of body size.
    Call(String, Vec<Token>),
    /// Return from a subroutine.
    Ret,
}

// ---------------------------------------------------------------------------
// Skill — a learned subroutine
// ---------------------------------------------------------------------------

/// A compiled capability. Rather than a frozen instruction sequence, a skill
/// stores the *structure* of the pattern it was compiled from — its body is
/// re-derived at call time from whatever concrete values match that structure.
/// This lets one skill generalize across every sequence with the same shape,
/// not just the instance it was first observed from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    /// Human-readable name, usually auto-generated from the pattern signature.
    pub name: String,
    /// The structural template this skill was compiled from.
    pub shape: PatternShape,
    /// Usage-dependent strength score; decays when unused.
    pub strength: i32,
    /// Number of times this skill has been invoked.
    pub uses: u32,
    /// Structural signature for generalization.
    pub signature: String,
}

impl Skill {
    /// Create a new skill with default strength.
    pub fn new(name: String, shape: PatternShape) -> Self {
        Skill {
            name,
            shape,
            strength: 10,
            uses: 0,
            signature: String::new(),
        }
    }

    /// Number of instructions the compiled body would contain (complexity measure).
    pub fn instruction_count(&self) -> usize {
        self.shape.instruction_count()
    }
}

// ---------------------------------------------------------------------------
// Pattern types detected in token streams
// ---------------------------------------------------------------------------

/// A detected structural invariant in a token sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// All tokens equal: [x, x, x, ...]
    Constant { value: Token, len: usize },
    /// Arithmetic progression: [start, start+delta, start+2*delta, ...]
    Arithmetic { start: Token, delta: i32, len: usize },
    /// Repeated unit: [A,B, A,B, A,B, ...]
    Repeat { unit: Vec<Token>, count: usize },
}

impl Pattern {
    /// Generate the token sequence described by this pattern.
    pub fn generate(&self) -> Vec<Token> {
        match self {
            Pattern::Constant { value, len } => vec![*value; *len],
            Pattern::Arithmetic { start, delta, len } => {
                let mut v = *start;
                (0..*len).map(|_| { let out = v; v = token_step(v, *delta); out }).collect()
            }
            Pattern::Repeat { unit, count } => {
                unit.iter().copied().cycle().take(unit.len() * *count).collect()
            }
        }
    }

    /// Compile the pattern into VM instructions.
    pub fn to_instructions(&self) -> Vec<Instr> {
        match self {
            Pattern::Constant { value, len } => {
                vec![Instr::Seq(*value, 0, *len), Instr::Ret]
            }
            Pattern::Arithmetic { start, delta, len } => {
                vec![Instr::Seq(*start, *delta, *len), Instr::Ret]
            }
            Pattern::Repeat { unit, count } => {
                let mut body = Vec::with_capacity(unit.len() * 2 * count + 1);
                for _ in 0..*count {
                    for &t in unit {
                        body.push(Instr::Load(t));
                        body.push(Instr::Output);
                    }
                }
                body.push(Instr::Ret);
                body
            }
        }
    }

    /// Complexity score: lower is more compressed.
    pub fn complexity(&self) -> usize {
        match self {
            Pattern::Constant { .. } => 1,
            Pattern::Arithmetic { .. } => 1,
            Pattern::Repeat { unit, count } => unit.len() * 2 * count + 1,
        }
    }

    /// Structural signature for pattern deduplication.
    pub fn signature(&self) -> String {
        match self {
            Pattern::Constant { len, .. } => format!("const:x{}", len),
            Pattern::Arithmetic { delta, len, .. } => format!("arith:d{}x{}", delta, len),
            Pattern::Repeat { unit, count } => format!("rep:{}x{}", unit.len(), count),
        }
    }

    /// The value-free structural template for this pattern — what a compiled
    /// skill actually stores and matches future sequences against.
    pub fn shape(&self) -> PatternShape {
        match self {
            Pattern::Constant { len, .. } => PatternShape::Constant { len: *len },
            Pattern::Arithmetic { delta, len, .. } => {
                PatternShape::Arithmetic { delta: *delta, len: *len }
            }
            Pattern::Repeat { unit, count } => {
                PatternShape::Repeat { unit_len: unit.len(), count: *count }
            }
        }
    }

    /// The concrete values needed to re-derive this specific instance from
    /// its shape (e.g. the arithmetic start value, or the repeat unit).
    pub fn params(&self) -> Vec<Token> {
        match self {
            Pattern::Constant { value, .. } => vec![*value],
            Pattern::Arithmetic { start, .. } => vec![*start],
            Pattern::Repeat { unit, .. } => unit.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// PatternShape — a pattern's structure, stripped of concrete values
// ---------------------------------------------------------------------------

/// One position in a `PatternShape::Template`.
///
/// Real streams are template-plus-variable: `GET /api/user/8814 200 34ms` is
/// the same *shape* as `GET /api/user/2291 404 12ms`, differing only where the
/// id and status sit. `Fixed` pins the parts that never change, `Var` marks the
/// parts that do, and `Run` covers a counting stretch. `Constant`/`Arithmetic`/
/// `Repeat` cannot express that mixture, which is why they only ever matched
/// uniform sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    /// This exact token must appear here, or the match fails.
    Fixed(Token),
    /// Any single token. Its value is captured into params.
    Var,
    /// An arithmetic run of `len` tokens stepping by `delta`. The start value
    /// is captured into params; `delta` and `len` are part of the structure.
    Run { delta: i32, len: usize },
}

impl Slot {
    /// How many tokens this slot spans.
    fn span(&self) -> usize {
        match self {
            Slot::Fixed(_) | Slot::Var => 1,
            Slot::Run { len, .. } => *len,
        }
    }

    /// Whether matching this slot captures a param. Keeps `matches` and
    /// `to_instructions` agreeing on param order — the one way a template can
    /// silently scramble its own output.
    fn captures(&self) -> bool {
        matches!(self, Slot::Var | Slot::Run { .. })
    }
}

/// The structural template of a `Pattern`, with concrete values removed.
/// A `Skill` stores one of these instead of a frozen output sequence, so it
/// can recognize and reproduce *any* sequence with matching structure —
/// not just the specific instance it was compiled from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternShape {
    Constant { len: usize },
    Arithmetic { delta: i32, len: usize },
    Repeat { unit_len: usize, count: usize },
    /// The general case: a sequence of fixed, variable, and run slots.
    /// The three variants above are fast paths for shapes it could also
    /// express, kept because they encode more compactly and match faster.
    Template(Vec<Slot>),
}

impl PatternShape {
    /// Number of tokens a match against this shape spans.
    pub fn span_len(&self) -> usize {
        match self {
            PatternShape::Constant { len } => *len,
            PatternShape::Arithmetic { len, .. } => *len,
            PatternShape::Repeat { unit_len, count } => unit_len * count,
            PatternShape::Template(slots) => slots.iter().map(Slot::span).sum(),
        }
    }

    /// Number of instructions the compiled body would contain.
    pub fn instruction_count(&self) -> usize {
        match self {
            PatternShape::Constant { .. } => 2,
            PatternShape::Arithmetic { .. } => 2,
            PatternShape::Repeat { unit_len, count } => unit_len * 2 * count + 1,
            // Fixed/Var emit Load+Output, Run emits a single Seq, plus one Ret.
            PatternShape::Template(slots) => {
                slots
                    .iter()
                    .map(|s| match s {
                        Slot::Fixed(_) | Slot::Var => 2,
                        Slot::Run { .. } => 1,
                    })
                    .sum::<usize>()
                    + 1
            }
        }
    }

    /// How many params a match against this shape captures.
    fn param_count(&self) -> usize {
        match self {
            PatternShape::Constant { .. } | PatternShape::Arithmetic { .. } => 1,
            PatternShape::Repeat { unit_len, .. } => *unit_len,
            PatternShape::Template(slots) => slots.iter().filter(|s| s.captures()).count(),
        }
    }

    /// Check whether `slice` structurally fits this shape. On success,
    /// returns the params needed to regenerate `slice` from this shape.
    pub fn matches(&self, slice: &[Token]) -> Option<Vec<Token>> {
        match self {
            PatternShape::Constant { len } => {
                if slice.len() != *len {
                    return None;
                }
                slice.iter().all(|&t| t == slice[0]).then(|| vec![slice[0]])
            }
            PatternShape::Arithmetic { delta, len } => {
                if slice.len() != *len {
                    return None;
                }
                // `len < 2` would also have indexed slice[1] out of bounds.
                if *len < 2 {
                    return None;
                }
                slice
                    .windows(2)
                    .all(|w| token_delta(w[0], w[1]) == Some(*delta))
                    .then(|| vec![slice[0]])
            }
            PatternShape::Repeat { unit_len, count } => {
                if slice.len() != unit_len * count {
                    return None;
                }
                let unit = &slice[0..*unit_len];
                slice
                    .chunks(*unit_len)
                    .all(|chunk| chunk == unit)
                    .then(|| unit.to_vec())
            }
            PatternShape::Template(slots) => {
                // A zero-span shape would be a skill that consumes no input.
                // `compose` would never make progress with it; refuse it here
                // rather than let Phase 4 accidentally construct one.
                if slots.is_empty() || slice.len() != self.span_len() {
                    return None;
                }
                let mut params = Vec::with_capacity(self.param_count());
                let mut at = 0;
                for slot in slots {
                    match slot {
                        // Exact, or no match. This is the lossless guarantee:
                        // there is no "close enough" branch here by design.
                        Slot::Fixed(t) => {
                            if slice[at] != *t {
                                return None;
                            }
                        }
                        Slot::Var => params.push(slice[at]),
                        Slot::Run { delta, len } => {
                            let run = &slice[at..at + len];
                            if !run.windows(2).all(|w| token_delta(w[0], w[1]) == Some(*delta)) {
                                return None;
                            }
                            params.push(run[0]);
                        }
                    }
                    at += slot.span();
                }
                Some(params)
            }
        }
    }

    /// Compile instructions for this shape, bound to the given params.
    pub fn to_instructions(&self, params: &[Token]) -> Vec<Instr> {
        match self {
            PatternShape::Constant { len } => {
                vec![Instr::Seq(params[0], 0, *len), Instr::Ret]
            }
            PatternShape::Arithmetic { delta, len } => {
                vec![Instr::Seq(params[0], *delta, *len), Instr::Ret]
            }
            PatternShape::Repeat { unit_len, count } => {
                let mut body = Vec::with_capacity(unit_len * 2 * count + 1);
                for _ in 0..*count {
                    for &t in &params[..*unit_len] {
                        body.push(Instr::Load(t));
                        body.push(Instr::Output);
                    }
                }
                body.push(Instr::Ret);
                body
            }
            PatternShape::Template(slots) => {
                let mut body = Vec::with_capacity(self.instruction_count());
                // Params are consumed in slot order — the same order
                // `matches` produced them. If these two ever disagree, output
                // comes out scrambled while every length check still passes.
                let mut next = 0;
                for slot in slots {
                    match slot {
                        Slot::Fixed(t) => {
                            body.push(Instr::Load(*t));
                            body.push(Instr::Output);
                        }
                        Slot::Var => {
                            body.push(Instr::Load(params[next]));
                            body.push(Instr::Output);
                            next += 1;
                        }
                        Slot::Run { delta, len } => {
                            body.push(Instr::Seq(params[next], *delta, *len));
                            next += 1;
                        }
                    }
                }
                body.push(Instr::Ret);
                body
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Virtual Machine
// ---------------------------------------------------------------------------

/// The execution engine. Holds a single register, a program counter, an
/// output buffer, a call stack, and the subroutine table — which is the
/// only persistent memory in the system.
pub struct VM {
    reg: Token,
    pc: usize,
    program: Vec<Instr>,
    output: Vec<Token>,
    call_stack: Vec<usize>,
    subroutines: HashMap<String, Skill>,
    instr_count: u64,
    call_depth: u32,
    last_error: Option<String>,
}

impl VM {
    /// Create a new VM with no loaded program and an empty subroutine table.
    pub fn new() -> Self {
        VM {
            reg: 0,
            pc: 0,
            program: Vec::new(),
            output: Vec::new(),
            call_stack: Vec::new(),
            subroutines: HashMap::new(),
            instr_count: 0,
            call_depth: 0,
            last_error: None,
        }
    }

    /// Load instructions into program memory.
    pub fn load_program(&mut self, prog: Vec<Instr>) {
        self.program = prog;
        self.pc = 0;
    }

    /// Reset the VM state without clearing the subroutine table.
    pub fn reset(&mut self) {
        self.reg = 0;
        self.pc = 0;
        self.output.clear();
        self.call_stack.clear();
        self.instr_count = 0;
        self.call_depth = 0;
        self.last_error = None;
    }

    /// Execute a single instruction. Returns `true` if execution should
    /// continue, `false` if the program terminated.
    pub fn step(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return false;
        }
        let instr = self.program[self.pc].clone();
        self.pc += 1;
        self.instr_count += 1;

        match instr {
            Instr::Load(v) => {
                self.reg = v;
            }
            Instr::Output => {
                self.output.push(self.reg);
            }
            Instr::Seq(start, delta, len) => {
                let mut v = start;
                for _ in 0..len {
                    self.output.push(v);
                    v = token_step(v, delta);
                }
            }
            Instr::Call(name, params) => {
                const MAX_CALL_DEPTH: u32 = 64;
                if self.call_depth >= MAX_CALL_DEPTH {
                    self.last_error = Some(format!("call depth limit ({}) exceeded", MAX_CALL_DEPTH));
                    return false;
                }
                // Record the use and reinforce before running, then drop the
                // borrow so the recursive step below can take `self` again.
                let body = self.subroutines.get_mut(&name).map(|skill| {
                    skill.uses = skill.uses.saturating_add(1);
                    skill.strength = skill.strength.saturating_add(STRENGTH_ON_USE).min(STRENGTH_MAX);
                    skill.shape.to_instructions(&params)
                });
                match body {
                    Some(body) => {
                        self.call_stack.push(self.pc);
                        self.call_depth += 1;
                        let saved = std::mem::replace(&mut self.program, body);
                        self.pc = 0;
                        while self.step() {}
                        self.program = saved;
                        self.pc = self.call_stack.pop().unwrap();
                        self.call_depth -= 1;
                        // An error inside the body must abort the caller too,
                        // otherwise the outer program keeps emitting output and
                        // still looks like it succeeded — the exact silent
                        // failure this branch exists to prevent, one level down.
                        if self.last_error.is_some() {
                            return false;
                        }
                    }
                    None => {
                        self.last_error = Some(format!("skill not found: {}", name));
                        return false;
                    }
                }
            }
            Instr::Ret => {
                return false;
            }
        }
        true
    }

    /// Execute the loaded program to completion.
    pub fn run(&mut self) {
        while self.step() {}
    }

    /// Insert a skill into the subroutine table.
    pub fn add_skill(&mut self, skill: Skill) {
        self.subroutines.insert(skill.name.clone(), skill);
    }

    /// Check if a subroutine exists by name.
    pub fn has_skill(&self, name: &str) -> bool {
        self.subroutines.contains_key(name)
    }

    /// Read the output buffer.
    pub fn output(&self) -> &[Token] {
        &self.output
    }

    /// Number of instructions executed since last reset.
    pub fn instruction_count(&self) -> u64 {
        self.instr_count
    }

    /// Number of learned subroutines.
    pub fn skill_count(&self) -> usize {
        self.subroutines.len()
    }

    /// Get the last error that occurred during execution, if any.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Return all skills sorted by strength (strongest first).
    pub fn skills(&self) -> Vec<&Skill> {
        let mut skills: Vec<_> = self.subroutines.values().collect();
        skills.sort_by_key(|s| -s.strength);
        skills
    }

    /// Return read-only access to the subroutine table.
    pub fn subroutines(&self) -> &HashMap<String, Skill> {
        &self.subroutines
    }

    /// Mutable access to the subroutine table, for strength bookkeeping.
    pub fn subroutines_mut(&mut self) -> &mut HashMap<String, Skill> {
        &mut self.subroutines
    }

    /// Forget a skill. Callers holding the learner should prefer
    /// `Learner::forget_skill`, which also prunes the bookkeeping that would
    /// otherwise stop the shape from ever being learned again.
    pub fn remove_skill(&mut self, name: &str) -> Option<Skill> {
        self.subroutines.remove(name)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Persistence — save/load the subroutine table
// ---------------------------------------------------------------------------
//
// The subroutine table is the only memory in the system, so it's the only
// thing that needs to survive a process restart. Serialized as plain text,
// one skill per line — no external dependencies, consistent with the crate's
// zero-dependency design.

// V2 adds the `tmpl:` shape kind. Writers emit the newest version; readers
// accept every version ever written, so a table saved by an older build keeps
// loading forever. Add to ACCEPTED, never remove from it.
const SKILLS_FORMAT_VERSION: &str = "XDPD_SKILLS_V2";
const SKILLS_FORMAT_ACCEPTED: &[&str] = &["XDPD_SKILLS_V1", "XDPD_SKILLS_V2"];

impl Slot {
    // Slot separator is ',' and the Run field separator is '|', so neither
    // collides with the ':' that splits shape kind from body, or the '\t' that
    // splits line fields.
    fn encode(&self) -> String {
        match self {
            Slot::Fixed(t) => format!("F{}", t),
            Slot::Var => "V".to_string(),
            Slot::Run { delta, len } => format!("R{}|{}", delta, len),
        }
    }

    fn decode(s: &str) -> Option<Slot> {
        if s == "V" {
            return Some(Slot::Var);
        }
        if let Some(rest) = s.strip_prefix('F') {
            return Some(Slot::Fixed(rest.parse().ok()?));
        }
        if let Some(rest) = s.strip_prefix('R') {
            let (delta, len) = rest.split_once('|')?;
            return Some(Slot::Run {
                delta: delta.parse().ok()?,
                len: len.parse().ok()?,
            });
        }
        None
    }
}

impl PatternShape {
    fn encode(&self) -> String {
        match self {
            PatternShape::Constant { len } => format!("const:{}", len),
            PatternShape::Arithmetic { delta, len } => format!("arith:{}:{}", delta, len),
            PatternShape::Repeat { unit_len, count } => format!("repeat:{}:{}", unit_len, count),
            PatternShape::Template(slots) => {
                let body: Vec<String> = slots.iter().map(Slot::encode).collect();
                format!("tmpl:{}", body.join(","))
            }
        }
    }

    fn decode(s: &str) -> Option<PatternShape> {
        let mut parts = s.split(':');
        match parts.next()? {
            "const" => Some(PatternShape::Constant {
                len: parts.next()?.parse().ok()?,
            }),
            "arith" => Some(PatternShape::Arithmetic {
                delta: parts.next()?.parse().ok()?,
                len: parts.next()?.parse().ok()?,
            }),
            "repeat" => Some(PatternShape::Repeat {
                unit_len: parts.next()?.parse().ok()?,
                count: parts.next()?.parse().ok()?,
            }),
            "tmpl" => {
                let body = parts.next()?;
                if body.is_empty() {
                    return None;
                }
                // Every slot must decode. A partial template would match
                // fewer tokens than it was saved with and corrupt output.
                let slots: Option<Vec<Slot>> = body.split(',').map(Slot::decode).collect();
                Some(PatternShape::Template(slots?))
            }
            _ => None,
        }
    }
}

impl Skill {
    fn encode_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.name,
            self.shape.encode(),
            self.strength,
            self.uses,
            self.signature
        )
    }

    fn decode_line(line: &str) -> Option<Skill> {
        let mut f = line.splitn(5, '\t');
        let name = f.next()?.to_string();
        let shape = PatternShape::decode(f.next()?)?;
        let strength = f.next()?.parse().ok()?;
        let uses = f.next()?.parse().ok()?;
        let signature = f.next()?.to_string();
        Some(Skill {
            name,
            shape,
            strength,
            uses,
            signature,
        })
    }
}

impl VM {
    /// Serialize the subroutine table to a writer.
    pub fn save_skills<W: std::io::Write>(&self, mut w: W) -> std::io::Result<()> {
        writeln!(w, "{}", SKILLS_FORMAT_VERSION)?;
        for skill in self.subroutines.values() {
            writeln!(w, "{}", skill.encode_line())?;
        }
        Ok(())
    }

    /// Save the subroutine table to a file.
    pub fn save_skills_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        self.save_skills(std::io::BufWriter::new(file))
    }

    /// Load skills from a reader, merging them into the current subroutine
    /// table (existing skills with the same name are overwritten). Returns
    /// the number of skills loaded. Lines that fail to parse are skipped.
    pub fn load_skills<R: std::io::Read>(&mut self, r: R) -> std::io::Result<usize> {
        use std::io::BufRead;
        let mut lines = std::io::BufReader::new(r).lines();

        match lines.next() {
            Some(Ok(header)) if SKILLS_FORMAT_ACCEPTED.contains(&header.as_str()) => {}
            Some(Ok(_)) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "unrecognized skills file format",
                ));
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(0),
        }

        let mut count = 0;
        for line in lines {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Some(skill) = Skill::decode_line(&line) {
                self.subroutines.insert(skill.name.clone(), skill);
                count += 1;
            }
        }
        Ok(count)
    }

    /// Load skills from a file, merging them into the current subroutine table.
    pub fn load_skills_from_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<usize> {
        let file = std::fs::File::open(path)?;
        self.load_skills(file)
    }
}

// ---------------------------------------------------------------------------
// Pattern detection
// ---------------------------------------------------------------------------

/// Detect invariants in a token sequence.
/// Returns `None` if no structural pattern is found.
pub fn detect_pattern(seq: &[Token]) -> Option<Pattern> {
    let n = seq.len();
    if n < 2 {
        return None;
    }

    // Constant: all tokens identical
    if seq.iter().all(|&t| t == seq[0]) {
        return Some(Pattern::Constant { value: seq[0], len: n });
    }

    // Arithmetic: constant difference between consecutive tokens
    if n >= 3 {
        if let Some(delta) = token_delta(seq[0], seq[1]) {
            if delta != 0 && seq.windows(2).all(|w| token_delta(w[0], w[1]) == Some(delta)) {
                return Some(Pattern::Arithmetic { start: seq[0], delta, len: n });
            }
        }
    }

    // Repeat: sequence divides evenly into identical chunks
    for unit_len in 1..=n / 2 {
        if n % unit_len == 0 {
            let unit = &seq[0..unit_len];
            if seq.chunks(unit_len).all(|chunk| chunk == unit) {
                return Some(Pattern::Repeat {
                    unit: unit.to_vec(),
                    count: n / unit_len,
                });
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Segmentation — finding patterns inside a longer stream
// ---------------------------------------------------------------------------

/// Find every maximal constant or arithmetic run in `seq` of at least
/// `min_run` tokens.
///
/// `detect_pattern` only answers "is this *whole* slice one invariant", so a
/// pattern surrounded by unrelated tokens is invisible to it and the caller has
/// to pre-cut the stream. This scans instead, so `[1,2,3, 91, 50,50,50,50]`
/// yields the ascending run and the constant run and ignores the noise between.
///
/// Runs are maximal and non-overlapping: the longest run at each position wins,
/// and scanning resumes after it. Without that, one long run would also emit
/// every shorter run inside itself.
pub fn scan_runs(seq: &[Token], min_run: usize) -> Vec<Pattern> {
    let mut out = Vec::new();
    let n = seq.len();
    if min_run < 2 || n < min_run {
        return out;
    }
    let mut i = 0;
    while i + min_run <= n {
        let delta = match token_delta(seq[i], seq[i + 1]) {
            Some(d) => d,
            // Difference too wide to store as a delta; cannot start a run here.
            None => {
                i += 1;
                continue;
            }
        };
        let mut j = i + 1;
        while j + 1 < n && token_delta(seq[j], seq[j + 1]) == Some(delta) {
            j += 1;
        }
        let len = j - i + 1;
        if len >= min_run {
            out.push(if delta == 0 {
                Pattern::Constant { value: seq[i], len }
            } else {
                Pattern::Arithmetic {
                    start: seq[i],
                    delta,
                    len,
                }
            });
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Learn a template by aligning two records position by position: where they
/// agree the value is structural (`Fixed`), where they differ it is data
/// (`Var`).
///
/// Two examples of a thing are enough to separate its skeleton from its
/// payload. This is how a real template gets discovered from real records, and
/// it makes no assumption about *where* the fixed parts sit — unlike parsers
/// that assume the leading tokens are constant.
///
/// **all-`Var` is rejected**: that is a wildcard matching *any* sequence of that
/// length. It would compress pure noise, which silently destroys the anomaly
/// signal — everything would look familiar.
///
/// An all-`Fixed` result *is* returned. It describes a record type that carries
/// no payload at all, which sounds degenerate but is extremely common in real
/// logs — measured on `Apache_2k`, two of six event types are wholly constant
/// messages. Rejecting those as "memorized literals" silently discarded whole
/// event types. Callers decide whether a literal earns a place:
/// `Learner::observe` keeps one only when no structural shape already describes
/// the sequence, so `[7,7,7,7]` still compiles a constant run rather than a
/// redundant four-slot literal. Use [`PatternShape::is_literal_template`].
pub fn align_template(a: &[Token], b: &[Token]) -> Option<PatternShape> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let slots: Vec<Slot> = a
        .iter()
        .zip(b)
        .map(|(&x, &y)| if x == y { Slot::Fixed(x) } else { Slot::Var })
        .collect();

    let fixed = slots.iter().filter(|s| matches!(s, Slot::Fixed(_))).count();
    if fixed == 0 {
        return None;
    }
    // ponytail: require at least half the slots fixed, so two unrelated records
    // that happen to share one position don't become a "template". Crude but
    // it holds the false-positive rate down; revisit against real data, where
    // genuine mostly-variable records may exist.
    if fixed * 2 < slots.len() {
        return None;
    }
    Some(PatternShape::Template(slots))
}

// ---------------------------------------------------------------------------
// Composition — Dynamic Programming over skills
// ---------------------------------------------------------------------------

/// A skill lookup key, chosen so it can be computed two ways that must agree:
/// from a stored shape, and from a candidate slice of the target. That is what
/// turns candidate selection into a hash probe instead of a table scan.
///
/// A key only has to be *necessary* for a match, never sufficient —
/// `PatternShape::matches` still verifies every candidate the probe returns.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MatchKey {
    Const(usize),
    Arith(usize, i32),
    Repeat(usize, usize),
    /// A template whose first slot is `Fixed`: span plus that token. The common
    /// case for real records, which usually start with something structural.
    TmplLead(usize, Token),
    /// A template whose first slot is not `Fixed`, so the leading token says
    /// nothing about it. These can only be narrowed by span.
    TmplVarLead(usize),
}

impl PatternShape {
    /// Whether this is a template with no variable slots at all: it matches
    /// exactly one token sequence. Useful for record types that carry no
    /// payload, redundant when a structural shape already covers the sequence.
    pub fn is_literal_template(&self) -> bool {
        matches!(self, PatternShape::Template(slots)
            if slots.iter().all(|s| matches!(s, Slot::Fixed(_))))
    }

    fn match_key(&self) -> MatchKey {
        match self {
            PatternShape::Constant { len } => MatchKey::Const(*len),
            PatternShape::Arithmetic { delta, len } => MatchKey::Arith(*len, *delta),
            PatternShape::Repeat { unit_len, count } => MatchKey::Repeat(*unit_len, *count),
            PatternShape::Template(slots) => match slots.first() {
                Some(Slot::Fixed(t)) => MatchKey::TmplLead(self.span_len(), *t),
                _ => MatchKey::TmplVarLead(self.span_len()),
            },
        }
    }
}

/// Fill `keys` with every key under which a skill could match this slice.
///
/// Writes into a caller-owned buffer rather than returning a fresh `Vec`: this
/// runs once per (position, span) pair, so allocating here dominated the cost
/// for small tables — it made a 100-skill compose slower than the unindexed
/// scan it replaced.
fn fill_probe_keys(
    keys: &mut Vec<MatchKey>,
    slice: &[Token],
    span: usize,
    divisors: &[usize],
    present: &Present,
) {
    keys.clear();
    if present.consts {
        keys.push(MatchKey::Const(span));
    }
    if present.ariths && span >= 2 {
        if let Some(delta) = token_delta(slice[0], slice[1]) {
            keys.push(MatchKey::Arith(span, delta));
        }
    }
    // Only divisors that some Repeat skill actually uses as a unit length.
    // Probing all of them was the single biggest cost in this loop.
    for &unit in divisors {
        if present.repeat_units.contains(&unit) {
            keys.push(MatchKey::Repeat(unit, span / unit));
        }
    }
    if present.tmpl_lead {
        keys.push(MatchKey::TmplLead(span, slice[0]));
    }
    if present.tmpl_varlead {
        keys.push(MatchKey::TmplVarLead(span));
    }
}

/// Which key families the skill table actually contains. Without this the probe
/// loop asks the index about kinds of skill that were never stored, which for a
/// small table costs more than the scan the index replaced.
#[derive(Default)]
struct Present {
    consts: bool,
    ariths: bool,
    repeat_units: HashSet<usize>,
    tmpl_lead: bool,
    tmpl_varlead: bool,
}

impl Present {
    fn note(&mut self, shape: &PatternShape) {
        match shape {
            PatternShape::Constant { .. } => self.consts = true,
            PatternShape::Arithmetic { .. } => self.ariths = true,
            PatternShape::Repeat { unit_len, .. } => {
                self.repeat_units.insert(*unit_len);
            }
            PatternShape::Template(slots) => match slots.first() {
                Some(Slot::Fixed(_)) => self.tmpl_lead = true,
                _ => self.tmpl_varlead = true,
            },
        }
    }
}

/// Compose a minimal-instruction program to produce the target sequence
/// using the available skills. Uses dynamic programming: for each position
/// in the target, it selects the cheapest option — either naive byte-by-byte
/// emission (2 ops per token) or calling a learned skill (1 Call + body).
pub fn compose(
    skills: &HashMap<String, Skill>,
    target: &[Token],
) -> (Vec<Instr>, u64) {
    let n = target.len();
    if n == 0 {
        return (vec![Instr::Ret], 0);
    }

    // Index every skill under a key that can be recomputed from a candidate
    // slice, so finding candidates is a hash probe rather than a scan of the
    // whole table. Indexing by span alone is not enough: thousands of skills
    // can share a span, and if none of them match, every one still gets tested.
    //
    // Entries are sorted by name so composition is deterministic instead of
    // dependent on HashMap iteration order.
    let mut entries: Vec<(&String, &Skill)> = skills.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    // Below this, scanning every skill beats building and probing an index.
    // Measured on a 512-token target: at 100 skills the scan is roughly twice
    // as fast, by 1000 the index is five times faster.
    const INDEX_THRESHOLD: usize = 256;
    let use_index = entries.len() >= INDEX_THRESHOLD;

    let mut index: HashMap<MatchKey, Vec<usize>> = HashMap::new();
    let mut spans: Vec<usize> = Vec::new();
    let mut present = Present::default();
    if use_index {
        for (idx, (_, skill)) in entries.iter().enumerate() {
            let span = skill.shape.span_len();
            if span == 0 || span > n {
                continue;
            }
            index.entry(skill.shape.match_key()).or_default().push(idx);
            present.note(&skill.shape);
            spans.push(span);
        }
    }
    spans.sort_unstable();
    spans.dedup();

    // Pair each span with its proper divisors, for probing Repeat shapes.
    // A Vec rather than a HashMap so the hot loop indexes instead of hashing.
    let spans: Vec<(usize, Vec<usize>)> = spans
        .into_iter()
        .map(|len| (len, (1..=len / 2).filter(|u| len % u == 0).collect()))
        .collect();
    let mut keys: Vec<MatchKey> = Vec::with_capacity(32);

    // `None` in the choice slot means the naive per-token step was taken.
    type Choice = Option<(usize, Option<(usize, Vec<Token>)>)>;
    let mut dp = vec![u64::MAX; n + 1];
    let mut choice: Vec<Choice> = vec![None; n + 1];
    dp[0] = 0;

    for i in 0..n {
        if dp[i] == u64::MAX {
            continue;
        }

        // Naive: emit one token (2 ops: Load + Output)
        if dp[i] + 2 < dp[i + 1] {
            dp[i + 1] = dp[i] + 2;
            choice[i + 1] = Some((i, None));
        }

        // Try skills by span. Every Call costs exactly 1, so once one skill of
        // a given span matches there is nothing to gain from testing the rest.
        if !use_index {
            // Few enough skills that testing them all is cheaper than building
            // and querying an index. The `dp` guard means the first skill to
            // cover a given span wins, matching the indexed path's behaviour.
            for (idx, (_, skill)) in entries.iter().enumerate() {
                let span = skill.shape.span_len();
                if span == 0 || i + span > n || dp[i] + 1 >= dp[i + span] {
                    continue;
                }
                if let Some(params) = skill.shape.matches(&target[i..i + span]) {
                    dp[i + span] = dp[i] + 1;
                    choice[i + span] = Some((i, Some((idx, params))));
                }
            }
            continue;
        }

        for (span, divisors) in &spans {
            let span = *span;
            if i + span > n {
                break; // spans are sorted, so nothing longer fits either
            }
            if dp[i] + 1 >= dp[i + span] {
                continue; // cannot improve; skip the match attempt entirely
            }
            let slice = &target[i..i + span];
            let mut hit = None;
            fill_probe_keys(&mut keys, slice, span, divisors, &present);
            'probe: for key in keys.iter() {
                if let Some(candidates) = index.get(key) {
                    for &idx in candidates {
                        if let Some(params) = entries[idx].1.shape.matches(slice) {
                            hit = Some((idx, params));
                            break 'probe;
                        }
                    }
                }
            }
            if let Some((idx, params)) = hit {
                dp[i + span] = dp[i] + 1;
                choice[i + span] = Some((i, Some((idx, params))));
            }
        }
    }

    // Backtrack. Steps are collected end-to-start and reversed at the end, so
    // the two instructions of a naive step are pushed in reverse order too —
    // pushing Load then Output here would emit `Output, Load` after the
    // reverse, which emits the register's previous value instead of the token.
    let mut prog = Vec::new();
    let mut pos = n;
    while pos > 0 {
        match &choice[pos] {
            Some((prev, None)) => {
                prog.push(Instr::Output);
                prog.push(Instr::Load(target[*prev]));
                pos = *prev;
            }
            Some((prev, Some((idx, params)))) => {
                prog.push(Instr::Call(entries[*idx].0.clone(), params.clone()));
                pos = *prev;
            }
            None => {
                prog.push(Instr::Output);
                prog.push(Instr::Load(target[pos - 1]));
                pos -= 1;
            }
        }
    }

    prog.reverse();
    prog.push(Instr::Ret);
    (prog, dp[n])
}

// ---------------------------------------------------------------------------
// Learner — the core learning engine
// ---------------------------------------------------------------------------

/// Configuration for the learner.
#[derive(Debug, Clone)]
pub struct LearnerConfig {
    /// Minimum observations before a repeat pattern is compiled into a skill.
    pub min_occurrences: u32,
    /// Maximum number of observations kept in the temporary window.
    pub window_size: usize,
}

impl Default for LearnerConfig {
    fn default() -> Self {
        LearnerConfig {
            min_occurrences: 3,
            window_size: 100,
        }
    }
}

/// The capability acquisition engine.
///
/// Observes token sequences, detects invariant patterns, compiles them into
/// subroutines, and uses DP composition to generate minimal programs.
/// Raw observations are consumed and discarded; only the subroutine table
/// persists as memory.
/// Upper bound on tokens buffered by `observe_token`/`observe_chunk` before an
/// automatic flush. A caller that never calls `flush()` would otherwise grow
/// this buffer without limit.
///
/// Deliberately a private const rather than a `LearnerConfig` field: that
/// struct has public fields and is built with struct-literal syntax by
/// downstream code (see `examples/gateway`), so adding a field to it would be
/// a breaking change.
const MAX_PENDING_TOKENS: usize = 4096;

// Forgetting. A skill table that only grows becomes slow and full of junk in a
// long-running process, so strength is now load-bearing: calling a skill
// reinforces it, time erodes it, and what nothing calls eventually goes.
//
// These are consts for the same reason as `MAX_PENDING_TOKENS`: `LearnerConfig`
// has public fields and downstream code builds it with struct-literal syntax,
// so growing it would be a breaking change.

/// Observations between decay ticks.
const DECAY_INTERVAL: u64 = 100;
/// Strength lost per decay tick.
const DECAY_AMOUNT: i32 = 1;
/// Strength gained each time a skill is actually called.
const STRENGTH_ON_USE: i32 = 2;
/// Ceiling, so a hot skill cannot become permanently immortal.
const STRENGTH_MAX: i32 = 100;
/// How many fixed positions one record may turn variable. Bounds how fast a
/// skeleton can generalize, which is what keeps a different record type from
/// dissolving it in a single step.

const WIDEN_NUM: usize = 1;
const WIDEN_DEN: usize = 4;
/// At or below this, a skill is forgotten.
const STRENGTH_FLOOR: i32 = 0;
/// Hard cap on table size. The weakest skills go first.
const MAX_SKILLS: usize = 4096;

/// How many recent raw records of a *given length* are kept for alignment.
/// Each new record is aligned only against records of its own length, so this
/// bounds per-observation work.
const RECENT_RECORDS: usize = 16;

/// Hard cap on raw records held for alignment across all lengths. Bounds memory
/// independently of how many distinct record lengths a stream contains.
const RECENT_TOTAL: usize = 256;

/// How much of a record a skeleton must already pin down before the record is
/// treated as another instance of that type, as a fraction of record length.
/// Too high and one record type shatters into several skeletons; too low and
/// unrelated types collapse into one that matches everything. Expressed as a
/// ratio to keep the comparison in integers.
const TEMPLATE_SIM_NUM: usize = 2;
const TEMPLATE_SIM_DEN: usize = 5;

pub struct Learner {
    vm: VM,
    /// Signatures contributed by the last `window_size` observations, oldest
    /// first. `None` means that observation had no detectable pattern.
    ///
    /// Note what this does *not* hold: raw tokens. The window used to keep a
    /// copy of every observed sequence purely to recount frequencies. Only the
    /// signature is needed for that, so the raw tokens are now dropped as soon
    /// as they are examined — which is what the design claimed all along.
    window: VecDeque<Vec<String>>,
    /// Live occurrence count per signature across the window, alongside the
    /// shape needed to compile it. Maintained incrementally (+1 on push,
    /// -1 on evict) instead of recounting the whole window on every call.
    freq: HashMap<String, (PatternShape, u32)>,
    learned_signatures: HashSet<String>,
    /// Tokens accumulated by the streaming API, awaiting `flush()`.
    pending: Vec<Token>,
    /// The last few raw records, kept only to align new records against,
    /// bucketed by record length.
    ///
    /// Alignment only ever succeeds between records of equal length, so a flat
    /// FIFO wastes its whole budget whenever record types interleave: a record
    /// can be pushed out by sixteen unrelated ones before its own kind comes
    /// round again, and the two never meet. Keeping a short queue per length
    /// means two records of the same shape align however far apart they arrive.
    ///
    /// Working state with a hard cap, not storage: `recent_order` bounds the
    /// total across all buckets, so nothing accumulates and the skill table is
    /// still the only thing that persists.
    recent: HashMap<usize, VecDeque<Vec<Token>>>,
    /// Lengths of the buckets holding each stored record, oldest first — the
    /// eviction order that keeps `recent` bounded to `RECENT_TOTAL`.
    recent_order: VecDeque<usize>,
    /// Observations seen, used only to pace decay ticks.
    ticks: u64,
    config: LearnerConfig,
}

impl Learner {
    /// Create a new learner with default configuration.
    pub fn new() -> Self {
        Self::with_config(LearnerConfig::default())
    }

    /// Create a learner with custom configuration.
    pub fn with_config(config: LearnerConfig) -> Self {
        Learner {
            vm: VM::new(),
            window: VecDeque::new(),
            freq: HashMap::new(),
            learned_signatures: HashSet::new(),
            pending: Vec::new(),
            recent: HashMap::new(),
            recent_order: VecDeque::new(),
            ticks: 0,
            config,
        }
    }

    /// Forget a skill completely.
    ///
    /// The subroutine table is not the only bookkeeping involved.
    /// `learned_signatures` records "I have already compiled this shape" and
    /// exists to stop duplicate skills — so dropping a skill while leaving its
    /// signature behind makes that shape **permanently unlearnable**: the
    /// learner would keep believing it already knows it and never recompile it.
    /// `freq` has to go too, otherwise a still-high window count would
    /// resurrect the skill on the very next matching observation and eviction
    /// would achieve nothing. All three are pruned together, always.
    pub fn forget_skill(&mut self, name: &str) -> bool {
        match self.vm.remove_skill(name) {
            Some(skill) => {
                self.learned_signatures.remove(&skill.signature);
                self.freq.remove(&skill.signature);
                true
            }
            None => false,
        }
    }

    /// Erode every skill's strength and forget whatever falls to the floor.
    /// Called automatically every `DECAY_INTERVAL` observations.
    fn decay(&mut self) {
        let mut doomed = Vec::new();
        for (name, skill) in self.vm.subroutines_mut() {
            skill.strength -= DECAY_AMOUNT;
            if skill.strength <= STRENGTH_FLOOR {
                doomed.push(name.clone());
            }
        }
        for name in doomed {
            self.forget_skill(&name);
        }
    }

    /// Enforce the hard table cap, weakest first.
    fn enforce_cap(&mut self) {
        if self.vm.skill_count() <= MAX_SKILLS {
            return;
        }
        let mut ranked: Vec<(i32, String)> = self
            .vm
            .subroutines()
            .values()
            .map(|s| (s.strength, s.name.clone()))
            .collect();
        ranked.sort();
        let excess = self.vm.skill_count() - MAX_SKILLS;
        for (_, name) in ranked.into_iter().take(excess) {
            self.forget_skill(&name);
        }
    }

    /// Observe a token sequence. The system looks for patterns in its
    /// temporary observation window. If a pattern repeats enough times,
    /// it is compiled into a permanent subroutine. The raw sequence
    /// is kept only in the window, then discarded.
    ///
    /// Returns names of any newly learned skills.
    /// Fit a record into the skeleton that already describes its type, widening
    /// that skeleton wherever the record disagrees with it.
    ///
    /// Returns whether a home was found. `false` means no learned template is
    /// close enough and the record has to seed a new one by alignment.
    ///
    /// Judging the record against the *template* is what makes this converge,
    /// and it is the difference between competing with Drain and not. Comparing
    /// two templates to each other — the obvious move — measures agreement that
    /// shrinks as they widen, so the moment a skeleton has generalized enough to
    /// be useful it stops accepting anything and its record type shatters across
    /// several skeletons. Grouping accuracy scores every one of those zero.
    fn absorb_record(&mut self, seq: &[Token]) -> bool {
        let mut best: Option<(String, usize, Vec<Slot>)> = None;
        for skill in self.vm.subroutines().values() {
            let PatternShape::Template(slots) = &skill.shape else {
                continue;
            };
            // Run slots carry a span beyond one token; widening one would change
            // the shape's length, so leave those templates to exact matching.
            if slots.len() != seq.len() || slots.iter().any(|s| s.span() != 1) {
                continue;
            }
            let hits = slots
                .iter()
                .zip(seq)
                .filter(|(s, &t)| matches!(s, Slot::Fixed(v) if *v == t))
                .count();
            if hits * TEMPLATE_SIM_DEN < seq.len() * TEMPLATE_SIM_NUM {
                continue;
            }
            // Widening is bounded per record. Another instance of the same type
            // disagrees with its skeleton in a *few* places — one field that
            // turned out to vary. A record that contradicts many fixed
            // positions at once is a different type that happens to share a
            // prefix, and letting it in is expensive out of all proportion:
            // grouping accuracy demands exact set equality, so a single
            // contaminating line scores a four-hundred-line cluster zero.
            //
            // `Received disconnect from IP: 11: Bye Bye [preauth]` and
            // `Received disconnect from IP: 11: disconnected by user` are the
            // real case — same length, same first five tokens, different events.
            // Similarity alone cannot separate them; how much the skeleton has
            // to give up to accept the record can.
            let contradictions = slots
                .iter()
                .zip(seq)
                .filter(|(s, &t)| matches!(s, Slot::Fixed(v) if *v != t))
                .count();
            if contradictions * WIDEN_DEN > seq.len() * WIDEN_NUM {
                continue;
            }
            if best.as_ref().is_some_and(|b| hits <= b.1) {
                continue;
            }
            let merged = slots
                .iter()
                .zip(seq)
                .map(|(s, &t)| match s {
                    Slot::Fixed(v) if *v == t => Slot::Fixed(*v),
                    Slot::Fixed(_) => Slot::Var,
                    other => other.clone(),
                })
                .collect();
            best = Some((skill.name.clone(), hits, merged));
        }

        let Some((name, _, merged)) = best else {
            return false;
        };
        let Some(skill) = self.vm.subroutines_mut().get_mut(&name) else {
            return false;
        };
        // A skeleton that keeps explaining incoming records is in use, whether
        // or not anything has called it. Without this, decay kills every
        // template a long ingest learns early: `observe` reinforces nothing and
        // ten decay ticks is all it takes to reach the floor.
        skill.strength = skill
            .strength
            .saturating_add(STRENGTH_ON_USE)
            .min(STRENGTH_MAX);
        if skill.shape == PatternShape::Template(merged.clone()) {
            return true;
        }

        let old_sig = skill.signature.clone();
        let wider = PatternShape::Template(merged);
        let sig = wider.encode();
        self.forget_skill(&name);
        // Deliberately keep the narrow signature marked as learned. `forget_skill`
        // clears it so a shape can be relearned, which is right when a skill
        // decayed away — but here it was replaced by something strictly more
        // general, and letting it come back would undo the widening every time.
        self.learned_signatures.insert(old_sig);
        if self.learned_signatures.insert(sig.clone()) {
            let mut skill = Skill::new(format!("skill_{}", sig), wider);
            skill.signature = sig;
            self.vm.add_skill(skill);
        }
        true
    }

    pub fn observe(&mut self, sequence: &[Token]) -> Vec<String> {
        if sequence.len() < 2 {
            return Vec::new();
        }

        // Evict the oldest observation, dropping its contribution to the counts.
        if self.window.len() >= self.config.window_size {
            if let Some(old_sigs) = self.window.pop_front() {
                for sig in old_sigs {
                    if let Some(entry) = self.freq.get_mut(&sig) {
                        entry.1 -= 1;
                        if entry.1 == 0 {
                            self.freq.remove(&sig);
                        }
                    }
                }
            }
        }

        // Everything this one observation contributes, keyed by signature so a
        // shape found by two different routes still counts once. Without the
        // dedupe, a sequence that is entirely one run would be counted twice —
        // by `detect_pattern` and again by `scan_runs` — and reach the
        // threshold in half the observations it should.
        let mut found: HashMap<String, PatternShape> = HashMap::new();

        // 1. Whole-sequence invariant. Also the only route that finds Repeat.
        if let Some(pattern) = detect_pattern(sequence) {
            found.insert(pattern.signature(), pattern.shape());
        }

        // 2. Runs buried anywhere inside the sequence.
        for pattern in scan_runs(sequence, 3) {
            found.insert(pattern.signature(), pattern.shape());
        }

        // 3. Templates. A record that an existing skeleton already describes
        // goes there, widening it; only a record no skeleton recognizes falls
        // through to alignment to seed a new one.
        let absorbed = self.absorb_record(sequence);
        let bucket = self.recent.entry(sequence.len()).or_default();
        let mut aligned = Vec::new();
        for prev in bucket.iter() {
            if let Some(shape) = align_template(prev, sequence) {
                // A literal earns its place only when nothing structural
                // already describes this sequence. Otherwise [7,7,7,7] would
                // compile a constant run *and* a redundant 4-slot literal.
                if shape.is_literal_template() && !found.is_empty() {
                    continue;
                }
                aligned.push(shape);
            }
        }

        if bucket.len() >= RECENT_RECORDS {
            bucket.pop_front();
        } else {
            self.recent_order.push_back(sequence.len());
        }
        self.recent
            .get_mut(&sequence.len())
            .expect("bucket just created")
            .push_back(sequence.to_vec());

        // Templates do not go through the frequency threshold. Alignment needs
        // two records before it can produce a template at all, so the evidence
        // a threshold exists to demand has already been supplied — while making
        // a template wait for its *signature* to repeat is the thing that stops
        // real record types ever being learned, because each pair of records
        // freezes a different set of coincidences and no two agree.
        let mut new_skills = Vec::new();
        // Alignment only ever *seeds* skeletons now. Letting it also widen the
        // ones already learned is what lets a lone record of some other type
        // dissolve a big skeleton: `absorb_record` refuses it for contradicting
        // too much, then alignment pairs it with a neighbour of the type it is
        // contaminating and merges the result straight back in through the side
        // door. Records join skeletons in exactly one place, under one rule.
        for shape in aligned.into_iter().take(if absorbed { 0 } else { usize::MAX }) {
            let sig = shape.encode();
            if self.learned_signatures.insert(sig.clone()) {
                let name = format!("skill_{}", sig);
                let mut skill = Skill::new(name.clone(), shape);
                skill.signature = sig;
                self.vm.add_skill(skill);
                new_skills.push(name);
            }
        }

        // Evict across buckets once the global budget is spent, so a stream of
        // many distinct lengths cannot grow this without bound.
        while self.recent_order.len() > RECENT_TOTAL {
            if let Some(len) = self.recent_order.pop_front() {
                if let Some(b) = self.recent.get_mut(&len) {
                    b.pop_front();
                    if b.is_empty() {
                        self.recent.remove(&len);
                    }
                }
            }
        }

        // Count them, and compile any that just crossed the threshold. Only
        // signatures touched here can have crossed it — eviction only lowers
        // counts — so there is no reason to re-scan the whole table.
        let mut sigs = Vec::with_capacity(found.len());
        for (sig, shape) in found {
            let entry = self.freq.entry(sig.clone()).or_insert((shape, 0));
            entry.1 += 1;
            let crossed = entry.1 >= self.config.min_occurrences;
            if crossed && !self.learned_signatures.contains(&sig) {
                let name = format!("skill_{}", sig);
                let mut skill = Skill::new(name.clone(), entry.0.clone());
                skill.signature = sig.clone();
                self.vm.add_skill(skill);
                self.learned_signatures.insert(sig.clone());
                new_skills.push(name);
            }
            sigs.push(sig);
        }
        self.window.push_back(sigs);

        self.ticks += 1;
        if self.ticks % DECAY_INTERVAL == 0 {
            self.decay();
        }
        self.enforce_cap();

        new_skills
    }

    /// Feed a single token into the stream.
    ///
    /// Tokens accumulate in a pending buffer until `flush()` marks the end of a
    /// record. That boundary has to be explicit: a stream carries no signal for
    /// where one observation stops and the next begins, and detecting patterns
    /// on every prefix would compile a separate skill for every length the
    /// record passed through on its way to being complete.
    ///
    /// Normally returns an empty list. If the pending buffer reaches
    /// `MAX_PENDING_TOKENS` it flushes automatically and returns whatever that
    /// produced, so a caller that never flushes cannot leak memory.
    pub fn observe_token(&mut self, t: Token) -> Vec<String> {
        self.pending.push(t);
        if self.pending.len() >= MAX_PENDING_TOKENS {
            return self.flush();
        }
        Vec::new()
    }

    /// Feed several tokens at once. Same buffering rules as `observe_token`.
    pub fn observe_chunk(&mut self, ts: &[Token]) -> Vec<String> {
        self.pending.extend_from_slice(ts);
        if self.pending.len() >= MAX_PENDING_TOKENS {
            return self.flush();
        }
        Vec::new()
    }

    /// Mark the end of a record: treat everything buffered as one complete
    /// observation. Feeding a sequence token by token and then flushing is
    /// equivalent to passing that whole sequence to `observe()`.
    pub fn flush(&mut self) -> Vec<String> {
        if self.pending.is_empty() {
            return Vec::new();
        }
        let seq = std::mem::take(&mut self.pending);
        self.observe(&seq)
    }

    /// Number of tokens buffered and not yet flushed.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Returns (output_tokens, program_level_instruction_count).
    /// Program level: 1 Call = 1 instruction regardless of body size.
    pub fn generate(&mut self, target: &[Token], use_learned: bool) -> (Vec<Token>, u64) {
        self.vm.reset();

        if use_learned && !self.vm.subroutines().is_empty() {
            let (prog, dp_cost) = compose(self.vm.subroutines(), target);
            self.vm.load_program(prog);
            self.vm.run();
            return (self.vm.output().to_vec(), dp_cost);
        }

        let mut prog: Vec<Instr> = target
            .iter()
            .flat_map(|&t| vec![Instr::Load(t), Instr::Output])
            .collect();
        prog.push(Instr::Ret);
        self.vm.load_program(prog);
        self.vm.run();
        (self.vm.output().to_vec(), self.vm.instruction_count())
    }

    /// Check whether a sequence matches any learned pattern.
    /// Returns a compression ratio. Values near 1.0 indicate the sequence
    /// does not match learned patterns — useful for anomaly detection.
    pub fn check_anomaly(&mut self, sequence: &[Token]) -> f64 {
        let (_, naive) = self.generate(sequence, false);
        let (_, learned) = self.generate(sequence, true);
        if learned == 0 {
            return 1.0;
        }
        naive as f64 / learned as f64
    }

    /// Return the number of learned skills.
    pub fn skill_count(&self) -> usize {
        self.vm.skill_count()
    }

    /// Return all learned skills, sorted by strength.
    pub fn skills(&self) -> Vec<&Skill> {
        self.vm.skills()
    }

    /// Access the underlying VM (for advanced use).
    pub fn vm(&self) -> &VM {
        &self.vm
    }

    /// Mutable access to the VM.
    pub fn vm_mut(&mut self) -> &mut VM {
        &mut self.vm
    }

    /// Persist the subroutine table to a file — the only state that needs
    /// to survive a restart. The observation window and config are not
    /// persisted; they're working state, not learned capability.
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.vm.save_skills_to_file(path)
    }

    /// Load a previously saved subroutine table, merging it into whatever
    /// this learner already knows. Returns the number of skills loaded.
    /// `learned_signatures` is rebuilt from the loaded skills so they won't
    /// be recompiled from scratch on the next matching observation.
    pub fn load_from_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<usize> {
        let count = self.vm.load_skills_from_file(path)?;
        let sigs: Vec<String> = self.vm.skills().into_iter().map(|s| s.signature.clone()).collect();
        self.learned_signatures.extend(sigs);
        Ok(count)
    }
}

impl Default for Learner {
    fn default() -> Self {
        Self::new()
    }
}

// Needed for the learned_signatures field
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_basic_execution() {
        let mut vm = VM::new();
        let prog = vec![
            Instr::Load(42),
            Instr::Output,
            Instr::Load(7),
            Instr::Output,
            Instr::Ret,
        ];
        vm.load_program(prog);
        vm.run();
        assert_eq!(vm.output(), &[42, 7]);
    }

    #[test]
    fn vm_seq_instruction() {
        let mut vm = VM::new();
        let prog = vec![Instr::Seq(10, 5, 4), Instr::Ret];
        vm.load_program(prog);
        vm.run();
        assert_eq!(vm.output(), &[10, 15, 20, 25]);
    }

    #[test]
    fn vm_call_subroutine() {
        let mut vm = VM::new();
        let skill = Skill::new("double".into(), PatternShape::Constant { len: 1 });
        vm.add_skill(skill);
        let prog = vec![Instr::Call("double".into(), vec![100]), Instr::Ret];
        vm.load_program(prog);
        vm.run();
        assert_eq!(vm.output(), &[100]);
    }

    #[test]
    fn skill_generalizes_across_values() {
        // Same shape (delta=2, len=5), three different value ranges. The
        // learner should compile exactly one skill after the third
        // occurrence, and that skill should apply to all three — and to
        // any other sequence with the same shape, not just the one it was
        // first observed from.
        let mut learner = Learner::new();
        let sequences = [
            vec![0, 2, 4, 6, 8],
            vec![100, 102, 104, 106, 108],
            vec![9000, 9002, 9004, 9006, 9008],
        ];
        for seq in &sequences {
            learner.observe(seq);
        }
        assert_eq!(learner.skill_count(), 1);

        for seq in &sequences {
            let (out, cost) = learner.generate(seq, true);
            assert_eq!(&out, seq);
            assert_eq!(cost, 1, "expected a single Call instruction for {:?}", seq);
        }

        // A same-shape sequence never observed before should also hit the skill.
        let unseen = vec![50, 52, 54, 56, 58];
        let (out, cost) = learner.generate(&unseen, true);
        assert_eq!(out, unseen);
        assert_eq!(cost, 1);
    }

    #[test]
    fn constant_skill_dedupes_across_values() {
        // Same shape (len=4), three different constant values. Constant's
        // dedup signature must ignore the value like Arithmetic/Repeat do,
        // or every new constant value would compile a redundant duplicate
        // skill even though PatternShape::Constant already matches any value.
        let mut learner = Learner::new();
        for value in [7u32, 200, 9999] {
            for _ in 0..5 {
                learner.observe(&vec![value; 4]);
            }
        }
        assert_eq!(learner.skill_count(), 1);

        let (out, cost) = learner.generate(&vec![42, 42, 42, 42], true);
        assert_eq!(out, vec![42, 42, 42, 42]);
        assert_eq!(cost, 1);
    }

    #[test]
    fn skills_round_trip_through_memory_buffer() {
        let mut learner = Learner::new();
        let seq = vec![0, 2, 4, 6, 8];
        for _ in 0..5 {
            learner.observe(&seq);
        }
        assert_eq!(learner.skill_count(), 1);

        let mut buf = Vec::new();
        learner.vm().save_skills(&mut buf).unwrap();

        // Simulates a process restart: a brand new VM with no skills.
        let mut fresh_vm = VM::new();
        assert_eq!(fresh_vm.skill_count(), 0);
        let loaded = fresh_vm.load_skills(buf.as_slice()).unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(fresh_vm.skill_count(), 1);

        // The restored skill still generalizes across values, not just the
        // exact sequence it was saved from.
        let unseen = vec![100, 102, 104, 106, 108];
        let (prog, cost) = compose(fresh_vm.subroutines(), &unseen);
        fresh_vm.load_program(prog);
        fresh_vm.run();
        assert_eq!(fresh_vm.output(), unseen.as_slice());
        assert_eq!(cost, 1);
    }

    #[test]
    fn skills_round_trip_through_file_survives_restart() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("xdpd_test_skills_{:?}.tsv", std::thread::current().id()));

        let mut learner = Learner::new();
        for _ in 0..5 {
            learner.observe(&vec![1, 0, 1, 0]);
        }
        assert_eq!(learner.skill_count(), 1);
        learner.save_to_file(&path).unwrap();

        // A fresh Learner stands in for a new process after restart.
        let mut restarted = Learner::new();
        assert_eq!(restarted.skill_count(), 0);
        let loaded = restarted.load_from_file(&path).unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(restarted.skill_count(), 1);

        let (out, cost) = restarted.generate(&vec![1, 0, 1, 0], true);
        assert_eq!(out, vec![1, 0, 1, 0]);
        assert_eq!(cost, 1);

        // Loaded skills' signatures are known, so re-observing the same
        // shape must not compile a duplicate skill.
        for _ in 0..5 {
            restarted.observe(&vec![1, 0, 1, 0]);
        }
        assert_eq!(restarted.skill_count(), 1);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_skills_rejects_unrecognized_format() {
        let mut vm = VM::new();
        let bogus = b"not a real header\nsome garbage\n";
        assert!(vm.load_skills(bogus.as_slice()).is_err());
    }

    #[test]
    fn detect_constant_pattern() {
        let seq = vec![5, 5, 5, 5];
        let p = detect_pattern(&seq).unwrap();
        assert!(matches!(p, Pattern::Constant { value: 5, len: 4 }));
    }

    #[test]
    fn detect_arithmetic_pattern() {
        let seq = vec![0, 2, 4, 6, 8];
        let p = detect_pattern(&seq).unwrap();
        assert!(matches!(p, Pattern::Arithmetic { start: 0, delta: 2, len: 5 }));
    }

    #[test]
    fn detect_repeat_pattern() {
        let seq = vec![1, 0, 1, 0, 1, 0];
        let p = detect_pattern(&seq).unwrap();
        assert!(matches!(p, Pattern::Repeat { .. }));
        assert_eq!(p.generate(), vec![1, 0, 1, 0, 1, 0]);
    }

    #[test]
    fn no_pattern_in_varied_sequence() {
        let seq = vec![1, 3, 5, 7, 11];
        assert!(detect_pattern(&seq).is_none());
    }

    #[test]
    fn pattern_generate_constant() {
        let p = Pattern::Constant { value: 9, len: 3 };
        assert_eq!(p.generate(), vec![9, 9, 9]);
    }

    #[test]
    fn pattern_generate_arithmetic() {
        let p = Pattern::Arithmetic { start: 10, delta: 10, len: 4 };
        assert_eq!(p.generate(), vec![10, 20, 30, 40]);
    }

    #[test]
    fn learner_learns_repeated_pattern() {
        let mut learner = Learner::new();
        let seq = vec![0, 1, 2];
        for _ in 0..5 {
            learner.observe(&seq);
        }
        assert!(learner.skill_count() > 0);
    }

    #[test]
    fn learner_shows_compression_speedup() {
        let mut learner = Learner::new();
        let seq = vec![0, 1, 2];
        for _ in 0..5 {
            learner.observe(&seq);
        }
        let target = vec![0, 1, 2, 0, 1, 2];
        let (_, naive) = learner.generate(&target, false);
        let (_, learned) = learner.generate(&target, true);
        let ratio = naive as f64 / learned as f64;
        // With learned patterns, we should need fewer instructions
        assert!(ratio >= 1.0, "expected speedup, got ratio {}", ratio);
    }

    #[test]
    fn anomaly_detection_low_ratio_for_unknown() {
        let mut learner = Learner::new();
        // Learn a constant baseline
        for _ in 0..5 {
            learner.observe(&vec![42, 42, 42, 42]);
        }
        // Known pattern should show compression
        let ratio_known = learner.check_anomaly(&vec![42, 42, 42, 42]);
        // Unknown pattern should show less compression
        let ratio_unknown = learner.check_anomaly(&vec![1, 3, 7, 15]);
        assert!(ratio_known >= ratio_unknown);
    }

    #[test]
    fn compose_uses_skills_when_available() {
        let mut learner = Learner::new();
        let seq = vec![5, 5, 5, 5, 5];
        for _ in 0..5 {
            learner.observe(&seq);
        }
        let target = vec![5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
        let (out, _) = learner.generate(&target, true);
        assert_eq!(out, target);
    }

    #[test]
    fn vm_missing_skill_reports_error() {
        let mut vm = VM::new();
        let prog = vec![Instr::Call("nonexistent".into(), vec![]), Instr::Ret];
        vm.load_program(prog);
        vm.run();
        assert!(vm.last_error().is_some());
        assert!(vm.last_error().unwrap().contains("skill not found"));
        assert_eq!(vm.output(), &[]);
    }

    #[test]
    fn vm_call_depth_is_balanced_and_bounded() {
        // Two halves. First: a normal call must leave call_depth back at 0 —
        // this is what proves the increment/decrement pair is correct, and it
        // is the half a preloaded counter can never check.
        let mut vm = VM::new();
        vm.add_skill(Skill::new(
            "s".into(),
            PatternShape::Arithmetic { delta: 2, len: 3 },
        ));
        vm.load_program(vec![Instr::Call("s".into(), vec![10]), Instr::Ret]);
        vm.run();
        assert_eq!(vm.output(), &[10, 12, 14]);
        assert_eq!(vm.call_depth, 0, "depth must unwind after a successful call");
        assert!(vm.last_error().is_none());

        // Second: at the ceiling, the guard refuses the call instead of
        // recursing. `call_depth` is set directly because `mod tests` is a
        // child module — no test-only public API needed on VM.
        //
        // ponytail: a genuine cycle cannot be built yet. No PatternShape emits
        // Instr::Call, so no skill body can call anything, so recursion depth
        // is structurally capped at 1 today. This guard is here for the
        // hierarchical skills in AGENTS.md Phase 4 — write the real cycle test
        // the moment a shape can emit a Call.
        vm.reset();
        vm.load_program(vec![Instr::Call("s".into(), vec![10]), Instr::Ret]);
        vm.call_depth = 64;
        vm.run();
        assert!(vm.last_error().unwrap().contains("call depth limit"));
        assert_eq!(vm.output(), &[], "a refused call must emit nothing");
    }

    #[test]
    fn failed_call_halts_execution_and_emits_nothing() {
        // A failed call must stop the program, not just record an error while
        // the following instructions keep appending output.
        //
        // Scope note: this covers the TOP-LEVEL call only. The sibling guard
        // for a failure inside a nested body (see Instr::Call in `step`) is
        // deliberately NOT covered — verified by deleting that guard and
        // watching this test still pass. It cannot be covered until a
        // PatternShape can emit an Instr::Call. Do not claim it is tested.
        let mut vm = VM::new();
        vm.load_program(vec![
            Instr::Call("missing".into(), vec![]),
            Instr::Load(5),
            Instr::Output,
            Instr::Ret,
        ]);
        vm.run();
        assert!(vm.last_error().unwrap().contains("skill not found"));
        assert_eq!(
            vm.output(),
            &[],
            "execution must halt at the failed call, not continue to Load(5)"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 2 — slotted templates
    // -----------------------------------------------------------------------

    /// Runs `shape` with `params` and returns what it emitted.
    fn run_shape(shape: &PatternShape, params: Vec<Token>) -> Vec<Token> {
        let mut vm = VM::new();
        vm.add_skill(Skill::new("t".into(), shape.clone()));
        vm.load_program(vec![Instr::Call("t".into(), params), Instr::Ret]);
        vm.run();
        assert!(vm.last_error().is_none(), "{:?}", vm.last_error());
        vm.output().to_vec()
    }

    #[test]
    fn template_match_is_lossless_across_varying_values() {
        // Three "log lines" as tokens: a fixed verb, a varying id, a fixed
        // status, a varying duration. One shape covers all three.
        let shape = PatternShape::Template(vec![
            Slot::Fixed(71),
            Slot::Var,
            Slot::Fixed(200),
            Slot::Var,
        ]);
        assert_eq!(shape.span_len(), 4);

        for input in [
            vec![71, 8814, 200, 34],
            vec![71, 2291, 200, 41],
            vec![71, 9007, 200, 12],
            vec![71, 0, 200, 4294967295], // extremes still exact
        ] {
            let params = shape.matches(&input).expect("should match");
            assert_eq!(params, vec![input[1], input[3]], "captured in slot order");
            assert_eq!(
                run_shape(&shape, params),
                input,
                "THE LOSSLESS GUARANTEE: output must equal input byte-for-byte"
            );
        }
    }

    #[test]
    fn template_refuses_any_mismatch_with_no_near_misses() {
        let shape =
            PatternShape::Template(vec![Slot::Fixed(71), Slot::Var, Slot::Fixed(200)]);
        assert!(shape.matches(&[71, 8814, 200]).is_some());
        // Every one of these differs from the template. None may match.
        assert!(shape.matches(&[72, 8814, 200]).is_none(), "leading Fixed");
        assert!(shape.matches(&[71, 8814, 404]).is_none(), "trailing Fixed");
        assert!(shape.matches(&[71, 8814]).is_none(), "too short");
        assert!(shape.matches(&[71, 8814, 200, 5]).is_none(), "too long");
        assert!(shape.matches(&[]).is_none(), "empty");
        // A shape with no slots consumes no input and must never match.
        assert!(PatternShape::Template(vec![]).matches(&[]).is_none());
    }

    #[test]
    fn template_run_slot_verifies_step_and_captures_start() {
        // Fixed, then a counting run, then a Var. Param order is
        // [run start, var] — if `matches` and `to_instructions` disagreed on
        // ordering, the run would start at 77 and the tail would print 100.
        let shape = PatternShape::Template(vec![
            Slot::Fixed(9),
            Slot::Run { delta: 2, len: 4 },
            Slot::Var,
        ]);
        assert_eq!(shape.span_len(), 6);

        let input = vec![9, 100, 102, 104, 106, 77];
        let params = shape.matches(&input).expect("should match");
        assert_eq!(params, vec![100, 77], "run start first, then the var");
        assert_eq!(run_shape(&shape, params), input);

        // Wrong step size inside the run must be refused.
        assert!(shape.matches(&[9, 100, 103, 104, 106, 77]).is_none());
        // Negative deltas work too.
        let down = PatternShape::Template(vec![Slot::Run { delta: -1, len: 3 }, Slot::Var]);
        let input = vec![50, 49, 48, 7];
        assert_eq!(run_shape(&down, down.matches(&input).unwrap()), input);
    }

    #[test]
    fn compose_covers_repeated_log_lines_with_one_call_each() {
        // Two 4-token lines that share a shape. Naive cost would be 16 ops
        // (2 per token); with the template it should be 2 Calls.
        let mut vm = VM::new();
        vm.add_skill(Skill::new(
            "logline".into(),
            PatternShape::Template(vec![
                Slot::Fixed(71),
                Slot::Var,
                Slot::Fixed(200),
                Slot::Var,
            ]),
        ));
        let target = vec![71, 8814, 200, 34, 71, 2291, 200, 41];
        let (prog, cost) = compose(vm.subroutines(), &target);
        assert_eq!(cost, 2, "one Call per line, got program {:?}", prog);

        vm.load_program(prog);
        vm.run();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.output(), target.as_slice(), "must reproduce exactly");
    }

    #[test]
    fn template_survives_save_and_restart() {
        let shape = PatternShape::Template(vec![
            Slot::Fixed(71),
            Slot::Var,
            Slot::Run { delta: -1, len: 3 },
        ]);
        let mut vm = VM::new();
        vm.add_skill(Skill::new("logline".into(), shape.clone()));

        let mut buf = Vec::new();
        vm.save_skills(&mut buf).unwrap();
        assert!(
            String::from_utf8_lossy(&buf).starts_with("XDPD_SKILLS_V2"),
            "writer must emit the newest format version"
        );

        // Stand-in for a fresh process.
        let mut restarted = VM::new();
        assert_eq!(restarted.load_skills(buf.as_slice()).unwrap(), 1);
        assert_eq!(
            restarted.subroutines()["logline"].shape, shape,
            "shape must survive the round trip exactly"
        );

        let input = vec![71, 8814, 50, 49, 48];
        let params = shape.matches(&input).unwrap();
        let (prog, _) = compose(restarted.subroutines(), &input);
        restarted.load_program(prog);
        restarted.run();
        assert_eq!(restarted.output(), input.as_slice());
        assert_eq!(params, vec![8814, 50]);
    }

    #[test]
    fn v1_skills_files_still_load_after_v2_bump() {
        // Hand-written V1 file, exactly as an older build would have emitted.
        // This must keep working forever — see SKILLS_FORMAT_ACCEPTED.
        let v1 = "XDPD_SKILLS_V1\nskill_arith:d2x5\tarith:2:5\t10\t0\tarith:d2x5\n";
        let mut vm = VM::new();
        assert_eq!(vm.load_skills(v1.as_bytes()).unwrap(), 1);

        // And the old skill still generalizes to unseen values.
        let unseen = vec![100, 102, 104, 106, 108];
        let (prog, cost) = compose(vm.subroutines(), &unseen);
        assert_eq!(cost, 1);
        vm.load_program(prog);
        vm.run();
        assert_eq!(vm.output(), unseen.as_slice());
    }

    #[test]
    fn every_slot_kind_round_trips_through_its_encoding() {
        // Regression: the first version of Slot::decode split on the second
        // character, so the single-character "V" returned None and every
        // template containing a Var silently failed to load. Each kind gets
        // checked on its own here so a future encoding change can't repeat it.
        for slot in [
            Slot::Var,
            Slot::Fixed(0),
            Slot::Fixed(71),
            Slot::Fixed(4294967295),
            Slot::Run { delta: 2, len: 4 },
            Slot::Run { delta: -1, len: 3 },
            Slot::Run { delta: 0, len: 1 },
        ] {
            let encoded = slot.encode();
            assert_eq!(
                Slot::decode(&encoded).as_ref(),
                Some(&slot),
                "round trip failed for {:?} (encoded as {:?})",
                slot,
                encoded
            );
        }
    }

    #[test]
    fn malformed_template_encoding_is_rejected() {
        // A half-decodable template would span fewer tokens than it was saved
        // with and silently corrupt output, so any bad slot fails the line.
        for bad in [
            "XDPD_SKILLS_V2\nx\ttmpl:F1,ZZZ,F2\t10\t0\tsig\n",
            "XDPD_SKILLS_V2\nx\ttmpl:\t10\t0\tsig\n",
            "XDPD_SKILLS_V2\nx\ttmpl:R5\t10\t0\tsig\n", // Run missing |len
        ] {
            let mut vm = VM::new();
            assert_eq!(
                vm.load_skills(bad.as_bytes()).unwrap(),
                0,
                "should have skipped the bad line: {}",
                bad
            );
            assert_eq!(vm.skill_count(), 0);
        }
    }

    // -----------------------------------------------------------------------
    // Phase 3 — streaming ingestion
    // -----------------------------------------------------------------------

    #[test]
    fn streaming_one_token_at_a_time_matches_batch_observe() {
        // Same input, same result, different delivery.
        let records = [
            vec![0, 2, 4, 6, 8],
            vec![100, 102, 104, 106, 108],
            vec![7, 7, 7, 7],
            vec![50, 52, 54, 56, 58],
            vec![9, 9, 9, 9],
            vec![1, 1, 1, 1],
        ];

        let mut batch = Learner::new();
        for r in &records {
            batch.observe(r);
        }

        let mut streamed = Learner::new();
        for r in &records {
            for &t in r {
                streamed.observe_token(t);
            }
            streamed.flush(); // record boundary
        }

        assert_eq!(streamed.skill_count(), batch.skill_count());
        let mut a: Vec<_> = streamed.skills().iter().map(|s| s.name.clone()).collect();
        let mut b: Vec<_> = batch.skills().iter().map(|s| s.name.clone()).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "streaming must learn exactly the same skills");
    }

    #[test]
    fn observe_chunk_matches_token_by_token() {
        let seq = vec![3, 6, 9, 12];
        let mut by_token = Learner::new();
        let mut by_chunk = Learner::new();
        for _ in 0..4 {
            for &t in &seq {
                by_token.observe_token(t);
            }
            by_token.flush();
            by_chunk.observe_chunk(&seq);
            by_chunk.flush();
        }
        assert_eq!(by_chunk.skill_count(), by_token.skill_count());
        assert!(by_chunk.skill_count() > 0);
    }

    #[test]
    fn flush_is_required_to_end_a_record() {
        // Without a flush nothing is learned, because the record is not over.
        let mut learner = Learner::new();
        for _ in 0..10 {
            for &t in &[0, 2, 4, 6, 8] {
                learner.observe_token(t);
            }
        }
        assert_eq!(learner.skill_count(), 0, "no boundary means no observation");
        assert_eq!(learner.pending_len(), 50);
        learner.flush();
        assert_eq!(learner.pending_len(), 0);
    }

    #[test]
    fn pending_buffer_auto_flushes_and_cannot_grow_without_bound() {
        // A caller that never flushes must not leak. Feed well past the cap.
        let mut learner = Learner::new();
        for i in 0..(MAX_PENDING_TOKENS * 3) {
            learner.observe_token(i as Token);
        }
        assert!(
            learner.pending_len() < MAX_PENDING_TOKENS,
            "pending grew to {} without flushing",
            learner.pending_len()
        );
    }

    #[test]
    fn streams_100k_tokens_without_quadratic_blowup() {
        // The old window re-ran detect_pattern over every entry on every call
        // and memmoved the whole window on eviction, so cost grew with
        // window_size. A large window here would have been the worst case.
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 3,
            window_size: 1000,
        });
        let start = std::time::Instant::now();
        let mut n = 0u32;
        for i in 0..20_000u32 {
            // 5 tokens per record, 100k tokens total.
            for k in 0..5 {
                learner.observe_token(i.wrapping_mul(7).wrapping_add(k * 2));
                n += 1;
            }
            learner.flush();
        }
        let elapsed = start.elapsed();
        assert_eq!(n, 100_000);
        // Generous bound so this can never flake on a loaded machine or in a
        // debug build; the real figure is printed with --nocapture.
        assert!(
            elapsed.as_secs() < 20,
            "100k tokens took {:?}, expected far less",
            elapsed
        );
        println!("100k tokens in {:?} ({} skills)", elapsed, learner.skill_count());
    }

    #[test]
    fn window_evicts_and_decrements_counts_incrementally() {
        // A shape seen twice, then pushed out of a size-3 window by unrelated
        // observations, must not later reach the threshold from stale counts.
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 3,
            window_size: 3,
        });
        learner.observe(&[0, 2, 4]); // arith:d2x3, count 1
        learner.observe(&[0, 2, 4]); // count 2
        assert_eq!(learner.skill_count(), 0);

        // Three unrelated observations evict both of the above.
        learner.observe(&[5, 5, 5]);
        learner.observe(&[6, 6, 6]);
        learner.observe(&[7, 7, 7]);

        // The window no longer holds any arith:d2x3, so one more must not tip
        // it over the threshold — it should be counted as the first, not third.
        learner.observe(&[0, 2, 4]);
        assert!(
            !learner.skills().iter().any(|s| s.signature == "arith:d2x3"),
            "evicted observations must not still count toward the threshold"
        );
        // Sanity: the constants did repeat 3 times, so they did get learned.
        assert!(learner.skills().iter().any(|s| s.signature == "const:x3"));
    }

    // -----------------------------------------------------------------------
    // Phase 4 — finding patterns inside a stream
    // -----------------------------------------------------------------------

    #[test]
    fn scan_runs_finds_patterns_buried_in_noise() {
        // detect_pattern sees nothing here: the slice as a whole is not one
        // invariant. scan_runs finds both runs and ignores the noise.
        let seq = vec![1, 2, 3, 4, 91, 50, 50, 50, 50, 7, 33];
        assert!(detect_pattern(&seq).is_none(), "whole-slice detection fails");

        let runs = scan_runs(&seq, 3);
        assert_eq!(runs.len(), 2, "got {:?}", runs);
        assert_eq!(
            runs[0],
            Pattern::Arithmetic { start: 1, delta: 1, len: 4 }
        );
        assert_eq!(runs[1], Pattern::Constant { value: 50, len: 4 });
    }

    #[test]
    fn scan_runs_reports_maximal_non_overlapping_runs() {
        // One long run must yield exactly one Pattern, not every shorter run
        // nested inside it.
        let runs = scan_runs(&[0, 2, 4, 6, 8, 10], 3);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0], Pattern::Arithmetic { start: 0, delta: 2, len: 6 });

        // Too short to qualify.
        assert!(scan_runs(&[5, 6], 3).is_empty());
        assert!(scan_runs(&[1, 9, 4, 77], 3).is_empty());
    }

    #[test]
    fn learner_finds_a_pattern_buried_in_noise() {
        let mut learner = Learner::new();
        // Same constant run each time, surrounded by changing noise.
        for i in 0..5u32 {
            learner.observe(&[i * 977 + 1, 42, 42, 42, 42, i * 331 + 7]);
        }
        assert!(
            learner.skills().iter().any(|s| s.signature == "const:x4"),
            "learned: {:?}",
            learner.skills().iter().map(|s| &s.signature).collect::<Vec<_>>()
        );
    }

    #[test]
    fn alignment_separates_skeleton_from_payload() {
        let a = [71, 8814, 200, 34];
        let b = [71, 2291, 200, 41];
        let shape = align_template(&a, &b).expect("should align");
        assert_eq!(
            shape,
            PatternShape::Template(vec![
                Slot::Fixed(71),
                Slot::Var,
                Slot::Fixed(200),
                Slot::Var,
            ])
        );
        // And it reproduces both records exactly.
        for rec in [a, b] {
            let params = shape.matches(&rec).unwrap();
            assert_eq!(run_shape(&shape, params), rec.to_vec());
        }
    }

    #[test]
    fn alignment_rejects_wildcards_but_keeps_literals() {
        // Identical records -> all Fixed. Originally rejected as a "memorized
        // literal"; the loghub benchmark showed that threw away whole event
        // types (two of six on Apache_2k are wholly constant messages), so it
        // is now returned and the caller decides whether it earns a place.
        let literal = align_template(&[1, 2, 3], &[1, 2, 3]).expect("kept");
        assert!(literal.is_literal_template());
        assert!(!PatternShape::Constant { len: 3 }.is_literal_template());
        // Nothing in common -> all Var -> a wildcard that matches any 3 tokens.
        // Accepting this would compress noise and destroy the anomaly signal.
        assert!(align_template(&[1, 2, 3], &[9, 8, 7]).is_none());
        // Mostly variable -> too weak to trust as structure.
        assert!(align_template(&[1, 2, 3, 4], &[1, 9, 8, 7]).is_none());
        // Different lengths cannot be aligned positionally.
        assert!(align_template(&[1, 2, 3], &[1, 2]).is_none());
        assert!(align_template(&[], &[]).is_none());
    }

    #[test]
    fn a_template_widens_to_cover_every_position_that_ever_varies() {
        // Each *pair* of these agrees somewhere the type as a whole does not:
        // the first two share status 200, the last two share duration 12.
        // Pairwise alignment alone freezes those coincidences into a template
        // that matches almost nothing. The skeleton must widen instead.
        //
        // The trailing 88, 99 are the record type's own constant context. They
        // matter: widening only merges skeletons that still agree on at least
        // half their positions, so a record type carrying almost no fixed
        // context — three varying fields out of four — can still settle into
        // two skeletons rather than one. That guard is what stops unrelated
        // types collapsing together, and it is the known limit of this method.
        let records = [
            vec![71, 8814, 200, 34, 88, 99],
            vec![71, 2291, 200, 41, 88, 99],
            vec![71, 9007, 404, 12, 88, 99],
            vec![71, 5150, 500, 12, 88, 99],
        ];
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 2,
            window_size: 100,
        });
        for r in &records {
            learner.observe(r);
        }

        let templates: Vec<_> = learner
            .skills()
            .into_iter()
            .filter(|s| matches!(s.shape, PatternShape::Template(_)))
            .collect();
        assert_eq!(
            templates.len(),
            1,
            "one record type must yield one skeleton, got {:?}",
            templates.iter().map(|s| &s.shape).collect::<Vec<_>>()
        );

        // The surviving skeleton covers all four, and still reproduces each
        // exactly — widening must never cost losslessness.
        let t = &templates[0];
        for r in &records {
            let params = t.shape.matches(r).expect("widened template should match");
            assert_eq!(run_shape(&t.shape, params), *r);
        }
        // ...and it did not widen into a shape that matches anything at all.
        assert!(t.shape.matches(&[99, 1, 2, 3]).is_none());
    }

    #[test]
    fn a_template_that_keeps_matching_survives_a_long_ingest() {
        // Decay runs on a timer, and `observe` alone used to reinforce nothing,
        // so a skeleton learned early died mid-stream even while records of its
        // own type kept arriving — the table forgot exactly what it was seeing.
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 2,
            window_size: 100,
        });
        let ticks_to_starve = (10 / DECAY_AMOUNT as u64 + 2) * DECAY_INTERVAL;
        for i in 0..ticks_to_starve {
            learner.observe(&[71, 1000 + i as Token, 200, 34]);
        }

        let survivor = learner
            .skills()
            .into_iter()
            .find(|s| matches!(s.shape, PatternShape::Template(_)))
            .expect("a template matched on every observation must not decay away");
        assert!(survivor.shape.matches(&[71, 4242, 200, 34]).is_some());
    }

    #[test]
    fn learner_learns_a_template_from_realistic_records() {
        // Three "log lines" as tokens: fixed verb, varying id, fixed status,
        // varying duration. One template must cover all three.
        let records = [
            vec![71, 8814, 200, 34],
            vec![71, 2291, 200, 41],
            vec![71, 9007, 200, 12],
        ];
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 2,
            window_size: 100,
        });
        for r in &records {
            learner.observe(r);
        }

        let template = learner
            .skills()
            .into_iter()
            .find(|s| matches!(s.shape, PatternShape::Template(_)))
            .expect("a template should have been learned");

        // Every record — including ones it was not aligned from — reproduces
        // exactly through the learned template.
        for r in &records {
            let params = template.shape.matches(r).expect("template should match");
            assert_eq!(run_shape(&template.shape, params), *r);
        }

        // And an unseen record with the same skeleton also matches.
        let unseen = vec![71, 55555, 200, 99];
        let params = template.shape.matches(&unseen).unwrap();
        assert_eq!(run_shape(&template.shape, params), unseen);

        // A record with a different skeleton must not match.
        assert!(template.shape.matches(&[72, 8814, 404, 34]).is_none());
    }

    #[test]
    fn wide_token_values_do_not_overflow() {
        // Regression: delta was computed as `b as i32 - a as i32`, which
        // reinterprets large u32 tokens as negative and then overflows the
        // subtraction — a debug-build panic on ordinary data like hashed ids.
        // Every one of these would have crashed.
        let wide = vec![4_000_000_000, 100, 4_294_967_295, 7, 2_147_483_648];
        assert!(detect_pattern(&wide).is_none());
        scan_runs(&wide, 3);
        align_template(&wide, &[4_000_000_000, 1, 4_294_967_295, 2, 9]);

        let mut learner = Learner::new();
        for _ in 0..5 {
            learner.observe(&wide);
        }
        learner.check_anomaly(&wide);

        // A run near the top of the range must still round-trip exactly.
        let shape = PatternShape::Arithmetic { delta: 1, len: 3 };
        let near_max = vec![4_294_967_293, 4_294_967_294, 4_294_967_295];
        let params = shape.matches(&near_max).expect("should match");
        assert_eq!(run_shape(&shape, params), near_max);

        // And a delta too wide for i32 is simply not a run, rather than a panic.
        assert_eq!(token_delta(0, 4_000_000_000), None);
        assert_eq!(token_delta(0, 5), Some(5));
        assert_eq!(token_delta(5, 0), Some(-5));
    }

    #[test]
    fn pure_noise_learns_nothing() {
        // False patterns are worse than no patterns. Deterministic LCG so this
        // test can never flake.
        let mut learner = Learner::new();
        let mut x: u32 = 12345;
        for _ in 0..200 {
            let rec: Vec<Token> = (0..6)
                .map(|_| {
                    x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                    x
                })
                .collect();
            learner.observe(&rec);
        }
        assert_eq!(
            learner.skill_count(),
            0,
            "learned from noise: {:?}",
            learner.skills().iter().map(|s| &s.signature).collect::<Vec<_>>()
        );
    }

    // -----------------------------------------------------------------------
    // Phase 5 — forgetting
    // -----------------------------------------------------------------------

    #[test]
    fn calling_a_skill_records_the_use_and_reinforces_it() {
        let mut learner = Learner::new();
        let seq = vec![0, 2, 4, 6, 8];
        for _ in 0..3 {
            learner.observe(&seq);
        }
        let before = learner.skills()[0].clone();
        assert_eq!(before.uses, 0, "unused to begin with");

        learner.generate(&seq, true);
        let after = learner.skills()[0].clone();
        assert_eq!(after.uses, 1, "a call must be recorded");
        assert!(
            after.strength > before.strength,
            "use must reinforce: {} -> {}",
            before.strength,
            after.strength
        );

        // Strength is capped so a hot skill cannot become immortal.
        for _ in 0..200 {
            learner.generate(&seq, true);
        }
        assert!(learner.skills()[0].strength <= STRENGTH_MAX);
    }

    #[test]
    fn unused_skills_decay_and_are_forgotten() {
        let mut learner = Learner::new();
        for _ in 0..3 {
            learner.observe(&[0, 2, 4, 6, 8]);
        }
        assert_eq!(learner.skill_count(), 1);

        // Observe unrelated noise long enough for decay ticks to erode it.
        // Nothing calls the skill, so nothing reinforces it.
        let mut x: u32 = 999;
        let ticks_needed = (10 / DECAY_AMOUNT) as u64 + 1;
        for _ in 0..(DECAY_INTERVAL * ticks_needed + 10) {
            let rec: Vec<Token> = (0..6)
                .map(|_| {
                    x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                    x
                })
                .collect();
            learner.observe(&rec);
        }
        assert_eq!(
            learner.skill_count(),
            0,
            "an unused skill should eventually be forgotten"
        );
    }

    #[test]
    fn forgotten_shapes_can_be_learned_again() {
        // THE TRAP. `learned_signatures` means "already compiled this shape".
        // Evicting a skill without pruning it there makes the shape
        // permanently unlearnable — the learner keeps thinking it knows it.
        let mut learner = Learner::new();
        let seq = vec![0, 2, 4, 6, 8];
        for _ in 0..3 {
            learner.observe(&seq);
        }
        assert_eq!(learner.skill_count(), 1);
        let name = learner.skills()[0].name.clone();

        assert!(learner.forget_skill(&name));
        assert_eq!(learner.skill_count(), 0);

        // Feed the same shape again. It must come back.
        for _ in 0..3 {
            learner.observe(&seq);
        }
        assert_eq!(
            learner.skill_count(),
            1,
            "a forgotten shape must be relearnable"
        );
        // And still work.
        let (out, cost) = learner.generate(&seq, true);
        assert_eq!(out, seq);
        assert_eq!(cost, 1);
    }

    #[test]
    fn forget_skill_reports_whether_anything_was_removed() {
        let mut learner = Learner::new();
        assert!(!learner.forget_skill("nonexistent"));
    }

    #[test]
    fn table_stays_bounded_under_a_long_varied_run() {
        // Many distinct shapes over a long run must not grow without bound.
        let mut learner = Learner::with_config(LearnerConfig {
            min_occurrences: 2,
            window_size: 200,
        });
        for len in 2..60usize {
            for delta in 1..40i32 {
                let rec: Vec<Token> =
                    (0..len).map(|k| (k as i32 * delta) as Token).collect();
                learner.observe(&rec);
                learner.observe(&rec);
            }
        }
        assert!(
            learner.skill_count() <= MAX_SKILLS,
            "table grew to {}",
            learner.skill_count()
        );
    }

    // -----------------------------------------------------------------------
    // Phase 6 — composition correctness and scaling
    // -----------------------------------------------------------------------

    /// Composes `target` against `skills`, runs it, and asserts the output is
    /// byte-identical to the target. Every compose result must satisfy this
    /// regardless of how much of the target the skills covered.
    fn assert_composes_exactly(skills: &HashMap<String, Skill>, target: &[Token]) -> u64 {
        let (prog, cost) = compose(skills, target);
        let mut vm = VM::new();
        for (name, skill) in skills {
            let mut s = skill.clone();
            s.name = name.clone();
            vm.add_skill(s);
        }
        vm.load_program(prog.clone());
        vm.run();
        assert!(vm.last_error().is_none(), "{:?}", vm.last_error());
        assert_eq!(
            vm.output(),
            target,
            "compose must reproduce the target exactly; program was {:?}",
            prog
        );
        cost
    }

    #[test]
    fn compose_reproduces_uncovered_targets_exactly() {
        // Regression: the naive per-token step pushed Load then Output and then
        // the whole program was reversed, so it executed as Output,Load and
        // emitted the register's stale value. `compose(&{}, &[7,8,9])` produced
        // [0,7,8]. Every prior compose test used a fully covered target, so
        // nothing caught it.
        let empty: HashMap<String, Skill> = HashMap::new();
        assert_eq!(assert_composes_exactly(&empty, &[7, 8, 9]), 6);
        assert_eq!(assert_composes_exactly(&empty, &[42]), 2);
        assert_composes_exactly(&empty, &[1, 3, 7, 15, 31]);
    }

    #[test]
    fn compose_reproduces_partially_covered_targets_exactly() {
        // Mixed coverage: some spans hit a skill, the rest fall back to naive.
        // This is the common real case and the one the bug corrupted.
        let mut skills = HashMap::new();
        skills.insert(
            "run5".to_string(),
            Skill::new("run5".into(), PatternShape::Arithmetic { delta: 2, len: 5 }),
        );

        // noise, then a matching run, then more noise.
        let target = vec![91, 77, 0, 2, 4, 6, 8, 55];
        let cost = assert_composes_exactly(&skills, &target);
        // 3 naive tokens (2 ops each) + 1 Call = 7
        assert_eq!(cost, 7, "expected the run to be covered by one Call");

        // Run at the very start and very end too.
        assert_composes_exactly(&skills, &[0, 2, 4, 6, 8, 99]);
        assert_composes_exactly(&skills, &[99, 0, 2, 4, 6, 8]);
    }

    #[test]
    fn compose_is_deterministic_across_runs() {
        // Skills are sorted by name before indexing, so the same inputs must
        // always produce the same program — HashMap order must not leak in.
        let mut skills = HashMap::new();
        for (name, shape) in [
            ("a", PatternShape::Constant { len: 3 }),
            ("b", PatternShape::Constant { len: 3 }),
            ("c", PatternShape::Arithmetic { delta: 1, len: 3 }),
        ] {
            skills.insert(name.to_string(), Skill::new(name.into(), shape));
        }
        let target = vec![5, 5, 5, 1, 2, 3, 8];
        let (first, cost) = compose(&skills, &target);
        for _ in 0..20 {
            let (again, c) = compose(&skills, &target);
            assert_eq!(again, first, "composition must be deterministic");
            assert_eq!(c, cost);
        }
        assert_composes_exactly(&skills, &target);
    }

    #[test]
    fn composition_stays_fast_as_the_table_grows() {
        // Before the span index, every position tested every skill. Timing is
        // printed rather than asserted tightly so this cannot flake; the shape
        // of the curve is the point.
        let target: Vec<Token> = (0..512).map(|i| (i * 3) as Token).collect();
        let mut timings = Vec::new();
        for count in [100usize, 1000, 10_000] {
            let mut skills = HashMap::new();
            for k in 0..count {
                // Distinct shapes, almost none of which can match.
                let name = format!("s{:06}", k);
                skills.insert(
                    name.clone(),
                    Skill::new(
                        name,
                        PatternShape::Arithmetic {
                            delta: (k % 977 + 7) as i32,
                            len: 3 + (k % 29),
                        },
                    ),
                );
            }
            let start = std::time::Instant::now();
            let (_, _) = compose(&skills, &target);
            let elapsed = start.elapsed();
            timings.push((count, elapsed));
            println!("{:>6} skills -> {:?}", count, elapsed);
        }
        // 100x more skills must not cost anywhere near 100x more time.
        let (_, small) = timings[0];
        let (_, large) = timings[2];
        assert!(
            large.as_nanos() < small.as_nanos().saturating_mul(25).max(1_000_000),
            "scaling looks linear in table size: {:?} -> {:?}",
            small,
            large
        );
    }

    #[test]
    fn check_anomaly_empty_sequence_returns_finite() {
        let mut learner = Learner::new();
        let ratio = learner.check_anomaly(&[]);
        assert!(ratio.is_finite());
        assert_eq!(ratio, 1.0);
    }
}

