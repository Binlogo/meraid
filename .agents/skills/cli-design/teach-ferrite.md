---
description: >
  Teach Ferrite the project's CLI context once, then save it to `.ferrite.md` so future
  Ferrite commands can tailor recommendations to the real audience, tone, and constraints.
user-invocable: true
---

Use this as a one-time setup command near the start of working on a CLI codebase.

## Workflow

1. Read `Cargo.toml`, CLI entry points, shared output modules, and any existing docs or screenshots that reveal the current terminal style.
2. Ask exactly 5 targeted questions covering:
   - Audience
   - Environment
   - Tone
   - Constraints
   - Existing patterns worth preserving
3. Distill the answers into a concise `.ferrite.md` file in the repository root.
4. Keep the file practical: short bullets, not a manifesto.

## `.ferrite.md` format

Use this structure:

```markdown
# Ferrite Project Context

- Audience:
- Environment:
- Tone:
- Constraints:
- Existing patterns to preserve:
- Commands that must stay pipe-safe:
- Notes:
```

## Important

All other Ferrite commands should read `.ferrite.md` first if it exists. Treat it as local context, not as a replacement for reading the code.
