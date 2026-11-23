# OpenCAD LISP Engine (`opencad-lisp-engine`)

> [!WARNING]
> **Status: Extreme Alpha / Active Development**  
> This project is in an early experimental stage and under active development. Language coverage, CAD bridge primitives, and internal interfaces are subject to rapid evolution.

High-performance, memory-safe, cross-platform OpenCAD LISP, LISP extension (`vl-*`), and Object Model (`vlax-*`) runtime engine written in Rust for **OpenCAD Studio** and standalone CAD scripting.

---

## Overview

`opencad-lisp-engine` provides a complete, authoritative implementation of the OpenCAD LISP programming language. It includes a pure Lisp runtime, CAD bridge abstraction layer, interactive terminal REPL, C-compatible shared library (`opencad_lisp.dll`), and standalone CLI runner.

---

## Workspace Crates

| Crate | Purpose |
|---|---|
| **`lisp-core`** | Pure Lisp runtime: lexer, reader/parser, AST, tree-walking evaluator, lexical scoping, closures, tail recursion, error diagnostics with stack traces, and standard `vl-*` string, list, math, and file utilities. Zero external CAD dependencies. |
| **`lisp-cad`** | CAD domain integration: `(command ...)`, `(entmake ...)`, `(entget ...)`, `(entmod ...)`, `(ssget ...)`, `(getpoint ...)`, `(tblsearch ...)`, `(tblnext ...)`, curve and lightweight polyline geometry, `(defun c:...)` command auto-registration, `oc-panel-*` dynamic GUI sidebars, and DCL dialogs via the `CadBridge` trait. |
| **`lisp-repl`** | Interactive REPL services: terminal line editing, bracket balancing, history management, and JSON-RPC IDE server protocol. |
| **`lisp-c-api`** | C-compatible shared library (`opencad_lisp.dll` / `.so`) and C header (`opencad_lisp.h`) for embedding the engine into foreign CAD plugins or C/C++ applications. |
| **`lisp-cli`** | Standalone CLI binary (`oclisp`) for terminal REPL and batch script execution. |

---

## Features

- **Language Completeness**:
  - Full arithmetic, logic, conditional forms (`cond`, `if`, `when`, `unless`), and loops (`while`, `repeat`, `foreach`).
  - List manipulation (`cons`, `car`, `cdr`, `assoc`, `nth`, `mapcar`, `apply`, `append`, `reverse`, `subst`).
  - Special forms: `defun`, `lambda`, `quote`, `setq`, `progn`, `let`.
  - S-expression breakpoints and stepping hooks for IDE debuggers.
- **Extended LISP & Utility Library**:
  - Comprehensive `vl-*` file manipulation (`vl-file-copy`, `vl-file-delete`, `vl-file-size`, `vl-directory-files`).
  - String functions (`vl-string-search`, `vl-string-subst`, `vl-string-trim`, `vl-string->list`).
  - Error catching (`vl-catch-all-apply`, `vl-catch-all-error-p`, `vl-catch-all-error-message`).
- **CAD Domain Integration**:
  - Decoupled from host implementations via the `CadBridge` trait.
  - Entity inspection and modification using DXF dotted pairs.
  - Interactive prompts (`getpoint`, `getdist`, `getangle`, `getstring`, `getint`, `getkword`).
  - Symbol table iteration and queries across layers, line types, text styles, viewports, UCS, and blocks.
  - Dynamic UI panels and DCL dialogs.

---

## Building & Testing

### Prerequisites
- Rust 1.80+ (edition 2021)

### Build
```bash
cargo build --workspace
```

### Run All Tests
```bash
cargo test --workspace
```
Runs 58 comprehensive tests covering:
- Core arithmetic, scoping, error stacks, and recursion depth limits.
- Extended LISP functions and string operations.
- Polyline curve geometry, arc interpolation, and coordinate transformations.
- Symbol table search and iteration (`tblsearch`, `tblnext`).
- Selection set operations (`ssget`, `sslength`, `ssname`).
- Dynamic UI panel and dialog definition roundtrips.

---

## Standalone CLI Usage

The `lisp-cli` crate produces the `oclisp` command-line binary:

```bash
# Start an interactive REPL session:
cargo run -p lisp-cli

# Execute an OpenCAD LISP file:
cargo run -p lisp-cli -- path/to/script.lsp
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for pull request guidelines, repository boundaries, CAD conventions, and testing expectations. AI coding tools should also follow [AGENTS.md](AGENTS.md).

---

## License

GNU General Public License v3.0 (GPL-3.0-only).
