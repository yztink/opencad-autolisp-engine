# Contributing to OpenCAD AutoLISP Engine

Thank you for contributing! This guide summarizes the conventions for pull requests in `opencad-autolisp-engine` (see [AGENTS.md](./AGENTS.md) for detailed instructions).

## Workspace Overview

| Crate | Purpose |
|---|---|
| `crates/lisp-core` | Core lexer, parser, evaluator, and standard AutoLISP functions |
| `crates/lisp-cad` | Host-agnostic CAD bridge traits (`CadBridge`) and CAD built-ins |
| `crates/lisp-repl` | Standalone REPL binary |
| `crates/lisp-c-api` | C FFI bindings |
| `crates/lisp-cli` | CLI script runner |

## Core Principles

1. **Respect layer boundaries — avoid workarounds.**
   Keep `crates/lisp-core` independent of OpenCAD Studio host IPC and UI internals. When an issue requires host changes in `OpenCADStudio` or plugin wiring in `opencad-autolisp-and-ide`, update the authoritative repo (and link paired PRs) rather than adding workarounds in the engine or asking consumers to work around engine bugs.
2. **Adhere to established CAD standards.**
   Match standard AutoLISP language semantics and DXF/DWG entity, group-code, geometry, and coordinate conventions. Keep `AUTOLISP_REFERENCE.md` and `AUTOLISP_REFERENCE.html` updated when language behavior changes.
3. **Keep naming clean.**
   Never mention names of third-party or proprietary CAD software products, commercial add-on libraries, or CAD vendors in code, comments, docs, commit messages, or PR descriptions.
4. **Write clean, safe Rust.**
   Return structured errors (`LispError` / `Result`) instead of panicking on script or FFI inputs, and keep diffs focused without unrelated formatting or bulk `rustfmt`.

## Testing & Pull Requests

1. Verify the entire workspace test suite passes:
   ```bash
   cargo test --workspace
   ```
2. Fill out [.github/PULL_REQUEST_TEMPLATE.md](./.github/PULL_REQUEST_TEMPLATE.md), including **Problem & Why This PR Is Needed**, summary of changes, downstream `opencad-autolisp-and-ide` / `OpenCADStudio` coordination notes, and test results.
