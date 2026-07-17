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

// ---------------------------------------------------------------------------
// Token type
// ---------------------------------------------------------------------------

/// Unified token type for all discrete observations, actions, and symbols.
pub type Token = u32;

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
                let mut s = *start as i32;
                (0..*len).map(|_| { let v = s as Token; s += delta; v }).collect()
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

/// The structural template of a `Pattern`, with concrete values removed.
/// A `Skill` stores one of these instead of a frozen output sequence, so it
/// can recognize and reproduce *any* sequence with matching structure —
/// not just the specific instance it was compiled from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternShape {
    Constant { len: usize },
    Arithmetic { delta: i32, len: usize },
    Repeat { unit_len: usize, count: usize },
}

impl PatternShape {
    /// Number of tokens a match against this shape spans.
    pub fn span_len(&self) -> usize {
        match self {
            PatternShape::Constant { len } => *len,
            PatternShape::Arithmetic { len, .. } => *len,
            PatternShape::Repeat { unit_len, count } => unit_len * count,
        }
    }

    /// Number of instructions the compiled body would contain.
    pub fn instruction_count(&self) -> usize {
        match self {
            PatternShape::Constant { .. } => 2,
            PatternShape::Arithmetic { .. } => 2,
            PatternShape::Repeat { unit_len, count } => unit_len * 2 * count + 1,
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
                let actual_delta = slice[1] as i32 - slice[0] as i32;
                if actual_delta != *delta {
                    return None;
                }
                slice
                    .windows(2)
                    .all(|w| w[1] as i32 - w[0] as i32 == *delta)
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
                let mut val = start as i32;
                for _ in 0..len {
                    self.output.push(val as Token);
                    val += delta;
                }
            }
            Instr::Call(name, params) => {
                if let Some(skill) = self.subroutines.get(&name) {
                    let body = skill.shape.to_instructions(&params);
                    self.call_stack.push(self.pc);
                    let saved = std::mem::replace(&mut self.program, body);
                    self.pc = 0;
                    while self.step() {}
                    self.program = saved;
                    self.pc = self.call_stack.pop().unwrap();
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

const SKILLS_FORMAT_VERSION: &str = "XDPD_SKILLS_V1";

impl PatternShape {
    fn encode(&self) -> String {
        match self {
            PatternShape::Constant { len } => format!("const:{}", len),
            PatternShape::Arithmetic { delta, len } => format!("arith:{}:{}", delta, len),
            PatternShape::Repeat { unit_len, count } => format!("repeat:{}:{}", unit_len, count),
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
            Some(Ok(header)) if header == SKILLS_FORMAT_VERSION => {}
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
        let delta = seq[1] as i32 - seq[0] as i32;
        if delta != 0 && seq.windows(2).all(|w| w[1] as i32 - w[0] as i32 == delta) {
            return Some(Pattern::Arithmetic { start: seq[0], delta, len: n });
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
// Composition — Dynamic Programming over skills
// ---------------------------------------------------------------------------

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

    let mut dp = vec![u64::MAX; n + 1];
    let mut choice: Vec<Option<(usize, String, Vec<Token>)>> = vec![None; n + 1];
    dp[0] = 0;

    for i in 0..n {
        if dp[i] == u64::MAX {
            continue;
        }

        // Naive: emit one token (2 ops: Load + Output)
        if dp[i] + 2 < dp[i + 1] {
            dp[i + 1] = dp[i] + 2;
            choice[i + 1] = Some((i, String::new(), Vec::new()));
        }

        // Try each learned skill whose *structure* matches at position i —
        // not just the exact values it was originally compiled from.
        for (name, skill) in skills {
            let slen = skill.shape.span_len();
            if i + slen <= n {
                if let Some(params) = skill.shape.matches(&target[i..i + slen]) {
                    // Cost: 1 Call instruction (atomic — body size doesn't count)
                    let cost = 1u64;
                    if dp[i] + cost < dp[i + slen] {
                        dp[i + slen] = dp[i] + cost;
                        choice[i + slen] = Some((i, name.clone(), params));
                    }
                }
            }
        }
    }

    // Backtrack to construct the program
    let mut prog = Vec::new();
    let mut pos = n;
    while pos > 0 {
        match &choice[pos] {
            Some((prev, name, _)) if name.is_empty() => {
                prog.push(Instr::Load(target[*prev]));
                prog.push(Instr::Output);
                pos = *prev;
            }
            Some((prev, name, params)) => {
                prog.push(Instr::Call(name.clone(), params.clone()));
                pos = *prev;
            }
            None => {
                prog.push(Instr::Load(target[pos - 1]));
                prog.push(Instr::Output);
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
pub struct Learner {
    vm: VM,
    observation_window: Vec<Vec<Token>>,
    learned_signatures: HashSet<String>,
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
            observation_window: Vec::new(),
            learned_signatures: HashSet::new(),
            config,
        }
    }

    /// Observe a token sequence. The system looks for patterns in its
    /// temporary observation window. If a pattern repeats enough times,
    /// it is compiled into a permanent subroutine. The raw sequence
    /// is kept only in the window, then discarded.
    ///
    /// Returns names of any newly learned skills.
    pub fn observe(&mut self, sequence: &[Token]) -> Vec<String> {
        if sequence.len() < 2 {
            return Vec::new();
        }

        // Maintain the sliding window
        if self.observation_window.len() >= self.config.window_size {
            self.observation_window.remove(0);
        }
        self.observation_window.push(sequence.to_vec());

        // Count pattern frequencies across the window
        let mut freq: HashMap<String, (Pattern, u32)> = HashMap::new();
        for window_seq in &self.observation_window {
            if let Some(pattern) = detect_pattern(window_seq) {
                let sig = pattern.signature();
                let entry = freq.entry(sig).or_insert_with(|| (pattern, 0));
                entry.1 += 1;
            }
        }

        // Compile patterns that meet the occurrence threshold
        let mut new_skills = Vec::new();
        for (sig, (pattern, count)) in freq {
            if count >= self.config.min_occurrences
                && !self.learned_signatures.contains(&sig)
            {
                let name = format!("skill_{}", sig);
                let mut skill = Skill::new(name.clone(), pattern.shape());
                skill.signature = sig.clone();

                self.vm.add_skill(skill);
                self.learned_signatures.insert(sig);
                new_skills.push(name);
            }
        }

        new_skills
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
}
