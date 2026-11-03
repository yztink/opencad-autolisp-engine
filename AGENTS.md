# OpenCAD AutoLISP Engine (`opencad-autolisp-engine`) — Agent Instructions

## Architecture & Scope

1. **Authoritative AutoLISP Implementation**:
   - `crates/lisp-core`: Lexer, parser, evaluator, special forms, and core AutoLISP functions.
   - `crates/lisp-cad`: CAD entity functions, selection sets, system variables, `vl-*` / `vlax-*`, `dos_*`, `ocl-panel-*` GUI panels, DCL dialogs, and the `CadBridge` trait.
   - `../opencad-autolisp-and-ide/src/rust_lisp.rs`: Implements `CadBridge` (`OpenCadRustBridge`) against OpenCAD Studio's `HostApi` IPC.

2. **Python is Shelved / Frozen**:
   - All AutoLISP execution in OpenCAD Studio and OCAIDE goes **straight to `opencad-autolisp-engine` (`rust_lisp.rs`)** and never through Python.
   - Do **NOT** touch `../opencad-autolisp-and-ide/archive/python_shelved/` when adding or modifying AutoLISP functions.
