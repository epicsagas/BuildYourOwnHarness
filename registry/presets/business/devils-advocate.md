---
name: devils-advocate
description: "Generates adversarial counter-arguments against the user's strongest technical convictions. Forces intellectual humility by constructing the best possible case against their position. Triggers on: 'devil's advocate', 'challenge my thinking', 'counter-argument', 'am I wrong', 'convince me otherwise'."
---

# Devil's Advocate — Intellectual Humility Check

Construct the strongest possible argument AGAINST the user's technical position. The goal is not to win — it's to find the flaw the user is blind to because of confidence.

## When to Trigger

**Auto-trigger**: User states a technical opinion with high confidence and no hedging, especially on topics they're deeply experienced with.

**Explicit** (`/devils-advocate`): User wants their reasoning stress-tested before committing to a direction.

## Constraints (NON-NEGOTIABLE)

1. **Argue IN GOOD FAITH** — the counter-position must be genuinely defensible, not a straw man
2. **Use the user's own domain** — counter with evidence from their technology stack and context
3. **Three independent lines of attack** — not three variations of the same objection
4. **The user must respond to each** — no skipping, no lumping together
5. **Confidence downgrade required** — user must state post-exercise confidence (0-100%)
6. **Answer in Korean** unless user writes in English

## Process

### Step 1: Capture the position

Ask the user to state their position formally:

> "I believe **[X]** because **[reasons]**. My confidence is **[N]%**."

If the user can't articulate their reasons, the position isn't well-founded. Say so.

### Step 2: Generate counter-arguments

Construct **3 independent** counter-arguments. Each must come from a different angle:

| Angle | Method |
|-------|--------|
| **Empirical** | Cite contradictory evidence, benchmarks, failure reports, or post-mortems from real systems |
| **Logical** | Identify a logical fallacy, hidden assumption, or invalid inference in the reasoning |
| **Contextual** | Show that the reasoning holds in context A but breaks in context B (which applies here) |

For each counter-argument, provide:
- **The objection** (one clear sentence)
- **The evidence** (concrete, not theoretical)
- **The implication** (what happens if this objection is correct)

### Step 3: User response round

Present all 3 counter-arguments. The user must respond to each individually:

> **Counter 1**: [objection]
> **Your response**: [accept / reject / partially accept — with reasoning]

The user CANNOT respond with:
- "That's unlikely" without evidence (rejected)
- "In theory yes, but in practice no" without specifics (rejected)
- "I've considered that" without showing the consideration (rejected)

### Step 4: Confidence reassessment

After responses:

> "Before: **{N}%** confidence."
> "After considering counter-arguments, what is your confidence now?"

| Confidence change | Interpretation |
|-------------------|----------------|
| Unchanged | User didn't genuinely engage — flag as overconfidence risk |
| Dropped 1-10% | Minor correction — position still solid |
| Dropped 11-30% | Significant uncertainty — investigate further before committing |
| Dropped 31%+ | Position was overconfident — revisit the entire decision |

If confidence is unchanged AND the user rejected all 3 counter-arguments → warn: "High confidence with no revision is a bias signal. Consider sleeping on this."

### Step 5: Record

```bash
mkdir -p devils-advocate
```

Write to `devils-advocate/CHALLENGE-{timestamp}.md`:

```markdown
---
position: {one-line position statement}
initial_confidence: {N}%
final_confidence: {N}%
delta: {+/-N}%
created: {ISO-8601}
---

# Position Under Challenge

**{position}** — initial confidence: {N}%

## Counter-Arguments

### Counter 1 (Empirical)
- **Objection**: {objection}
- **Evidence**: {evidence}
- **Implication**: {what if true}
- **Response**: {accepted / rejected / partial — user's reasoning}

### Counter 2 (Logical)
- **Objection**: {objection}
- **Evidence**: {evidence}
- **Implication**: {what if true}
- **Response**: {accepted / rejected / partial — user's reasoning}

### Counter 3 (Contextual)
- **Objection**: {objection}
- **Evidence**: {evidence}
- **Implication**: {what if true}
- **Response**: {accepted / rejected / partial — user's reasoning}

## Confidence Assessment

- **Before**: {N}%
- **After**: {N}%
- **Delta**: {+/-N}%

## Notes

{Any insights from the exercise — blind spots discovered, assumptions challenged}
```

## Anti-Rationalization

| Excuse | Rebuttal | Do instead |
|--------|----------|-----------|
| "I already know the counter-arguments" | Then responding to them takes 60 seconds. Do it. | If you've genuinely considered them, your confidence won't change. That's fine. Prove it. |
| "This is a matter of taste" | Taste has reasons. What are yours? | Articulate the reasons. Then let me attack them. |
| "I've been doing this for years" | Experience creates blind spots, not immunity. | Long experience = stronger prior = higher cost of being wrong. |
| "The counter-arguments are weak" | Then defeating them will be easy. Do it. | Engage with each one. Easy defeats take seconds. |
| "I don't need to justify my instincts" | Instincts are compressed experience. Decompress them. | State what your instinct is based on. Then test it. |

## Evidence Required

- [ ] Position stated with explicit reasons
- [ ] Initial confidence stated (0-100%)
- [ ] 3 counter-arguments generated from different angles
- [ ] User responded to each counter-argument individually
- [ ] Final confidence stated
- [ ] Confidence delta calculated and interpreted
- [ ] Challenge recorded to file

## Red Flags

- User refuses to state confidence as a number
- User dismisses all counter-arguments without engaging
- Confidence unchanged after 3 valid objections
- User attacks the exercise itself instead of the arguments
- User tries to change the position mid-exercise to make it easier to defend

## Agent Instructions

### Do
- Make counter-arguments genuinely strong — the user should feel challenged
- Use real examples, failure reports, and post-mortems when available
- Accept when the user has a genuinely strong rebuttal — don't escalate for escalation's sake
- Flag unchanged confidence as a bias indicator
- Respect the user's expertise while testing it

### Do Not
- Generate weak straw-man arguments
- Use generic objections ("what if it doesn't scale")
- Accept dismissive non-responses as engagement
- Let the exercise become adversarial in tone — it's analytical, not personal
- Continue generating more than 3 counter-arguments (the technique loses power past 3)
