---
name: mvp-force
description: "Forces minimum viable implementation check — can the project ship at 50% of current technical capability within 3 days? Strips scope to survival-level features. Triggers on: 'MVP', 'minimum viable', 'scope check', 'too big', '3-day rule', 'strip scope'."
---

# MVP Force — Minimum Implementation Gate

Enforce the 3-day MVP rule: if you can't build a working version at 50% of your technical ability in 3 days, the design is wrong. No exceptions.

## When to Trigger

**Auto-trigger**: User describes a project plan with >5 features, or when scope exceeds what's needed for initial validation.

**Explicit** (`/mvp-force`): User wants to define minimum scope for a new project, or feels current scope has grown too large.

## Constraints (NON-NEGOTIABLE)

1. **3-day hard limit** — not 3 business days, not "about 3 days" — 72 hours
2. **50% capability rule** — use half your technical skill; no optimization, no polish, no elegance
3. **Survival features only** — if removing a feature means the product still works, it's cut
4. **No infrastructure first** — don't build the pipeline before the product
5. **Answer in Korean** unless user writes in English

## Process

### Step 1: List everything

Ask the user to list ALL planned features/capabilities. Write them down.

### Step 2: Survival filter

For each feature, ask:

> "If this feature is missing, does the product still solve the core problem?"

| Answer | Action |
|--------|--------|
| No — product is useless without it | **KEEP** |
| Product works but is awkward | **CUT** — add to v2 list |
| Product is fine, this is polish | **CUT** — add to v3 list |
| User is unsure | **CUT** — uncertainty means it's not survival-critical |

The survival filter must reduce the list to **≤3 features**. If more than 3 survive, re-run the filter with a stricter standard.

### Step 3: 3-day estimate

For each surviving feature:

> "Can you implement this in 1 day at 50% of your ability?"

| Answer | Action |
|--------|--------|
| Yes, easily | Keep — might be overthinking it |
| Yes, barely | Keep — this is the right size |
| No | Split the feature OR cut it — it's too complex for MVP |

If total estimate > 3 days, cut the lowest-priority survivor.

### Step 4: Generate the MVP spec

Output the MVP scope:

```markdown
# MVP Scope: {project name}

## Timeline
3 days (72 hours)

## Survival Features (max 3)
1. {feature} — {1-day implementation plan}
2. {feature} — {1-day implementation plan}
3. {feature} — {1-day implementation plan}

## Cut to v2
- {cut feature}: {why cut}
- {cut feature}: {why cut}

## Cut to v3
- {cut feature}: {why cut}

## Success Criterion
{One measurable outcome — "User can complete X in Y steps" or "System processes Z requests"}
```

### Step 5: Commitment check

Ask the user:

> "Can you commit to shipping this MVP in 3 days? Yes or no."

- **Yes** → Record the commitment. Set the deadline.
- **No** → The scope is still too large. Loop back to Step 2.
- **"Yes, but..."** → The "but" is scope creep in disguise. Return to Step 2.

### Step 6: Record

```bash
mkdir -p mvp-force
```

Write to `mvp-force/MVP-{timestamp}.md`.

## Anti-Rationalization

| Excuse | Rebuttal | Do instead |
|--------|----------|-----------|
| "The MVP wouldn't represent the product" | The MVP tests whether the product should exist at all. | Ship ugly. Learn fast. Rebuild later. |
| "Users will judge the quality" | You have zero users right now. Judgment requires an audience. | Get users first. Then worry about judgment. |
| "I need the infrastructure first" | Infrastructure without a product is a bridge to nowhere. | Build the product on whatever works. Replace infrastructure when it hurts. |
| "3 days isn't enough" | For 3 survival features at 50% effort, it is. If not, the features are too big. | Split features until each fits in a day. |
| "I'll do it properly from the start" | "Properly" is how you end up with nothing shipped after 3 months. | Ship improperly. Learn what "properly" actually means from users. |

## Evidence Required

- [ ] Full feature list documented
- [ ] Survival filter applied to every feature
- [ ] ≤3 survival features remain
- [ ] 3-day estimate passes for each survivor
- [ ] MVP spec generated
- [ ] User committed (yes/no) without qualifications
- [ ] MVP recorded to file

## Red Flags

- Survival filter keeps >5 features (too lenient)
- "This feature is essential" for everything (no discipline)
- 3-day estimate is padded with caveats
- User can't state a single measurable success criterion
- "Yes, but..." commitment (hidden scope creep)
- User tries to redefine MVP as "phase 1" with 10 features

## Agent Instructions

### Do
- Be ruthless with the survival filter — most features are wants, not needs
- Challenge every feature that survives: "Are you sure?"
- Force the user to commit to a deadline
- Celebrate cuts — each cut is a day saved
- Keep the v2/v3 lists visible — nothing is lost, just deferred

### Do Not
- Let "nice to have" features survive the filter
- Accept "3 days per feature" — it's 3 days TOTAL
- Allow infrastructure/boilerplate/setup to count as features
- Let the MVP spec grow beyond 3 features
- Confuse "simpler implementation" with "less valuable product"
