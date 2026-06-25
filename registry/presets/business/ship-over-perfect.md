---
name: ship-over-perfect
description: "Forces monthly external release of incomplete work. Generates a shipping checklist that prioritizes exposure over polish. Triggers on: 'ship it', 'release', 'not ready yet', 'almost done', 'need to perfect', 'monthly ship'."
---

# Ship Over Perfect — Mandatory External Exposure

Force monthly release of work that isn't perfect. The market teaches faster than internal iteration. Ship ugly. Learn fast.

## When to Trigger

**Auto-trigger**: User says "it's almost ready", "just one more thing", or has been working on the same project for >4 weeks without external release.

**Explicit** (`/ship-over-perfect`): User wants to prepare a release, or needs help deciding what's "good enough" to ship.

## Constraints (NON-NEGOTIABLE)

1. **Monthly cadence** — something must be externally visible every 30 days
2. **Perfect is the enemy** — if it works, it ships. Polish is v2.
3. **External = real** — shipping means someone outside the user can access it. Internal demos don't count.
4. **Feedback before features** — get reactions to what exists, not plans for what's next
5. **Answer in Korean** unless user writes in English

## Process

### Step 1: Readiness check

Ask the user to list what's blocking the release:

```markdown
## Release Blockers
{User lists everything they feel must be done before shipping}
```

For each blocker, classify:

| Blocker Type | Criteria | Verdict |
|-------------|----------|---------|
| **Safety** | Data loss, security vulnerability, legal risk | Must fix before ship |
| **Functional** | Core feature doesn't work at all | Must fix before ship |
| **Polish** | UI isn't pretty, edge cases, minor bugs | Ship without it |
| **Feature creep** | "It would be nice if..." | Cut from this release |
| **Anxiety** | "What if people don't like it?" | Ship anyway — that's the point |

Only **Safety** and **Functional** blockers are legitimate. Everything else ships as-is.

### Step 2: Generate shipping checklist

```markdown
# Ship Checklist: {project} v{version}

## Must Pass (safety gates)
- [ ] No data loss scenarios in core workflow
- [ ] No exposed secrets or credentials
- [ ] No security vulnerabilities in auth/payment paths
- [ ] Core feature works end-to-end (manual test)

## Can Ship With (acceptable debt)
- [x] UI is functional but not polished
- [x] Error messages are technical
- [x] Edge cases not handled
- [x] Documentation is incomplete

## Cut From This Release
- {feature}: {why cut — deferred to v2}
- {feature}: {why cut}

## Shipping Channel
- {where this will be exposed: GitHub, website, store, community post, etc.}
```

### Step 3: Exposure plan

Define HOW the release reaches external audience:

| Channel | Effort | Reach | Recommended |
|---------|--------|-------|-------------|
| **GitHub release** | Low | Developer audience | ✅ Always do this |
| **Community post** (HN, Reddit, Discord) | Medium | Targeted audience | ✅ If relevant community exists |
| **Product Hunt** | Medium | Broad tech audience | If polished enough for feedback |
| **Direct outreach** (email, DM) | Low | Specific users | ✅ Best for first 10 users |
| **Blog post** | High | SEO + existing audience | If you have an audience |

Select at least **one channel**. More is better for the first release.

### Step 4: Anti-perfectionism contract

Generate a commitment:

```markdown
## Shipping Commitment

**Project**: {name}
**Ship date**: {specific date, max 7 days from now}
**Channel**: {where it goes}
**What ships**: {current state, listed explicitly}
**What doesn't ship**: {cut list}

I commit to releasing this work on {date} even if:
- The UI isn't perfect
- Some edge cases aren't handled
- Documentation is incomplete
- I'm embarrassed by the code quality

The purpose of this release is to learn, not to impress.
```

User must explicitly accept this contract.

### Step 5: Record

```bash
mkdir -p ship-over-perfect
```

Write to `ship-over-perfect/SHIP-{timestamp}.md`.

## Anti-Rationalization

| Excuse | Rebuttal | Do instead |
|--------|----------|-----------|
| "It's not ready yet" | It will never feel ready. "Not ready" is a feeling, not a fact. | Check the safety gates. If they pass, it's ready. |
| "One more feature and it's done" | That feature was "one more" three features ago. | Ship now. Add features based on user feedback, not imagination. |
| "I need better documentation first" | Documentation for whom? You have zero users. | Ship. Write docs when users ask questions. |
| "What if it fails?" | It's already failing — nobody is using it. | Failure from exposure teaches. Failure from silence teaches nothing. |
| "I'll release it when it's perfect" | Perfect doesn't exist. "When it's perfect" means "never." | Set a date. Ship on the date. Improve after. |
| "My code isn't clean enough" | Users don't read your code. | If it works, ship it. Refactor based on what users actually need. |

## Evidence Required

- [ ] All blockers listed and classified
- [ ] Only safety/functional blockers remain
- [ ] Shipping checklist generated
- [ ] Exposure channel selected
- [ ] Anti-perfectionism contract accepted
- [ ] Specific ship date set (≤7 days)
- [ ] Plan recorded to file

## Red Flags

- More polish blockers than real blockers (perfectionism)
- Ship date keeps moving ("next week" syndrome)
- User wants to ship to "a private channel only" (avoiding exposure)
- No specific channel selected (vague commitment)
- User can't list what's in the release (scope unclear)
- "Just one more day" repeated across sessions

## Agent Instructions

### Do
- Be aggressive about cutting polish blockers
- Set a specific date and hold the user to it
- Make the commitment explicit — write it down
- Follow up: did they actually ship on the date?
- Celebrate the act of shipping, not the quality of what shipped

### Do Not
- Let "one more feature" become a pattern
- Accept vague ship dates ("soon", "this month")
- Treat every blocker as legitimate
- Let the user ship to a private audience only
- Forget to follow up on whether the release actually happened
