# XDPD: Research Brief

**Origin**: Web research conducted via Kimi (kimi.moonshot.cn), July 2026.
The raw chat log is preserved in `op-raw.txt`.
This document extracts the research findings and hypothesis.

---

## 1. Research Motivation

Build a learning mechanism ("Project A") fundamentally different from current AI:

- No neural networks — no weights, no gradient descent, no GPU
- No storage of raw experiences — no database, no vector store, no RAG
- Structural self-modification — the system changes ITSELF after experience
- CPU-only, lightweight
- General-purpose — not domain-specific

Question: Does anything like this already exist?

---

## 2. Internet Findings

After 30+ searches across AI, neuroscience, biology, compression,
cognitive architectures, and self-modifying systems.

### Finding A: Compression as Intelligence

Marcus Hutter's AIXI frames general intelligence as equivalent to
data compression. A 2011 paper analyzing PAQ8 found it outperformed RNNs
on text prediction and sentiment analysis — by up to 6%.

Key distinction: "Data Compression algorithms are predictors while
Recurrent Neural Networks are imitators."

PAQ builds a predictive model, then discards raw data. It does online
learning continuously. It aggressively forgets via counter halving.
This is learning through prediction-error-driven adaptive weighting,
not gradient descent.

### Finding B: Self-Modifying Code

Schmidhuber's Godel Machine (2007): theoretical self-improving AI
that proves code changes are beneficial. Remained hypothetical.

Darwin Godel Machine (2025): open-source implementation that empirically
validates self-modifications. Modifies its own Python codebase.
Improved SWE-bench from 20% to 50%.

Critical limitation: DGM uses Claude 3.5 Sonnet as the reasoning engine
for self-modification. The intelligence driving the changes is external.

### Finding C: Structural Memory Without Storage

A 2025 paper on Structurally Dynamic Cellular Automata demonstrates that
memory can exist purely in graph topology — "regenerating a memory function
without long-term storage of any specific inputs or outputs, resulting in a
very compact and flexible representation of the past within the topology
and conductances of the coincident graph itself."

The mechanism uses de-inforcement (weakening connections) rather than
reinforcement. No raw experiences stored. This is a computational proof
that memory without storage is possible.

### Finding D: Biological Learning Without Brains

Slime mold (*Physarum polycephalum*) learns through habituation without
neurons. Solves mazes, anticipates periodic events, transfers learned
memories to other slime molds through physical contact.

"The organism weaves memories of food encounters directly into the
architecture of the network-like body and uses the stored information
when making future decisions."

Memory is the structure of the tubular network, not a separate system.

### Finding E: Cognitive Architectures — Rule Compilation

SOAR's chunking: When problem-solving produces a result, SOAR compiles
the entire reasoning trace into a single production rule. Future similar
situations fire this rule in one step. Converts deliberate reasoning into
automatic skill. The system changes its own rule set.

ACT-R's production compilation: Combines two sequential rules into one new
rule. Eliminates memory retrieval between them. How novices become experts.

Both change the system's rule set, not store memories.

### Finding F: Emergent Models

A 2025 proposal for "Emergent Models (EMs)" suggests replacing neural
networks entirely with "iterative application of simple, fixed rules over
large state spaces, inspired by CA and Turing machines." Remains theoretical.

### Finding G: Sparse Distributed Memory (Kanerva)

Models human memory as high-dimensional binary vectors. Uses Hamming distance,
not exact match. Writes distribute data across thousands of locations.
Not a neural network. Recent 2026 work extends to generative models.

### Finding H: Structural Plasticity (Partial Match)

SMGrNN (2025): grows and prunes neural network topology based on local
activity statistics. But weights are still updated by backpropagation.
Structural changes are separate from synaptic changes.

### Finding I: Active Inference / Free Energy Principle

Karl Friston's framework unifies perception, action, and learning as
prediction-error minimization. Current implementations: pymdp (hand-designed
probability matrices, explode combinatorially), deep active inference (uses
neural networks, loses Bayesian nature). "The field has theory without
engineering." No lightweight CPU-only implementation exists.

### Finding J: Hyperdimensional Computing

Uses high-dimensional vectors with addition/subtraction for learning.
No backpropagation. Performance capped at simple tasks (MNIST works,
CIFAR-10 fails). Encoding requires manual design per application.

---

## 3. Gap Analysis

### What Does NOT Exist

After 30+ searches, no system satisfies all of these together:

- **Learns without storing raw experiences** — Almost everything stores
  something: weights, memories, rules, episodes, or vectors. The structurally
  dynamic CA comes closest but is theoretical.

- **Changes its own internal structure from experience** — DGM changes
  code but uses LLMs. SOAR/ACT-R compile rules but are symbolic and static
  in architecture. Slime mold changes structure but is biological.

- **No neural networks, no gradient descent** — Most "alternative"
  approaches still use NNs somewhere (deep active inference, NCA, SMGrNN).

- **CPU-only, lightweight** — PAQ is CPU-only. SOAR/ACT-R are CPU-only.
  But neither is a general learning engine.

- **General-purpose** — PAQ compresses text. SOAR solves symbolic problems.
  Neither generalizes to arbitrary environments.

### The Real Gap

> No one has built a system where the architecture itself is the learning
> mechanism — where experiences are consumed to change structure, and
> where the changed structure is the only thing that remains.

### Why Existing Approaches Fall Short

| Approach | Trap |
|---|---|
| Neural networks | Store experiences as weight updates. Weights are opaque. |
| RAG / Vector DB | Store raw text. Retrieve later. Not learning. |
| SOAR/ACT-R | Learn rules, but rules are hand-coded initially. No continuous adaptation. |
| PAQ compression | Builds models, discards data. Models are statistical, not structural. |
| DGM | Changes code. Uses external LLM to decide changes. |
| Cellular automata | Structure can compute. No validated learning mechanism. |
| Active inference | Theory without implementation. |

---

## 4. Hypothesis: Capability Acquisition Through Vocabulary Expansion

### Core Idea

The system starts with a small set of primitive operations (like a CPU
instruction set). It observes token streams. When it detects an invariant
(repeating pattern), it compiles that pattern into a new subroutine.
That subroutine becomes a permanent first-class operation. Raw observations
are discarded. The subroutine table IS the only memory.

### What Changes Inside the System

Not weights. Not stored text. Not vectors.

What changes:
- Which subroutines exist
- The instruction sequences they contain
- The signatures that match them to future observations
- The strength scores that decay with disuse

### Sources Combined

| Source | What XDPD Takes |
|---|---|
| PAQ compression | Online adaptation. Aggressive forgetting. CPU-only. |
| Slime mold | Memory as structure. No separate storage. |
| SOAR chunking | Compile experience into rules. Change the rule set. |
| Structurally Dynamic CA | Topology itself encodes the past. |
| DGM | Self-modification loop. |
| MDL Principle (Rissanen) | Compression as learning criterion. |
| Miller's Chunking (1956) | Hierarchy: primitives into chunks into meta-chunks. |

### How It Differs From Everything

| System | Why XDPD is different |
|---|---|
| Neural network | No weights. No backprop. Instruction set grows. |
| RAG | No stored documents. No retrieval. |
| SOAR/ACT-R | No hand-coded rules. Rules emerge from observations. |
| PAQ | Not statistical prediction. Structural compilation. |
| Expert system | Rules learned, not written. |
| Cellular automata | Not a grid. Rules themselves change. |

### What We Proved First (Now Complete)

1. Can it learn arithmetic patterns? YES — `Seq` instruction.
2. Can it learn constant patterns? YES — `Seq(start, 0, len)`.
3. Can it learn repeat patterns? YES — subroutine with Load/Output body.
4. Can it compose learned skills? YES — DP over skill table.
5. Does it detect anomalies? YES — compression ratio < 1.3x flags unknown patterns.
6. Does the subroutine table hold all state? YES — no raw data retained after observation.
7. Does it run on CPU with zero dependencies? YES — pure Rust stdlib.

### Known Limitations

- Patterns match exact token values, not parameterized templates.
  A sequence [100,102,104] with delta=2 does not match a skill learned
  from [0,2,4] despite sharing the same delta pattern.
  Template/parametric skills are a planned future extension.

- Repeat patterns compile to per-token bodies. Execution-level cost
  equals naive; savings are at program level (1 Call vs N Load+Output).

### Implementation

- Language: Rust
- Dependencies: Zero (standard library only)
- Platform: CPU-only, any architecture
- License: MIT

---

## 5. Honest Assessment

The concept — a learning mechanism that changes its own structure from
experience without storing raw data, without neural networks, and without
GPUs — did not exist as a working system when this research was conducted.

Individual pieces existed in isolation:
- Compression proves prediction without NNs is viable
- Slime mold proves structural memory without storage is real
- SOAR proves rule compilation is a valid learning mechanism
- Structurally dynamic CA proves topology can encode memory without stored inputs

XDPD is the first implementation that combines these into a working engine.
The core mechanism compiles and runs. The subroutine table holds all state.
Patterns are learned and reused. Anomalies are detected.

The project occupies genuinely open territory.

---

*Sources cited are from web searches conducted via Kimi in July 2026.
No training data was used for the findings. The hypothesis and
implementation are original synthesis.*
