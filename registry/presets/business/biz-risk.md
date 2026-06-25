---
name: biz-risk
description: "Business-first risk evaluation before writing code. Forces quantification of how a technical decision affects revenue, users, or market position within 30 days. Triggers on: 'biz risk', 'business value', 'is this worth it', 'prioritize', 'ROI'. No code without a business case."
---

# Biz Risk — Business Impact Gate

Evaluate whether a technical decision, feature, or project has measurable business impact within 30 days. If it doesn't, challenge whether the work should happen at all.

## When to Trigger

**Auto-trigger**: User is about to start coding without stating business impact, or when scope expands beyond the original business need.

**Explicit** (`/biz-risk`): User wants to prioritize features, evaluate a project's worth, or validate a technical investment.

## Constraints (NON-NEGOTIABLE)

1. **30-day horizon** — if the impact can't materialize in 30 days, it's a strategic bet, not a tactical priority
2. **Quantify or kill** — "it will be better" is not acceptable; demand numbers
3. **Opportunity cost is real** — time spent on X is time NOT spent on Y; make this explicit
4. **No "it will help later"** — future value is speculation; present value is evidence
5. **Answer in Korean** unless user writes in English

## Process

### Step 1: State the investment

Ask the user to describe what they're about to build/change:

> "I am investing **[N hours/days]** to **[build/change X]**."

### Step 2: Impact quantification

Ask these questions in order. Each must have a concrete answer:

| Question | Acceptable | Rejected |
|----------|-----------|----------|
| **Who uses this?** | Named user segment or persona | "Everyone", "Future users" |
| **What changes for them?** | Specific behavior or metric change | "Better experience", "Cleaner code" |
| **By how much?** | Number, percentage, or rate | "A lot", "Significantly" |
| **When do they notice?** | Within 30 days / Next release | "Eventually", "In the long run" |
| **What if we don't do this?** | Concrete negative outcome | "Code won't be as clean" |

### Step 3: Opportunity cost

State explicitly:

> "The **[N hours]** spent on this is **[N hours]** NOT spent on **[highest-priority alternative]**."

Ask: "Is this the highest-ROI use of your time right now?"

If the answer is anything other than a confident "yes" with evidence → **CAUTION** or **STOP**.

### Step 4: Score

Rate on three axes:

| Axis | 3 (High) | 2 (Medium) | 1 (Low) | 0 (None) |
|------|----------|------------|---------|----------|
| **Revenue impact** | Direct revenue change | Indirect revenue enabler | Long-term revenue potential | No revenue connection |
| **User impact** | Fixes critical user pain | Improves existing experience | Nice-to-have improvement | Internal-only change |
| **Time sensitivity** | Must ship this week | Should ship this month | No deadline pressure | Can ship anytime |

**Total score interpretation:**
- **7–9**: PROCEED — clear business case
- **4–6**: CAUTION — marginal, consider alternatives
- **0–3**: STOP — no business justification; reframe as learning or drop

### Step 5: Record

```bash
mkdir -p biz-risk
```

Write to `biz-risk/EVAL-{timestamp}.md`:

```markdown
---
investment: {N hours/days}
feature: {what}
score: {total}/9
verdict: PROCEED | CAUTION | STOP
created: {ISO-8601}
---

# Business Risk Evaluation: {feature}

## Investment
{N hours/days} to build/change {feature}

## Impact Analysis
- **Who**: {user segment}
- **What changes**: {specific change}
- **By how much**: {quantified}
- **When**: {timeframe}
- **If we don't**: {consequence}

## Opportunity Cost
{N hours} NOT spent on {alternative}

## Score
| Axis | Score | Justification |
|------|-------|---------------|
| Revenue | {0-3} | {why} |
| User | {0-3} | {why} |
| Time | {0-3} | {why} |
| **Total** | **{/9}** | |

## Verdict
{PROCEED/CAUTION/STOP}: {one-line justification}

## Recommendation
{What to do next — ship, defer, drop, or reframe}
```

## Anti-Rationalization

| Excuse | Rebuttal | Do instead |
|--------|----------|-----------|
| "This is infrastructure, it doesn't have direct users" | Infrastructure serves something. What? | Trace to the end user. If chain >3 hops, it's premature. |
| "It will make development faster" | By how much? For which features? When? | Quantify the developer-hours saved per month. |
| "I need this to ship the next feature" | Then the next feature is the business case, not this. | Evaluate the next feature instead. This is a dependency, not a product. |
| "Technical debt will kill us" | Is it killing you RIGHT NOW? Show the bug reports. | If no active incidents, it's preventive maintenance. Score accordingly. |
| "It's only a few hours" | A few hours × 10 "only a few hours" tasks = a week. | Compare to the week's total available hours. |

## Evidence Required

- [ ] Investment stated in hours/days
- [ ] All 5 impact questions answered concretely
- [ ] Opportunity cost explicitly stated
- [ ] Three-axis score calculated with justification
- [ ] Verdict assigned (PROCEED/CAUTION/STOP)
- [ ] Evaluation recorded to file

## Red Flags

- Can't name specific users who benefit
- "By how much" gets a vague answer after 2 attempts
- Opportunity cost question makes the user defensive
- Score is inflated by "it will help later" reasoning
- User argues with the scoring system instead of answering questions

## Agent Instructions

### Do
- Force quantification — "better" is not a number
- Name the highest-priority alternative explicitly for opportunity cost
- Accept "I want to learn this" as a valid motivation (but score it as 0 on all axes)
- Be direct about low scores — don't inflate to avoid discomfort
- Link score to action: 0-3 means stop, not "proceed with caution"

### Do Not
- Accept "technical excellence" as a business impact
- Let "future potential" count as present value
- Soften low scores with "but it's still good to do"
- Skip the opportunity cost step
- Confuse developer satisfaction with user impact
