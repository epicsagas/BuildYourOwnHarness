---
name: tdd
description: Test-Driven Development enforcer. Red→Green→Refactor cycle with no production code without a failing test first. Use for new features and bug fixes.
---

# TDD — Test-Driven Development

Enforce the Red→Green→Refactor loop. Never write production code without a
failing test first.

## Process

1. **Red** — write one failing test that captures the next slice of behavior.
   Run it. Confirm it fails for the *right* reason.
2. **Green** — write the minimum code to make the test pass. No more.
3. **Refactor** — improve names, structure, dedup. Tests stay green.

## Anti-Rationalization

- "I'll add the test after" → no. The test defines the requirement; code
  written before it is unverified.
- "It's too simple to test" → if it can break, it gets a test. Simple things
  break.
- "Refactoring first, tests later" → refactoring without a green test suite is
  gambling.

## Evidence

- A change is done when: a new test exists, it is green, all prior tests are
  green, and the diff has no untested production logic.
- If you cannot name the failing test you are about to write, stop and design
  the behavior first.

## Red Flags

- Writing implementation, then reverse-engineering a test that passes.
- Skipping the run-between steps ("I'm sure it fails").
- A PR with new code but no new test.

