---
description: >
  Critique a Rust CLI conceptually when the user wants a senior design review of
  its command structure, messaging, and interaction model before or after code changes.
user-invocable: true
---

Use this command when the user wants judgment, not just a lint list. Focus on whether the CLI feels trustworthy and coherent.

If `.ferrite.md` exists, read it first.

## Review lenses

### Mental Model
- What does the tool appear to believe the user is trying to accomplish?
- Does the command structure reflect goals or internal implementation details?
- Are command names and summaries legible to someone who did not write the crate?

### Information Architecture
- Is the most important information surfaced first?
- Are details layered beneath outcomes instead of dumped in arrival order?
- Do help text and summaries use a stable vocabulary?

### Interaction Flow
- Are prompts timed appropriately, with sensible defaults and bypass flags?
- Does long-running work give calm, truthful feedback?
- Does the command avoid unnecessary interruption or theatrical output?

### Trust & Reliability
- Are errors actionable, specific, and correctly routed to stderr?
- Does the CLI behave predictably in CI, pipes, and partial failure cases?
- Are colors and symbols stable enough that users can learn them?

### Emotional Journey
- How does the tool feel at the start, during progress, on success, and on failure?
- Does it become calmer or more chaotic under stress?
- Does the interface feel written by someone who has lived in terminals for years?

## Output format

Use this structure exactly:

### Overall Impression
One paragraph describing the CLI's current design character.

### Top 3 Improvements
1. Improvement
   - Why it matters
   - Concrete change to make
2. Improvement
   - Why it matters
   - Concrete change to make
3. Improvement
   - Why it matters
   - Concrete change to make

### What Works
List the strongest existing decisions worth preserving.
