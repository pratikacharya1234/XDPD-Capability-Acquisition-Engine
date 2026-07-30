# AGENTS.md — Build Instructions for Coding Agents

Read this whole file before writing any code. It is written in plain English on purpose.

You are working on **XDPD**, a small Rust library that learns patterns from streams of numbers
and turns them into reusable "skills". It has zero dependencies and 18 passing tests. It
works. Your job is to extend it **without breaking it**.

There are 9 phases. **Do one phase at a time. Finish it, test it, stop, and report.** Do not
start the next phase until a human tells you to. This is not negotiable — the whole point of
the phase structure is that a human checks your work between steps.

---

## 1. What this project is, in plain English

XDPD watches a stream of numbers go by. When it notices the same *shape* of pattern happening
over and over, it writes itself a tiny program that can reproduce that shape, and gives it a
name. That named program is called a **skill**.

Later, when it sees the same shape again, it doesn't need to spell out every number. It just
says "run skill #4 with these values." One instruction instead of hundreds.

Two things make this different from every other pattern system:

1. **It throws away the raw data.** After it finds a pattern, the original numbers are gone.
   The only thing kept is the *shape*. So the skill table is the memory. There is no database.
2. **A match is exact and lossless.** When a skill fires, the numbers that come out are
   byte-for-byte identical to the numbers that went in. It is a compression, not a guess.

That second point is the most important sentence in this document. Protect it.

## 2. The north star: why this makes an LLM stronger

Everything you build should serve one of these three. If a change doesn't, don't make it.

**A. It gives an agent more room to think.**
An AI agent burns huge amounts of its limited context on repetitive machine text — the same
tool schemas, the same log lines, the same trace formats, over and over. XDPD can collapse
those repeated structures into a short reference and expand them back perfectly. Same
information, less space. More space left for actual reasoning.

**B. It tells an agent when something is off, without another AI model.**
XDPD already knows how well it can compress something. Familiar structure compresses well.
Weird, never-seen-before structure doesn't. So the compression ratio is a free alarm bell for
"this doesn't look like what normally happens." Because it's pure arithmetic with no language
model inside, **you cannot prompt-inject it or talk it out of firing.** Security tools built
on language models can be fooled by clever text. This can't be.

**C. It builds a skill library that was earned, not invented.**
There is published research (SkillsBench) showing that when an LLM writes its own reusable
skills, the skills provide **no measurable benefit**. When humans write them, performance
jumps 16 points. XDPD's skills are neither — they are *mechanically extracted from things that
actually happened*. No language model decides what's worth remembering. Repetition decides.

## 3. Hard rules — breaking any of these fails the phase

1. **Never add a dependency.** `xdpd/Cargo.toml` must keep an empty `[dependencies]` section.
   Standard library only. If you think you need a crate, you don't — write the ten lines.
2. **Never delete or weaken an existing test.** All 18 must keep passing. You may add tests.
   If an existing test now fails, **you broke something** — fix your code, not the test.
   The only exception: a human explicitly tells you a behavior is intentionally changing.
3. **Never break the lossless guarantee.** If a skill matches some input, running that skill
   must reproduce that input exactly. No "close enough", no similarity scores, no thresholds,
   no fuzzy matching anywhere in the core. Ever.
4. **Never rewrite `lib.rs` wholesale.** Make small, surgical edits. If your diff touches more
   than ~150 lines in one phase, stop and ask.
5. **Never change the public API without saying so.** Anything `pub` is used by people outside
   this repo. Add new things freely; changing or removing existing ones needs a human's OK.
6. **Never change the saved-file format without a migration path.** Details in Phase 2.
7. **Never invent performance numbers.** If you didn't run it and see the output, you don't
   report it. Write "not measured" instead. This rule exists because the project's credibility
   depends entirely on its numbers being real.
8. **Don't touch these files unless a phase tells you to:** `README.md`, `RESEARCH.md`,
   `docs/ARCHITECTURE.md`, `CHANGELOG.md`, `xdpd/Cargo.toml` (version line only, when told),
   anything in `.github/`.
9. **Work on a branch, never commit straight to `main`.** One branch per phase.

## 4. Orientation — where everything lives

```
xdpd/src/lib.rs              THE ENTIRE LIBRARY. ~830 lines of code + ~250 of tests.
                             Everything you build goes here.
xdpd/Cargo.toml              Zero dependencies. Keep it that way.
examples/src/main.rs         The demo/benchmark you run to see it work.
examples/gateway/            A toy HTTP proxy. Demonstrates shape, not the mechanism.
docs/ARCHITECTURE.md         The full design, with evidence. Read §IV before Phase 2.
RESEARCH.md                  What's proven vs unproven, with sources.
```

The five pieces inside `lib.rs` you need to understand:

| Piece | What it does | Roughly where |
|---|---|---|
| `Instr` | The 5 instructions the machine can run: `Load`, `Output`, `Seq`, `Call`, `Ret` | line 32 |
| `Pattern` | A pattern *with* its actual numbers in it | line 94 |
| `PatternShape` | The same pattern with the numbers **stripped out**. This is the clever bit. | line 193 |
| `VM` | Runs instructions. Holds the skill table. | line 285 |
| `Learner` | Watches streams, decides what to turn into a skill | line 687 |

**The one idea you must understand before you write anything:**

A skill stores a *shape*, not an answer. `[0,2,4,6,8]` and `[9000,9002,9004,9006,9008]` are
the *same shape* — "go up by 2, five times." So one skill covers both, and covers every other
sequence like them, including ones never seen before. The actual starting number is passed in
at the last second as a **parameter**.

That's why `Instr::Call` looks like `Call(name, params)` and not just `Call(name)`. The skill
body doesn't exist sitting in memory — it gets built fresh each time it's called, from the
shape plus the parameters. Look at `PatternShape::to_instructions()` around line 255 to see
this happen.

Remember this. Phase 2 works because of it.

## 5. Before you start ANY phase

```bash
cd xdpd && cargo test          # must print: 18 passed; 0 failed
cd ../examples && cargo run --release   # must run and print results
git checkout -b phase-N-short-name
```

If the 18 tests don't pass before you change anything, **stop and tell the human.** Something
is wrong with the environment and it is not your fault — don't try to fix it by editing code.

## 6. How to finish a phase

A phase is done when **all** of these are true:

1. `cargo test` passes, showing *more* tests than before (yours were added).
2. You added at least one test that **fails if your feature is removed**. Prove it: comment
   out your feature, watch your test fail, uncomment it. This is the single best habit here.
3. `cargo build --release` gives no warnings you introduced.
4. The example still runs.
5. You wrote a short report: what you changed, what you tested, what surprised you, and
   anything you're unsure about.

Then **stop.** Say "Phase N complete, ready for review." Do not continue.

Commit message format:

```
Phase N: <short description>

<what changed and why, 2-4 lines>

Tests: 18 -> <new count>, all passing

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
```

---

# THE PHASES

---

## Phase 1 — Three small safety fixes

**This is a warm-up phase.** It is small on purpose. It lets a human confirm you work
carefully before you touch anything structural. Do not expand its scope.

**Goal:** fix three real bugs that are currently harmless but become dangerous later.

**Fix 1 — a missing skill must be an error, not silence.**
In `VM::step()`, the `Instr::Call` branch (~line 348) does `if let Some(skill) = ...` with no
`else`. So if a program calls a skill that isn't in the table, **nothing happens and the
program reports success.** It silently produces wrong output. Right now this can't be
triggered, but it will be able to be later.

Make it fail loudly. Since `step()` returns `bool`, the cleanest minimal option is to record
the failure on the VM (for example a `last_error: Option<String>` field with a public getter)
and stop execution. Do **not** panic — this is a library, and libraries should not crash the
caller's program. Do not change `step()`'s signature.

**Fix 2 — cap how deep skills can call each other.**
`step()` handles `Call` by recursing into itself (`while self.step() {}`, ~line 354). Today
skills are flat so this only goes one level deep. Later phases allow skills that call other
skills, and a cycle would blow the stack and crash the process. Add a depth counter with a
limit (64 is plenty) and treat exceeding it as an error, same mechanism as Fix 1.

**Fix 3 — division by zero in the anomaly score.**
`check_anomaly()` (~line 781) computes `naive / learned`. For an empty input, `learned` is 0,
so it returns infinity. Return something sensible instead — `1.0` means "no compression
happened", which is the honest answer for an empty sequence.

**Touch only:** `xdpd/src/lib.rs`.

**Done when:**
- A test loads a program that calls a skill name that doesn't exist, and asserts the VM
  reports an error instead of quietly producing empty output.
- A test builds a skill that calls itself (or a small cycle), runs it, and asserts the depth
  limit stops it without crashing the test process.
- A test asserts `check_anomaly(&[])` returns a finite number.
- All 18 original tests still pass. Total should be 21+.

---

## Phase 2 — Templates with variable slots ⭐ THE IMPORTANT ONE

**Read `docs/ARCHITECTURE.md` §IV.1 before starting.**

**Goal:** let XDPD recognize "the same thing with some parts changed."

**Why this matters more than anything else in this document:** right now XDPD can only
recognize three kinds of pattern — all-the-same, counting-up, and repeating-block. Real data
doesn't look like that. Real data looks like:

```
GET /api/user/8814 200 34ms
GET /api/user/2291 200 41ms
GET /api/user/9007 404 12ms
```

Those three lines are obviously the same *shape* with three parts that vary. **XDPD cannot
express that today.** This is the single reason it works on made-up test sequences but not on
real logs, real traces, or real tool calls. Everything in the north star (§2) is blocked on
this one feature.

**The design.** Add a fourth kind of shape built out of slots:

```rust
pub enum Slot {
    Fixed(Token),                    // this exact number must be here
    Var,                             // any one number; remember what it was
    Run { delta: i32, len: usize },  // a counting run; remember where it started
}
```

Then add **one** new variant to the existing `PatternShape` enum. Leave the three existing
variants exactly as they are — they're fast paths and their tests must keep passing:

```rust
pub enum PatternShape {
    Constant { len: usize },                    // unchanged
    Arithmetic { delta: i32, len: usize },      // unchanged
    Repeat { unit_len: usize, count: usize },   // unchanged
    Template(Vec<Slot>),                        // new
}
```

**Here is the beautiful part, and why this phase is safe:** you do not need to touch the VM at
all. `Instr::Call(name, params)` already carries a list of numbers. The captured `Var` values
*are* those parameters. No new instruction. No change to how programs execute. The design from
§4 pays off here — this is what the shape/parameter split was for.

You need to fill in the four existing methods for the new variant:

- `span_len()` — how many numbers this covers. `Fixed` and `Var` are 1 each, `Run` is its
  `len`. Add them up.
- `instruction_count()` — 2 per slot is fine.
- `matches(slice)` — walk the slots and the input together. `Fixed` must match exactly or you
  return `None`. `Var` captures whatever number is there. `Run` checks the step size is right
  and captures the starting number. Return the captured numbers **in slot order**.
- `to_instructions(params)` — build the body. `Fixed(t)` becomes `Load(t)` + `Output`. `Var`
  becomes `Load(next param)` + `Output`. `Run` becomes one `Seq`. Consume params in the same
  order `matches` produced them.

**Getting `matches` and `to_instructions` to agree on parameter order is the whole job.** If
they disagree, output comes out scrambled. Test this specifically and deliberately.

**Saving to disk — read this carefully, rule 6 applies.** The file format has a version header
`XDPD_SKILLS_V1`. You're adding a new shape kind, so an old version of the library cannot read
your new files.

- The writer now emits `XDPD_SKILLS_V2`.
- The reader **must accept both** `V1` and `V2`. Old files keep working forever.
- Add a test proving a V1 file still loads.

This also fixes a known limitation — the project currently has one format version and no
migration path. You're establishing the pattern for all future versions. Do it properly.

**Touch only:** `xdpd/src/lib.rs`.

**Done when:**
- A test builds a template like `[Fixed(71), Var, Fixed(200), Var]`, matches it against
  several inputs that differ only at the `Var` positions, and asserts **every one comes back
  byte-for-byte identical**. This is the lossless guarantee (rule 3). Make it loud and obvious.
- A test asserts a non-matching input returns `None` — a `Fixed` slot that disagrees must
  refuse to match. **No near-misses allowed.**
- A test saves a template skill, loads it into a fresh `VM`, and confirms it still works.
- A test loads a hand-written `XDPD_SKILLS_V1` file successfully.
- All previous tests pass.

---

## Phase 3 — Feed it a stream instead of a list

**Goal:** let data arrive a piece at a time, instead of all at once in a `Vec`.

Right now you must hand `observe()` a complete sequence. Real data doesn't arrive that way; it
trickles in. Add:

```rust
pub fn observe_token(&mut self, t: Token) -> Vec<String>
pub fn observe_chunk(&mut self, ts: &[Token]) -> Vec<String>
```

Keep the existing `observe()` working exactly as it does now — other people's code calls it.

**Also fix the slowness while you're in here.** `observe()` currently re-examines *every* entry
in its memory window on *every* single call, and it removes old entries with
`Vec::remove(0)`, which shuffles the entire window in memory each time. Use a `VecDeque` and
keep a running count instead of recounting from scratch. You'll see this at line ~723.

**Done when:**
- A test feeds a pattern in one token at a time and confirms the same skills get learned as
  when the whole sequence is passed at once. Same input, same result, different delivery.
- A test feeds 100,000 tokens and finishes fast (say under a second in release mode). Report
  the **actual measured** time (rule 7).
- All previous tests pass.

---

## Phase 4 — Find patterns *inside* a stream

**Goal:** stop requiring the caller to pre-cut the data into perfect pieces.

Here's the current limitation, and it's a subtle one. `detect_pattern()` only returns something
if the **entire** input is one single pattern. Give it 50 identical numbers followed by some
noise and it says "no pattern found" — even though there's an obvious pattern sitting right
there in the first 50.

So today, XDPD's apparent skill at finding patterns is partly just the caller slicing the data
up nicely beforehand. That's not learning, that's the caller doing the work.

Note the odd asymmetry: `compose()` **already** knows how to break a target into segments —
that's exactly what its dynamic-programming loop does. Only the *learning* side lacks it.

**The lazy, proven approach — alignment.** Do not try to implement a full grammar-induction
algorithm. Do this instead:

1. Slide a window over the stream. Find two places that look like they start the same way
   (share a couple of leading numbers).
2. Line those two stretches up side by side.
3. Where they **agree**, emit `Slot::Fixed`. Where they **disagree**, emit `Slot::Var`.
4. That's your template. You just learned it from real data.

That's it. Two examples of a thing, aligned, gives you the template. This is essentially how
production log-template miners work, and it drops straight into the `Template` shape you built
in Phase 2.

**Worth knowing:** the best-known production tool in this space (Drain) assumes the unchanging
parts of a message sit near the *front*, and that messages of the same type have the same
*length*. Both assumptions are documented sources of its errors. **Alignment has neither
assumption** — you can find fixed parts anywhere, and slots can differ in count. Don't copy
Drain's limitations by accident.

Mark any shortcut you take with a `ponytail:` comment naming what it can't handle yet.

**Done when:**
- A test feeds a stream where a pattern is buried in the middle of unrelated noise, and
  asserts the pattern is found anyway.
- A test feeds three realistic "log lines as numbers" that differ only in a couple of
  positions, and asserts a single template skill is learned covering all three, and that all
  three reproduce exactly.
- A test asserts pure random noise learns **nothing**. False patterns are worse than no
  patterns.
- All previous tests pass.

---

## Phase 5 — Let it forget

**Goal:** stop the memory growing forever.

The skill table only ever grows. Nothing is ever removed. A process running for a week
accumulates junk skills until it's slow and bloated.

Two fields already exist for this and are **completely unused**: `strength` (set to 10 once
and never touched again) and `uses` (never incremented, ever). They get faithfully saved to
disk carrying no information. Make them real:

- Bump `uses` every time a skill actually gets called.
- Lower `strength` slowly for skills that aren't being used.
- Drop skills whose strength falls below a floor. Cap total table size.

**The trap — do not miss this.** There is a *second* structure that also grows forever:
`learned_signatures` (line ~690). It remembers "I already learned this shape, don't learn it
again." If you evict a skill but leave its signature in that set, that shape becomes
**permanently unlearnable** — XDPD will never rediscover it, because it thinks it already
knows it. **Prune both, in the same operation.**

There's a well-established idea worth borrowing here: a rule that only gets used once isn't
worth keeping as a rule. That's the eviction policy, and it's been proven since 1997.

**Done when:**
- A test confirms `uses` goes up when a skill is called.
- A test runs long enough to trigger eviction and asserts the table stays under its cap.
- A test evicts a skill, then feeds that exact shape again, and asserts **it gets relearned**.
  This is the trap above. If it doesn't relearn, you forgot to prune `learned_signatures`.
- All previous tests pass.

---

## Phase 6 — Make matching fast when there are many skills

**Goal:** keep it fast at 10,000 skills, not just 3.

`compose()` currently tries **every single skill at every single position** in the input, and
clones strings and vectors while doing it (~line 617). With 3 skills that's free. With 10,000
it's the entire cost of the system.

Group skills by what kind they are and how long they span, so each position only tests the
handful that could possibly fit. Cut the clones.

**Do this phase before any GPU work.** Making a wasteful scan run in parallel is just wasting
more hardware. Fix the algorithm first.

**Done when:**
- A benchmark shows composition time stays roughly flat going from 100 to 10,000 skills.
  Report the **real measured** numbers, before and after (rule 7).
- Output is byte-identical to before the change for the same inputs. Add a test that pins this.
- All previous tests pass.

---

## Phase 7 — Test it against reality, and publish whatever happens

**Goal:** replace synthetic numbers with real ones.

Everything so far has been tested on sequences we made up. The headline claims in the README
(16×, 93.8%) are real arithmetic on **synthetic** data, counted at the program level. They are
not measurements on real-world data, and they must never be presented as if they were.

This phase is where the project either earns credibility or learns something. Both are wins.

1. Get a real dataset — public log collections (loghub is the standard one) or captured agent
   tool-call traces.
2. Run XDPD over it. Measure: how much did it compress? How many templates did it find? How
   many were right?
3. Compare against Drain3, the established production tool for this.
4. **Write down the real number, whatever it is.**

**If Drain3 beats us, publish that.** The differentiators survive losing a template-accuracy
benchmark: lossless exact reproduction, no assumptions about where the fixed parts sit, zero
dependencies, and anomaly detection for free. A project that publishes its losing numbers is
the one people trust when it reports winning ones. Hiding a bad benchmark is the single
fastest way to destroy this project's credibility permanently.

**Done when:**
- A runnable command reproduces the benchmark from a named, downloadable dataset.
- Results are written down honestly, including anything that went badly.
- No claim exists anywhere without a dataset name and a way to reproduce it.

---

## Phase 8 — The agent application (the north star, built)

**Goal:** point it at the thing from §2 and see if it actually helps.

Only start this after Phase 7 gives real numbers. Two deliverables:

**8a. Trace compression.** Feed real agent tool-call sequences in. Learn the repeated
structure. Measure how much smaller the trace gets when repeated structures are replaced with
skill references — and prove the expansion is byte-exact. This is north star A: an agent that
spends less of its context on boilerplate has more left for thinking.

**8b. Behavioral baseline.** Use the compression ratio as an alarm. Learn what an agent's
normal tool-call sequences look like, then measure the score when the sequence is unusual.
Improve `check_anomaly` to report **which part** of the sequence failed to compress, not just
one overall number — "unusual" is useless without "where."

This lands on a problem the research literature explicitly calls unsolved: defining what
"normal" looks like for an agent, stably. Current approaches use machine-learning models,
which can be fooled by adversarial input. This one is arithmetic. That's the pitch — and it's
only a real pitch once 8b has numbers behind it.

**Done when:**
- Both are runnable on real captured data with reported measurements.
- Expansion is proven byte-exact.
- Anomaly output identifies specific spans, not just a scalar.

---

## Phase 9 — Reach (only when 1–8 are done)

Pick based on where interest actually shows up, not by guessing:

- **WASM build** — nearly free, since there are no dependencies. Put file saving behind a
  feature flag, and it runs in browsers and on edge workers.
- **GPU** — only after Phase 6. Pattern detection and matching are naturally parallel.
- **Shared skill tables** — the save format is already one-skill-per-line and mergeable, so
  syncing is a set union. Needs Phase 1's error handling first, so a mismatched table fails
  loudly instead of silently producing wrong output.

---

## 7. Things you will be tempted to do. Don't.

| Temptation | Why not |
|---|---|
| "Similar enough" matching, or a similarity threshold | Kills the lossless guarantee, which is the entire competitive advantage. Systems that guess produce confidently wrong answers. This one can't, by construction. Keep it that way. |
| Adding embeddings or a small model to improve matching | Then it's just a worse version of tools that already exist. The value is being the thing that has *no* model in it. |
| Adding `serde`, `rayon`, `tokio`, anything | Zero dependencies is a feature people choose this library for. |
| Refactoring `lib.rs` into modules "for cleanliness" | Not asked for, creates a huge diff, hides real changes in noise. If a human wants it, they'll ask. |
| Building the LLM prompt-cache idea from the old README | Measured production data says 60–70% of real queries are unique, so cache hit rates from research papers don't hold up. It's the weakest use case, not the flagship. |
| Quoting the "gzip beats BERT" result as support | It was **debunked** — the reported accuracy used an oracle tie-break. Citing it in this specific field is the fastest way to lose expert readers. |
| Doing two phases at once because they seem related | The human review between phases is the safety mechanism. Skipping it defeats the point. |
| Reporting a speed or accuracy number you estimated | Rule 7. Say "not measured." Every fake number found later makes every real number suspect. |

## 8. If you get stuck

Say so. Report what you tried, what happened, and what you think the options are. A phase
reported as "blocked, here's why" is a good outcome. A phase reported as "done" when it isn't
is the worst possible outcome, because the next phase gets built on sand.

If a phase's instructions here contradict what the code actually does, **trust the code and
tell the human.** This document was written against the code at version 0.2.1 and the code may
have moved.

---

*Written against XDPD 0.2.1 (18 tests passing). Full design rationale with sources:
`docs/ARCHITECTURE.md`. Evidence for every claim, including what's disproven:
`RESEARCH.md`.*
