# AGENTS.md

Repository instructions for AI coding agents working in `opencad-autolisp-engine`.

## Architecture & Crate Boundaries

This workspace provides the host-agnostic Rust AutoLISP engine consumed by `opencad-autolisp-and-ide` and standalone tools:

| Crate | Responsibility |
|---|---|
| `crates/lisp-core` | Lexer, parser, AST/values, evaluator, special forms, and standard AutoLISP runtime library |
| `crates/lisp-cad` | Host-agnostic CAD bridge trait definitions (`CadBridge` and CAD data/geometry/selection/entity abstractions) and CAD built-ins |
| `crates/lisp-repl` | Interactive REPL binary for standalone evaluation and testing |
| `crates/lisp-c-api` | C FFI bindings over the engine |
| `crates/lisp-cli` | Command-line runner for scripts and batch verification |

- Keep `crates/lisp-core` decoupled from host CAD IPC, OpenCAD Studio internals, and GUI frameworks. Host integrations belong behind clean traits in `crates/lisp-cad` and implementations in `opencad-autolisp-and-ide`.
- Note: The plugin consumer's `archive/python_shelved/` is frozen; all AutoLISP evaluation is implemented here in Rust.

## Fix the Right Layer — No Workarounds

- Implement language, evaluator, standard library, and `CadBridge` contract fixes here in `opencad-autolisp-engine` when this engine is the authoritative layer — do not expect downstream consumers (`opencad-autolisp-and-ide`) to patch around engine limitations.
- Conversely, if an issue belongs in the host application (`OpenCADStudio`) or plugin runtime (`opencad-autolisp-and-ide`), fix it in that repository (or coordinate a paired PR) rather than adding brittle assumptions, host-specific hacks, timing sleeps, or fallback workarounds inside `lisp-core` or `lisp-cad`.
- Document any cross-repo contract or downstream consumer changes clearly in your PR description.

## CAD Standards & Language Fidelity

- Follow established CAD standards, DXF/DWG entity/object models and group-code specifications, and standard AutoLISP language semantics.
- Maintain consistent coordinate, geometry, unit, selection, and interactive error-handling behavior across engine built-ins and bridge traits.
- Keep reference documentation (`AUTOLISP_REFERENCE.md` / `AUTOLISP_REFERENCE.html`) accurate when public functions or language behavior change.

## Naming & Trademark Hygiene

- **Never** mention the names of third-party or proprietary CAD software products, commercial add-on libraries, or CAD vendors anywhere in code, comments, identifiers, docstrings, tests, documentation, commit messages, or PR descriptions.
- Refer strictly to AutoLISP, DXF/DWG, OpenCAD Studio, and established CAD standards.

## Engineering & PR Best Practices

- **Tightly scoped diffs**: Keep changes focused; preserve unrelated comments and formatting, and never run bulk `rustfmt` over untouched files.
- **Safe error handling**: Use structured `LispError` / `Result` values for script, type, arity, and bridge failures; avoid `unwrap()` / `expect()` on runtime or FFI inputs.
- **API & FFI stability**: Keep `CadBridge` trait changes clear and well-tested; preserve FFI soundness in `crates/lisp-c-api`.
- **Verification**: Run the full workspace test suite before finishing:
  ```bash
  cargo test --workspace
  ```
- **PR quality**: Complete `.github/PULL_REQUEST_TEMPLATE.md` with the problem statement, rationale, architectural layer justification, cross-repo impacts, and test results.
