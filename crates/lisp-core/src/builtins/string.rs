use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

pub fn register(env: &mut Env) {
    env.def_subr("STRCAT", |args, _| {
        let mut s = String::new();
        for arg in args {
            match arg {
                Value::Str(piece) => s.push_str(piece),
                other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
            }
        }
        Ok(Value::Str(s))
    });

    env.def_subr("STRLEN", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.len() as i64)),
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("SUBSTR", |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3", actual: args.len() });
        }
        let s = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        // AutoLISP substr is 1-indexed!
        let start_1based = match &args[1] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer start", actual: other.to_string() }),
        };
        if start_1based < 1 {
            return Err(LispError::Eval("substr start index must be >= 1".to_string()));
        }
        let start_idx = (start_1based - 1) as usize;
        let char_count = s.chars().count();
        if start_idx >= char_count {
            return Ok(Value::Str(String::new()));
        }

        let len = if args.len() == 3 {
            match &args[2] {
                Value::Int(n) => Some(*n.max(&0) as usize),
                other => return Err(LispError::TypeError { expected: "integer length", actual: other.to_string() }),
            }
        } else {
            None
        };

        let sub: String = s
            .chars()
            .skip(start_idx)
            .take(len.unwrap_or(char_count))
            .collect();

        Ok(Value::Str(sub))
    });

    env.def_subr("STRCASE", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2", actual: args.len() });
        }
        let s = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        let to_lower = if args.len() == 2 {
            args[1].is_truthy()
        } else {
            false
        };

        if to_lower {
            Ok(Value::Str(s.to_lowercase()))
        } else {
            Ok(Value::Str(s.to_uppercase()))
        }
    });

    env.def_subr("ITOA", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Str(n.to_string())),
            other => Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
        }
    });

    env.def_subr("ATOI", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.trim().parse::<i64>().unwrap_or(0))),
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("ATOF", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => Ok(Value::Real(s.trim().parse::<f64>().unwrap_or(0.0))),
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("RTOS", |args, _| {
        if args.is_empty() || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "1, 2, or 3", actual: args.len() });
        }
        let val = match &args[0] {
            Value::Int(n) => *n as f64,
            Value::Real(r) => *r,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let prec = if args.len() == 3 {
            match &args[2] {
                Value::Int(p) => *p.max(&0) as usize,
                _ => 4,
            }
        } else {
            4
        };

        Ok(Value::Str(format!("{:.prec$}", val, prec = prec)))
    });

    env.def_subr("ANGTOS", |args, _| {
        if args.is_empty() || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "1, 2, or 3", actual: args.len() });
        }
        let rad = match &args[0] {
            Value::Int(n) => *n as f64,
            Value::Real(r) => *r,
            other => return Err(LispError::TypeError { expected: "angle in radians", actual: other.to_string() }),
        };
        // Convert to degrees by default
        let deg = rad.to_degrees();
        Ok(Value::Str(format!("{:.4}", deg)))
    });

    env.def_subr("ANGTOF", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (string [mode])", actual: args.len() });
        }
        let s = match &args[0] {
            Value::Str(s) => s.trim(),
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        // Mode 0 = degrees (default)
        let mode = if args.len() == 2 {
            match &args[1] {
                Value::Int(m) => *m,
                _ => 0,
            }
        } else {
            0
        };

        if let Ok(val) = s.parse::<f64>() {
            let rad = match mode {
                0 => val.to_radians(), // degrees -> rad
                1 => val.to_radians(), // deg/min/sec approx
                2 => val * (std::f64::consts::PI / 200.0), // grads -> rad
                3 => val, // already radians
                _ => val.to_radians(),
            };
            Ok(Value::Real(rad))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("DISTOF", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (string [mode])", actual: args.len() });
        }
        let s = match &args[0] {
            Value::Str(s) => s.trim(),
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        if let Ok(val) = s.parse::<f64>() {
            Ok(Value::Real(val))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("ASCII", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.chars().next().map(|c| c as i64).unwrap_or(0))),
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("CHR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => {
                let c = char::from_u32(*n as u32).unwrap_or('\0');
                Ok(Value::Str(c.to_string()))
            }
            other => Err(LispError::TypeError { expected: "integer ASCII code", actual: other.to_string() }),
        }
    });

    env.def_subr("READ", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => {
                let parsed = crate::parser::parse_one(s.trim());
                Ok(parsed.unwrap_or(Value::Nil))
            }
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("WCMATCH", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let text = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        let pattern = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "pattern string", actual: other.to_string() }),
        };

        if wcmatch(text, pattern) { Ok(Value::T) } else { Ok(Value::Nil) }
    });
}

/// Wildcard matching for AutoLISP `wcmatch` patterns (*, ?, etc.)
pub(crate) fn wcmatch(text: &str, pattern: &str) -> bool {
    let patterns = pattern.split(',');
    for pat in patterns {
        if wcmatch_single(text, pat) {
            return true;
        }
    }
    false
}

fn wcmatch_single(text: &str, pattern: &str) -> bool {
    let t_chars: Vec<char> = text.chars().collect();
    let p_chars: Vec<char> = pattern.chars().collect();
    let (mut ti, mut pi) = (0, 0);
    let (mut star_pi, mut star_ti) = (None, 0);

    while ti < t_chars.len() {
        if pi < p_chars.len() && (p_chars[pi] == '?' || p_chars[pi] == t_chars[ti]) {
            ti += 1;
            pi += 1;
        } else if pi < p_chars.len() && p_chars[pi] == '*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < p_chars.len() && p_chars[pi] == '*' {
        pi += 1;
    }

    pi == p_chars.len()
}
