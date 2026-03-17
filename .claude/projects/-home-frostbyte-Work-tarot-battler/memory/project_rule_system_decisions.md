---
name: Rule system design decisions
description: Decisions on rule conditions - companion targeting, stat queries, comparators, target reference
type: project
---

Rule condition design decisions (2026-03-16):

- **Companion/ally conditions**: "any" semantics — if any companion/ally matches the condition, the rule triggers. That matching character is also a valid target for the ability if it targets companions/allies.
- **Queryable values**: All 9 stats (effective values) plus `hp` and `spi` as special runtime values.
- **Comparators**: `>=` and `<=` only (no exact equality, no strict `>` or `<`).
- **Target stat reference**: Refers to the character's current sticky target.

**Why:** Keep the initial rule system simple but expressive enough for the Emperor example rules (Crush when target HP <= 3, Embolden when any companion SPI < 2).
**How to apply:** Use these constraints when implementing the rule condition evaluation system.
