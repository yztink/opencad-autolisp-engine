use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

pub fn register(env: &mut Env) {
    env.def_subr("CAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car())
    });

    env.def_subr("CDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr())
    });

    // 2-level
    env.def_subr("CAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car())
    });
    env.def_subr("CADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car())
    });
    env.def_subr("CDAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr())
    });
    env.def_subr("CDDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr())
    });

    // 3-level
    env.def_subr("CAAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().car())
    });
    env.def_subr("CAADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().car())
    });
    env.def_subr("CADAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().car())
    });
    env.def_subr("CADDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().car())
    });
    env.def_subr("CDAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().cdr())
    });
    env.def_subr("CDADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().cdr())
    });
    env.def_subr("CDDAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().cdr())
    });
    env.def_subr("CDDDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().cdr())
    });

    // 4-level
    env.def_subr("CAAAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().car().car())
    });
    env.def_subr("CAAADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().car().car())
    });
    env.def_subr("CAADAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().car().car())
    });
    env.def_subr("CAADDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().car().car())
    });
    env.def_subr("CADAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().cdr().car())
    });
    env.def_subr("CADADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().cdr().car())
    });
    env.def_subr("CADDAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().cdr().car())
    });
    env.def_subr("CADDDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().cdr().car())
    });
    env.def_subr("CDAAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().car().cdr())
    });
    env.def_subr("CDAADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().car().cdr())
    });
    env.def_subr("CDADAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().car().cdr())
    });
    env.def_subr("CDADDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().car().cdr())
    });
    env.def_subr("CDDAAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().car().cdr().cdr())
    });
    env.def_subr("CDDADR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().car().cdr().cdr())
    });
    env.def_subr("CDDDAR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].car().cdr().cdr().cdr())
    });
    env.def_subr("CDDDDR", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].cdr().cdr().cdr().cdr())
    });

    env.def_subr("CONS", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        Ok(Value::cons(args[0].clone(), args[1].clone()))
    });

    env.def_subr("LIST", |args, _| {
        Ok(Value::from_slice(args))
    });

    env.def_subr("APPEND", |args, _| {
        let mut flat = Vec::new();
        for arg in args {
            if let Some(items) = arg.to_vec() {
                flat.extend(items);
            } else if arg.is_nil() {
                // skip nil
            } else {
                return Err(LispError::TypeError { expected: "list", actual: arg.to_string() });
            }
        }
        Ok(Value::from_slice(&flat))
    });

    env.def_subr("REVERSE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if let Some(mut items) = args[0].to_vec() {
            items.reverse();
            Ok(Value::from_slice(&items))
        } else if args[0].is_nil() {
            Ok(Value::Nil)
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[0].to_string() })
        }
    });

    env.def_subr("LENGTH", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if let Some(items) = args[0].to_vec() {
            Ok(Value::Int(items.len() as i64))
        } else if args[0].is_nil() {
            Ok(Value::Int(0))
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[0].to_string() })
        }
    });

    env.def_subr("NTH", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let idx = match &args[0] {
            Value::Int(n) => *n,
            other => return Err(LispError::TypeError { expected: "integer index", actual: other.to_string() }),
        };
        if idx < 0 { return Ok(Value::Nil); }
        if let Some(items) = args[1].to_vec() {
            Ok(items.get(idx as usize).cloned().unwrap_or(Value::Nil))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("LAST", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if let Some(items) = args[0].to_vec() {
            Ok(items.last().cloned().unwrap_or(Value::Nil))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("ASSOC", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let key = &args[0];
        let mut curr = args[1].clone();

        while let Value::Cons(c_rc) = curr {
            let pair = c_rc.borrow().car.clone();
            if let Value::Cons(p_rc) = &pair {
                if &p_rc.borrow().car == key {
                    return Ok(pair);
                }
            }
            curr = c_rc.borrow().cdr.clone();
        }

        Ok(Value::Nil)
    });

    env.def_subr("MEMBER", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let item = &args[0];
        let mut curr = args[1].clone();

        while let Value::Cons(c_rc) = curr.clone() {
            if &c_rc.borrow().car == item {
                return Ok(curr);
            }
            curr = c_rc.borrow().cdr.clone();
        }

        Ok(Value::Nil)
    });

    env.def_subr("SUBST", |args, _| {
        if args.len() != 3 { return Err(LispError::WrongArgCount { expected: "3", actual: args.len() }); }
        let new_item = &args[0];
        let old_item = &args[1];
        let tree = &args[2];
        Ok(subst(new_item, old_item, tree))
    });

    env.def_subr("APPLY", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let func = &args[0];
        let arg_list = match args[1].to_vec() {
            Some(v) => v,
            None => if args[1].is_nil() { Vec::new() } else {
                return Err(LispError::TypeError { expected: "list", actual: args[1].to_string() });
            },
        };

        crate::eval::apply(func, &arg_list, env_rc)
    });

    env.def_subr("MAPCAR", |args, env_rc| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        let func = &args[0];
        let mut lists = Vec::new();

        for arg in &args[1..] {
            match arg.to_vec() {
                Some(items) => lists.push(items),
                None => if arg.is_nil() { lists.push(Vec::new()) } else {
                    return Err(LispError::TypeError { expected: "list", actual: arg.to_string() });
                },
            }
        }

        let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
        let mut results = Vec::new();

        for i in 0..min_len {
            let call_args: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
            results.push(crate::eval::apply(func, &call_args, env_rc)?);
        }

        Ok(Value::from_slice(&results))
    });
}

fn subst(new_item: &Value, old_item: &Value, tree: &Value) -> Value {
    if tree == old_item {
        return new_item.clone();
    }
    match tree {
        Value::Cons(c_rc) => {
            let (car, cdr) = {
                let c = c_rc.borrow();
                (c.car.clone(), c.cdr.clone())
            };
            Value::cons(subst(new_item, old_item, &car), subst(new_item, old_item, &cdr))
        }
        _ => tree.clone(),
    }
}
