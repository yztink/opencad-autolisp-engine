use std::path::Path;

use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

pub fn register(env: &mut Env) {
    // ── Visual LISP String Utilities ──────────────────────────────────────────

    env.def_subr("VL-STRING-SEARCH", |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3", actual: args.len() });
        }
        let pattern = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "pattern string", actual: other.to_string() }),
        };
        let string = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "target string", actual: other.to_string() }),
        };
        let start_pos = if args.len() == 3 {
            match &args[2] {
                Value::Int(n) => *n.max(&0) as usize,
                other => return Err(LispError::TypeError { expected: "integer start position", actual: other.to_string() }),
            }
        } else {
            0
        };

        if start_pos >= string.len() {
            return Ok(Value::Nil);
        }

        if let Some(pos) = string[start_pos..].find(pattern) {
            Ok(Value::Int((start_pos + pos) as i64))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("VL-STRING-SUBST", |args, _| {
        if args.len() < 3 || args.len() > 4 {
            return Err(LispError::WrongArgCount { expected: "3 or 4", actual: args.len() });
        }
        let new_str = match &args[0] {
            Value::Str(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "new string", actual: other.to_string() }),
        };
        let pattern = match &args[1] {
            Value::Str(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "pattern string", actual: other.to_string() }),
        };
        let string = match &args[2] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "target string", actual: other.to_string() }),
        };
        let start_pos = if args.len() == 4 {
            match &args[3] {
                Value::Int(n) => *n.max(&0) as usize,
                other => return Err(LispError::TypeError { expected: "integer start position", actual: other.to_string() }),
            }
        } else {
            0
        };

        if start_pos >= string.len() {
            return Ok(Value::Str(string.clone()));
        }

        let prefix = &string[..start_pos];
        let remainder = &string[start_pos..];

        if let Some(pos) = remainder.find(pattern) {
            let mut result = String::with_capacity(string.len() + new_str.len());
            result.push_str(prefix);
            result.push_str(&remainder[..pos]);
            result.push_str(new_str);
            result.push_str(&remainder[pos + pattern.len()..]);
            Ok(Value::Str(result))
        } else {
            Ok(Value::Str(string.clone()))
        }
    });

    env.def_subr("VL-STRING-TRIM", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let char_set = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "char set string", actual: other.to_string() }),
        };
        let string = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "target string", actual: other.to_string() }),
        };
        let trimmed = string.trim_matches(|c| char_set.contains(c));
        Ok(Value::Str(trimmed.to_string()))
    });

    env.def_subr("VL-STRING-LEFT-TRIM", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let char_set = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "char set string", actual: other.to_string() }),
        };
        let string = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "target string", actual: other.to_string() }),
        };
        let trimmed = string.trim_start_matches(|c| char_set.contains(c));
        Ok(Value::Str(trimmed.to_string()))
    });

    env.def_subr("VL-STRING-RIGHT-TRIM", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let char_set = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "char set string", actual: other.to_string() }),
        };
        let string = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "target string", actual: other.to_string() }),
        };
        let trimmed = string.trim_end_matches(|c| char_set.contains(c));
        Ok(Value::Str(trimmed.to_string()))
    });

    env.def_subr("VL-STRING-TRANSLATE", |args, _| {
        if args.len() != 3 {
            return Err(LispError::WrongArgCount { expected: "3 (from-str to-str str)", actual: args.len() });
        }
        let from_str = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        let to_str = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        let target = match &args[2] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };

        let from_chars: Vec<char> = from_str.chars().collect();
        let to_chars: Vec<char> = to_str.chars().collect();

        let mut result = String::with_capacity(target.len());
        for c in target.chars() {
            if let Some(pos) = from_chars.iter().position(|&fc| fc == c) {
                if pos < to_chars.len() {
                    result.push(to_chars[pos]);
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        Ok(Value::Str(result))
    });

    env.def_subr("VL-STRING->LIST", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => {
                let list: Vec<Value> = s.bytes().map(|b| Value::Int(b as i64)).collect();
                Ok(Value::from_slice(&list))
            }
            other => Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        }
    });

    env.def_subr("VL-LIST->STRING", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if let Some(items) = args[0].to_vec() {
            let mut bytes = Vec::new();
            for item in items {
                match item {
                    Value::Int(n) => bytes.push(n as u8),
                    other => return Err(LispError::TypeError { expected: "integer ASCII code", actual: other.to_string() }),
                }
            }
            Ok(Value::Str(String::from_utf8_lossy(&bytes).into_owned()))
        } else if args[0].is_nil() {
            Ok(Value::Str(String::new()))
        } else {
            Err(LispError::TypeError { expected: "list of integer character codes", actual: args[0].to_string() })
        }
    });

    env.def_subr("VL-PRIN1-TO-STRING", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(Value::Str(format!("{}", args[0])))
    });

    env.def_subr("VL-PRINC-TO-STRING", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.clone())),
            other => Ok(Value::Str(format!("{}", other))),
        }
    });

    env.def_subr("VL-SYMBOL-NAME", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Sym(s) => Ok(Value::Str(s.to_string())),
            other => Err(LispError::TypeError { expected: "symbol", actual: other.to_string() }),
        }
    });

    // ── Visual LISP List Utilities ───────────────────────────────────────────

    env.def_subr("VL-REMOVE", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let item = &args[0];
        if let Some(list) = args[1].to_vec() {
            let filtered: Vec<Value> = list.into_iter().filter(|x| x != item).collect();
            Ok(Value::from_slice(&filtered))
        } else if args[1].is_nil() {
            Ok(Value::Nil)
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[1].to_string() })
        }
    });

    env.def_subr("VL-REMOVE-IF", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let pred = &args[0];
        if let Some(list) = args[1].to_vec() {
            let mut filtered = Vec::new();
            for item in list {
                let res = crate::eval::apply(pred, &[item.clone()], env_rc)?;
                if !res.is_truthy() {
                    filtered.push(item);
                }
            }
            Ok(Value::from_slice(&filtered))
        } else if args[1].is_nil() {
            Ok(Value::Nil)
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[1].to_string() })
        }
    });

    env.def_subr("VL-REMOVE-IF-NOT", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let pred = &args[0];
        if let Some(list) = args[1].to_vec() {
            let mut filtered = Vec::new();
            for item in list {
                let res = crate::eval::apply(pred, &[item.clone()], env_rc)?;
                if res.is_truthy() {
                    filtered.push(item);
                }
            }
            Ok(Value::from_slice(&filtered))
        } else if args[1].is_nil() {
            Ok(Value::Nil)
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[1].to_string() })
        }
    });

    env.def_subr("VL-POSITION", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let item = &args[0];
        if let Some(list) = args[1].to_vec() {
            for (idx, el) in list.iter().enumerate() {
                if el == item {
                    return Ok(Value::Int(idx as i64));
                }
            }
            Ok(Value::Nil)
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("VL-SORT", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let mut list = match args[0].to_vec() {
            Some(v) => v,
            None => if args[0].is_nil() { return Ok(Value::Nil); } else {
                return Err(LispError::TypeError { expected: "list", actual: args[0].to_string() });
            },
        };
        let comparator = &args[1];

        // Custom sort using the comparator lambda or subr
        let mut sort_error: Option<LispError> = None;
        list.sort_by(|a, b| {
            if sort_error.is_some() {
                return std::cmp::Ordering::Equal;
            }
            match crate::eval::apply(comparator, &[a.clone(), b.clone()], env_rc) {
                Ok(res) => {
                    if res.is_truthy() {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
                Err(e) => {
                    sort_error = Some(e);
                    std::cmp::Ordering::Equal
                }
            }
        });

        if let Some(err) = sort_error {
            return Err(err);
        }

        Ok(Value::from_slice(&list))
    });

    env.def_subr("VL-SORT-I", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let list = match args[0].to_vec() {
            Some(v) => v,
            None => if args[0].is_nil() { return Ok(Value::Nil); } else {
                return Err(LispError::TypeError { expected: "list", actual: args[0].to_string() });
            },
        };
        let comparator = &args[1];

        let mut indices: Vec<usize> = (0..list.len()).collect();
        let mut sort_error: Option<LispError> = None;
        indices.sort_by(|&i, &j| {
            if sort_error.is_some() {
                return std::cmp::Ordering::Equal;
            }
            match crate::eval::apply(comparator, &[list[i].clone(), list[j].clone()], env_rc) {
                Ok(res) => {
                    if res.is_truthy() {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
                Err(e) => {
                    sort_error = Some(e);
                    std::cmp::Ordering::Equal
                }
            }
        });

        if let Some(err) = sort_error {
            return Err(err);
        }

        let result_vals: Vec<Value> = indices.into_iter().map(|idx| Value::Int(idx as i64)).collect();
        Ok(Value::from_slice(&result_vals))
    });

    env.def_subr("VL-LIST-EXPORT", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        Ok(args[0].clone())
    });

    env.def_subr("VL-LIST-LENGTH", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        if let Some(items) = args[0].to_vec() {
            Ok(Value::Int(items.len() as i64))
        } else if args[0].is_nil() {
            Ok(Value::Int(0))
        } else {
            Err(LispError::TypeError { expected: "list", actual: args[0].to_string() })
        }
    });

    env.def_subr("VL-MEMBER-IF", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2 (predicate list)", actual: args.len() }); }
        let pred = &args[0];
        let mut curr = args[1].clone();
        while !curr.is_nil() {
            let car = curr.car();
            let res = crate::eval::apply(pred, &[car], env_rc)?;
            if res.is_truthy() {
                return Ok(curr);
            }
            curr = curr.cdr();
        }
        Ok(Value::Nil)
    });

    env.def_subr("VL-MEMBER-IF-NOT", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2 (predicate list)", actual: args.len() }); }
        let pred = &args[0];
        let mut curr = args[1].clone();
        while !curr.is_nil() {
            let car = curr.car();
            let res = crate::eval::apply(pred, &[car], env_rc)?;
            if !res.is_truthy() {
                return Ok(curr);
            }
            curr = curr.cdr();
        }
        Ok(Value::Nil)
    });

    env.def_subr("VL-EVERY", |args, env_rc| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        let pred = &args[0];
        let mut lists = Vec::new();
        for arg in &args[1..] {
            match arg.to_vec() {
                Some(items) => lists.push(items),
                None => return Ok(Value::Nil),
            }
        }
        let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
        for i in 0..min_len {
            let call_args: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
            let res = crate::eval::apply(pred, &call_args, env_rc)?;
            if !res.is_truthy() {
                return Ok(Value::Nil);
            }
        }
        Ok(Value::T)
    });

    env.def_subr("VL-SOME", |args, env_rc| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        let pred = &args[0];
        let mut lists = Vec::new();
        for arg in &args[1..] {
            match arg.to_vec() {
                Some(items) => lists.push(items),
                None => return Ok(Value::Nil),
            }
        }
        let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
        for i in 0..min_len {
            let call_args: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
            let res = crate::eval::apply(pred, &call_args, env_rc)?;
            if res.is_truthy() {
                return Ok(res);
            }
        }
        Ok(Value::Nil)
    });

    // ── Visual LISP File & Path Utilities ────────────────────────────────────

    env.def_subr("VL-FILENAME-BASE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(p) => {
                let path = Path::new(p);
                let base = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                Ok(Value::Str(base.to_string()))
            }
            other => Err(LispError::TypeError { expected: "path string", actual: other.to_string() }),
        }
    });

    env.def_subr("VL-FILENAME-DIRECTORY", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(p) => {
                let path = Path::new(p);
                let dir = path.parent().and_then(|d| d.to_str()).unwrap_or("");
                Ok(Value::Str(dir.to_string()))
            }
            other => Err(LispError::TypeError { expected: "path string", actual: other.to_string() }),
        }
    });

    env.def_subr("VL-FILENAME-EXTENSION", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Str(p) => {
                let path = Path::new(p);
                let ext = path.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
                Ok(Value::Str(ext))
            }
            other => Err(LispError::TypeError { expected: "path string", actual: other.to_string() }),
        }
    });

    env.def_subr("VL-FILE-COPY", |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3 (src dst [append])", actual: args.len() });
        }
        let src = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "source file string", actual: other.to_string() }),
        };
        let dst = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "dest file string", actual: other.to_string() }),
        };

        match std::fs::copy(src, dst) {
            Ok(bytes) => Ok(Value::Int(bytes as i64)),
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-FILE-DELETE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1 (filename)", actual: args.len() }); }
        let filename = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };
        match std::fs::remove_file(filename) {
            Ok(_) => Ok(Value::T),
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-FILE-RENAME", |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2 (old new)", actual: args.len() }); }
        let old_p = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "old filename string", actual: other.to_string() }),
        };
        let new_p = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "new filename string", actual: other.to_string() }),
        };
        match std::fs::rename(old_p, new_p) {
            Ok(_) => Ok(Value::T),
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-FILE-SIZE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1 (filename)", actual: args.len() }); }
        let filename = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };
        match std::fs::metadata(filename) {
            Ok(m) => Ok(Value::Int(m.len() as i64)),
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-FILE-SYSTIME", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1 (filename)", actual: args.len() }); }
        let filename = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };
        match std::fs::metadata(filename).and_then(|m| m.modified()) {
            Ok(time) => {
                let duration = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                Ok(Value::Int(duration.as_secs() as i64))
            }
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-DIRECTORY-FILES", |args, _| {
        if args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "0, 1, 2, or 3 ([dir] [pattern] [directories-p])", actual: args.len() });
        }
        let dir_path = if !args.is_empty() && !args[0].is_nil() {
            match &args[0] {
                Value::Str(s) => s.as_str(),
                other => return Err(LispError::TypeError { expected: "directory string", actual: other.to_string() }),
            }
        } else {
            "."
        };

        let pattern = if args.len() >= 2 && !args[1].is_nil() {
            match &args[1] {
                Value::Str(s) => Some(s.clone()),
                other => return Err(LispError::TypeError { expected: "pattern string", actual: other.to_string() }),
            }
        } else {
            None
        };

        let dir_mode = if args.len() >= 3 {
            match &args[2] {
                Value::Int(n) => *n,
                _ => 0,
            }
        } else {
            0 // 0 = files and dirs, -1 = directories only, 1 = files only
        };

        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                if dir_mode == -1 && !is_dir {
                    continue;
                }
                if dir_mode == 1 && is_dir {
                    continue;
                }

                if let Some(pat) = &pattern {
                    if !crate::builtins::string::wcmatch(&file_name, pat) {
                        continue;
                    }
                }

                results.push(Value::Str(file_name));
            }
        }
        Ok(Value::from_slice(&results))
    });

    // ── Visual LISP Catch-All Error Handling ─────────────────────────────────

    env.def_subr("VL-CATCH-ALL-APPLY", |args, env_rc| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let func = &args[0];
        let arg_list = match args[1].to_vec() {
            Some(v) => v,
            None => if args[1].is_nil() { Vec::new() } else {
                return Err(LispError::TypeError { expected: "list", actual: args[1].to_string() });
            },
        };

        match crate::eval::apply(func, &arg_list, env_rc) {
            Ok(val) => Ok(val),
            Err(e) => {
                crate::error::clear_unwind_state();
                Ok(Value::CatchError(e.to_string()))
            }
        }
    });

    env.def_subr("VL-CATCH-ALL-ERROR-P", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::CatchError(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("VL-CATCH-ALL-ERROR-MESSAGE", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::CatchError(msg) => Ok(Value::Str(msg.clone())),
            other => Err(LispError::TypeError { expected: "vl-catch-all-apply-error", actual: other.to_string() }),
        }
    });

    env.def_subr("VL-BT", |_args, _| {
        let mut frames = crate::error::get_active_call_stack();
        if let Some(diag) = crate::error::get_last_error_diagnostic() {
            for f in diag.frames {
                if !frames.iter().any(|existing| existing.call_expr == f.call_expr) {
                    frames.push(f);
                }
            }
        }
        let mut out_items = Vec::with_capacity(frames.len());
        if frames.is_empty() {
            crate::write_output("; Backtrace: (top-level)\n");
        } else {
            crate::write_output("; Backtrace:\n");
            for (idx, frame) in frames.iter().enumerate() {
                let line = format!(";   [{}] {}  -->  {}", idx, frame.call_expr, frame.defun_source);
                crate::write_output(&format!("{line}\n"));
                out_items.push(Value::Str(frame.call_expr.clone()));
            }
        }
        Ok(Value::from_slice(&out_items))
    });

    env.def_subr("VL-LOAD-COM", |_args, _| {
        // Initializes ActiveX/COM support
        Ok(Value::T)
    });
}
