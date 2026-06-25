---
name: reproducibility
description: Reproducibility checker for computational research and analyses. Demands pinned seeds, versions, and data hashes so a result can be re-run. Use when producing any number, figure, or benchmark a decision will rest on.
---

# Reproducibility — Pin It or It Didn't Happen

A result that can't be re-run is an anecdote. Pin the inputs so someone else
(the reader, future-you, the auditor) can reproduce the exact output.

## Process

1. **Pin the stack** — record language/tool versions, library versions, and OS.
2. **Pin the data** — version + hash (sha256) of every input dataset.
3. **Pin the randomness** — fix the seed; record it alongside the result.
4. **Pin the commands** — the exact invocation, not a paraphrase.

## Anti-Rationalization

- "It's deterministic, no seed needed" — then stating the seed costs nothing and
  removes doubt.
- "The data won't change" — hash it anyway; "won't" is an unverified bet.
- "Nobody will re-run this" — the figure will outlive your memory of how it was
  made.

## Evidence

- A result is reproducible when a third party, given the pinned stack/data/seed/
  commands, obtains the same numbers within tolerance.

## Red Flags

- A headline number with no seed, version, or data hash attached.
- "Latest" versions pinned (floats rather than locks).
- Re-running once and trusting the first output.
