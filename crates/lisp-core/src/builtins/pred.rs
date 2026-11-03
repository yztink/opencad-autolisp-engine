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
        match (&args[0], &args[1]) {
            (Value::Nil, Value::Nil) => Ok(Value::T),
            (Value::T, Value::T) => Ok(Value::T),
            (Value::Int(a), Value::Int(b)) => if a == b { Ok(Value::T) } else { Ok(Value::Nil) },
            (Value::Sym(a), Value::Sym(b)) => if a == b { Ok(Value::T) } else { Ok(Value::Nil) },
            (Value::Cons(a), Value::Cons(b)) => if std::rc::Rc::ptr_eq(a, b) { Ok(Value::T) } else { Ok(Value::Nil) },
            _ => Ok(Value::Nil),
        }
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
