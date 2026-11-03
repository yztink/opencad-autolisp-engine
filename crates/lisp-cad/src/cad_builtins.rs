use std::sync::{Arc, Mutex};

use lisp_core::{Env, LispError, Value};

use crate::bridge::{CadBridge, DxfValue};
use crate::dxf::{dxf_to_lisp, lisp_to_dxf};

fn parse_point(val: &Value) -> Option<[f64; 3]> {
    let coords = val.to_vec()?;
    if (coords.len() == 2 || coords.len() == 3)
        && coords.iter().all(|c| matches!(c, Value::Int(_) | Value::Real(_)))
    {
        let x = match coords.first()? {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            _ => return None,
        };
        let y = match coords.get(1)? {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            _ => return None,
        };
        let z = match coords.get(2) {
            Some(Value::Real(r)) => *r,
            Some(Value::Int(n)) => *n as f64,
            None => 0.0,
            _ => return None,
        };
        Some([x, y, z])
    } else {
        None
    }
}

fn parse_point_list(val: &Value) -> Option<Vec<[f64; 3]>> {
    let items = val.to_vec()?;
    if items.is_empty() {
        return None;
    }
    let mut pts = Vec::with_capacity(items.len());
    for item in &items {
        pts.push(parse_point(item)?);
    }
    Some(pts)
}

pub fn register_cad_builtins(env: &mut Env, bridge: Arc<Mutex<dyn CadBridge>>) {
    // Set auto-command registration hook for (defun c:NAME ...)
    let b_cmd = bridge.clone();
    env.command_hook = Some(Arc::new(move |cmd_name: &str| {
        let mut br = b_cmd.lock().unwrap();
        let _ = br.register_command(cmd_name, "AutoLISP Registered Command");
    }));

    let b = bridge.clone();
    env.def_subr("COMMAND", move |args, _| {
        let mut tokens = Vec::new();
        for arg in args {
            match arg {
                Value::Str(s) => {
                    if s.trim().is_empty() {
                        tokens.push("ENTER".to_string());
                    } else {
                        tokens.push(s.clone());
                    }
                }
                Value::Int(n) => tokens.push(n.to_string()),
                Value::Real(r) => tokens.push(format!("{:.4}", r)),
                Value::Sym(s) => tokens.push(s.to_string()),
                Value::Cons(_) => {
                    // Coordinates list '(x y [z])
                    if let Some(coords) = arg.to_vec() {
                        let formatted: Vec<String> = coords
                            .iter()
                            .map(|c| match c {
                                Value::Real(r) => format!("{:.4}", r),
                                Value::Int(n) => format!("{:.4}", *n as f64),
                                other => other.to_string(),
                            })
                            .collect();
                        tokens.push(formatted.join(","));
                    } else {
                        tokens.push(arg.to_string());
                    }
                }
                Value::Ename(h) => tokens.push(format!("{:X}", h)),
                Value::PickSet(ss_id) => {
                    let len = {
                        let br = b.lock().unwrap();
                        br.ss_length(*ss_id)
                    };
                    let br = b.lock().unwrap();
                    for idx in 0..len {
                        if let Some(ename) = br.ss_name(*ss_id, idx) {
                            tokens.push(format!("{:X}", ename));
                        }
                    }
                }
                Value::Nil => tokens.push(String::new()),
                other => tokens.push(other.to_string()),
            }
        }

        let mut br = b.lock().unwrap();
        br.execute_command_args(&tokens).map_err(LispError::Cad)?;
        Ok(Value::Nil)
    });

    // ── 2. Entity Database Functions ──────────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("ENTGET", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2", actual: args.len() });
        }
        let ename = match &args[0] {
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        };

        let app_list: Option<Vec<String>> = if args.len() == 2 {
            match &args[1] {
                Value::Cons(_) | Value::Nil => {
                    let mut list = Vec::new();
                    if let Some(items) = args[1].to_vec() {
                        for it in items {
                            if let Value::Str(s) = it {
                                list.push(s);
                            }
                        }
                    }
                    Some(list)
                }
                Value::Str(s) => Some(vec![s.clone()]),
                _ => None,
            }
        } else {
            None
        };

        let br = b.lock().unwrap();
        let records = br.ent_get(ename, app_list.as_deref()).map_err(LispError::Cad)?;
        Ok(dxf_to_lisp(&records))
    });

    let b = bridge.clone();
    env.def_subr("ENTMOD", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let records = lisp_to_dxf(&args[0])?;
        let mut br = b.lock().unwrap();
        br.ent_mod(&records).map_err(LispError::Cad)?;
        Ok(args[0].clone())
    });

    let b = bridge.clone();
    env.def_subr("ENTMAKE", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let records = lisp_to_dxf(&args[0])?;
        let mut br = b.lock().unwrap();
        br.ent_make(&records).map_err(LispError::Cad)?;
        Ok(args[0].clone())
    });

    let b = bridge.clone();
    env.def_subr("ENTMAKEX", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let records = lisp_to_dxf(&args[0])?;
        let mut br = b.lock().unwrap();
        let handle = br.ent_make(&records).map_err(LispError::Cad)?;
        Ok(Value::Ename(handle))
    });

    let b = bridge.clone();
    env.def_subr("ENTDEL", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let ename = match &args[0] {
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        };
        let mut br = b.lock().unwrap();
        br.ent_del(ename).map_err(LispError::Cad)?;
        Ok(args[0].clone())
    });

    let b = bridge.clone();
    env.def_subr("ENTNEXT", move |args, _| {
        if args.len() > 1 {
            return Err(LispError::WrongArgCount { expected: "0 or 1", actual: args.len() });
        }
        let handle = if args.len() == 1 {
            match &args[0] {
                Value::Ename(h) => Some(*h),
                Value::Nil => None,
                other => return Err(LispError::TypeError { expected: "ename or nil", actual: other.to_string() }),
            }
        } else {
            None
        };

        let br = b.lock().unwrap();
        match br.ent_next(handle) {
            Some(h) => Ok(Value::Ename(h)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("ENTLAST", move |_args, _| {
        let br = b.lock().unwrap();
        match br.ent_last() {
            Some(h) => Ok(Value::Ename(h)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("ENTUPD", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let ename = match &args[0] {
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        };
        // Verify entity exists
        let br = b.lock().unwrap();
        let _ = br.ent_get(ename, None).map_err(LispError::Cad)?;
        Ok(args[0].clone())
    });

    let b = bridge.clone();
    env.def_subr("HANDENT", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (hex handle string)", actual: args.len() });
        }
        let handle_str = match &args[0] {
            Value::Str(s) => s.trim(),
            other => return Err(LispError::TypeError { expected: "string handle", actual: other.to_string() }),
        };
        match u64::from_str_radix(handle_str, 16) {
            Ok(h) => {
                let br = b.lock().unwrap();
                if br.ent_get(h, None).is_ok() {
                    Ok(Value::Ename(h))
                } else {
                    Ok(Value::Nil)
                }
            }
            Err(_) => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("ENTSEL", move |args, _| {
        if args.len() > 1 {
            return Err(LispError::WrongArgCount { expected: "0 or 1 ([prompt])", actual: args.len() });
        }
        let prompt = if args.len() == 1 {
            match &args[0] {
                Value::Str(s) => Some(s.as_str()),
                _ => None,
            }
        } else {
            None
        };
        let mut br = b.lock().unwrap();
        match br.ent_sel(prompt).map_err(LispError::Cad)? {
            Some((h, pt)) => {
                let pt_val = Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]);
                Ok(Value::from_slice(&[Value::Ename(h), pt_val]))
            }
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("NENTSEL", move |args, _| {
        if args.len() > 1 {
            return Err(LispError::WrongArgCount { expected: "0 or 1 ([prompt])", actual: args.len() });
        }
        let prompt = if args.len() == 1 {
            match &args[0] {
                Value::Str(s) => Some(s.as_str()),
                _ => None,
            }
        } else {
            None
        };
        let mut br = b.lock().unwrap();
        match br.ent_sel(prompt).map_err(LispError::Cad)? {
            Some((h, pt)) => {
                let pt_val = Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]);
                Ok(Value::from_slice(&[Value::Ename(h), pt_val]))
            }
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("TBLSEARCH", move |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3 (table-name symbol [set-next])", actual: args.len() });
        }
        let tbl_name = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "table name string", actual: other.to_string() }),
        };
        let sym_name = match &args[1] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "symbol name string", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        match br.tbl_search(tbl_name, sym_name) {
            Some(records) => Ok(crate::dxf::dxf_to_lisp(&records)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("TBLNEXT", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (table-name [rewind])", actual: args.len() });
        }
        let tbl_name = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "table name string", actual: other.to_string() }),
        };
        let rewind = args.len() == 2 && args[1] != Value::Nil;
        let mut br = b.lock().unwrap();
        match br.tbl_next(tbl_name, rewind) {
            Some(records) => Ok(crate::dxf::dxf_to_lisp(&records)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("REGAPP", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let app_name = match &args[0] {
            Value::Str(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "string", actual: other.to_string() }),
        };
        let mut br = b.lock().unwrap();
        let newly_registered = br.reg_app(app_name).map_err(LispError::Cad)?;
        if newly_registered {
            Ok(Value::Str(app_name.to_string()))
        } else {
            Ok(Value::Nil)
        }
    });

    // ── 3. Selection Sets: (ssget ...) ────────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("SSGET", move |args, _| {
        let mut mode_str = String::new();
        let mut points: Option<Vec<[f64; 3]>> = None;
        let mut filter = None;

        let mut arg_idx = 0;
        if arg_idx < args.len() {
            match &args[arg_idx] {
                Value::Str(s) => {
                    mode_str = s.clone();
                    arg_idx += 1;
                }
                Value::Nil => {
                    arg_idx += 1;
                }
                _ => {}
            }
        }

        // Check for point or points list
        if arg_idx < args.len() {
            if args[arg_idx] == Value::Nil {
                arg_idx += 1;
            } else if let Some(pt1) = parse_point(&args[arg_idx]) {
                arg_idx += 1;
                if arg_idx < args.len() {
                    if let Some(pt2) = parse_point(&args[arg_idx]) {
                        arg_idx += 1;
                        points = Some(vec![pt1, pt2]);
                    } else if args[arg_idx] == Value::Nil {
                        arg_idx += 1;
                        points = Some(vec![pt1]);
                    } else {
                        points = Some(vec![pt1]);
                    }
                } else {
                    points = Some(vec![pt1]);
                }
            } else if let Some(pt_list) = parse_point_list(&args[arg_idx]) {
                arg_idx += 1;
                points = Some(pt_list);
            }
        }

        // Check for DXF filter list
        if arg_idx < args.len() {
            if args[arg_idx] != Value::Nil {
                filter = Some(lisp_to_dxf(&args[arg_idx])?);
            }
        }

        let mut br = b.lock().unwrap();
        let ss_handle = br.ss_get(&mode_str, points.as_deref(), filter.as_deref()).map_err(LispError::Cad)?;
        match ss_handle {
            Some(handle) => Ok(Value::PickSet(handle)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("SSLENGTH", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let ss = match &args[0] {
            Value::PickSet(h) => *h,
            other => return Err(LispError::TypeError { expected: "pickset", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        Ok(Value::Int(br.ss_length(ss) as i64))
    });

    let b = bridge.clone();
    env.def_subr("SSNAME", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2", actual: args.len() });
        }
        let ss = match &args[0] {
            Value::PickSet(h) => *h,
            other => return Err(LispError::TypeError { expected: "pickset", actual: other.to_string() }),
        };
        let idx = match &args[1] {
            Value::Int(n) => *n.max(&0) as usize,
            other => return Err(LispError::TypeError { expected: "integer index", actual: other.to_string() }),
        };

        let br = b.lock().unwrap();
        match br.ss_name(ss, idx) {
            Some(ename) => Ok(Value::Ename(ename)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("SSADD", move |args, _| {
        if args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "0, 1, or 2", actual: args.len() });
        }
        let ename = if let Some(first) = args.first() {
            match first {
                Value::Ename(h) => Some(*h),
                Value::Int(n) => Some(*n as u64),
                Value::Nil => None,
                other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
            }
        } else {
            None
        };

        let ss_handle = if let Some(second) = args.get(1) {
            match second {
                Value::PickSet(h) => Some(*h),
                other => return Err(LispError::TypeError { expected: "pickset", actual: other.to_string() }),
            }
        } else {
            None
        };

        let mut br = b.lock().unwrap();
        if let Some(en) = ename {
            let res_ss = br.ss_add(en, ss_handle).map_err(LispError::Cad)?;
            Ok(Value::PickSet(res_ss))
        } else {
            let res_ss = br.ss_add(0, None).map_err(LispError::Cad)?;
            Ok(Value::PickSet(res_ss))
        }
    });

    let b = bridge.clone();
    env.def_subr("SSDEL", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2", actual: args.len() });
        }
        let ename = match &args[0] {
            Value::Ename(h) => *h,
            Value::Int(n) => *n as u64,
            other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        };
        let ss = match &args[1] {
            Value::PickSet(h) => *h,
            other => return Err(LispError::TypeError { expected: "pickset", actual: other.to_string() }),
        };
        let mut br = b.lock().unwrap();
        br.ss_del(ename, ss).map_err(LispError::Cad)?;
        Ok(args[1].clone())
    });

    let b = bridge.clone();
    env.def_subr("SSMEMB", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2", actual: args.len() });
        }
        let ename = match &args[0] {
            Value::Ename(h) => *h,
            Value::Int(n) => *n as u64,
            other => return Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        };
        let ss = match &args[1] {
            Value::PickSet(h) => *h,
            other => return Err(LispError::TypeError { expected: "pickset", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        if br.ss_memb(ename, ss) { Ok(Value::T) } else { Ok(Value::Nil) }
    });

    // ── 4. Interactive User Input ─────────────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("GETPOINT", move |args, _| {
        let mut base_pt = None;
        let mut prompt = "";

        for arg in args {
            match arg {
                Value::Str(p) => prompt = p.as_str(),
                Value::Cons(_) => {
                    if let Some(coords) = arg.to_vec() {
                        let x = match coords.first() { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let y = match coords.get(1) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        base_pt = Some([x, y, z]);
                    }
                }
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        match br.get_point(base_pt, prompt).map_err(LispError::Cad)? {
            Some(pt) => Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])])),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETCORNER", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (base-pt [prompt])", actual: args.len() });
        }
        let base_pt = match &args[0] {
            Value::Cons(_) => {
                let coords = args[0].to_vec().unwrap_or_default();
                let x = match coords.first() { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                let y = match coords.get(1) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                Some([x, y, z])
            }
            other => return Err(LispError::TypeError { expected: "base point list", actual: other.to_string() }),
        };
        let prompt = args.get(1).and_then(|a| match a { Value::Str(s) => Some(s.as_str()), _ => None }).unwrap_or("");

        let mut br = b.lock().unwrap();
        match br.get_point(base_pt, prompt).map_err(LispError::Cad)? {
            Some(pt) => Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])])),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETANGLE", move |args, _| {
        let mut base_pt = None;
        let mut prompt = "";

        for arg in args {
            match arg {
                Value::Str(p) => prompt = p.as_str(),
                Value::Cons(_) => {
                    if let Some(coords) = arg.to_vec() {
                        let x = match coords.first() { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let y = match coords.get(1) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        base_pt = Some([x, y, z]);
                    }
                }
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        match br.get_point(base_pt, prompt).map_err(LispError::Cad)? {
            Some(pt) => {
                let origin = base_pt.unwrap_or([0.0, 0.0, 0.0]);
                let dx = pt[0] - origin[0];
                let dy = pt[1] - origin[1];
                let mut ang = dy.atan2(dx);
                if ang < 0.0 { ang += 2.0 * std::f64::consts::PI; }
                Ok(Value::Real(ang))
            }
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETORIENT", move |args, _| {
        let mut base_pt = None;
        let mut prompt = "";

        for arg in args {
            match arg {
                Value::Str(p) => prompt = p.as_str(),
                Value::Cons(_) => {
                    if let Some(coords) = arg.to_vec() {
                        let x = match coords.first() { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let y = match coords.get(1) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        base_pt = Some([x, y, z]);
                    }
                }
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        match br.get_point(base_pt, prompt).map_err(LispError::Cad)? {
            Some(pt) => {
                let origin = base_pt.unwrap_or([0.0, 0.0, 0.0]);
                let dx = pt[0] - origin[0];
                let dy = pt[1] - origin[1];
                let mut ang = dy.atan2(dx);
                if ang < 0.0 { ang += 2.0 * std::f64::consts::PI; }
                Ok(Value::Real(ang))
            }
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETDIST", move |args, _| {
        let mut base_pt = None;
        let mut prompt = "";

        for arg in args {
            match arg {
                Value::Str(p) => prompt = p.as_str(),
                Value::Cons(_) => {
                    if let Some(coords) = arg.to_vec() {
                        let x = match coords.first() { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let y = match coords.get(1) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        base_pt = Some([x, y, z]);
                    }
                }
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        match br.get_dist(base_pt, prompt).map_err(LispError::Cad)? {
            Some(d) => Ok(Value::Real(d)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETSTRING", move |args, _| {
        let mut has_spaces = false;
        let mut prompt = "";

        for arg in args {
            match arg {
                Value::Str(p) => prompt = p.as_str(),
                Value::Int(n) => has_spaces = *n != 0,
                Value::T => has_spaces = true,
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        match br.get_string(has_spaces, prompt).map_err(LispError::Cad)? {
            Some(s) => Ok(Value::Str(s)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETINT", move |args, _| {
        let prompt = args.first().and_then(|a| match a { Value::Str(s) => Some(s.as_str()), _ => None }).unwrap_or("");
        let mut br = b.lock().unwrap();
        match br.get_int(prompt).map_err(LispError::Cad)? {
            Some(n) => Ok(Value::Int(n)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETREAL", move |args, _| {
        let prompt = args.first().and_then(|a| match a { Value::Str(s) => Some(s.as_str()), _ => None }).unwrap_or("");
        let mut br = b.lock().unwrap();
        match br.get_real(prompt).map_err(LispError::Cad)? {
            Some(r) => Ok(Value::Real(r)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("GETKWORD", move |args, _| {
        let prompt = args.first().and_then(|a| match a { Value::Str(s) => Some(s.as_str()), _ => None }).unwrap_or("");
        let mut br = b.lock().unwrap();
        match br.get_kword(prompt).map_err(LispError::Cad)? {
            Some(k) => Ok(Value::Str(k)),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("INITGET", move |args, _| {
        let mut bits = 0i32;
        let mut keywords = "";

        for arg in args {
            match arg {
                Value::Int(n) => bits = *n as i32,
                Value::Str(s) => keywords = s.as_str(),
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        br.init_get(bits, keywords);
        Ok(Value::Nil)
    });

    // ── 5. System Variables ───────────────────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("GETVAR", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let name = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "string or symbol variable name", actual: other.to_string() }),
        };

        let br = b.lock().unwrap();
        match br.get_var(name) {
            Ok(DxfValue::Str(s)) => Ok(Value::Str(s)),
            Ok(DxfValue::Int(n)) => Ok(Value::Int(n)),
            Ok(DxfValue::Real(r)) => Ok(Value::Real(r)),
            Ok(DxfValue::Point(pt)) => Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])])),
            Ok(DxfValue::Handle(h)) => Ok(Value::Str(format!("{:X}", h))),
            Ok(DxfValue::XData(_)) => Ok(Value::Nil),
            Err(_) => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("SETVAR", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2", actual: args.len() });
        }
        let name = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "string or symbol variable name", actual: other.to_string() }),
        };

        let dxf_val = match &args[1] {
            Value::Str(s) => DxfValue::Str(s.clone()),
            Value::Int(n) => DxfValue::Int(*n),
            Value::Real(r) => DxfValue::Real(*r),
            Value::Nil => DxfValue::Int(0),
            Value::T => DxfValue::Int(1),
            Value::Cons(_) => {
                if let Some(coords) = args[1].to_vec() {
                    if coords.len() >= 2 && coords.len() <= 3 && coords.iter().all(|c| matches!(c, Value::Int(_) | Value::Real(_))) {
                        let x = match &coords[0] { Value::Real(r) => *r, Value::Int(n) => *n as f64, _ => 0.0 };
                        let y = match &coords[1] { Value::Real(r) => *r, Value::Int(n) => *n as f64, _ => 0.0 };
                        let z = match coords.get(2) { Some(Value::Real(r)) => *r, Some(Value::Int(n)) => *n as f64, _ => 0.0 };
                        DxfValue::Point([x, y, z])
                    } else {
                        DxfValue::Str(args[1].to_string())
                    }
                } else {
                    DxfValue::Str(args[1].to_string())
                }
            }
            other => DxfValue::Str(other.to_string()),
        };

        let mut br = b.lock().unwrap();
        br.set_var(name, dxf_val).map_err(LispError::Cad)?;
        Ok(args[1].clone())
    });

    // ── 6. Dialogs & UI ───────────────────────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("ALERT", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1", actual: args.len() });
        }
        let msg = args[0].to_string();
        let mut br = b.lock().unwrap();
        br.alert(&msg);
        Ok(Value::Nil)
    });

    // ── 7. ActiveX Emulation: (vlax-*) ─────────────────────────────────────────

    env.def_subr("VLAX-ENAME->VLA-OBJECT", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::Ename(h) => Ok(Value::VlaObject(*h)),
            other => Err(LispError::TypeError { expected: "ename", actual: other.to_string() }),
        }
    });

    env.def_subr("VLAX-VLA-OBJECT->ENAME", |args, _| {
        if args.len() != 1 { return Err(LispError::WrongArgCount { expected: "1", actual: args.len() }); }
        match &args[0] {
            Value::VlaObject(h) => Ok(Value::Ename(*h)),
            other => Err(LispError::TypeError { expected: "vla-object", actual: other.to_string() }),
        }
    });

    let b = bridge.clone();
    env.def_subr("VLAX-GET-PROPERTY", move |args, _| {
        if args.len() != 2 { return Err(LispError::WrongArgCount { expected: "2", actual: args.len() }); }
        let handle = match &args[0] {
            Value::VlaObject(h) => *h,
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "vla-object", actual: other.to_string() }),
        };
        let prop_name = match &args[1] {
            Value::Sym(s) => s.as_str().to_ascii_uppercase(),
            Value::Str(s) => s.to_ascii_uppercase(),
            other => return Err(LispError::TypeError { expected: "property symbol or string", actual: other.to_string() }),
        };

        if prop_name == "ACTIVEDOCUMENT" || prop_name == "DOCUMENT" {
            return Ok(Value::VlaObject(2));
        }

        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;

        match prop_name.as_str() {
            "LAYER" => {
                let layer = records.iter().find(|(c, _)| *c == 8).map(|(_, v)| match v {
                    DxfValue::Str(s) => Value::Str(s.clone()),
                    _ => Value::Nil,
                }).unwrap_or(Value::Str("0".into()));
                Ok(layer)
            }
            "COLOR" => {
                let color = records.iter().find(|(c, _)| *c == 62).map(|(_, v)| match v {
                    DxfValue::Int(n) => Value::Int(*n),
                    _ => Value::Int(256), // ByLayer
                }).unwrap_or(Value::Int(256));
                Ok(color)
            }
            "ENTITYNAME" | "OBJECTNAME" => {
                let ent_type = records.iter().find(|(c, _)| *c == 0).map(|(_, v)| match v {
                    DxfValue::Str(s) => Value::Str(format!("AcDb{}", s)),
                    _ => Value::Str("AcDbEntity".into()),
                }).unwrap_or(Value::Str("AcDbEntity".into()));
                Ok(ent_type)
            }
            "HANDLE" => Ok(Value::Str(format!("{:X}", handle))),
            "COORDINATES" => {
                let info = crate::curve::extract_curve_info(&records);
                let mut flat = Vec::with_capacity(info.points.len() * 2);
                for pt in info.points {
                    flat.push(Value::Real(pt[0]));
                    flat.push(Value::Real(pt[1]));
                }
                Ok(Value::from_slice(&flat))
            }
            "AREA" => {
                let info = crate::curve::extract_curve_info(&records);
                Ok(Value::Real(crate::curve::curve_area(&info)))
            }
            "LENGTH" => {
                let info = crate::curve::extract_curve_info(&records);
                Ok(Value::Real(crate::curve::curve_total_length(&info)))
            }
            "CLOSED" => {
                let info = crate::curve::extract_curve_info(&records);
                Ok(if info.is_closed { Value::T } else { Value::Nil })
            }
            _ => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("VLAX-PUT-PROPERTY", move |args, _| {
        if args.len() != 3 { return Err(LispError::WrongArgCount { expected: "3", actual: args.len() }); }
        let handle = match &args[0] {
            Value::VlaObject(h) => *h,
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "vla-object", actual: other.to_string() }),
        };
        let prop_name = match &args[1] {
            Value::Sym(s) => s.as_str().to_ascii_uppercase(),
            Value::Str(s) => s.to_ascii_uppercase(),
            other => return Err(LispError::TypeError { expected: "property symbol", actual: other.to_string() }),
        };

        let mut br = b.lock().unwrap();
        let mut records = br.ent_get(handle, None).map_err(LispError::Cad)?;

        match prop_name.as_str() {
            "LAYER" => {
                let new_layer = match &args[2] {
                    Value::Str(s) => s.clone(),
                    other => other.to_string(),
                };
                if let Some(pos) = records.iter().position(|(c, _)| *c == 8) {
                    records[pos] = (8, DxfValue::Str(new_layer));
                } else {
                    records.push((8, DxfValue::Str(new_layer)));
                }
                br.ent_mod(&records).map_err(LispError::Cad)?;
                Ok(Value::Nil)
            }
            "COLOR" => {
                let new_color = match &args[2] {
                    Value::Int(n) => *n,
                    _ => 256,
                };
                if let Some(pos) = records.iter().position(|(c, _)| *c == 62) {
                    records[pos] = (62, DxfValue::Int(new_color));
                } else {
                    records.push((62, DxfValue::Int(new_color)));
                }
                br.ent_mod(&records).map_err(LispError::Cad)?;
                Ok(Value::Nil)
            }
            _ => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("VLAX-INVOKE-METHOD", move |args, _| {
        if args.len() < 2 { return Err(LispError::WrongArgCount { expected: "at least 2", actual: args.len() }); }
        let handle = match &args[0] {
            Value::VlaObject(h) => *h,
            Value::Ename(h) => *h,
            other => return Err(LispError::TypeError { expected: "vla-object", actual: other.to_string() }),
        };
        let method_name = match &args[1] {
            Value::Sym(s) => s.as_str().to_ascii_uppercase(),
            Value::Str(s) => s.to_ascii_uppercase(),
            other => return Err(LispError::TypeError { expected: "method symbol", actual: other.to_string() }),
        };

        let mut br = b.lock().unwrap();
        match method_name.as_str() {
            "DELETE" | "ERASE" => {
                br.ent_del(handle).map_err(LispError::Cad)?;
                Ok(Value::Nil)
            }
            "STARTUNDOMARK" => {
                br.start_undo_mark().map_err(LispError::Cad)?;
                Ok(Value::Nil)
            }
            "ENDUNDOMARK" => {
                br.end_undo_mark().map_err(LispError::Cad)?;
                Ok(Value::Nil)
            }
            _ => Ok(Value::Nil),
        }
    });

    // ── 8. LData Persistent Storage: (vlax-ldata-*) ───────────────────────────

    let b = bridge.clone();
    env.def_subr("VLAX-LDATA-PUT", move |args, _| {
        if args.len() != 3 {
            return Err(LispError::WrongArgCount { expected: "3 (dict key val)", actual: args.len() });
        }
        let dict = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "dict string/symbol", actual: other.to_string() }),
        };
        let key = match &args[1] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "key string/symbol", actual: other.to_string() }),
        };
        let val_repr = format!("{}", args[2]);
        let mut br = b.lock().unwrap();
        br.ldata_put(dict, key, &val_repr).map_err(LispError::Cad)?;
        Ok(args[2].clone())
    });

    let b = bridge.clone();
    env.def_subr("VLAX-LDATA-GET", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (dict key)", actual: args.len() });
        }
        let dict = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "dict string/symbol", actual: other.to_string() }),
        };
        let key = match &args[1] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "key string/symbol", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        match br.ldata_get(dict, key) {
            Some(s) => Ok(lisp_core::parse_one(&s).unwrap_or(Value::Str(s))),
            None => Ok(Value::Nil),
        }
    });

    let b = bridge.clone();
    env.def_subr("VLAX-LDATA-DELETE", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (dict key)", actual: args.len() });
        }
        let dict = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "dict string/symbol", actual: other.to_string() }),
        };
        let key = match &args[1] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "key string/symbol", actual: other.to_string() }),
        };
        let mut br = b.lock().unwrap();
        br.ldata_delete(dict, key).map_err(LispError::Cad)?;
        Ok(Value::T)
    });

    let b = bridge.clone();
    env.def_subr("VLAX-LDATA-LIST", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (dict)", actual: args.len() });
        }
        let dict = match &args[0] {
            Value::Str(s) => s.as_str(),
            Value::Sym(s) => s.as_str(),
            other => return Err(LispError::TypeError { expected: "dict string/symbol", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let items = br.ldata_list(dict);
        let mut list_pairs = Vec::new();
        for (k, v) in items {
            let val_node = lisp_core::parse_one(&v).unwrap_or(Value::Str(v));
            list_pairs.push(Value::cons(Value::Str(k), val_node));
        }
        Ok(Value::from_slice(&list_pairs))
    });

    // VLAX aliases
    env.def_subr("VLAX-GET", move |args, env_rc| {
        let func_opt = env_rc.borrow().get(&lisp_core::Symbol::new("VLAX-GET-PROPERTY"));
        if let Some(f) = func_opt {
            lisp_core::apply(&f, args, env_rc)
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("VLAX-PUT", move |args, env_rc| {
        let func_opt = env_rc.borrow().get(&lisp_core::Symbol::new("VLAX-PUT-PROPERTY"));
        if let Some(f) = func_opt {
            lisp_core::apply(&f, args, env_rc)
        } else {
            Ok(Value::Nil)
        }
    });

    env.def_subr("VLAX-INVOKE", move |args, env_rc| {
        let func_opt = env_rc.borrow().get(&lisp_core::Symbol::new("VLAX-INVOKE-METHOD"));
        if let Some(f) = func_opt {
            lisp_core::apply(&f, args, env_rc)
        } else {
            Ok(Value::Nil)
        }
    });

    // ActiveX Reflection & Type introspection
    env.def_subr("VLAX-METHOD-APPLICABLE-P", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (vla-object method-sym)", actual: args.len() });
        }
        match &args[0] {
            Value::VlaObject(_) | Value::Ename(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    env.def_subr("VLAX-PROPERTY-AVAILABLE-P", move |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3 (vla-object prop-sym [act])", actual: args.len() });
        }
        match &args[0] {
            Value::VlaObject(_) | Value::Ename(_) => Ok(Value::T),
            _ => Ok(Value::Nil),
        }
    });

    // ActiveX Variants and Safearrays
    env.def_subr("VLAX-MAKE-VARIANT", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (val [type])", actual: args.len() });
        }
        Ok(args[0].clone())
    });

    env.def_subr("VLAX-VARIANT-VALUE", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (var)", actual: args.len() });
        }
        Ok(args[0].clone())
    });

    env.def_subr("VLAX-SAFEARRAY->LIST", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (safearray)", actual: args.len() });
        }
        match &args[0] {
            Value::Cons(_) | Value::Nil => Ok(args[0].clone()),
            other => Ok(Value::cons(other.clone(), Value::Nil)),
        }
    });

    env.def_subr("VLAX-SAFEARRAY-GET-U-BOUND", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (safearray [dim])", actual: args.len() });
        }
        match &args[0] {
            Value::Cons(_) => {
                let len = args[0].to_vec().map(|v| v.len()).unwrap_or(0);
                Ok(Value::Int((len as i64).saturating_sub(1)))
            }
            Value::Nil => Ok(Value::Int(-1)),
            _ => Ok(Value::Int(0)),
        }
    });

    env.def_subr("VLAX-SAFEARRAY-GET-L-BOUND", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount { expected: "1 or 2 (safearray [dim])", actual: args.len() });
        }
        Ok(Value::Int(0))
    });

    env.def_subr("VLAX-MAKE-SAFEARRAY", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (type bounds-list)", actual: args.len() });
        }
        Ok(Value::Nil)
    });

    env.def_subr("VLAX-SAFEARRAY-VALUE", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (safearray)", actual: args.len() });
        }
        Ok(args[0].clone())
    });

    // ── 9. Curve Functions: (vlax-curve-*) ─────────────────────────────────────

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETSTARTPOINT", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (curve-obj)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let pt = info.points.first().copied().unwrap_or([0.0, 0.0, 0.0]);
        Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETENDPOINT", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (curve-obj)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let pt = info.points.last().copied().unwrap_or([0.0, 0.0, 0.0]);
        Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]))
    });

    env.def_subr("VLAX-CURVE-GETSTARTPARAM", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (curve-obj)", actual: args.len() });
        }
        Ok(Value::Real(0.0))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETENDPARAM", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (curve-obj)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        Ok(Value::Real(crate::curve::curve_end_param(&info)))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETPOINTATPARAM", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (curve-obj param)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let param = match &args[1] {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let pt = crate::curve::curve_point_at_param(&info, param);
        Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETFIRSTDERIV", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (curve-obj param)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let param = match &args[1] {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let deriv = crate::curve::curve_first_deriv(&info, param);
        Ok(Value::from_slice(&[Value::Real(deriv[0]), Value::Real(deriv[1]), Value::Real(deriv[2])]))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETPOINTATDIST", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (curve-obj dist)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let dist = match &args[1] {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let pt = crate::curve::curve_point_at_dist(&info, dist);
        Ok(Value::from_slice(&[Value::Real(pt[0]), Value::Real(pt[1]), Value::Real(pt[2])]))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETDISTATPARAM", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (curve-obj param)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let param = match &args[1] {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        Ok(Value::Real(crate::curve::curve_dist_at_param(&info, param)))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETPARAMATDIST", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount { expected: "2 (curve-obj dist)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let dist = match &args[1] {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            other => return Err(LispError::TypeError { expected: "number", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        Ok(Value::Real(crate::curve::curve_param_at_dist(&info, dist)))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-GETCLOSESTPOINTTO", move |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount { expected: "2 or 3 (curve-obj point [extend])", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let pt = parse_point(&args[1]).ok_or_else(|| LispError::TypeError { expected: "point list", actual: args[1].to_string() })?;
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        let closest = crate::curve::curve_closest_point_to(&info, pt);
        Ok(Value::from_slice(&[Value::Real(closest[0]), Value::Real(closest[1]), Value::Real(closest[2])]))
    });

    let b = bridge.clone();
    env.def_subr("VLAX-CURVE-ISCLOSED", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (curve-obj)", actual: args.len() });
        }
        let handle = match &args[0] {
            Value::Ename(h) | Value::VlaObject(h) => *h,
            other => return Err(LispError::TypeError { expected: "curve ename/vla-object", actual: other.to_string() }),
        };
        let br = b.lock().unwrap();
        let records = br.ent_get(handle, None).map_err(LispError::Cad)?;
        let info = crate::curve::extract_curve_info(&records);
        Ok(if info.is_closed { Value::T } else { Value::Nil })
    });

    // ── 10. Root Application and Undo Markers: (vlax-get-acad-object, vla-*) ───

    env.def_subr("VLAX-GET-ACAD-OBJECT", |args, _| {
        if !args.is_empty() {
            return Err(LispError::WrongArgCount { expected: "0", actual: args.len() });
        }
        Ok(Value::VlaObject(1))
    });

    env.def_subr("VLA-GET-ACTIVEDOCUMENT", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (acad-obj)", actual: args.len() });
        }
        Ok(Value::VlaObject(2))
    });

    let b = bridge.clone();
    env.def_subr("VLA-STARTUNDOMARK", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (doc-obj)", actual: args.len() });
        }
        let mut br = b.lock().unwrap();
        br.start_undo_mark().map_err(LispError::Cad)?;
        Ok(Value::Nil)
    });

    let b = bridge.clone();
    env.def_subr("VLA-ENDUNDOMARK", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (doc-obj)", actual: args.len() });
        }
        let mut br = b.lock().unwrap();
        br.end_undo_mark().map_err(LispError::Cad)?;
        Ok(Value::Nil)
    });

    crate::ui_builtins::register_ui_builtins(env, bridge);
}
