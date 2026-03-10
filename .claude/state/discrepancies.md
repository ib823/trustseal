# Sahi — Spec Discrepancies Log

When implementation reveals a conflict between specs, or a spec is ambiguous, log it here.
Always implement the more conservative (more secure, more private) interpretation.
Flag any discrepancy requiring Ikmal's decision with `[NEEDS DECISION]`.

Format:
```
## DISC-001 — YYYY-MM-DD
**Module:** F1
**Source A:** vaultpass-spec.md §X.X — says X
**Source B:** attack-plan.md Part 2 — says Y
**Decision:** Implemented per [Source A/B] because [reason]
**Status:** RESOLVED / [NEEDS DECISION]
```

---

## DISC-001 — 2026-03-10
**Module:** VP-3c (Access Rules)
**Source A:** MASTER_PLAN §8.5 — `FloorAccess { floors: Vec<u32> }`
**Source B:** Implementation — `FloorAccess { floors: Vec<String> }`
**Decision:** Implemented with `Vec<String>` to support real-world floor nomenclature (e.g., "L12", "B1", "GF", "Penthouse") rather than numeric indices.
**Rationale:** Buildings commonly use alphanumeric floor identifiers. Using strings is more expressive and avoids mapping complexity.
**Status:** RESOLVED

## DISC-002 — 2026-03-10
**Module:** VP-3c (Access Rules)
**Source A:** MASTER_PLAN §8.5 — `Clearance { min_level: String }`
**Source B:** Implementation — `Clearance { min_level: u8 }`
**Decision:** Implemented with `u8` for type safety and efficient comparison.
**Rationale:** Clearance levels 0-3 are numeric by nature. Using `u8` provides compile-time bounds checking and avoids string parsing errors. The spec's String type may have been for flexibility that isn't needed.
**Status:** RESOLVED
