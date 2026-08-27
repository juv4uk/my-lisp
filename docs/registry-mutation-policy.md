# Registry Mutation Policy

**Status:** ACTIVE (Codified 2026-08-27)

**Motivation:** Incremental string-surgery (e.g., regex replacements, `sed` hacks, or partial string splits) has caused multiple file corruption incidents in registries and configuration files (such as `tasks.my`, `manifest.scm`, or Lisp metadata).

## The Rule: Rebuild from Clean Base

Agents must **NEVER** use incremental string surgery to mutate structured registries or databases. 

When modifying a registry, you must follow this exact sequence:

1. **Read full file**: Load the entire contents into memory.
2. **Extract valid entries**: Parse the file using a robust structural parser (e.g., a proper S-expression parser, JSON parser, or strict block-level text extraction).
3. **Mutate in memory**: Apply the required additions, modifications, or deletions to the parsed data structures directly.
4. **Rebuild entire file**: Serialize the entire data structure back into the canonical textual format.
5. **Verify balance**: Before overwriting the original file on disk, verify that the new output is structurally sound (e.g., perfectly balanced parentheses, valid syntax).

By strictly adhering to this pattern, the ecosystem eliminates the risk of mismatched delimiters, malformed trees, and partial data loss caused by naive string replacements.
