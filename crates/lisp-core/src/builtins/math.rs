use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

pub fn register(env: &mut Env) {
    env.def_subr("+", |args, _| {
        let mut is_real = false;
        let mut sum_int = 0i64;
        let mut sum_real = 0.0f64;

        for arg in args {
            match arg {
                Value::Int(n) => {
                    if is_real {
                        sum_real += *n as f64;
                    } else {
                        sum_int += n;
                    }
                }
                Value::Real(r) => {
                    if !is_real {
                        is_real = true;
                        sum_real = (sum_int as f64) + r;
                    } else {
                        sum_real += r;
                    }
                }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }

        if is_real { Ok(Value::Real(sum_real)) } else { Ok(Value::Int(sum_int)) }
    });

    env.def_subr("-", |args, _| {
        if args.is_empty() {
            return Err(LispError::WrongArgCount { expected: "at least 1", actual: 0 });
        }
        if args.len() == 1 {
            return match &args[0] {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Real(r) => Ok(Value::Real(-r)),
                other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            };
        }

        let mut is_real = false;
        let mut acc_int = match &args[0] {
            Value::Int(n) => *n,
            Value::Real(_r) => { is_real = true; 0 },
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let mut acc_real = match &args[0] {
            Value::Real(r) => *r,
            _ => 0.0,
        };

        for arg in &args[1..] {
            match arg {
                Value::Int(n) => {
                    if is_real {
                        acc_real -= *n as f64;
                    } else {
                        acc_int -= n;
                    }
                }
                Value::Real(r) => {
                    if !is_real {
                        is_real = true;
                        acc_real = (acc_int as f64) - r;
                    } else {
                        acc_real -= r;
                    }
                }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }

        if is_real { Ok(Value::Real(acc_real)) } else { Ok(Value::Int(acc_int)) }
    });

    env.def_subr("*", |args, _| {
        let mut is_real = false;
        let mut prod_int = 1i64;
        let mut prod_real = 1.0f64;

        for arg in args {
            match arg {
                Value::Int(n) => {
                    if is_real {
                        prod_real *= *n as f64;
                    } else {
                        prod_int *= n;
                    }
                }
                Value::Real(r) => {
                    if !is_real {
                        is_real = true;
                        prod_real = (prod_int as f64) * r;
                    } else {
                        prod_real *= r;
                    }
                }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }

        if is_real { Ok(Value::Real(prod_real)) } else { Ok(Value::Int(prod_int)) }
    });

    env.def_subr("/", |args, _| {
        if args.is_empty() {
            return Err(LispError::WrongArgCount { expected: "at least 1", actual: 0 });
        }
        if args.len() == 1 {
            return match &args[0] {
                Value::Int(0) => Err(LispError::DivisionByZero),
                Value::Real(r) if *r == 0.0 => Err(LispError::DivisionByZero),
                Value::Int(n) => Ok(Value::Int(1 / n)),
                Value::Real(r) => Ok(Value::Real(1.0 / r)),
                other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            };
        }

        let mut is_real = false;
        let mut acc_int = match &args[0] {
            Value::Int(n) => *n,
            Value::Real(_r) => { is_real = true; 0 },
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let mut acc_real = match &args[0] {
            Value::Real(r) => *r,
            _ => 0.0,
        };

        for arg in &args[1..] {
            match arg {
                Value::Int(0) => return Err(LispError::DivisionByZero),
                Value::Real(r) if *r == 0.0 => return Err(LispError::DivisionByZero),
                Value::Int(n) => {
                    if is_real {
                        acc_real /= *n as f64;
                    } else {
                        acc_int /= n;
                    }
                }
                Value::Real(r) => {
                    if !is_real {
                        is_real = true;
                        acc_real = (acc_int as f64) / r;
                    } else {
                        acc_real /= r;
                    }
                }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }

        if is_real { Ok(Value::Real(acc_real)) } else { Ok(Value::Int(acc_int)) }
    });

    env.def_subr("1+", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Int(n + 1)),
            Value::Real(r) => Ok(Value::Real(r + 1.0)),
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("1-", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Int(n - 1)),
            Value::Real(r) => Ok(Value::Real(r - 1.0)),
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("ABS", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Int(n.abs())),
            Value::Real(r) => Ok(Value::Real(r.abs())),
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("MAX", |args, _| {
        if args.is_empty() { return Err(LispError::WrongArgCount { expected: "at least 1", actual: 0 }); }
        let mut max_val = f64::NEG_INFINITY;
        let mut is_real = false;

        for arg in args {
            match arg {
                Value::Int(n) => { max_val = max_val.max(*n as f64); }
                Value::Real(r) => { is_real = true; max_val = max_val.max(*r); }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }
        if is_real { Ok(Value::Real(max_val)) } else { Ok(Value::Int(max_val as i64)) }
    });

    env.def_subr("MIN", |args, _| {
        if args.is_empty() { return Err(LispError::WrongArgCount { expected: "at least 1", actual: 0 }); }
        let mut min_val = f64::INFINITY;
        let mut is_real = false;

        for arg in args {
            match arg {
                Value::Int(n) => { min_val = min_val.min(*n as f64); }
                Value::Real(r) => { is_real = true; min_val = min_val.min(*r); }
                other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
            }
        }
        if is_real { Ok(Value::Real(min_val)) } else { Ok(Value::Int(min_val as i64)) }
    });

    env.def_subr("REM", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        match (&args[0], &args[1]) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 { return Err(LispError::DivisionByZero); }
                Ok(Value::Int(a % b))
            }
            (a, b) => {
                let af = to_float(a)?;
                let bf = to_float(b)?;
                if bf == 0.0 { return Err(LispError::DivisionByZero); }
                Ok(Value::Real(af % bf))
            }
        }
    });

    env.def_subr("SQRT", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        let f = to_float(&args[0])?;
        if f < 0.0 { return Err(LispError::Eval("Cannot take sqrt of negative number".to_string())); }
        Ok(Value::Real(f.sqrt()))
    });

    env.def_subr("EXPT", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        match (&args[0], &args[1]) {
            (Value::Int(base), Value::Int(exp)) if *exp >= 0 && *exp <= u32::MAX as i64 => {
                Ok(Value::Int(base.pow(*exp as u32)))
            }
            (base, exp) => {
                let bf = to_float(base)?;
                let ef = to_float(exp)?;
                Ok(Value::Real(bf.powf(ef)))
            }
        }
    });

    env.def_subr("SIN", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(Value::Real(to_float(&args[0])?.sin()))
    });

    env.def_subr("COS", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(Value::Real(to_float(&args[0])?.cos()))
    });

    env.def_subr("ATAN", |args, _| {
        match args.len() {
            1 => Ok(Value::Real(to_float(&args[0])?.atan())),
            2 => Ok(Value::Real(to_float(&args[0])?.atan2(to_float(&args[1])?))),
            n => Err(LispError::WrongArgCount { expected: "1 or 2", actual: n }),
        }
    });

    env.def_subr("FLOAT", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(Value::Real(to_float(&args[0])?))
    });

    env.def_subr("FIX", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Int(*n)),
            Value::Real(r) => Ok(Value::Int(*r as i64)),
            other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        }
    });

    env.def_subr("LOG", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        let n = to_float(&args[0])?;
        if n <= 0.0 {
            return Err(LispError::Eval("log: argument must be positive".into()));
        }
        Ok(Value::Real(n.ln()))
    });

    env.def_subr("EXP", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        let n = to_float(&args[0])?;
        Ok(Value::Real(n.exp()))
    });

    env.def_subr("LOGAND", |args, _| {
        if args.is_empty() {
            return Ok(Value::Int(-1));
        }
        let mut res = match &args[0] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
        };
        for arg in &args[1..] {
            match arg {
                Value::Int(n) => res &= *n,
                other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
            }
        }
        Ok(Value::Int(res))
    });

    env.def_subr("LOGIOR", |args, _| {
        let mut res = 0i64;
        for arg in args {
            match arg {
                Value::Int(n) => res |= *n,
                other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
            }
        }
        Ok(Value::Int(res))
    });

    env.def_subr("~", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Int(n) => Ok(Value::Int(!*n)),
            other => Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
        }
    });

    env.def_subr("LSH", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let val = match &args[0] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
        };
        let shift = match &args[1] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer shift count", actual: other.to_string() }),
        };
        let res = if shift >= 0 {
            val << (shift.min(63) as u32)
        } else {
            val >> ((-shift).min(63) as u32)
        };
        Ok(Value::Int(res))
    });

    env.def_subr("BOOLE", |args, _| {
        if args.len() < 3 { return Err(LispError::WrongArgCount { expected: "at least 3 (op int1 int2 ...)", actual: args.len() }); }
        let op = match &args[0] {
            Value::Int(n) => *n as u8 & 0x0F,
            other => return Err(LispError::TypeError { expected: "integer opcode (0-15)", actual: other.to_string() }),
        };

        let eval_boole_pair = |opcode: u8, a: i64, b: i64| -> i64 {
            match opcode {
                0 => 0,
                1 => a & b,
                2 => a & !b,
                3 => a,
                4 => !a & b,
                5 => b,
                6 => a ^ b,
                7 => a | b,
                8 => !(a | b),
                9 => !(a ^ b),
                10 => !b,
                11 => a | !b,
                12 => !a,
                13 => !a | b,
                14 => !(a & b),
                15 => -1,
                _ => 0,
            }
        };

        let mut acc = match &args[1] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
        };

        for arg in &args[2..] {
            match arg {
                Value::Int(n) => acc = eval_boole_pair(op, acc, *n),
                other => return Err(LispError::TypeError { expected: "integer", actual: other.to_string() }),
            }
        }
        Ok(Value::Int(acc))
    });

    env.def_subr("GCD", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        match (&args[0], &args[1]) {
            (Value::Int(mut a), Value::Int(mut b)) => {
                while b != 0 {
                    let t = b;
                    b = a % b;
                    a = t;
                }
                Ok(Value::Int(a.abs()))
            }
            _ => Err(LispError::TypeError { expected: "two integers", actual: format!("{}, {}", args[0], args[1]) }),
        }
    });

    env.def_subr("DISTANCE", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let p1 = to_point(&args[0])?;
        let p2 = to_point(&args[1])?;
        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        let dz = p2[2] - p1[2];
        Ok(Value::Real((dx * dx + dy * dy + dz * dz).sqrt()))
    });

    env.def_subr("ANGLE", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let p1 = to_point(&args[0])?;
        let p2 = to_point(&args[1])?;
        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        let mut a = dy.atan2(dx);
        if a < 0.0 { a += 2.0 * std::f64::consts::PI; }
        Ok(Value::Real(a))
    });

    env.def_subr("POLAR", |args, _| {
        if args.len() != 3 { return Err(LispError::WrongArgCount { expected: "3", actual: args.len() }); }
        let pt = to_point(&args[0])?;
        let ang = to_float(&args[1])?;
        let dist = to_float(&args[2])?;
        let x = pt[0] + dist * ang.cos();
        let y = pt[1] + dist * ang.sin();
        let z = pt[2];
        Ok(Value::from_slice(&[Value::Real(x), Value::Real(y), Value::Real(z)]))
    });

    env.def_subr("INTERS", |args, _| {
        if args.len() < 4 || args.len() > 5 {
            return Err(LispError::WrongArgCount { expected: "4 or 5 (pt1 pt2 pt3 pt4 [onseg])", actual: args.len() });
        }
        let pt1 = to_point(&args[0])?;
        let pt2 = to_point(&args[1])?;
        let pt3 = to_point(&args[2])?;
        let pt4 = to_point(&args[3])?;
        let test_on_seg = if args.len() == 5 {
            args[4].is_truthy()
        } else {
            true // AutoLISP default: nil means infinite lines, non-nil or omitted means bounded segments
        };

        // 2D line intersection: line 1 = pt1->pt2, line 2 = pt3->pt4
        let (x1, y1) = (pt1[0], pt1[1]);
        let (x2, y2) = (pt2[0], pt2[1]);
        let (x3, y3) = (pt3[0], pt3[1]);
        let (x4, y4) = (pt4[0], pt4[1]);

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denom.abs() < 1e-12 {
            return Ok(Value::Nil); // Parallel or coincident
        }

        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

        if test_on_seg {
            if t < -1e-9 || t > 1.0 + 1e-9 || u < -1e-9 || u > 1.0 + 1e-9 {
                return Ok(Value::Nil);
            }
        }

        let ix = x1 + t * (x2 - x1);
        let iy = y1 + t * (y2 - y1);
        let iz = pt1[2] + t * (pt2[2] - pt1[2]);

        Ok(Value::from_slice(&[Value::Real(ix), Value::Real(iy), Value::Real(iz)]))
    });

    // Pi constant
    env.set_local(crate::ast::Symbol::new("PI"), Value::Real(std::f64::consts::PI));
}

fn to_float(val: &Value) -> Result<f64, LispError> {
    match val {
        Value::Int(n) => Ok(*n as f64),
        Value::Real(r) => Ok(*r),
        other => Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
    }
}

fn to_point(val: &Value) -> Result<[f64; 3], LispError> {
    if let Some(vec) = val.to_vec() {
        if vec.len() >= 2 {
            let x = to_float(&vec[0])?;
            let y = to_float(&vec[1])?;
            let z = if vec.len() >= 3 { to_float(&vec[2])? } else { 0.0 };
            return Ok([x, y, z]);
        }
    }
    Err(LispError::TypeError { expected: "2D or 3D point list", actual: val.to_string() })
}
