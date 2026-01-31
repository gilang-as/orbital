# Claude Repository Instructions

This repository uses AI-first, code-verified documentation.

GLOBAL RULES

- Code is the source of truth
- Documentation must reflect actual implementation
- Documentation must be deterministic and explicit
- Outdated documentation must be deleted, not archived
- No duplicate or overlapping documents are allowed

DOCUMENTATION POLICY

- All documentation must:
  - Be validated against code
  - Clearly state implementation status
  - Avoid assumptions unless explicitly stated
- Legacy documentation must not coexist with new documentation
- When documentation is replaced, old files must be removed

AI-READABILITY STANDARD

All documents should follow predictable structure:
- Purpose
- Scope
- Responsibilities
- Inputs / Outputs (if applicable)
- Constraints
- Current Status
- TODO / Future Work

DEVELOPMENT PHASE AWARENESS

- Each major system must indicate:
  - Implemented features
  - Partially implemented features
  - Planned features
- Development phases must be explicit and up to date

WHEN MODIFYING DOCUMENTATION

- Prefer rewrite over edit
- Prefer deletion over preservation
- Avoid historical explanations unless required
- Ensure no ambiguous or duplicated content remains

ASSUMPTIONS

- If information is missing, infer carefully
- Always state assumptions explicitly
- Never invent behavior not supported by code

This file is authoritative.
Claude must follow these rules when interacting with this repository.
