# CLAUDE.md

@AGENTS.md

Quick rules for `opencad-autolisp-engine`:

- Keep `crates/lisp-core` host-agnostic; express host CAD interactions cleanly through traits in `crates/lisp-cad` rather than building host-specific workarounds in the engine.
- If a fix belongs in `OpenCADStudio` or `opencad-autolisp-and-ide`, implement or link the paired PR there instead of adding workarounds here.
- Adhere to established CAD standards, DXF/DWG conventions, and standard AutoLISP semantics.
- Never include names of third-party or proprietary CAD software, add-on libraries, or vendors in code, docs, commits, or PR descriptions.
- Keep diffs tightly scoped (no bulk `rustfmt`), return structured errors rather than panicking on script/FFI input, run `cargo test --workspace`, and fill out `.github/PULL_REQUEST_TEMPLATE.md`.
