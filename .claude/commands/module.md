# /module — Start or resume a specific module

Usage: `/module F1` or `/module VP-4` or `/module TM-2`

When this command is run:

1. Read `.claude/state/current.md` to check if this module is already in progress or completed
2. Read `.claude/state/completed.md` to check dependencies are met
3. Read the module specification from `specs/attack-plan.md` — find the section for this module
4. Cross-reference with the source spec (vaultpass-spec.md or trustmark-spec.md)
5. List every acceptance criterion for this module
6. Report what is already built (check the relevant package/app directory)
7. State exactly what will be built next and ask for confirmation before starting

Do not write any code until you have completed steps 1-7 and received confirmation.

Output format:
```
## Module: [MODULE_ID]
**Phase:** [0/1/2/3/4]
**Sprint:** [sprint number]
**Dependencies met:** [yes/no — list any unmet deps]

### Acceptance criteria:
1. [criterion from attack plan]
2. ...

### What exists already:
- [file/function that already exists]

### What will be built:
- [specific deliverable]

Ready to proceed? (yes/no)
```
