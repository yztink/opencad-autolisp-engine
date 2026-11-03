pub mod ast;
pub mod builtins;
pub mod env;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use ast::{Cons, ExprSpan, Symbol, UserFunction, Value};
pub use env::Env;
pub use error::{
    finalize_error_diagnostic, get_last_error_diagnostic, set_debug_breakpoints,
    set_debug_pause_callback, set_debug_step_mode, DebugEventKind, DebugPauseCallback, DebugPauseInfo,
    DebugResumeAction, DebugStepMode, ErrorDiagnostic, ErrorFrame, LispError,
};
pub use eval::{apply, apply_with_depth, eval, eval_with_depth, MAX_CALL_DEPTH};
pub use lexer::{tokenize, tokenize_spanned, Span, SpannedToken, Token};
pub use parser::{parse, parse_one};

/// Creates a standard AutoLISP environment loaded with all core and Visual LISP built-ins.
pub fn create_default_env() -> Rc<RefCell<Env>> {
    let env = Env::new();
    builtins::register_all(&mut env.borrow_mut());
    env
}

pub type OutputSink = Box<dyn Fn(&str) + Send + Sync>;
static OUTPUT_SINK: std::sync::Mutex<Option<OutputSink>> = std::sync::Mutex::new(None);

thread_local! {
    static IN_ERROR_HANDLER: Cell<bool> = const { Cell::new(false) };
}

/// Set a global hook for AutoLISP standard output functions (`princ`, `prin1`, `print`, `prompt`, `write-line`).
/// When None (the default), output is written to `std::io::stdout()`.
pub fn set_output_sink(sink: Option<OutputSink>) {
    *OUTPUT_SINK.lock().unwrap() = sink;
}

/// Write output to the current sink or stdout.
pub fn write_output(s: &str) {
    if let Ok(guard) = OUTPUT_SINK.lock() {
        if let Some(sink) = guard.as_ref() {
            sink(s);
            return;
        }
    }
    use std::io::Write;
    print!("{}", s);
    let _ = std::io::stdout().flush();
}

/// Parses and evaluates an AutoLISP source string in the given environment.
/// Automatically executes the user-defined `*error*` handler if an evaluation error occurs.
pub fn eval_str(source: &str, env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    error::clear_unwind_state();
    let exprs = match parse(source) {
        Ok(v) => v,
        Err(e) => {
            error::finalize_error_diagnostic(&e);
            return Err(e);
        }
    };
    let mut last_val = Value::Nil;
    for expr in exprs {
        error::clear_unwind_state();
        match eval(&expr, env) {
            Ok(val) => last_val = val,
            Err(e) => {
                if matches!(e, LispError::UserExit) {
                    error::clear_unwind_state();
                    return Err(e);
                }

                let diag = error::finalize_error_diagnostic(&e);

                // If not already inside *error*, check if *ERROR* is bound
                let in_handler = IN_ERROR_HANDLER.with(|c| c.get());
                if !in_handler {
                    let err_sym = Symbol::new("*ERROR*");
                    let err_handler = env.borrow().get(&err_sym);
                    if let Some(handler) = err_handler {
                        if !handler.is_nil() {
                            IN_ERROR_HANDLER.with(|c| c.set(true));
                            let err_msg = Value::Str(match &e {
                                LispError::UserError(msg) => msg.clone(),
                                other => other.to_string(),
                            });
                            let _ = apply(&handler, &[err_msg], env);
                            IN_ERROR_HANDLER.with(|c| c.set(false));
                            // Ensure the original error diagnostic remains the active last diagnostic
                            error::clear_unwind_state();
                            let _ = diag;
                        }
                    }
                }

                return Err(e);
            }
        }
    }
    Ok(last_val)
}

/// Parses and evaluates an AutoLISP source string without attaching source spans to top-level expressions.
/// Useful for synthetic command wrappers like `(c:mycmd)` so they do not trigger `(1, 1)` breakpoints,
/// while preserving all spans inside the user functions they invoke.
pub fn eval_str_unspanned(source: &str, env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    error::clear_unwind_state();
    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            error::finalize_error_diagnostic(&e);
            return Err(e);
        }
    };
    let spanned: Vec<SpannedToken> = tokens
        .into_iter()
        .map(|token| SpannedToken {
            token,
            span: Span { line: 0, col: 0 },
        })
        .collect();
    let mut parser = parser::Parser::from_spanned(spanned);
    let exprs = match parser.parse_all() {
        Ok(v) => v,
        Err(e) => {
            error::finalize_error_diagnostic(&e);
            return Err(e);
        }
    };
    let mut last_val = Value::Nil;
    for expr in exprs {
        error::clear_unwind_state();
        match eval(&expr, env) {
            Ok(val) => last_val = val,
            Err(e) => {
                if matches!(e, LispError::UserExit) {
                    error::clear_unwind_state();
                    return Err(e);
                }
                error::finalize_error_diagnostic(&e);
                return Err(e);
            }
        }
    }
    Ok(last_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let mut env = create_default_env();
        let res = eval_str("(+ 1 2 (* 3 4))", &mut env).unwrap();
        assert_eq!(res, Value::Int(15));

        let res2 = eval_str("(/ 10 2 2)", &mut env).unwrap();
        assert_eq!(res2, Value::Int(2));
    }

    #[test]
    fn test_multiline_setq() {
        let mut env = create_default_env();
        let code = r#"
            (setq a 10
                  b 20
                  c (+ a b))
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Int(30));
        assert_eq!(eval_str("a", &mut env).unwrap(), Value::Int(10));
        assert_eq!(eval_str("b", &mut env).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_defun_and_locals() {
        let mut env = create_default_env();
        let code = r#"
            (defun add-and-square (x y / sum)
                (setq sum (+ x y))
                (* sum sum)
            )
            (add-and-square 3 4)
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Int(49));
        // Ensure local `sum` is not leaked to global scope (evaluates to nil in AutoLISP)
        assert_eq!(eval_str("sum", &mut env).unwrap(), Value::Nil);
    }

    #[test]
    fn test_list_assoc_and_dxf_pairs() {
        let mut env = create_default_env();
        let code = r#"
            (setq elist '((0 . "LINE") (8 . "WALLS") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0)))
            (cdr (assoc 0 elist))
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Str("LINE".into()));

        let layer = eval_str("(cdr (assoc 8 elist))", &mut env).unwrap();
        assert_eq!(layer, Value::Str("WALLS".into()));
    }

    #[test]
    fn test_mapcar_lambda() {
        let mut env = create_default_env();
        let code = r#"
            (mapcar '(lambda (x) (* x 2)) '(1 2 3 4))
        "#;
        let res = eval_str(code, &mut env).unwrap();
        let expected = Value::from_slice(&[Value::Int(2), Value::Int(4), Value::Int(6), Value::Int(8)]);
        assert_eq!(res, expected);
    }

    #[test]
    fn test_vl_string_and_sort() {
        let mut env = create_default_env();
        let code = r#"
            (vl-string-subst "OpenCAD" "AutoLISP" "AutoLISP is great")
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Str("OpenCAD is great".into()));

        let sort_code = r#"
            (vl-sort '(5 2 8 1 9) '<)
        "#;
        let sorted = eval_str(sort_code, &mut env).unwrap();
        let expected = Value::from_slice(&[Value::Int(1), Value::Int(2), Value::Int(5), Value::Int(8), Value::Int(9)]);
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_vl_catch_all() {
        let mut env = create_default_env();
        let code = r#"
            (setq err (vl-catch-all-apply '/ '(10 0)))
            (vl-catch-all-error-p err)
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::T);
    }

    #[test]
    fn test_file_io() {
        let mut env = create_default_env();
        let test_file = std::env::temp_dir().join("opencad_lisp_test_io.txt");
        let path_str = test_file.to_str().unwrap().replace('\\', "/");

        let write_code = format!(
            r#"
            (setq f (open "{}" "w"))
            (write-line "Line 1: Hello from AutoLISP" f)
            (write-line "Line 2: 12345" f)
            (close f)
        "#,
            path_str
        );
        eval_str(&write_code, &mut env).unwrap();

        let read_code = format!(
            r#"
            (setq f (open "{}" "r"))
            (setq l1 (read-line f))
            (setq l2 (read-line f))
            (close f)
            (list l1 l2)
        "#,
            path_str
        );
        let res = eval_str(&read_code, &mut env).unwrap();
        let expected = Value::from_slice(&[
            Value::Str("Line 1: Hello from AutoLISP".into()),
            Value::Str("Line 2: 12345".into()),
        ]);
        assert_eq!(res, expected);

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_geometry_math() {
        let mut env = create_default_env();
        let code = r#"
            (setq p1 '(10.0 20.0 0.0))
            (setq p2 '(40.0 60.0 0.0))
            (setq d (distance p1 p2))
            (setq a (angle '(0.0 0.0 0.0) '(10.0 10.0 0.0)))
            (setq p3 (polar '(0.0 0.0 0.0) 0.0 50.0))
            (list d (< (abs (- a (/ pi 4.0))) 1e-4) p3)
        "#;
        let res = eval_str(code, &mut env).unwrap();
        let items = res.to_vec().unwrap();
        assert_eq!(items[0], Value::Real(50.0));
        assert_eq!(items[1], Value::T);
        let p3_items = items[2].to_vec().unwrap();
        assert_eq!(p3_items[0], Value::Real(50.0));
        assert_eq!(p3_items[1], Value::Real(0.0));
        assert_eq!(p3_items[2], Value::Real(0.0));
    }

    #[test]
    fn test_unassigned_symbols_eval_to_nil() {
        let mut env = create_default_env();
        let res1 = eval_str("sdfsdf", &mut env).unwrap();
        assert_eq!(res1, Value::Nil);

        let res2 = eval_str("(null sdfsdf)", &mut env).unwrap();
        assert_eq!(res2, Value::T);

        let res3 = eval_str("(if sdfsdf \"yes\" \"no\")", &mut env).unwrap();
        assert_eq!(res3, Value::Str("no".into()));
    }

    #[test]
    fn test_equality_and_comparison_strings() {
        let mut env = create_default_env();
        assert_eq!(eval_str(r#"(= "+" "+")"#, &mut env).unwrap(), Value::T);
        assert_eq!(eval_str(r#"(= "+" "-")"#, &mut env).unwrap(), Value::Nil);
        assert_eq!(eval_str(r#"(/= "+" "-")"#, &mut env).unwrap(), Value::T);
        assert_eq!(eval_str(r#"(< "abc" "def")"#, &mut env).unwrap(), Value::T);
        assert_eq!(eval_str(r#"(<= "abc" "abc")"#, &mut env).unwrap(), Value::T);
        assert_eq!(eval_str(r#"(> "def" "abc")"#, &mut env).unwrap(), Value::T);
    }

    #[test]
    fn test_quasiquote_unquote_and_splice() {
        let mut env = create_default_env();

        // Basic unquote
        let res = eval_str("`(+ 1 ,(+ 2 3) 4)", &mut env).unwrap();
        assert_eq!(
            res,
            Value::from_slice(&[
                Value::Sym(Symbol::new("+")),
                Value::Int(1),
                Value::Int(5),
                Value::Int(4),
            ])
        );

        // Unquote-splicing
        let res2 = eval_str("`(a ,@'(b c) d)", &mut env).unwrap();
        assert_eq!(
            res2,
            Value::from_slice(&[
                Value::Sym(Symbol::new("A")),
                Value::Sym(Symbol::new("B")),
                Value::Sym(Symbol::new("C")),
                Value::Sym(Symbol::new("D")),
            ])
        );

        // Unquote-splicing nil
        let res3 = eval_str("`(a ,@nil d)", &mut env).unwrap();
        assert_eq!(
            res3,
            Value::from_slice(&[
                Value::Sym(Symbol::new("A")),
                Value::Sym(Symbol::new("D")),
            ])
        );

        // Quasiquote with dotted pair
        let res4 = eval_str("(setq val 42) `(code . ,val)", &mut env).unwrap();
        assert_eq!(res4, Value::cons(Value::Sym(Symbol::new("CODE")), Value::Int(42)));
    }

    #[test]
    fn test_recursion_and_mutual_recursion() {
        std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(|| {
                let mut env = create_default_env();

                // Factorial recursion
                let fact_code = r#"
                    (defun fact (n)
                        (if (<= n 1)
                            1
                            (* n (fact (1- n)))))
                    (fact 6)
                "#;
                assert_eq!(eval_str(fact_code, &mut env).unwrap(), Value::Int(720));

                // Mutual recursion
                let mutual_code = r#"
                    (defun is-even (n)
                        (if (zerop n)
                            t
                            (is-odd (1- n))))
                    (defun is-odd (n)
                        (if (zerop n)
                            nil
                            (is-even (1- n))))
                    (list (is-even 10) (is-odd 10) (is-even 7) (is-odd 7))
                "#;
                let expected = Value::from_slice(&[Value::T, Value::Nil, Value::Nil, Value::T]);
                assert_eq!(eval_str(mutual_code, &mut env).unwrap(), expected);

                // Deep 2000-level recursion
                let deep_code = r#"
                    (defun test-recursion (n)
                        (if (> n 0)
                            (test-recursion (1- n))
                            n
                        )
                    )
                    (test-recursion 2000)
                "#;
                assert_eq!(eval_str(deep_code, &mut env).unwrap(), Value::Int(0));
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn test_recursion_depth_limit_stack_overflow() {
        std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(|| {
                let mut env = create_default_env();
                let inf_code = r#"
                    (defun inf-loop () (inf-loop))
                    (inf-loop)
                "#;
                let res = eval_str(inf_code, &mut env);
                assert!(matches!(res, Err(LispError::StackOverflow)));
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn test_setq_global_scoping_in_functions() {
        let mut env = create_default_env();

        // 'loc' is local; 'glob' is undeclared so it must be global
        let code = r#"
            (defun test-scope (x / loc)
                (setq loc x)
                (setq glob x)
                loc
            )
            (test-scope 999)
        "#;
        assert_eq!(eval_str(code, &mut env).unwrap(), Value::Int(999));
        // glob should exist in top-level env
        assert_eq!(eval_str("glob", &mut env).unwrap(), Value::Int(999));
        // loc should NOT exist (nil)
        assert_eq!(eval_str("loc", &mut env).unwrap(), Value::Nil);
    }

    #[test]
    fn test_error_handling_and_custom_error_function() {
        let mut env = create_default_env();

        let code = r#"
            (setq err-result nil)
            (defun *error* (msg)
                (setq err-result (strcat "Intercepted: " msg))
            )
            (/ 1 0)
        "#;
        let res = eval_str(code, &mut env);
        assert!(res.is_err());
        // Verify *error* was invoked and captured the error message
        assert_eq!(
            eval_str("err-result", &mut env).unwrap(),
            Value::Str("Intercepted: Division by zero".into())
        );

        // Verify explicit (error "Custom failure")
        let code2 = r#"
            (error "Custom failure")
        "#;
        let res2 = eval_str(code2, &mut env);
        assert!(res2.is_err());
        assert_eq!(
            eval_str("err-result", &mut env).unwrap(),
            Value::Str("Intercepted: Custom failure".into())
        );
    }

    #[test]
    fn test_exit_and_quit() {
        let mut env = create_default_env();
        let res1 = eval_str("(exit)", &mut env);
        assert_eq!(res1, Err(LispError::UserExit));
        let res2 = eval_str("(quit)", &mut env);
        assert_eq!(res2, Err(LispError::UserExit));
    }

    #[test]
    fn test_error_diagnostic_captures_defun_and_call_stack() {
        let mut env = create_default_env();
        let code = r#"
            (defun my-helper (x / y)
                (setq y (+ x 10))
                (* y 2)
            )
            (defun c:mycmd ()
                (my-helper "bad-arg")
            )
            (c:mycmd)
        "#;
        let res = eval_str(code, &mut env);
        assert!(res.is_err());

        let diag = get_last_error_diagnostic().expect("Expected last error diagnostic");
        assert_eq!(diag.primitive.as_deref(), Some("+"));
        assert_eq!(diag.failing_expr.as_deref(), Some("(+ X 10)"));
        assert_eq!(diag.frames.len(), 2);
        assert_eq!(diag.frames[0].func_name, "MY-HELPER");
        assert_eq!(diag.frames[0].call_expr, "(MY-HELPER \"bad-arg\")");
        assert!(
            diag.frames[0]
                .defun_source
                .contains("(DEFUN MY-HELPER (X / Y) (SETQ Y (+ X 10)) (* Y 2))"),
            "Unexpected defun_source: {}",
            diag.frames[0].defun_source
        );
        assert_eq!(diag.frames[1].func_name, "C:MYCMD");
        assert_eq!(diag.frames[1].call_expr, "(C:MYCMD)");

        let full = diag.format_full();
        assert!(full.contains("Type error in +: expected number, got \"bad-arg\""), "Full: {full}");
        assert!(full.contains("; in expression: (+ X 10)"), "Full: {full}");
        assert!(full.contains("; in function:   (MY-HELPER \"bad-arg\")"), "Full: {full}");
        assert!(full.contains("; definition:    (DEFUN MY-HELPER (X / Y) (SETQ Y (+ X 10)) (* Y 2))"), "Full: {full}");
        assert!(full.contains("; called from:   (C:MYCMD)"), "Full: {full}");
    }

    #[test]
    fn test_trace_untrace_and_vl_bt() {
        let mut env = create_default_env();
        let code = r#"
            (defun double-val (n) (* n 2))
            (trace double-val)
            (setq r1 (double-val 21))
            (untrace double-val)
            (setq r2 (double-val 5))
            (defun inner-bt () (vl-bt))
            (defun outer-bt () (inner-bt))
            (setq bt-frames (outer-bt))
        "#;
        eval_str(code, &mut env).unwrap();
        assert_eq!(eval_str("r1", &mut env).unwrap(), Value::Int(42));
        assert_eq!(eval_str("r2", &mut env).unwrap(), Value::Int(10));
        let bt = eval_str("bt-frames", &mut env).unwrap();
        let bt_vec = bt.to_vec().expect("bt-frames should be a list");
        assert_eq!(bt_vec.len(), 2);
        assert_eq!(bt_vec[0], Value::Str("(INNER-BT)".into()));
        assert_eq!(bt_vec[1], Value::Str("(OUTER-BT)".into()));
    }

    #[test]
    fn test_sexpr_breakpoints_and_stepping() {
        use std::sync::{Arc, Mutex};

        let mut env = create_default_env();
        let code = "(defun c:test (/ a b)\n  (setq a (+ 10 (* 2 3)))\n  (setq b (* a 5))\n)";
        eval_str(code, &mut env).unwrap();

        // 1. Test StepInto through nested S-expressions inside (setq a (+ 10 (* 2 3)))
        let visited_into: Arc<Mutex<Vec<(u32, u32, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let visited_clone = visited_into.clone();
        set_debug_breakpoints(vec![]);
        set_debug_step_mode(DebugStepMode::StepInto);
        set_debug_pause_callback(Some(Arc::new(move |info| {
            visited_clone.lock().unwrap().push((
                info.span.start_line,
                info.span.start_col,
                info.expr_preview.clone(),
            ));
            DebugResumeAction::StepInto
        })));

        let res = eval_str_unspanned("(c:test)", &mut env).unwrap();
        assert_eq!(res, Value::Int(80));

        set_debug_pause_callback(None);
        set_debug_step_mode(DebugStepMode::None);

        let steps = visited_into.lock().unwrap().clone();
        assert_eq!(
            steps,
            vec![
                (2, 3, "(SETQ A (+ 10 (* 2 3)))".to_string()),
                (2, 11, "(+ 10 (* 2 3))".to_string()),
                (2, 17, "(* 2 3)".to_string()),
                (2, 17, "(* 2 3) => 6".to_string()),
                (2, 11, "(+ 10 (* 2 3)) => 16".to_string()),
                (2, 3, "(SETQ A (+ 10 (* 2 3))) => 16".to_string()),
                (3, 3, "(SETQ B (* A 5))".to_string()),
                (3, 11, "(* A 5)".to_string()),
                (3, 11, "(* A 5) => 80".to_string()),
                (3, 3, "(SETQ B (* A 5)) => 80".to_string()),
            ]
        );

        // 2. Test StepOver skips inner (+ 10 (* 2 3)) and (* 2 3) and pauses at sibling (setq b (* a 5))
        let visited_over: Arc<Mutex<Vec<(u32, u32, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let visited_over_clone = visited_over.clone();
        set_debug_breakpoints(vec![]);
        set_debug_step_mode(DebugStepMode::StepInto);
        set_debug_pause_callback(Some(Arc::new(move |info| {
            visited_over_clone.lock().unwrap().push((
                info.span.start_line,
                info.span.start_col,
                info.expr_preview.clone(),
            ));
            DebugResumeAction::StepOver
        })));

        let res2 = eval_str_unspanned("(c:test)", &mut env).unwrap();
        assert_eq!(res2, Value::Int(80));

        set_debug_pause_callback(None);
        set_debug_step_mode(DebugStepMode::None);

        let over_steps = visited_over.lock().unwrap().clone();
        assert_eq!(
            over_steps,
            vec![
                (2, 3, "(SETQ A (+ 10 (* 2 3)))".to_string()),
                (3, 3, "(SETQ B (* A 5))".to_string()),
            ]
        );

        // 3. Test ParenBreakpoint on inner (* 2 3) at line 2, col 17
        let hit_bp: Arc<Mutex<Vec<(u32, u32, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let hit_bp_clone = hit_bp.clone();
        set_debug_breakpoints(vec![(2, 17)]);
        set_debug_step_mode(DebugStepMode::None);
        set_debug_pause_callback(Some(Arc::new(move |info| {
            hit_bp_clone.lock().unwrap().push((
                info.span.start_line,
                info.span.start_col,
                info.expr_preview.clone(),
            ));
            DebugResumeAction::Continue
        })));

        let res3 = eval_str_unspanned("(c:test)", &mut env).unwrap();
        assert_eq!(res3, Value::Int(80));

        set_debug_pause_callback(None);
        set_debug_breakpoints(vec![]);

        let bp_hits = hit_bp.lock().unwrap().clone();
        assert_eq!(bp_hits, vec![(2, 17, "(* 2 3)".to_string())]);
    }
}
