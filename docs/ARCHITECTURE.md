# XDPD Architecture

## Design Philosophy

XDPD embodies a specific hypothesis about learning:

> Learning is not storing information. Learning is creating new computational capabilities. The subroutine table is the memory. When the system learns, it adds a word to its language. That word is computational, not textual.

This hypothesis is synthesized from multiple fields:

- **SOAR Chunking** (Laird/Newell, 1983): Compiles reasoning traces into production rules. XDPD generalizes this from symbolic reasoning to raw observation streams.
- **MDL Principle** (Rissanen, 1978): Learning is finding the shortest description of data. XDPD compiles patterns into subroutines when the subroutine is shorter than the raw sequence.
- **Miller's Chunking** (1956): Humans group small units into chunks, then chunks into super-chunks. XDPD's subroutine hierarchy mirrors this.
- **Structurally Dynamic CA** (2025): Proved memory can exist purely in topology without storing inputs/outputs. XDPD's subroutine table is the computational analog.
- **AIXI** (Hutter, 2005): Mathematical proof that compression equals intelligence. XDPD evaluates learning by instruction-count reduction — a form of compression.

## Core Mechanism

### 1. Observation

The learner receives a token sequence — a slice of `u32` values. This could represent anything: sensor readings, log entries, API response codes, LLM output tokens, financial data points. The system makes no assumptions about token semantics.

### 2. Pattern Detection

`detect_pattern()` scans the sequence for three structural invariants:

- **Constant**: All tokens identical. Detected in O(n) by checking if all elements equal the first.
- **Arithmetic**: Constant difference between consecutive tokens. Detected in O(n) by computing delta and verifying all pairs.
- **Repeat**: The sequence divides evenly into identical chunks. Detected in O(n^2) worst case by testing divisors and comparing chunks.

If no invariant is found, the function returns `None`. The observation is discarded.

### 3. Compilation

When a pattern is detected and has been observed enough times (configurable via `min_occurrences`), it is compiled into a `Skill`:

```rust
struct Skill {
    name: String,         // e.g., "skill_arith:d2x5"
    body: Vec<Instr>,     // Compiled instruction sequence
    output_seq: Vec<Token>, // Expected output for matching
    strength: i32,        // Decays when unused
    uses: u32,            // Invocation count
    signature: String,    // Structural fingerprint
}
```

The body depends on the pattern type:
- **Constant** / **Arithmetic**: `[Seq(start, delta, len), Ret]` — 2 instructions, regardless of sequence length.
- **Repeat**: Per-token `[Load(t), Output]` pairs — N*2 instructions for N tokens. The compression is at the program level: 1 `Call` replaces all N pairs.

### 4. DP Composition

`compose()` finds the minimal-instruction program to produce a target sequence using the available skills. It uses dynamic programming:

```
dp[i] = minimum instructions to produce target[0..i]

For each position i:
  Option A: emit token i naively         -> dp[i] + 2  operations
  Option B: use a matching skill of len L -> dp[i] + 1  operation (1 Call)

choice[i] stores which option was selected.

Backtracking reconstructs the program from choice[].
```

This is the key innovation. Instead of always emitting tokens one by one, the system tries to cover the target with the largest learned subroutines first. A 100-token sequence that matches a learned pattern executes in 1 `Call` instead of 200 `Load`+`Output` pairs.

### 5. Anomaly Detection

`check_anomaly()` computes the compression ratio between naive and learned execution. A low ratio (<1.3x) means the sequence does not match any learned pattern — it is anomalous. This works because:

- Sequences matching learned patterns use few program-level instructions (1 `Call` per matched segment).
- Sequences that don't match fall back to per-token emission (2 instructions per token).
- The ratio reflects how much the system can compress the sequence.

### 6. Forgetting

The observation window has a configurable maximum size. When full, the oldest observation is evicted. This means:

- The system only remembers recent observations for pattern detection.
- Skills persist in the subroutine table regardless of window eviction.
- The subroutine table grows monotonically (no skill deletion in current version).

## Data Flow

```
External input (CSV, sensor, API)
        |
        v
Token sequence [u32, u32, ...]
        |
        v
Learner::observe() ------> observation_window (bounded, evicting)
        |                       |
        |                  detect_pattern()
        |                       |
        |                  Pattern found?
        |                  /         \
        |               Yes           No
        |                |             |
        |        Compile to skill     Discard
        |                |
        |        Add to subroutine table
        |
        v
Learner::generate()
        |
        v
compose() — DP over skill table
        |
        v
Program: [Call("skill_arith:d2x5"), Call("skill_const:45x6"), ..., Ret]
        |
        v
VM::run() — execute program
        |
        v
Output tokens + instruction count
```

## VM Design

The VM is intentionally minimal:

| Component | Description |
|---|---|
| `reg` | Single 32-bit register |
| `pc` | Program counter |
| `program` | Current instruction sequence |
| `output` | Output buffer (appended to by `Output` and `Seq`) |
| `call_stack` | Return addresses for nested `Call`/`Ret` |
| `subroutines` | The persistent memory — `HashMap<String, Skill>` |
| `instr_count` | Total instructions executed (for metrics) |

The `Call` instruction saves the current program counter, replaces `program` with the subroutine body, resets `pc` to 0, runs the subroutine, then restores the original program.

## Key Design Decisions

### Token = u32

Rather than a complex type hierarchy, all discrete values map to `u32`. This keeps the system simple and domain-agnostic. The mapping from domain concepts to tokens is external to the engine.

### No parameters in patterns

Current patterns match exact token sequences. A sequence `[100, 102, 104, 106, 108]` with delta=2 matches `Arithmetic { start: 100, delta: 2, len: 5 }`. A different sequence with the same delta but different start (`[200, 202, 204, 206, 208]`) does NOT match. Template/parametric skills are a planned future extension.

### Program-level vs execution-level costs

The DP composition optimizes for **program-level** instruction count, where 1 `Call` = 1 instruction. The VM's `instr_count` tracks **execution-level** instructions, which includes all instructions inside subroutine bodies. The demo reports program-level costs to demonstrate the compression. Both metrics are available.

### Zero dependencies

The engine uses only the Rust standard library. No serde, no tokio, no ndarray, no ML frameworks. This is intentional: the learning mechanism should be embeddable in any environment without dependency management.

## Future Directions

| Feature | Description |
|---|---|
| Template skills | Parametric subroutines matching same pattern structure with different values |
| Skill decay/GC | Remove unused skills from the subroutine table |
| Persistent storage | Serialize/deserialize the subroutine table to disk |
| Parallel observation | Concurrent observation processing with sharded skill tables |
| Python bindings | PyO3 bindings for Python integration |
| C FFI | C-compatible API for embedding in C/C++ applications |
| Hierarchical composition | Skills that call other skills, building deep abstraction hierarchies |
