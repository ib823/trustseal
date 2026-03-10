# /spec — Look up specification for a module, topic, or screen

Usage: `/spec F1` or `/spec SD-JWT` or `/spec VP-S3` or `/spec PAdES`

When this command is run:

1. Search across all spec files in `specs/` for the requested topic
2. Return the relevant sections with their source file and section number
3. If multiple sources cover the topic, show all of them ranked by authority:
   - Source specs (vaultpass-spec.md, trustmark-spec.md) first
   - Attack plan second
   - Batch docs third

Output format:
```
## Spec lookup: [topic]

### From: specs/[filename] — §[section]
[relevant content]

### From: specs/[filename] — §[section]
[relevant content]

### Cross-reference notes:
[any discrepancies or things to watch out for]
```

This is a read-only command — it never modifies files.
