---
name: debug
description: Systematic root-cause isolation for test failures, runtime errors, or unexpected behavior. Narrow hypothesis → verify → fix. Avoids shotgun fixes and symptom-chasing.
---

# Debug — Systematic Root-Cause Isolation

Isolate the true cause before changing code. A fix that removes a symptom but
not the cause will regress.

## Process

1. **Reproduce** — establish a minimal, deterministic reproduction. If it
   cannot be reproduced, it cannot be verified fixed.
2. **Form one hypothesis** — the smallest change in input/state that could
   produce the observed output.
3. **Verify the hypothesis** — probe it directly (log, assert, bisection) before
   editing production code.
4. **Fix the verified cause** — one change, scoped to the root cause.
5. **Confirm** — reproduction now passes; no new regressions.

## Anti-Rationalization

- "Let me just try X" → that is guessing, not debugging. Verify first.
- "It's probably Y" → "probably" is an unverified hypothesis. Probe it.
- "I'll fix the symptom for now" → symptom fixes mask the cause and breed
  regressions.

## Evidence

- A bug is fixed when: the reproduction no longer reproduces, the root cause is
  named in the commit/PR, and a regression test guards it.

## Red Flags

- Editing multiple places "to be safe" (shotgun).
- Fixing without a reproduction.
- Treating the error message as the bug rather than the condition that raised
  it.

