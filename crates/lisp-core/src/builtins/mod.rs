pub mod clipboard;
pub mod io;
pub mod list;
pub mod math;
pub mod pred;
pub mod string;
pub mod vl;

use crate::ast::Value;
use crate::env::Env;

pub fn register_all(env: &mut Env) {
    math::register(env);
    pred::register(env);
    list::register(env);
    string::register(env);
    vl::register(env);
    io::register(env);
    clipboard::register(env);

    // Standard OpenCAD LISP version query
    env.def_subr("VER", |_, _| {
        Ok(Value::Str("OpenCAD LISP 2026 (opencad-lisp-engine native Rust engine v0.1.0)".into()))
    });
    env.def_subr("VL-VERSION", |_, _| {
        Ok(Value::Str("OpenCAD LISP 2026 (opencad-lisp-engine native Rust engine v0.1.0)".into()))
    });
    env.def_subr("LISP-IMPLEMENTATION-TYPE", |_, _| {
        Ok(Value::Str("opencad-lisp-engine (Rust)".into()))
    });

    // Error & Execution Flow
    env.def_subr("ERROR", |args, _| {
        let msg = if !args.is_empty() {
            match &args[0] {
                Value::Str(s) => s.clone(),
                other => other.to_string(),
            }
        } else {
            "explicit error".to_string()
        };
        Err(crate::error::LispError::UserError(msg))
    });
    env.def_subr("EXIT", |_, _| Err(crate::error::LispError::UserExit));
    env.def_subr("QUIT", |_, _| Err(crate::error::LispError::UserExit));

    // OpenCAD LISP predefined symbols
    env.set_local(crate::ast::Symbol::new("PAUSE"), Value::Str("\\".into()));
}
