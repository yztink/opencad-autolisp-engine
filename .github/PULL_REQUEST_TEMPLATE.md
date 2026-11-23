## Problem & Why This PR Is Needed

<!-- Describe the problem, missing language/bridge capability, or spec discrepancy and why this change is needed. Link related issues if applicable. -->

## Summary of Changes

<!-- Concisely explain what changed across crates (`lisp-core`, `lisp-cad`, `lisp-repl`, `lisp-c-api`, `lisp-cli`) and any reference doc updates. -->

## Cross-Repo Dependencies

<!-- Does this change require paired updates in `opencad-lisp-and-ide` or `OpenCADStudio`? Link downstream/upstream PRs or state "None". -->
- None

## Verification

<!-- List exact test commands run (e.g., `cargo test --workspace`). -->
- [ ] `cargo test --workspace` passes

## Checklist

- [ ] Changes live in the proper crate/repository layer without unnecessary workarounds and preserve `lisp-core` host independence
- [ ] Language semantics and CAD interfaces adhere to established CAD standards and DXF/DWG conventions
- [ ] No names of third-party or proprietary CAD software, add-on libraries, or CAD vendors appear in code, comments, docs, commits, or this PR description
- [ ] Error handling returns structured results rather than panicking on untrusted script/FFI input
- [ ] Diff is tightly scoped with no unrelated reformatting
