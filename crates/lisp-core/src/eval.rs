use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::{Symbol, UserFunction, Value};
use crate::env::Env;
use crate::error::{
    active_call_depth, debug_enter_cons, debug_exit_cons, debug_pop_depth, is_debug_hook_enabled,
    is_function_traced, pop_active_frame, push_active_call, record_error_frame,
    record_error_primitive, record_failing_expr, LispError,
};

/// Maximum user function call recursion depth before raising `LispError::StackOverflow`.
pub const MAX_CALL_DEPTH: usize = 2500;

/// Maximum depth tracked in live `ACTIVE_CALL_STACK` for `(vl-bt)`.
const MAX_ACTIVE_BT_DEPTH: usize = 64;

struct ActiveFrameGuard(bool);

impl Drop for ActiveFrameGuard {
    fn drop(&mut self) {
        if self.0 {
            pop_active_frame();
        }
    }
}

struct DebugConsGuard {
    active: bool,
    completed: bool,
}

impl Drop for DebugConsGuard {
    fn drop(&mut self) {
        if self.active && !self.completed {
            debug_pop_depth();
        }
    }
}

#[inline(never)]
fn trace_enter_subr(name: &str, args: &[Value]) {
    let indent = "  ".repeat(active_call_depth());
    let call_str = if args.is_empty() {
        format!("({name})")
    } else {
        let a_str = args
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        format!("({name} {a_str})")
    };
    crate::write_output(&format!("{indent}Entering {call_str}\n"));
}

#[inline(never)]
fn trace_exit_subr(name: &str, res: &Result<Value, LispError>) {
    let indent = "  ".repeat(active_call_depth());
    match res {
        Ok(val) => crate::write_output(&format!("{indent}{name} -> {val}\n")),
        Err(e) => crate::write_output(&format!("{indent}{name} !-> {e}\n")),
    }
}

#[inline(never)]
fn trace_enter_usubr(user_fn: &UserFunction, args: &[Value], depth: usize) {
    let indent = "  ".repeat(depth);
    crate::write_output(&format!("{indent}Entering {}\n", user_fn.format_call(args)));
}

#[inline(never)]
fn trace_exit_usubr(user_fn: &UserFunction, res: &Result<Value, LispError>, depth: usize) {
    let indent = "  ".repeat(depth);
    let name = user_fn.display_name();
    match res {
        Ok(val) => crate::write_output(&format!("{indent}{name} -> {val}\n")),
        Err(e) => crate::write_output(&format!("{indent}{name} !-> {e}\n")),
    }
}

/// Evaluates a Lisp expression in the given environment.
pub fn eval(expr: &Value, env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    eval_with_depth(expr, env, 0)
}

/// Evaluates a Lisp expression tracking user function call depth to prevent OS thread stack overflow.
pub fn eval_with_depth(
    expr: &Value,
    env: &mut Rc<RefCell<Env>>,
    call_depth: usize,
) -> Result<Value, LispError> {
    if call_depth > MAX_CALL_DEPTH {
        return Err(LispError::StackOverflow);
    }

    match expr {
        Value::Nil | Value::T | Value::Int(_) | Value::Real(_) | Value::Str(_) => Ok(expr.clone()),
        Value::Subr { .. }
        | Value::USubr(_)
        | Value::Ename(_)
        | Value::PickSet(_)
        | Value::VlaObject(_)
        | Value::CatchError(_)
        | Value::File(_) => Ok(expr.clone()),
        Value::Sym(sym) => Ok(env.borrow().get(sym).unwrap_or(Value::Nil)),
        Value::Cons(cons_rc) => {
            let is_debug = is_debug_hook_enabled();
            let mut dbg_guard = DebugConsGuard {
                active: if is_debug {
                    debug_enter_cons(expr, env)?
                } else {
                    false
                },
                completed: false,
            };

            let res = (|| -> Result<Value, LispError> {
                let (car, cdr) = {
                    let c = cons_rc.borrow();
                    (c.car.clone(), c.cdr.clone())
                };

            // Check for special forms if car is a symbol
            if let Value::Sym(ref op_sym) = car {
                let op_name = op_sym.as_str();
                match op_name {
                    "QUOTE" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid quote syntax".into()))?;
                        if args.is_empty() {
                            record_error_primitive("QUOTE");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: 0,
                            });
                        }
                        return Ok(args[0].clone());
                    }

                    "QUASIQUOTE" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid quasiquote syntax".into()))?;
                        if args.len() != 1 {
                            record_error_primitive("QUASIQUOTE");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: args.len(),
                            });
                        }
                        return eval_quasiquote(&args[0], env, 0, call_depth);
                    }

                    "UNQUOTE" | "UNQUOTE-SPLICING" => {
                        record_error_primitive(op_name);
                        record_failing_expr(expr);
                        return Err(LispError::Syntax(
                            "unquote or unquote-splicing outside of quasiquote".into(),
                        ));
                    }

                    "SETQ" => {
                        let res = eval_setq(&cdr, env, call_depth);
                        if let Err(ref e) = res {
                            if !matches!(e, LispError::UserExit) {
                                record_error_primitive("SETQ");
                                record_failing_expr(expr);
                            }
                        }
                        return res;
                    }
                    "DEFUN" => {
                        let res = eval_defun(&cdr, env);
                        if let Err(ref e) = res {
                            if !matches!(e, LispError::UserExit) {
                                record_error_primitive("DEFUN");
                                record_failing_expr(expr);
                            }
                        }
                        return res;
                    }
                    "LAMBDA" => {
                        let res = eval_lambda(&cdr, env);
                        if let Err(ref e) = res {
                            if !matches!(e, LispError::UserExit) {
                                record_error_primitive("LAMBDA");
                                record_failing_expr(expr);
                            }
                        }
                        return res;
                    }

                    "IF" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid if syntax".into()))?;
                        if args.len() < 2 || args.len() > 3 {
                            record_error_primitive("IF");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "2 or 3 (test then [else])",
                                actual: args.len(),
                            });
                        }
                        let test_res = eval_with_depth(&args[0], env, call_depth)?;
                        if test_res.is_truthy() {
                            return eval_with_depth(&args[1], env, call_depth);
                        } else if args.len() == 3 {
                            return eval_with_depth(&args[2], env, call_depth);
                        } else {
                            return Ok(Value::Nil);
                        }
                    }

                    "COND" => return eval_cond(&cdr, env, call_depth),
                    "WHILE" => return eval_while(&cdr, env, call_depth),
                    "REPEAT" => return eval_repeat(&cdr, env, call_depth),
                    "FOREACH" => return eval_foreach(&cdr, env, call_depth),

                    "PROGN" => {
                        let exprs = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid progn syntax".into()))?;
                        let mut last = Value::Nil;
                        for sub in exprs {
                            last = eval_with_depth(&sub, env, call_depth)?;
                        }
                        return Ok(last);
                    }

                    "AND" => {
                        let exprs = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid and syntax".into()))?;
                        if exprs.is_empty() {
                            return Ok(Value::T);
                        }
                        let mut last = Value::T;
                        for sub in exprs {
                            let res = eval_with_depth(&sub, env, call_depth)?;
                            if !res.is_truthy() {
                                return Ok(Value::Nil);
                            }
                            last = res;
                        }
                        return Ok(last);
                    }

                    "OR" => {
                        let exprs = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid or syntax".into()))?;
                        for sub in exprs {
                            let res = eval_with_depth(&sub, env, call_depth)?;
                            if res.is_truthy() {
                                return Ok(res);
                            }
                        }
                        return Ok(Value::Nil);
                    }

                    "EVAL" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid eval syntax".into()))?;
                        if args.len() != 1 {
                            record_error_primitive("EVAL");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: args.len(),
                            });
                        }
                        let evaluated_arg = eval_with_depth(&args[0], env, call_depth)?;
                        return eval_with_depth(&evaluated_arg, env, call_depth);
                    }

                    "SET" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid set syntax".into()))?;
                        if args.len() != 2 {
                            record_error_primitive("SET");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "2 (sym-expr val-expr)",
                                actual: args.len(),
                            });
                        }
                        let target = eval_with_depth(&args[0], env, call_depth)?;
                        let sym = match target {
                            Value::Sym(s) => s,
                            other => {
                                record_error_primitive("SET");
                                record_failing_expr(expr);
                                return Err(LispError::TypeError {
                                    expected: "symbol",
                                    actual: other.to_string(),
                                });
                            }
                        };
                        let val = eval_with_depth(&args[1], env, call_depth)?;
                        Env::setq(env, sym, val.clone());
                        return Ok(val);
                    }

                    "BOUNDP" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid boundp syntax".into()))?;
                        if args.len() != 1 {
                            record_error_primitive("BOUNDP");
                            record_failing_expr(expr);
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: args.len(),
                            });
                        }
                        let target = eval_with_depth(&args[0], env, call_depth)?;
                        let sym = match target {
                            Value::Sym(s) => s,
                            _ => return Ok(Value::Nil),
                        };
                        if env.borrow().get(&sym).is_some() {
                            return Ok(Value::T);
                        } else {
                            return Ok(Value::Nil);
                        }
                    }

                    "TRACE" => return eval_trace(&cdr, env, call_depth),
                    "UNTRACE" => return eval_untrace(&cdr, env, call_depth),
                    "LET" => return eval_let(&cdr, env, call_depth),

                    _ => {}
                }
            }

            // Normal function application: evaluate operator and all arguments
            let func = match &car {
                Value::Sym(op_sym) => {
                    let maybe_f = env.borrow().get(op_sym);
                    if let Some(f) = maybe_f {
                        f
                    } else if let Some(vla_res) = try_eval_dynamic_vla(op_sym.as_str(), &cdr, env, call_depth)? {
                        return Ok(vla_res);
                    } else {
                        record_failing_expr(expr);
                        return Err(LispError::UnboundSymbol(op_sym.to_string()));
                    }
                }
                _ => eval_with_depth(&car, env, call_depth)?,
            };

            let raw_args = cdr
                .to_vec()
                .ok_or_else(|| LispError::Syntax("Malformed function argument list".into()))?;
            let mut evaluated_args = Vec::with_capacity(raw_args.len());
            for arg in raw_args {
                evaluated_args.push(eval_with_depth(&arg, env, call_depth)?);
            }

            let res = apply_with_depth(&func, &evaluated_args, env, call_depth);
            if let Err(ref e) = res {
                if !matches!(e, LispError::UserExit) {
                    record_failing_expr(expr);
                }
            }
            res
        })();

        match res {
            Ok(val) => {
                if dbg_guard.active {
                    dbg_guard.completed = true;
                    debug_exit_cons(expr, &val, env)?;
                }
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }
    }
}

#[inline(never)]
fn eval_trace(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid trace syntax".into()))?;
    let mut last = Value::Nil;
    for arg in args {
        let sym_name = match &arg {
            Value::Sym(s) => s.as_str().to_string(),
            Value::Str(s) => s.to_ascii_uppercase(),
            _ => match eval_with_depth(&arg, env, call_depth)? {
                Value::Sym(s) => s.as_str().to_string(),
                Value::Str(s) => s.to_ascii_uppercase(),
                other => {
                    record_error_primitive("TRACE");
                    return Err(LispError::TypeError {
                        expected: "function symbol",
                        actual: other.to_string(),
                    });
                }
            },
        };
        crate::error::trace_function(&sym_name);
        last = Value::Sym(Symbol::new(&sym_name));
    }
    Ok(last)
}

#[inline(never)]
fn eval_untrace(
    cdr: &Value,
    env: &mut Rc<RefCell<Env>>,
    call_depth: usize,
) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid untrace syntax".into()))?;
    if args.is_empty() {
        let removed = crate::error::untrace_all();
        return Ok(removed
            .last()
            .map(|s| Value::Sym(Symbol::new(s)))
            .unwrap_or(Value::Nil));
    }
    let mut last = Value::Nil;
    for arg in args {
        let sym_name = match &arg {
            Value::Sym(s) => s.as_str().to_string(),
            Value::Str(s) => s.to_ascii_uppercase(),
            _ => match eval_with_depth(&arg, env, call_depth)? {
                Value::Sym(s) => s.as_str().to_string(),
                Value::Str(s) => s.to_ascii_uppercase(),
                other => {
                    record_error_primitive("UNTRACE");
                    return Err(LispError::TypeError {
                        expected: "function symbol",
                        actual: other.to_string(),
                    });
                }
            },
        };
        crate::error::untrace_function(&sym_name);
        last = Value::Sym(Symbol::new(&sym_name));
    }
    Ok(last)
}

#[inline(never)]
fn eval_setq(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid setq syntax".into()))?;
    if args.len() % 2 != 0 {
        return Err(LispError::Syntax(
            "setq requires an even number of arguments (sym val ...)".into(),
        ));
    }
    let mut last_val = Value::Nil;
    for chunk in args.chunks(2) {
        let sym = match &chunk[0] {
            Value::Sym(s) => s.clone(),
            other => {
                return Err(LispError::TypeError {
                    expected: "symbol to set",
                    actual: other.to_string(),
                })
            }
        };
        let val = eval_with_depth(&chunk[1], env, call_depth)?;
        Env::setq(env, sym, val.clone());
        last_val = val;
    }
    Ok(last_val)
}

#[inline(never)]
fn eval_defun(cdr: &Value, env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid defun syntax".into()))?;
    if args.len() < 2 {
        return Err(LispError::WrongArgCount {
            expected: "at least 2 (name args body...)",
            actual: args.len(),
        });
    }
    let name_sym = match &args[0] {
        Value::Sym(s) => s.clone(),
        other => {
            return Err(LispError::TypeError {
                expected: "function name symbol",
                actual: other.to_string(),
            })
        }
    };

    let (params, locals) = parse_param_list(&args[1])?;
    let body = args[2..].to_vec();

    let user_fn = Arc::new(UserFunction {
        name: Some(name_sym.clone()),
        params,
        locals,
        body,
        env: env.clone(),
    });

    if name_sym.as_str().starts_with("C:") {
        let cmd_name = &name_sym.as_str()[2..];
        if let Some(hook) = &env.borrow().command_hook {
            hook(cmd_name);
        }
    }

    env.borrow_mut()
        .set_local(name_sym.clone(), Value::USubr(user_fn));
    Ok(Value::Sym(name_sym))
}

#[inline(never)]
fn eval_lambda(cdr: &Value, env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid lambda syntax".into()))?;
    if args.is_empty() {
        return Err(LispError::WrongArgCount {
            expected: "at least 1 (args body...)",
            actual: 0,
        });
    }
    let (params, locals) = parse_param_list(&args[0])?;
    let body = args[1..].to_vec();

    let user_fn = Arc::new(UserFunction {
        name: None,
        params,
        locals,
        body,
        env: env.clone(),
    });

    Ok(Value::USubr(user_fn))
}

#[inline(never)]
fn eval_cond(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let clauses = cdr
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Invalid cond syntax".into()))?;
    for clause in clauses {
        let clause_items = clause
            .to_vec()
            .ok_or_else(|| LispError::Syntax("cond clause must be a list".into()))?;
        if clause_items.is_empty() {
            continue;
        }
        let test_val = eval_with_depth(&clause_items[0], env, call_depth)?;
        if test_val.is_truthy() {
            if clause_items.len() == 1 {
                return Ok(test_val);
            }
            let mut last = Value::Nil;
            for expr in &clause_items[1..] {
                last = eval_with_depth(expr, env, call_depth)?;
            }
            return Ok(last);
        }
    }
    Ok(Value::Nil)
}

#[inline(never)]
fn eval_while(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| {
            record_error_primitive("WHILE");
            LispError::Syntax("Invalid while syntax".into())
        })?;
    if args.is_empty() {
        record_error_primitive("WHILE");
        return Err(LispError::WrongArgCount {
            expected: "at least 1 (test body...)",
            actual: 0,
        });
    }
    let test_expr = &args[0];
    let body = &args[1..];
    let mut last_val = Value::Nil;

    while eval_with_depth(test_expr, env, call_depth)?.is_truthy() {
        for expr in body {
            last_val = eval_with_depth(expr, env, call_depth)?;
        }
    }
    Ok(last_val)
}

#[inline(never)]
fn eval_repeat(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| {
            record_error_primitive("REPEAT");
            LispError::Syntax("Invalid repeat syntax".into())
        })?;
    if args.is_empty() {
        record_error_primitive("REPEAT");
        return Err(LispError::WrongArgCount {
            expected: "at least 1 (count body...)",
            actual: 0,
        });
    }
    let count = match eval_with_depth(&args[0], env, call_depth)? {
        Value::Int(n) => n.max(0) as usize,
        Value::Real(r) => r.max(0.0) as usize,
        other => {
            record_error_primitive("REPEAT");
            return Err(LispError::TypeError {
                expected: "count number",
                actual: other.to_string(),
            });
        }
    };
    let body = &args[1..];
    let mut last_val = Value::Nil;

    for _ in 0..count {
        for expr in body {
            last_val = eval_with_depth(expr, env, call_depth)?;
        }
    }
    Ok(last_val)
}

#[inline(never)]
fn eval_let(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| {
            record_error_primitive("LET");
            LispError::Syntax("Invalid let syntax".into())
        })?;
    if args.is_empty() {
        record_error_primitive("LET");
        return Err(LispError::WrongArgCount {
            expected: "at least 1 ((bindings...) body...)",
            actual: 0,
        });
    }

    let binding_list = args[0]
        .to_vec()
        .ok_or_else(|| LispError::Syntax("let bindings must be a list of (var val) pairs".into()))?;

    let mut evaluated_bindings = Vec::with_capacity(binding_list.len());
    for item in binding_list {
        match item {
            Value::Cons(_) => {
                let pair = item.to_vec().ok_or_else(|| LispError::Syntax("Malformed let binding pair".into()))?;
                if pair.is_empty() || pair.len() > 2 {
                    return Err(LispError::Syntax("let binding must be (var) or (var val)".into()));
                }
                let sym = match &pair[0] {
                    Value::Sym(s) => s.clone(),
                    other => return Err(LispError::TypeError { expected: "symbol", actual: other.to_string() }),
                };
                let val = if pair.len() == 2 {
                    eval_with_depth(&pair[1], env, call_depth)?
                } else {
                    Value::Nil
                };
                evaluated_bindings.push((sym, val));
            }
            Value::Sym(sym) => {
                evaluated_bindings.push((sym, Value::Nil));
            }
            other => return Err(LispError::TypeError { expected: "binding pair or symbol", actual: other.to_string() }),
        }
    }

    let let_env = Env::with_parent(env.clone());
    for (sym, val) in evaluated_bindings {
        let_env.borrow_mut().set_local(sym, val);
    }

    let mut let_env_rc = let_env;
    let body = &args[1..];
    let mut last_val = Value::Nil;
    for expr in body {
        last_val = eval_with_depth(expr, &mut let_env_rc, call_depth)?;
    }
    Ok(last_val)
}

fn try_eval_dynamic_vla(
    sym_name: &str,
    cdr: &Value,
    env: &mut Rc<RefCell<Env>>,
    call_depth: usize,
) -> Result<Option<Value>, LispError> {
    let upper = sym_name.to_ascii_uppercase();
    if upper.starts_with("VLA-GET-") && upper.len() > "VLA-GET-".len() {
        let prop = &upper["VLA-GET-".len()..];
        let raw_args = cdr.to_vec().ok_or_else(|| LispError::Syntax("Malformed arguments for vla-get".into()))?;
        if raw_args.len() != 1 {
            return Err(LispError::WrongArgCount {
                expected: "1 (vla-object)",
                actual: raw_args.len(),
            });
        }
        let obj = eval_with_depth(&raw_args[0], env, call_depth)?;
        let get_prop_sym = crate::ast::Symbol::new("VLAX-GET-PROPERTY");
        let func = env.borrow().get(&get_prop_sym).ok_or_else(|| {
            LispError::UnboundSymbol("VLAX-GET-PROPERTY".to_string())
        })?;
        let res = apply_with_depth(&func, &[obj, Value::Str(prop.to_string())], env, call_depth)?;
        return Ok(Some(res));
    } else if upper.starts_with("VLA-PUT-") && upper.len() > "VLA-PUT-".len() {
        let prop = &upper["VLA-PUT-".len()..];
        let raw_args = cdr.to_vec().ok_or_else(|| LispError::Syntax("Malformed arguments for vla-put".into()))?;
        if raw_args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (vla-object new-value)",
                actual: raw_args.len(),
            });
        }
        let obj = eval_with_depth(&raw_args[0], env, call_depth)?;
        let val = eval_with_depth(&raw_args[1], env, call_depth)?;
        let put_prop_sym = crate::ast::Symbol::new("VLAX-PUT-PROPERTY");
        let func = env.borrow().get(&put_prop_sym).ok_or_else(|| {
            LispError::UnboundSymbol("VLAX-PUT-PROPERTY".to_string())
        })?;
        let res = apply_with_depth(&func, &[obj, Value::Str(prop.to_string()), val], env, call_depth)?;
        return Ok(Some(res));
    } else if upper.starts_with("VLA-") && upper.len() > "VLA-".len() {
        let method = &upper["VLA-".len()..];
        let raw_args = cdr.to_vec().ok_or_else(|| LispError::Syntax("Malformed arguments for vla method".into()))?;
        if raw_args.is_empty() {
            return Err(LispError::WrongArgCount {
                expected: "at least 1 (vla-object [args...])",
                actual: 0,
            });
        }
        let mut evaluated = Vec::with_capacity(raw_args.len());
        for a in raw_args {
            evaluated.push(eval_with_depth(&a, env, call_depth)?);
        }
        let invoke_sym = crate::ast::Symbol::new("VLAX-INVOKE-METHOD");
        let func = env.borrow().get(&invoke_sym);
        if let Some(f) = func {
            let mut invoke_args = Vec::with_capacity(evaluated.len() + 1);
            invoke_args.push(evaluated[0].clone());
            invoke_args.push(Value::Str(method.to_string()));
            invoke_args.extend_from_slice(&evaluated[1..]);
            let res = apply_with_depth(&f, &invoke_args, env, call_depth)?;
            return Ok(Some(res));
        }
    }
    Ok(None)
}

#[inline(never)]
fn eval_foreach(cdr: &Value, env: &mut Rc<RefCell<Env>>, call_depth: usize) -> Result<Value, LispError> {
    let args = cdr
        .to_vec()
        .ok_or_else(|| {
            record_error_primitive("FOREACH");
            LispError::Syntax("Invalid foreach syntax".into())
        })?;
    if args.len() < 2 {
        record_error_primitive("FOREACH");
        return Err(LispError::WrongArgCount {
            expected: "at least 2 (var list body...)",
            actual: args.len(),
        });
    }
    let var_sym = match &args[0] {
        Value::Sym(s) => s.clone(),
        other => {
            record_error_primitive("FOREACH");
            return Err(LispError::TypeError {
                expected: "iteration variable symbol",
                actual: other.to_string(),
            });
        }
    };
    let list_val = eval_with_depth(&args[1], env, call_depth)?;
    let items = list_val.to_vec().unwrap_or_default();
    let body = &args[2..];
    let mut last_val = Value::Nil;

    for item in items {
        Env::setq(env, var_sym.clone(), item);
        for expr in body {
            last_val = eval_with_depth(expr, env, call_depth)?;
        }
    }
    Ok(last_val)
}

/// Recursively expands a quasiquoted expression.
fn eval_quasiquote(
    expr: &Value,
    env: &mut Rc<RefCell<Env>>,
    quote_level: usize,
    call_depth: usize,
) -> Result<Value, LispError> {
    if call_depth > MAX_CALL_DEPTH {
        return Err(LispError::StackOverflow);
    }
    match expr {
        Value::Cons(c_rc) => {
            let (car, cdr) = {
                let c = c_rc.borrow();
                (c.car.clone(), c.cdr.clone())
            };

            if let Value::Sym(ref s) = car {
                match s.as_str() {
                    "QUASIQUOTE" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid quasiquote".into()))?;
                        if args.is_empty() {
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: 0,
                            });
                        }
                        let expanded =
                            eval_quasiquote(&args[0], env, quote_level + 1, call_depth)?;
                        return Ok(Value::from_slice(&[
                            Value::Sym(Symbol::new("QUASIQUOTE")),
                            expanded,
                        ]));
                    }
                    "UNQUOTE" if quote_level == 0 => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid unquote".into()))?;
                        if args.is_empty() {
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: 0,
                            });
                        }
                        return eval_with_depth(&args[0], env, call_depth);
                    }
                    "UNQUOTE" => {
                        let args = cdr
                            .to_vec()
                            .ok_or_else(|| LispError::Syntax("Invalid unquote".into()))?;
                        if args.is_empty() {
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: 0,
                            });
                        }
                        let expanded =
                            eval_quasiquote(&args[0], env, quote_level - 1, call_depth)?;
                        return Ok(Value::from_slice(&[
                            Value::Sym(Symbol::new("UNQUOTE")),
                            expanded,
                        ]));
                    }
                    "UNQUOTE-SPLICING" if quote_level == 0 => {
                        return Err(LispError::Syntax(
                            ",@ (unquote-splicing) not valid outside of a list".into(),
                        ));
                    }
                    "UNQUOTE-SPLICING" => {
                        let args = cdr.to_vec().ok_or_else(|| {
                            LispError::Syntax("Invalid unquote-splicing".into())
                        })?;
                        if args.is_empty() {
                            return Err(LispError::WrongArgCount {
                                expected: "1",
                                actual: 0,
                            });
                        }
                        let expanded =
                            eval_quasiquote(&args[0], env, quote_level - 1, call_depth)?;
                        return Ok(Value::from_slice(&[
                            Value::Sym(Symbol::new("UNQUOTE-SPLICING")),
                            expanded,
                        ]));
                    }
                    _ => {}
                }
            }

            // Normal list traversal: check if car is (UNQUOTE-SPLICING ...) at quote_level == 0
            if quote_level == 0 {
                if let Value::Cons(ref car_c) = car {
                    let (car_car, car_cdr) = {
                        let c = car_c.borrow();
                        (c.car.clone(), c.cdr.clone())
                    };
                    if let Value::Sym(ref s) = car_car {
                        if s.as_str() == "UNQUOTE-SPLICING" {
                            let args = car_cdr.to_vec().ok_or_else(|| {
                                LispError::Syntax("Invalid unquote-splicing".into())
                            })?;
                            if args.is_empty() {
                                return Err(LispError::WrongArgCount {
                                    expected: "1",
                                    actual: 0,
                                });
                            }
                            let spliced_val = eval_with_depth(&args[0], env, call_depth)?;
                            let rest = eval_quasiquote(&cdr, env, 0, call_depth)?;

                            let spliced_items = match spliced_val {
                                Value::Nil => Vec::new(),
                                Value::Cons(_) => spliced_val.to_vec().ok_or_else(|| {
                                    LispError::TypeError {
                                        expected: "proper list for unquote-splicing",
                                        actual: spliced_val.to_string(),
                                    }
                                })?,
                                other => {
                                    return Err(LispError::TypeError {
                                        expected: "list for unquote-splicing",
                                        actual: other.to_string(),
                                    })
                                }
                            };
                            let mut result = rest;
                            for item in spliced_items.into_iter().rev() {
                                result = Value::cons(item, result);
                            }
                            return Ok(result);
                        }
                    }
                }
            }

            let expanded_car = eval_quasiquote(&car, env, quote_level, call_depth)?;
            let expanded_cdr = eval_quasiquote(&cdr, env, quote_level, call_depth)?;
            Ok(Value::cons(expanded_car, expanded_cdr))
        }
        _ => Ok(expr.clone()),
    }
}

/// Applies a function to evaluated arguments.
pub fn apply(func: &Value, args: &[Value], env: &mut Rc<RefCell<Env>>) -> Result<Value, LispError> {
    apply_with_depth(func, args, env, 0)
}

/// Applies a function to evaluated arguments, tracking user function call depth.
pub fn apply_with_depth(
    func: &Value,
    args: &[Value],
    env: &mut Rc<RefCell<Env>>,
    call_depth: usize,
) -> Result<Value, LispError> {
    if call_depth > MAX_CALL_DEPTH {
        return Err(LispError::StackOverflow);
    }

    match func {
        Value::Subr {
            name,
            func: native_fn,
        } => {
            let traced = is_function_traced(name);
            if traced {
                trace_enter_subr(name, args);
            }

            let res = native_fn(args, env);

            if traced {
                trace_exit_subr(name, &res);
            }
            if let Err(ref e) = res {
                if !matches!(e, LispError::UserExit) {
                    record_error_primitive(name);
                }
            }
            res
        }

        Value::USubr(user_fn) => {
            let traced = is_function_traced(user_fn.display_name());
            if traced {
                trace_enter_usubr(user_fn, args, call_depth);
            }

            let pushed = if call_depth < MAX_ACTIVE_BT_DEPTH {
                push_active_call(user_fn.clone(), args);
                true
            } else {
                false
            };
            let _frame_guard = ActiveFrameGuard(pushed);

            let call_env = Env::with_parent(user_fn.env.clone());

            // Bind formal parameters
            for (i, param) in user_fn.params.iter().enumerate() {
                let val = args.get(i).cloned().unwrap_or(Value::Nil);
                call_env.borrow_mut().set_local(param.clone(), val);
            }

            // Initialize local variables to nil
            for local in &user_fn.locals {
                call_env.borrow_mut().set_local(local.clone(), Value::Nil);
            }

            let mut call_env_ref = call_env;
            let mut last_val = Value::Nil;
            for expr in &user_fn.body {
                match eval_with_depth(expr, &mut call_env_ref, call_depth + 1) {
                    Ok(v) => last_val = v,
                    Err(e) => {
                        if traced {
                            let err_res: Result<Value, LispError> = Err(e.clone());
                            trace_exit_usubr(user_fn, &err_res, call_depth);
                        }
                        if !matches!(e, LispError::UserExit) && call_depth < MAX_ACTIVE_BT_DEPTH {
                            record_error_frame(user_fn, args);
                        }
                        return Err(e);
                    }
                }
            }

            if traced {
                let ok_res = Ok(last_val.clone());
                trace_exit_usubr(user_fn, &ok_res, call_depth);
            }

            Ok(last_val)
        }

        Value::Cons(c_rc) => {
            let car = c_rc.borrow().car.clone();
            if let Value::Sym(ref s) = car {
                if s.as_str() == "LAMBDA" {
                    let lambda_fn = eval_with_depth(func, env, call_depth)?;
                    return apply_with_depth(&lambda_fn, args, env, call_depth);
                }
            }
            Err(LispError::Eval(format!(
                "Cannot invoke non-function value: {}",
                func
            )))
        }

        Value::Sym(sym) => {
            let resolved = env
                .borrow()
                .get(sym)
                .ok_or_else(|| LispError::UnboundSymbol(sym.to_string()))?;
            apply_with_depth(&resolved, args, env, call_depth)
        }

        other => Err(LispError::Eval(format!(
            "Cannot invoke non-function value: {}",
            other
        ))),
    }
}

fn parse_param_list(val: &Value) -> Result<(Vec<Symbol>, Vec<Symbol>), LispError> {
    let items = val
        .to_vec()
        .ok_or_else(|| LispError::Syntax("Parameter list must be a list".into()))?;
    let mut params = Vec::new();
    let mut locals = Vec::new();
    let mut in_locals = false;

    for item in items {
        match item {
            Value::Sym(s) => {
                if s.as_str() == "/" {
                    in_locals = true;
                } else if in_locals {
                    locals.push(s);
                } else {
                    params.push(s);
                }
            }
            other => {
                return Err(LispError::TypeError {
                    expected: "symbol in parameter list",
                    actual: other.to_string(),
                })
            }
        }
    }

    Ok((params, locals))
}
