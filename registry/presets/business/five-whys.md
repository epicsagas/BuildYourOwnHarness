---
name: five-whys
description: "Forces 5-level root-cause questioning on any technical decision or justification. Reveals whether the real motive is business value or technical satisfaction. Triggers on: 'five whys', 'why', 'root cause', 'justify this', 'why this approach'. No surface-level answers accepted."
---

# Five Whys — Decision Root-Cause Drill

Interrogate a technical decision or justification through 5 consecutive "why" layers. Stop only when the answer reveals the true motive — or when it becomes clear the motive is self-referential.

## When to Trigger

**Auto-trigger**: User states a technical decision without explicit business justification, or when justification feels post-hoc.

**Explicit** (`/five-whys`): User wants to validate a decision before committing, or suspects their own reasoning is surface-level.

## Constraints (NON-NEGOTIABLE)

1. **No skipping levels** — every "why" must be answered before proceeding to the next
2. **No circular answers** — if answer references the question, flag it immediately
3. **No "because it's better"** — vague comparatives are rejected; demand specifics
4. **Terminate on "because I want to" or "because it's interesting"** — this IS the answer; don't go deeper
5. **Answer in Korean** unless user writes in English

## Process

### Step 1: State the decision

Ask the user to state the decision in one sentence:

> "I am choosing/doing **[X]** because **[reason].**"

If the user can't state it in one sentence, the decision isn't clear enough. Ask them to refine.

### Step 2: Begin the chain

For each level (1–5), ask:

> "Why **[previous answer]**?"

Record each answer. Classify the answer type:

| Answer Type | Signal | Verdict |
|-------------|--------|---------|
| **Business outcome** | Revenue, users, retention, cost reduction | ✅ Valid root — can stop early |
| **Technical necessity** | Performance, security, reliability requirement | ✅ Valid root — can stop early |
| **User need** | Solves a real user pain point | ✅ Valid root — can stop early |
| **Best practice** | "It's the standard way", "industry norm" | ⚠️ Weak — probe deeper |
| **Technical preference** | "Cleaner", "more elegant", "better DX" | 🔴 Alert — likely self-referential |
| **Interest/curiosity** | "I want to learn", "It's interesting" | 🔴 STOP — this IS the root |
| **Circular** | References the original decision | 🔴 STOP — no real justification |

### Step 3: Verdict

After the chain completes (or terminates early), deliver the verdict:

**PROCEED**: Root cause is a valid business/user need (levels 1-3 reached a concrete outcome).

**CAUTION**: Root cause is a best practice or technical preference. Ask: "Is there a simpler way to achieve the same outcome?"

**STOP**: Root cause is self-referential, or landed on "because I want to." Recommend either: drop the work, or reframe it explicitly as learning/exploration (not production work).

### Step 4: Record

```bash
mkdir -p five-whys
```

Write to `five-whys/CHAIN-{timestamp}.md`:

```markdown
---
decision: {one-line decision statement}
verdict: PROCEED | CAUTION | STOP
created: {ISO-8601}
---

# Decision: {decision}

## Chain

1. **Why?** {answer} — _{type}_
2. **Why?** {answer} — _{type}_
3. **Why?** {answer} — _{type}_
4. **Why?** {answer} — _{type}_
5. **Why?** {answer} — _{type}_

## Root Cause

{Final root — the deepest non-circular answer}

## Verdict

{PROCEED/CAUTION/STOP}: {one-line justification}

## Recommendation

{What to do next}
```

## Anti-Rationalization

| Excuse | Rebuttal | Do instead |
|--------|----------|-----------|
| "The reason is obvious" | Then stating it takes 10 seconds. State it. | Run the chain. Obvious reasons survive 5 whys easily. |
| "This is standard practice" | Standard practice has a reason. What is it HERE? | "Standard" is level 1. Go to level 2. |
| "I don't have time for this" | You have less time to build the wrong thing. | 2-minute exercise vs. days of wrong work. |
| "I already know why" | Then the chain will confirm it in 60 seconds. | Run it. If you're right, it reinforces confidence. |
| "It's just how I like it" | That IS a valid answer. Own it explicitly. | Label it "technical preference" and accept the verdict. |

## Evidence Required

- [ ] Decision stated in one sentence
- [ ] At least 3 "why" levels completed
- [ ] Each answer classified by type
- [ ] Root cause identified (non-circular)
- [ ] Verdict assigned (PROCEED/CAUTION/STOP)
- [ ] Chain recorded to file

## Red Flags

- User resists stating the decision in one sentence
- Answers get shorter and vaguer after level 3 (hiding)
- User changes the decision mid-chain (moving target)
- "Why" questions make the user defensive
- Circular reasoning at level 2+ (decision justifies itself)
- Skipping levels to reach a "safe" answer

## Agent Instructions

### Do
- Push through discomfort — the chain's value IS the discomfort
- Name answer types explicitly ("That's a technical preference, not a business need")
- Accept "because I want to" as a valid and honest answer
- Stop early if a clear business/user root is found (no need for all 5)
- Link the verdict to the root cause directly

### Do Not
- Accept vague answers without classification
- Let circular reasoning pass unflagged
- Continue past 5 levels (the technique breaks down)
- Soften the verdict — if it's STOP, say STOP
- Add commentary about the user's reasoning ability
