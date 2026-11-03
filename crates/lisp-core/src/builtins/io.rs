use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

enum FileStream {
    Reader(BufReader<File>),
    Writer(File),
}

static NEXT_FD: AtomicUsize = AtomicUsize::new(1);
static FILE_TABLE: Mutex<Option<HashMap<usize, FileStream>>> = Mutex::new(None);

fn with_file_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<usize, FileStream>) -> R,
{
    let mut guard = FILE_TABLE.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    f(guard.as_mut().unwrap())
}

pub fn register(env: &mut Env) {
    // ── Output Primitives ─────────────────────────────────────────────────────

    env.def_subr("PRINC", |args, _| {
        if args.is_empty() {
            return Ok(Value::Nil);
        }
        if args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2", actual: args.len() });
        }
        let val = &args[0];

        if args.len() == 2 {
            // Write to file descriptor
            if let Value::File(fd) = args[1] {
                with_file_table(|table| {
                    if let Some(FileStream::Writer(file)) = table.get_mut(&fd) {
                        let text = match val {
                            Value::Str(s) => s.clone(),
                            other => other.to_string(),
                        };
                        let _ = file.write_all(text.as_bytes());
                        let _ = file.flush();
                        Ok(val.clone())
                    } else {
                        Err(LispError::Io(format!("Invalid file descriptor for writing: {}", fd)))
                    }
                })?;
                return Ok(val.clone());
            }
        }

        match val {
            Value::Str(s) => crate::write_output(s),
            other => crate::write_output(&other.to_string()),
        }
        Ok(val.clone())
    });

    env.def_subr("PRIN1", |args, _| {
        if args.is_empty() {
            return Ok(Value::Str(String::new()));
        }
        if args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2", actual: args.len() });
        }
        let val = &args[0];

        if args.len() == 2 {
            // Write to file descriptor
            if let Value::File(fd) = args[1] {
                with_file_table(|table| {
                    if let Some(FileStream::Writer(file)) = table.get_mut(&fd) {
                        let text = match val {
                            Value::Str(s) => format!("\"{}\"", s),
                            other => other.to_string(),
                        };
                        let _ = file.write_all(text.as_bytes());
                        let _ = file.flush();
                        Ok(val.clone())
                    } else {
                        Err(LispError::Io(format!("Invalid file descriptor for writing: {}", fd)))
                    }
                })?;
                return Ok(val.clone());
            }
        }

        match val {
            Value::Str(s) => crate::write_output(&format!("\"{}\"", s)),
            other => crate::write_output(&other.to_string()),
        }
        Ok(val.clone())
    });

    env.def_subr("PRINT", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2", actual: args.len() });
        }
        let val = &args[0];
        let text = match val {
            Value::Str(s) => format!("\n\"{}\" ", s),
            other => format!("\n{} ", other),
        };
        crate::write_output(&text);
        Ok(val.clone())
    });

    env.def_subr("PROMPT", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        match &args[0] {
            Value::Str(s) => {
                crate::write_output(s);
                Ok(Value::Nil)
            }
            other => Err(LispError::TypeError { expected: "string prompt", actual: other.to_string() }),
        }
    });

    // ── File I/O Primitives ───────────────────────────────────────────────────

    env.def_subr("OPEN", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (filename mode)", actual: args.len() });
        }
        let filename = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };
        let mode = match &args[1] {
            Value::Str(s) => s.to_ascii_lowercase(),
            other => return Err(LispError::TypeError { expected: "mode string (\"r\", \"w\", \"a\")", actual: other.to_string() }),
        };

        let stream = match mode.as_str() {
            "r" => {
                let file = File::open(filename).map_err(|e| LispError::Io(format!("Failed to open '{}' for reading: {}", filename, e)))?;
                FileStream::Reader(BufReader::new(file))
            }
            "w" => {
                let file = File::create(filename).map_err(|e| LispError::Io(format!("Failed to create '{}' for writing: {}", filename, e)))?;
                FileStream::Writer(file)
            }
            "a" => {
                let file = OpenOptions::new().create(true).append(true).open(filename).map_err(|e| LispError::Io(format!("Failed to open '{}' for append: {}", filename, e)))?;
                FileStream::Writer(file)
            }
            other => return Err(LispError::Eval(format!("Unknown open mode: '{}' (must be \"r\", \"w\", or \"a\")", other))),
        };

        let fd = NEXT_FD.fetch_add(1, Ordering::SeqCst);
        with_file_table(|table| {
            table.insert(fd, stream);
        });

        Ok(Value::File(fd))
    });

    env.def_subr("CLOSE", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (file-descriptor)", actual: args.len() });
        }
        match &args[0] {
            Value::File(fd) => {
                with_file_table(|table| {
                    table.remove(fd);
                });
                Ok(Value::Nil)
            }
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("READ-LINE", |args, _| {
        if args.len() > 1 {
            return Err(LispError::WrongArgCount { expected: "0 or 1", actual: args.len() });
        }

        if args.len() == 1 {
            match &args[0] {
                Value::File(fd) => {
                    with_file_table(|table| {
                        if let Some(FileStream::Reader(reader)) = table.get_mut(fd) {
                            let mut line = String::new();
                            match reader.read_line(&mut line) {
                                Ok(0) => Ok(Value::Nil), // EOF
                                Ok(_) => {
                                    // Strip trailing CRLF
                                    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
                                    Ok(Value::Str(trimmed.to_string()))
                                }
                                Err(e) => Err(LispError::Io(format!("read-line error: {}", e))),
                            }
                        } else {
                            Err(LispError::Io(format!("Invalid file descriptor for reading: {}", fd)))
                        }
                    })
                }
                Value::Nil => {
                    // Read from stdin
                    let mut line = String::new();
                    let stdin = io::stdin();
                    if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
                        Ok(Value::Nil)
                    } else {
                        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
                        Ok(Value::Str(trimmed.to_string()))
                    }
                }
                other => Err(LispError::TypeError { expected: "file descriptor or nil", actual: other.to_string() }),
            }
        } else {
            // Read from stdin
            let mut line = String::new();
            let stdin = io::stdin();
            if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
                Ok(Value::Nil)
            } else {
                let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
                Ok(Value::Str(trimmed.to_string()))
            }
        }
    });

    env.def_subr("WRITE-LINE", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (string [file-descriptor])", actual: args.len() });
        }
        let text = match &args[0] {
            Value::Str(s) => s.clone(),
            other => other.to_string(),
        };

        if args.len() == 2 {
            match &args[1] {
                Value::File(fd) => {
                    with_file_table(|table| {
                        if let Some(FileStream::Writer(file)) = table.get_mut(fd) {
                            writeln!(file, "{}", text).map_err(|e| LispError::Io(format!("write-line error: {}", e)))?;
                            file.flush().map_err(|e| LispError::Io(format!("flush error: {}", e)))?;
                            Ok(Value::Str(text.clone()))
                        } else {
                            Err(LispError::Io(format!("Invalid file descriptor for writing: {}", fd)))
                        }
                    })
                }
                other => Err(LispError::TypeError { expected: "file descriptor", actual: other.to_string() }),
            }
        } else {
            crate::write_output(&format!("{}\n", text));
            Ok(Value::Str(text))
        }
    });

    env.def_subr("FINDFILE", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let filename = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };

        let path = Path::new(filename);
        if path.exists() {
            if let Ok(abs) = std::fs::canonicalize(path) {
                return Ok(Value::Str(abs.to_string_lossy().into_owned()));
            }
            Ok(Value::Str(filename.clone()))
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("LOAD", |args, env_rc| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (filename [on-failure])", actual: args.len() });
        }
        let mut filename = match &args[0] {
            Value::Str(s) => s.clone(),
            Value::Sym(s) => s.to_string(),
            other => return Err(LispError::TypeError { expected: "filename string", actual: other.to_string() }),
        };

        if !filename.ends_with(".lsp") && !filename.ends_with(".lisp") && !Path::new(&filename).exists() {
            let with_ext = format!("{}.lsp", filename);
            if Path::new(&with_ext).exists() {
                filename = with_ext;
            }
        }

        match std::fs::read_to_string(&filename) {
            Ok(code) => crate::eval_str(&code, env_rc),
            Err(e) => {
                if args.len() == 2 {
                    Ok(args[1].clone())
                } else {
                    Err(LispError::Io(format!("Cannot load file '{}': {}", filename, e)))
                }
            }
        }
    });

    env.def_subr("READ-CHAR", |args, _| {
        if args.len() > 1 {
            return Err(LispError::WrongArgCount { expected: "0 or 1 ([file-descriptor])", actual: args.len() });
        }
        if args.len() == 1 {
            match &args[0] {
                Value::File(fd) => {
                    with_file_table(|table| {
                        if let Some(FileStream::Reader(reader)) = table.get_mut(fd) {
                            let mut buf = [0u8; 1];
                            match reader.read(&mut buf) {
                                Ok(0) => Ok(Value::Nil), // EOF
                                Ok(_) => Ok(Value::Int(buf[0] as i64)),
                                Err(e) => Err(LispError::Io(format!("read-char error: {}", e))),
                            }
                        } else {
                            Err(LispError::Io(format!("Invalid file descriptor for reading: {}", fd)))
                        }
                    })
                }
                Value::Nil => {
                    let mut buf = [0u8; 1];
                    let mut stdin = io::stdin().lock();
                    match stdin.read(&mut buf) {
                        Ok(0) => Ok(Value::Nil),
                        Ok(_) => Ok(Value::Int(buf[0] as i64)),
                        Err(e) => Err(LispError::Io(format!("read-char error: {}", e))),
                    }
                }
                other => Err(LispError::TypeError { expected: "file descriptor or nil", actual: other.to_string() }),
            }
        } else {
            let mut buf = [0u8; 1];
            let mut stdin = io::stdin().lock();
            match stdin.read(&mut buf) {
                Ok(0) => Ok(Value::Nil),
                Ok(_) => Ok(Value::Int(buf[0] as i64)),
                Err(e) => Err(LispError::Io(format!("read-char error: {}", e))),
            }
        }
    });

    env.def_subr("WRITE-CHAR", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (char-code [file-descriptor])", actual: args.len() });
        }
        let code = match &args[0] {
            Value::Int(n) => *n as u8,
            other => return Err(LispError::TypeError { expected: "integer char code", actual: other.to_string() }),
        };

        if args.len() == 2 {
            match &args[1] {
                Value::File(fd) => {
                    with_file_table(|table| {
                        if let Some(FileStream::Writer(file)) = table.get_mut(fd) {
                            file.write_all(&[code]).map_err(|e| LispError::Io(format!("write-char error: {}", e)))?;
                            let _ = file.flush();
                            Ok(Value::Int(code as i64))
                        } else {
                            Err(LispError::Io(format!("Invalid file descriptor for writing: {}", fd)))
                        }
                    })
                }
                other => Err(LispError::TypeError { expected: "file descriptor", actual: other.to_string() }),
            }
        } else {
            let c_str = (code as char).to_string();
            crate::write_output(&c_str);
            Ok(Value::Int(code as i64))
        }
    });

    env.def_subr("GETENV", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (varname)", actual: args.len() });
        }
        let var_name = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string variable name", actual: other.to_string() }),
        };
        match std::env::var(var_name) {
            Ok(val) => Ok(Value::Str(val)),
            Err(_) => Ok(Value::Nil),
        }
    });

    env.def_subr("SETENV", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (varname value)", actual: args.len() });
        }
        let var_name = match &args[0] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string variable name", actual: other.to_string() }),
        };
        let val = match &args[1] {
            Value::Str(s) => s,
            other => return Err(LispError::TypeError { expected: "string value", actual: other.to_string() }),
        };
        std::env::set_var(var_name, val);
        Ok(Value::Str(val.clone()))
    });
}
