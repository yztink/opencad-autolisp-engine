# Copilot Instructions

Follow [AGENTS.md](../AGENTS.md) when working in `opencad-lisp-engine`:

- Keep `crates/lisp-core` host-agnostic and define clean CAD boundaries in `crates/lisp-cad`. Fix issues in the authoritative repo (`OpenCADStudio`, `opencad-lisp-engine`, or `opencad-lisp-and-ide`) rather than introducing workarounds.
- Follow established CAD standards, DXF/DWG specifications, and standard OpenCAD LISP semantics.
- Never include names of third-party or proprietary CAD software products, add-on libraries, or CAD vendors in code, comments, docs, commits, or PR descriptions.
- Return structured errors (`LispError` / `Result`) rather than panicking on script or FFI input, keep diffs scoped (no bulk `rustfmt`), run `cargo test --workspace`, and follow `.github/PULL_REQUEST_TEMPLATE.md`.
