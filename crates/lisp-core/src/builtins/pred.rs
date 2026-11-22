use crate::ast::{Symbol, Value};
use crate::env::Env;
use crate::error::LispError;

pub fn register(env: &mut Env) {
    env.def_subr("=", |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        match &args[0] {
            Value::Str(first_str) => {
                for arg in &args[1..] {
                    match arg {
                        Value::Str(s) if s == first_str => {}
                        _ => return Ok(Value::Nil),
                    }
                }
                Ok(Value::T)
            }
            Value::Sym(sym) => {
                for arg in &args[1..] {
                    match arg {
                        Value::Sym(s) if s == sym => {}
                        _ => return Ok(Value::Nil),
                    }
                }
                Ok(Value::T)
            }
            Value::Nil => {
                for arg in &args[1..] {
                    if !arg.is_nil() { return Ok(Value::Nil); }
                }
                Ok(Value::T)
            }
            _ => {
                let first = to_num(&args[0])?;
                for arg in &args[1..] {
                    if (to_num(arg)? - first).abs() > 1e-12 {
                        return Ok(Value::Nil);
                    }
                }
                Ok(Value::T)
            }
        }
    });

    env.def_subr("/=", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        match (&args[0], &args[1]) {
            (Value::Str(a), Value::Str(b)) => if a != b { Ok(Value::T) } else { Ok(Value::Nil) },
            (Value::Sym(a), Value::Sym(b)) => if a != b { Ok(Value::T) } else { Ok(Value::Nil) },
            (Value::Nil, Value::Nil) => Ok(Value::Nil),
            (Value::Nil, _) | (_, Value::Nil) => Ok(Value::T),
            (Value::Str(_), _) | (_, Value::Str(_)) => Ok(Value::T),
            (Value::Sym(_), _) | (_, Value::Sym(_)) => Ok(Value::T),
            _ => {
                let a = to_num(&args[0])?;
                let b = to_num(&args[1])?;
                if (a - b).abs() > 1e-12 { Ok(Value::T) } else { Ok(Value::Nil) }
            }
        }
    });

    env.def_subr("<", |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        if let Value::Str(first_str) = &args[0] {
            let mut prev = first_str.as_str();
            for arg in &args[1..] {
                match arg {
                    Value::Str(s) => {
                        if !(prev < s.as_str()) { return Ok(Value::Nil); }
                        prev = s.as_str();
                    }
                    other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
                }
            }
            return Ok(Value::T);
        }
        let mut prev = to_num(&args[0])?;
        for arg in &args[1..] {
            let curr = to_num(arg)?;
            if !(prev < curr) { return Ok(Value::Nil); }
            prev = curr;
        }
        Ok(Value::T)
    });

    env.def_subr("<=", |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        if let Value::Str(first_str) = &args[0] {
            let mut prev = first_str.as_str();
            for arg in &args[1..] {
                match arg {
                    Value::Str(s) => {
                        if !(prev <= s.as_str()) { return Ok(Value::Nil); }
                        prev = s.as_str();
                    }
                    other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
                }
            }
            return Ok(Value::T);
        }
        let mut prev = to_num(&args[0])?;
        for arg in &args[1..] {
            let curr = to_num(arg)?;
            if !(prev <= curr) { return Ok(Value::Nil); }
            prev = curr;
        }
        Ok(Value::T)
    });

    env.def_subr(">", |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        if let Value::Str(first_str) = &args[0] {
            let mut prev = first_str.as_str();
            for arg in &args[1..] {
                match arg {
                    Value::Str(s) => {
                        if !(prev > s.as_str()) { return Ok(Value::Nil); }
                        prev = s.as_str();
                    }
                    other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
                }
            }
            return Ok(Value::T);
        }
        let mut prev = to_num(&args[0])?;
        for arg in &args[1..] {
            let curr = to_num(arg)?;
            if !(prev > curr) { return Ok(Value::Nil); }
            prev = curr;
        }
        Ok(Value::T)
    });

    env.def_subr(">=", |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        if let Value::Str(first_str) = &args[0] {
            let mut prev = first_str.as_str();
            for arg in &args[1..] {
                match arg {
                    Value::Str(s) => {
                        if !(prev >= s.as_str()) { return Ok(Value::Nil); }
                        prev = s.as_str();
                    }
                    other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
                }
            }
            return Ok(Value::T);
        }
        let mut prev = to_num(&args[0])?;
        for arg in &args[1..] {
            let curr = to_num(arg)?;
            if !(prev >= curr) { return Ok(Value::Nil); }
            prev = curr;
        }
        Ok(Value::T)
    });

    env.def_subr("EQ", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        // Atoms compare by exact same-type value or handle (no Int/Real coercion);
        // lists and user functions compare by object identity.
        let is_eq = match (&args[0], &args[1]) {
            (Value::Nil, Value::Nil) => true,
            (Value::T, Value::T) => true,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Real(a), Value::Real(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Sym(a), Value::Sym(b)) => a == b,
            (Value::Cons(a), Value::Cons(b)) => std::rc::Rc::ptr_eq(a, b),
            (Value::Subr { name: a, .. }, Value::Subr { name: b, .. }) => a == b,
            (Value::USubr(a), Value::USubr(b)) => std::sync::Arc::ptr_eq(a, b),
            (Value::Ename(a), Value::Ename(b)) => a == b,
            (Value::PickSet(a), Value::PickSet(b)) => a == b,
            (Value::VlaObject(a), Value::VlaObject(b)) => a == b,
            (Value::File(a), Value::File(b)) => a == b,
            (Value::CatchError(a), Value::CatchError(b)) => a == b,
            _ => false,
        };
        Ok(if is_eq { Value::T } else { Value::Nil })
    });

    env.def_subr("EQUAL", |args, _| {
        match args.len() {
            2 => {
                if args[0] == args[1] { Ok(Value::T) } else { Ok(Value::Nil) }
            }
            3 => {
                let fuzz = match &args[2] {
                    Value::Int(n) => *n as f64,
                    Value::Real(r) => *r,
                    other => return Err(LispError::TypeError { expected: "fuzz real", actual: other.to_string() }),
                };
                if equal_with_fuzz(&args[0], &args[1], fuzz) { Ok(Value::T) } else { Ok(Value::Nil) }
            }
            n => Err(LispError::WrongArgCount { expected: "2 or 3", actual: n }),
        }
    });

    env.def_subr("NOT", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if args[0].is_nil() { Ok(Value::T) } else { Ok(Value::Nil) }
    });

    env.def_subr("NULL", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if args[0].is_nil() { Ok(Value::T) } else { Ok(Value::Nil) }
    });

    env.def_subr("ATOM", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Cons(_) => Ok(Value::Nil),
            _ => Ok(Value::T),
        }
    });

    env.def_subr("LISTP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Nil | Value::Cons(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-CONSP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Cons(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("NUMBERP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(_) | Value::Real(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("MINUSP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => if *n < 0 { Ok(Value::T) } else { Ok(Value::Nil) },
            Value::Real(r) => if *r < 0.0 { Ok(Value::T) } else { Ok(Value::Nil) },
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("ZEROP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => if *n == 0 { Ok(Value::T) } else { Ok(Value::Nil) },
            Value::Real(r) => if r.abs() < 1e-12 { Ok(Value::T) } else { Ok(Value::Nil) },
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("TYPE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if args[0].is_nil() {
            Ok(Value::Nil)
        } else {
            Ok(Value::Sym(Symbol::new(args[0].type_name())))
        }
    });
}

fn to_num(val: &Value) -> Result<f64, LispError> {
    match val {
        Value::Int(n) => Ok(*n as f64),
        Value::Real(r) => Ok(*r),
        other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
    }
}

fn equal_with_fuzz(a: &Value, b: &Value, fuzz: f64) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => (*x - *y).abs() <= fuzz as i64,
        (Value::Real(x), Value::Real(y)) => (x - y).abs() <= fuzz,
        (Value::Int(x), Value::Real(y)) => ((*x as f64) - y).abs() <= fuzz,
        (Value::Real(x), Value::Int(y)) => (x - (*y as f64)).abs() <= fuzz,
        (Value::Cons(x), Value::Cons(y)) => {
            let x_c = x.borrow();
            let y_c = y.borrow();
            equal_with_fuzz(&x_c.car, &y_c.car, fuzz) && equal_with_fuzz(&x_c.cdr, &y_c.cdr, fuzz)
        }
        (other_a, other_b) => other_a == other_b,
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Symbol, Value};
    use crate::error::LispError;
    use crate::{create_default_env, eval_str};

    fn eq_of(src: &str, env: &mut std::rc::Rc<std::cell::RefCell<crate::env::Env>>) -> Value {
        eval_str(src, env).unwrap_or_else(|e| panic!("{src} failed: {e}"))
    }

    fn assert_eq_t(src: &str, env: &mut std::rc::Rc<std::cell::RefCell<crate::env::Env>>) {
        assert_eq!(eq_of(src, env), Value::T, "expected T for {src}");
    }

    fn assert_eq_nil(src: &str, env: &mut std::rc::Rc<std::cell::RefCell<crate::env::Env>>) {
        assert_eq!(eq_of(src, env), Value::Nil, "expected nil for {src}");
    }

    #[test]
    fn eq_nil_and_t() {
        let mut env = create_default_env();
        assert_eq_t("(eq nil nil)", &mut env);
        assert_eq_t("(eq 'nil nil)", &mut env);
        assert_eq_t("(eq '() nil)", &mut env);
        assert_eq_t("(eq T T)", &mut env);
        assert_eq_t("(eq 't T)", &mut env);
        assert_eq_nil("(eq nil T)", &mut env);
    }

    #[test]
    fn eq_numbers_same_type_only() {
        let mut env = create_default_env();
        assert_eq_t("(eq 42 42)", &mut env);
        assert_eq_nil("(eq 42 43)", &mut env);
        assert_eq_t("(eq 1.5 1.5)", &mut env);
        assert_eq_t("(eq (+ 1.0 0.5) 1.5)", &mut env);
        assert_eq_t("(progn (setq r 1.5) (eq r r))", &mut env);
        assert_eq_nil("(eq 1.5 2.5)", &mut env);
        assert_eq_nil("(eq 1 1.0)", &mut env);
        assert_eq_nil("(eq \"1\" 1)", &mut env);
    }

    #[test]
    fn eq_strings_and_symbols() {
        let mut env = create_default_env();
        assert_eq_t("(eq \"LINE\" \"LINE\")", &mut env);
        assert_eq_t("(eq (strcat \"LI\" \"NE\") \"LINE\")", &mut env);
        assert_eq_t("(eq (strcase \"y\") \"Y\")", &mut env);
        assert_eq_t("(progn (setq s \"LINE\" s2 s) (eq s s2))", &mut env);
        assert_eq_nil("(eq \"LINE\" \"line\")", &mut env);
        assert_eq_t("(eq 'abc 'ABC)", &mut env);
        assert_eq_nil("(eq 'abc 'def)", &mut env);
    }

    #[test]
    fn eq_lists_by_identity() {
        let mut env = create_default_env();
        eq_of("(setq f1 '(a b c) f2 '(a b c) f3 f2)", &mut env);
        assert_eq_nil("(eq f1 f2)", &mut env);
        assert_eq_nil("(eq f1 f3)", &mut env);
        assert_eq_t("(eq f2 f3)", &mut env);
        assert_eq_nil("(eq (list 1 2) (list 1 2))", &mut env);
        assert_eq_t("(eq (cdr f1) (cdr f1))", &mut env);
        assert_eq_t("(equal f1 f2)", &mut env);
    }

    #[test]
    fn eq_functions() {
        let mut env = create_default_env();
        assert_eq_t("(eq + +)", &mut env);
        assert_eq_t("(eq car car)", &mut env);
        assert_eq_nil("(eq + -)", &mut env);
        eq_of("(defun foo (x) x)", &mut env);
        eq_of("(setq bar foo)", &mut env);
        assert_eq_t("(eq foo foo)", &mut env);
        assert_eq_t("(eq foo bar)", &mut env);
        assert_eq_nil("(eq (lambda (x) x) (lambda (x) x))", &mut env);
        eq_of("(defun g1 (x) x)", &mut env);
        eq_of("(defun g2 (x) x)", &mut env);
        assert_eq_nil("(eq g1 g2)", &mut env);
    }

    #[test]
    fn eq_handles() {
        let mut env = create_default_env();
        {
            let mut e = env.borrow_mut();
            e.set_local(Symbol::new("E1"), Value::Ename(7));
            e.set_local(Symbol::new("E1B"), Value::Ename(7));
            e.set_local(Symbol::new("E2"), Value::Ename(8));
            e.set_local(Symbol::new("SS1"), Value::PickSet(3));
            e.set_local(Symbol::new("SS2"), Value::PickSet(4));
            e.set_local(Symbol::new("O1"), Value::VlaObject(7));
            e.set_local(Symbol::new("O2"), Value::VlaObject(9));
            e.set_local(Symbol::new("F1"), Value::File(1));
            e.set_local(Symbol::new("F2"), Value::File(2));
        }
        assert_eq_t("(eq e1 e1b)", &mut env);
        assert_eq_nil("(eq e1 e2)", &mut env);
        assert_eq_t("(eq ss1 ss1)", &mut env);
        assert_eq_nil("(eq ss1 ss2)", &mut env);
        assert_eq_t("(eq o1 o1)", &mut env);
        assert_eq_nil("(eq o1 o2)", &mut env);
        assert_eq_nil("(eq e1 o1)", &mut env);
        assert_eq_t("(eq f1 f1)", &mut env);
        assert_eq_nil("(eq f1 f2)", &mut env);

        eq_of("(setq err (vl-catch-all-apply 'error '(\"boom\")) err2 err)", &mut env);
        assert_eq_t("(vl-catch-all-error-p err)", &mut env);
        assert_eq_t("(eq err err2)", &mut env);
    }

    #[test]
    fn eq_arity_errors() {
        let mut env = create_default_env();
        for src in ["(eq)", "(eq 1)", "(eq 1 2 3)"] {
            assert!(
                matches!(eval_str(src, &mut env), Err(LispError::WrongArgCount { .. })),
                "expected WrongArgCount for {src}"
            );
        }
    }
}
