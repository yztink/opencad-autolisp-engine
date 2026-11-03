use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lisp_core::{Env, LispError, Symbol, Value};

use crate::bridge::CadBridge;
use crate::ui_types::{
    DialogControl, DialogDef, DialogResponse, DialogRow, DialogValue, HorizontalAlign, PanelDef,
    VerticalAlign,
};

/// Look up a key case-insensitively in an association list.
/// Handles both dotted pairs `(k . v)` and standard 2-element lists `(k v)`.
pub fn lookup_alist(key: &str, alist: &Value) -> Option<Value> {
    let mut curr = alist.clone();
    while let Value::Cons(c_rc) = curr {
        let pair = c_rc.borrow().car.clone();
        if let Value::Cons(p_rc) = &pair {
            let p_borrow = p_rc.borrow();
            let k_matches = match &p_borrow.car {
                Value::Str(s) => s.eq_ignore_ascii_case(key),
                Value::Sym(s) => s.as_str().eq_ignore_ascii_case(key),
                _ => false,
            };

            if k_matches {
                return Some(p_borrow.cdr.clone());
            }
        }
        curr = c_rc.borrow().cdr.clone();
    }
    None
}

/// Helper to extract string from either Value::Str or Value::Sym.
fn extract_str(val: &Value) -> Option<String> {
    match val {
        Value::Str(s) => Some(s.clone()),
        Value::Sym(s) => Some(s.as_str().to_string()),
        Value::Cons(c) => {
            if c.borrow().cdr.is_nil() {
                extract_str(&c.borrow().car)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Helper to extract a number as f64.
fn extract_num(val: &Value) -> Option<f64> {
    match val {
        Value::Real(r) => Some(*r),
        Value::Int(i) => Some(*i as f64),
        Value::Cons(c) => {
            if c.borrow().cdr.is_nil() {
                extract_num(&c.borrow().car)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_align_h(val: &Value) -> Option<HorizontalAlign> {
    extract_str(val).and_then(|s| match s.to_ascii_lowercase().as_str() {
        "left" => Some(HorizontalAlign::Left),
        "center" => Some(HorizontalAlign::Center),
        "right" => Some(HorizontalAlign::Right),
        _ => None,
    })
}

fn parse_align_v(val: &Value) -> Option<VerticalAlign> {
    extract_str(val).and_then(|s| match s.to_ascii_lowercase().as_str() {
        "top" => Some(VerticalAlign::Top),
        "middle" => Some(VerticalAlign::Middle),
        "bottom" => Some(VerticalAlign::Bottom),
        _ => None,
    })
}

/// Helper to convert a Lisp value to a DialogControl.
pub fn lisp_to_control(val: &Value) -> Result<DialogControl, LispError> {
    let type_val = lookup_alist("type", val).ok_or_else(|| {
        LispError::Eval(format!("Control definition missing 'type': {}", val))
    })?;
    let ctype = extract_str(&type_val)
        .ok_or_else(|| LispError::Eval("Control 'type' must be string or symbol".into()))?
        .to_ascii_uppercase();

    match ctype.as_str() {
        "ROW" => {
            let controls_val = lookup_alist("controls", val).unwrap_or(Value::Nil);
            let ctrl_vec = controls_val.to_vec().unwrap_or_default();
            let mut controls = Vec::new();
            for c in ctrl_vec {
                controls.push(lisp_to_control(&c)?);
            }
            let align_h = lookup_alist("align-h", val).as_ref().and_then(parse_align_h);
            let align_v = lookup_alist("align-v", val).as_ref().and_then(parse_align_v);
            Ok(DialogControl::Row { controls, align_h, align_v })
        }
        "COLUMN" => {
            let controls_val = lookup_alist("controls", val).unwrap_or(Value::Nil);
            let ctrl_vec = controls_val.to_vec().unwrap_or_default();
            let mut controls = Vec::new();
            for c in ctrl_vec {
                controls.push(lisp_to_control(&c)?);
            }
            let align_h = lookup_alist("align-h", val).as_ref().and_then(parse_align_h);
            let align_v = lookup_alist("align-v", val).as_ref().and_then(parse_align_v);
            Ok(DialogControl::Column { controls, align_h, align_v })
        }
        "TEXT" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Text control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let default = extract_str(&lookup_alist("default", val).unwrap_or(Value::Nil))
                .unwrap_or_default();
            Ok(DialogControl::Text { id, label, default })
        }
        "NUMBER" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Number control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let default = lookup_alist("default", val)
                .as_ref()
                .and_then(extract_num)
                .unwrap_or(0.0);
            let min = lookup_alist("min", val).as_ref().and_then(extract_num);
            let max = lookup_alist("max", val).as_ref().and_then(extract_num);
            let step = lookup_alist("step", val).as_ref().and_then(extract_num);
            Ok(DialogControl::Number {
                id,
                label,
                default,
                min,
                max,
                step,
            })
        }
        "CHECKBOX" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Checkbox control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let default = lookup_alist("default", val)
                .map(|v| v.is_truthy())
                .unwrap_or(false);
            Ok(DialogControl::Checkbox { id, label, default })
        }
        "CHOICE" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Choice control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let options_raw = lookup_alist("options", val)
                .and_then(|v| v.to_vec())
                .unwrap_or_default();
            let mut options = Vec::new();
            for opt in &options_raw {
                if let Some(s) = extract_str(opt) {
                    options.push(s);
                } else {
                    options.push(opt.to_string());
                }
            }

            let default_index = match lookup_alist("default", val) {
                Some(Value::Int(i)) => {
                    if i >= 0 && (i as usize) < options.len() {
                        i as usize
                    } else {
                        0
                    }
                }
                Some(ref v) => {
                    if let Some(s) = extract_str(v) {
                        options
                            .iter()
                            .position(|opt| opt.eq_ignore_ascii_case(&s))
                            .unwrap_or(0)
                    } else {
                        0
                    }
                }
                None => 0,
            };

            Ok(DialogControl::Choice {
                id,
                label,
                options,
                default_index,
            })
        }
        "COLOR" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Color control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let mut rgb = [0u8, 0, 0];
            if let Some(rgb_list) = lookup_alist("default", val).and_then(|v| v.to_vec()) {
                if rgb_list.len() >= 3 {
                    if let Value::Int(r) = rgb_list[0] {
                        rgb[0] = r.clamp(0, 255) as u8;
                    }
                    if let Value::Int(g) = rgb_list[1] {
                        rgb[1] = g.clamp(0, 255) as u8;
                    }
                    if let Value::Int(b) = rgb_list[2] {
                        rgb[2] = b.clamp(0, 255) as u8;
                    }
                }
            }
            Ok(DialogControl::Color {
                id,
                label,
                default: rgb,
            })
        }
        "FILE" | "FILEPICKER" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("FilePicker control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let default_path = extract_str(&lookup_alist("default", val).unwrap_or(Value::Nil))
                .unwrap_or_default();
            let filter_name = extract_str(&lookup_alist("filter", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| "All Files".into());
            let mut extensions = Vec::new();
            if let Some(ext_list) = lookup_alist("extensions", val).and_then(|v| v.to_vec()) {
                for ext in ext_list {
                    if let Some(s) = extract_str(&ext) {
                        extensions.push(s);
                    }
                }
            }
            let save = lookup_alist("save", val)
                .map(|v| v.is_truthy())
                .unwrap_or(false);
            Ok(DialogControl::FilePicker {
                id,
                label,
                default_path,
                filter_name,
                extensions,
                save,
            })
        }
        "FIELDS" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| "fields".into());
            let title = extract_str(&lookup_alist("title", val).unwrap_or(Value::Nil));
            let collapsed = lookup_alist("collapsed", val)
                .map(|v| v.is_truthy())
                .unwrap_or(false);
            let controls_val = lookup_alist("controls", val).unwrap_or(Value::Nil);
            let ctrl_vec = controls_val.to_vec().unwrap_or_default();
            let mut controls = Vec::new();
            for c in ctrl_vec {
                controls.push(lisp_to_control(&c)?);
            }
            Ok(DialogControl::Fields {
                id,
                title,
                controls,
                collapsed,
            })
        }
        "TABLE" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Table control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil));
            let mut headers = Vec::new();
            if let Some(h_list) = lookup_alist("headers", val).and_then(|v| v.to_vec()) {
                for h in h_list {
                    if let Some(s) = extract_str(&h) {
                        headers.push(s);
                    } else {
                        headers.push(h.to_string());
                    }
                }
            }
            let mut rows = Vec::new();
            if let Some(r_list) = lookup_alist("rows", val).and_then(|v| v.to_vec()) {
                for r in r_list {
                    if let Some(cell_list) = r.to_vec() {
                        let row: Vec<String> = cell_list
                            .into_iter()
                            .map(|cell| extract_str(&cell).unwrap_or_else(|| cell.to_string()))
                            .collect();
                        rows.push(row);
                    }
                }
            }
            let allow_add = lookup_alist("allow_add", val)
                .map(|v| v.is_truthy())
                .unwrap_or(true);
            let allow_delete = lookup_alist("allow_delete", val)
                .map(|v| v.is_truthy())
                .unwrap_or(true);
            Ok(DialogControl::Table {
                id,
                label,
                headers,
                rows,
                allow_add,
                allow_delete,
            })
        }
        "BUTTON" => {
            let id = extract_str(&lookup_alist("id", val).unwrap_or(Value::Nil))
                .ok_or_else(|| LispError::Eval("Button control missing 'id'".into()))?;
            let label = extract_str(&lookup_alist("label", val).unwrap_or(Value::Nil))
                .unwrap_or_else(|| id.clone());
            let action = extract_str(&lookup_alist("action", val).unwrap_or(Value::Nil));
            let align_h = lookup_alist("align-h", val).as_ref().and_then(parse_align_h);
            let width = lookup_alist("width", val).as_ref().and_then(extract_num).map(|w| w as f32);
            Ok(DialogControl::Button { id, label, action, align_h, width })
        }
        "INFO" | "INFOTEXT" => {
            let text = extract_str(&lookup_alist("text", val).unwrap_or(Value::Nil))
                .unwrap_or_default();
            let align_h = lookup_alist("align-h", val).as_ref().and_then(parse_align_h);
            Ok(DialogControl::InfoText { text, align_h })
        }
        other => Err(LispError::Eval(format!("Unknown control type: '{other}'"))),
    }
}

/// Helper to convert a Lisp item into a DialogRow.
pub fn lisp_to_row(val: &Value) -> Result<DialogRow, LispError> {
    if let Some(ctype_val) = lookup_alist("type", val) {
        if let Some(ctype) = extract_str(&ctype_val) {
            if ctype.eq_ignore_ascii_case("ROW") {
                let controls_val = lookup_alist("controls", val).unwrap_or(Value::Nil);
                let ctrl_vec = controls_val.to_vec().unwrap_or_default();
                let mut controls = Vec::new();
                for c in ctrl_vec {
                    controls.push(lisp_to_control(&c)?);
                }
                let align_h = lookup_alist("align-h", val).as_ref().and_then(parse_align_h);
                let align_v = lookup_alist("align-v", val).as_ref().and_then(parse_align_v);
                let mut row = DialogRow::new(controls);
                row.align_h = align_h;
                row.align_v = align_v;
                return Ok(row);
            }
        }
    }

    // Single control wrapped in a row
    let ctrl = lisp_to_control(val)?;
    Ok(DialogRow::single(ctrl))
}

/// Convert a Lisp dialog structure into a DialogDef.
pub fn lisp_to_dialog_def(val: &Value) -> Result<DialogDef, LispError> {
    let title = extract_str(&lookup_alist("title", val).unwrap_or(Value::Nil))
        .unwrap_or_else(|| "OpenCAD Studio".to_string());
    let mut def = DialogDef::new(title);

    if let Some(w) = lookup_alist("width", val).as_ref().and_then(extract_num) {
        def.width = Some(w as f32);
    }
    if let Some(sub) = lookup_alist("submit", val).as_ref().and_then(extract_str) {
        def.submit_label = sub;
    }
    if let Some(can) = lookup_alist("cancel", val).as_ref().and_then(extract_str) {
        def.cancel_label = can;
    }

    let rows_val = lookup_alist("rows", val).unwrap_or(Value::Nil);
    if let Some(row_vec) = rows_val.to_vec() {
        for r in row_vec {
            def.rows.push(lisp_to_row(&r)?);
        }
    }

    Ok(def)
}

/// Convert a DialogValue from Rust back to a Lisp Value.
pub fn dialog_value_to_lisp(dv: DialogValue) -> Value {
    match dv {
        DialogValue::Text(s) => Value::Str(s),
        DialogValue::Number(n) => Value::Real(n),
        DialogValue::Bool(b) => {
            if b {
                Value::T
            } else {
                Value::Nil
            }
        }
        DialogValue::Choice { text, .. } => Value::Str(text),
        DialogValue::Color([r, g, b]) => Value::from_slice(&[
            Value::Int(r as i64),
            Value::Int(g as i64),
            Value::Int(b as i64),
        ]),
        DialogValue::FilePath(p) => Value::Str(p),
        DialogValue::Fields(map) => {
            let mut pairs = Vec::new();
            for (k, v) in map {
                pairs.push(Value::cons(Value::Str(k), dialog_value_to_lisp(v)));
            }
            Value::from_slice(&pairs)
        }
        DialogValue::Table(rows) => {
            let mut row_values = Vec::new();
            for row in rows {
                let cell_values: Vec<Value> = row.into_iter().map(Value::Str).collect();
                row_values.push(Value::from_slice(&cell_values));
            }
            Value::from_slice(&row_values)
        }
    }
}

/// Convert a Lisp Value into a DialogValue (e.g. for panel updates).
pub fn lisp_to_dialog_value(val: &Value) -> DialogValue {
    match val {
        Value::Str(s) => DialogValue::Text(s.clone()),
        Value::Sym(s) => DialogValue::Text(s.as_str().to_string()),
        Value::Real(r) => DialogValue::Number(*r),
        Value::Int(i) => DialogValue::Number(*i as f64),
        Value::T => DialogValue::Bool(true),
        Value::Nil => DialogValue::Bool(false),
        Value::Cons(_) => {
            if let Some(items) = val.to_vec() {
                // Check if color list of 3 integers
                if items.len() == 3 && items.iter().all(|i| matches!(i, Value::Int(_))) {
                    let r = if let Value::Int(v) = items[0] { v.clamp(0, 255) as u8 } else { 0 };
                    let g = if let Value::Int(v) = items[1] { v.clamp(0, 255) as u8 } else { 0 };
                    let b = if let Value::Int(v) = items[2] { v.clamp(0, 255) as u8 } else { 0 };
                    return DialogValue::Color([r, g, b]);
                }
                // Check if table (list of lists)
                if !items.is_empty() && items.iter().all(|i| i.to_vec().is_some()) {
                    let mut rows = Vec::new();
                    for row_val in items {
                        if let Some(cells) = row_val.to_vec() {
                            let row_strs: Vec<String> = cells
                                .into_iter()
                                .map(|c| extract_str(&c).unwrap_or_else(|| c.to_string()))
                                .collect();
                            rows.push(row_strs);
                        }
                    }
                    return DialogValue::Table(rows);
                }
                // Check if alist of fields
                let mut map = HashMap::new();
                let mut is_alist = true;
                for item in items {
                    if let Value::Cons(p_rc) = item {
                        let p = p_rc.borrow();
                        if let Some(k) = extract_str(&p.car) {
                            map.insert(k, lisp_to_dialog_value(&p.cdr));
                        } else {
                            is_alist = false;
                            break;
                        }
                    } else {
                        is_alist = false;
                        break;
                    }
                }
                if is_alist {
                    return DialogValue::Fields(map);
                }
            }
            DialogValue::Text(val.to_string())
        }
        _ => DialogValue::Text(val.to_string()),
    }
}

/// Helper to create a standard Lisp association list pair `(key . value)`.
fn pair(key: &str, val: Value) -> Value {
    Value::cons(Value::Sym(Symbol::new(key)), val)
}

/// Register all OpenCAD GUI functions into the Lisp environment.
pub fn register_ui_builtins(env: &mut Env, bridge: Arc<Mutex<dyn CadBridge>>) {
    // ── 1. Traditional MSGBOX / DOS_MSGBOX (Legacy CAD compatible) ────────────
    let b = bridge.clone();
    let msgbox_fn = move |args: &[Value], _env: &mut _| -> Result<Value, LispError> {
        if args.is_empty() || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "1 to 4 (prompt [title [buttons [icon]]])",
                actual: args.len(),
            });
        }
        let prompt = match &args[0] {
            Value::Str(s) => s.clone(),
            other => other.to_string(),
        };
        let title = if args.len() >= 2 && !args[1].is_nil() {
            Some(match &args[1] {
                Value::Str(s) => s.clone(),
                other => other.to_string(),
            })
        } else {
            Some("OpenCAD Studio".to_string())
        };

        let mut buttons = if args.len() >= 3 {
            match &args[2] {
                Value::Int(n) => *n as i32,
                _ => 0,
            }
        } else {
            0
        };

        if args.len() >= 4 {
            if let Value::Int(icon) = &args[3] {
                buttons |= *icon as i32;
            }
        }

        let mut br = b.lock().unwrap();
        let code = br
            .msg_box(&prompt, title.as_deref(), buttons)
            .map_err(LispError::Cad)?;
        Ok(Value::Int(code as i64))
    };

    env.def_subr("MSGBOX", msgbox_fn.clone());
    env.def_subr("DOS_MSGBOX", msgbox_fn);

    // ── 2. Enhanced (ocl-msgbox prompt [title] [buttons-arg]) ──────────────────
    let b = bridge.clone();
    env.def_subr("OCL-MSGBOX", move |args, _| {
        if args.is_empty() || args.len() > 3 {
            return Err(LispError::WrongArgCount {
                expected: "1 to 3 (prompt [title] [buttons-arg])",
                actual: args.len(),
            });
        }
        let prompt = match &args[0] {
            Value::Str(s) => s.clone(),
            other => other.to_string(),
        };
        let title = if args.len() >= 2 && !args[1].is_nil() {
            Some(match &args[1] {
                Value::Str(s) => s.clone(),
                other => other.to_string(),
            })
        } else {
            Some("OpenCAD Studio".to_string())
        };

        let mut buttons = 0;
        if args.len() >= 3 {
            match &args[2] {
                Value::Int(n) => buttons = *n as i32,
                Value::Sym(s) => {
                    let s_str = s.as_str();
                    if s_str.eq_ignore_ascii_case("OKCANCEL") {
                        buttons = 1;
                    } else if s_str.eq_ignore_ascii_case("ABORTRETRYIGNORE") {
                        buttons = 2;
                    } else if s_str.eq_ignore_ascii_case("YESNOCANCEL") {
                        buttons = 3;
                    } else if s_str.eq_ignore_ascii_case("YESNO") {
                        buttons = 4;
                    } else if s_str.eq_ignore_ascii_case("RETRYCANCEL") {
                        buttons = 5;
                    }
                }
                Value::Str(s) => {
                    if s.eq_ignore_ascii_case("OKCANCEL") {
                        buttons = 1;
                    } else if s.eq_ignore_ascii_case("ABORTRETRYIGNORE") {
                        buttons = 2;
                    } else if s.eq_ignore_ascii_case("YESNOCANCEL") {
                        buttons = 3;
                    } else if s.eq_ignore_ascii_case("YESNO") {
                        buttons = 4;
                    } else if s.eq_ignore_ascii_case("RETRYCANCEL") {
                        buttons = 5;
                    }
                }
                Value::Cons(_) => {
                    if let Some(b_val) = lookup_alist("buttons", &args[2]) {
                        if let Some(s) = extract_str(&b_val) {
                            if s.eq_ignore_ascii_case("OKCANCEL") {
                                buttons = 1;
                            } else if s.eq_ignore_ascii_case("ABORTRETRYIGNORE") {
                                buttons = 2;
                            } else if s.eq_ignore_ascii_case("YESNOCANCEL") {
                                buttons = 3;
                            } else if s.eq_ignore_ascii_case("YESNO") {
                                buttons = 4;
                            } else if s.eq_ignore_ascii_case("RETRYCANCEL") {
                                buttons = 5;
                            }
                        } else if let Value::Int(n) = b_val {
                            buttons = n as i32;
                        }
                    }
                }
                _ => {}
            }
        }

        let mut br = b.lock().unwrap();
        let code = br
            .msg_box(&prompt, title.as_deref(), buttons)
            .map_err(LispError::Cad)?;

        let sym = match code {
            1 => "OK",
            2 => "CANCEL",
            3 => "ABORT",
            4 => "RETRY",
            5 => "IGNORE",
            6 => "YES",
            7 => "NO",
            _ => return Ok(Value::Int(code as i64)),
        };
        Ok(Value::Sym(Symbol::new(sym)))
    });

    // ── 3. Dialog and Control Constructors ─────────────────────────────────────

    // (ocl-row [options] ctrl1 ctrl2 ...)
    env.def_subr("OCL-ROW", |args, _| {
        let mut align_h_val = Value::Nil;
        let mut align_v_val = Value::Nil;
        let mut controls_slice = args;

        if !args.is_empty() && matches!(args[0], Value::Cons(_)) && lookup_alist("type", &args[0]).is_none() {
            if let Some(h) = lookup_alist("align-h", &args[0]) {
                align_h_val = h;
            }
            if let Some(v) = lookup_alist("align-v", &args[0]) {
                align_v_val = v;
            }
            controls_slice = &args[1..];
        } else if args.len() > 1 && matches!(args.last().unwrap(), Value::Cons(_)) && lookup_alist("type", args.last().unwrap()).is_none() {
            let last_opt = args.last().unwrap();
            if let Some(h) = lookup_alist("align-h", last_opt) {
                align_h_val = h;
            }
            if let Some(v) = lookup_alist("align-v", last_opt) {
                align_v_val = v;
            }
            controls_slice = &args[..args.len() - 1];
        }

        let controls = Value::from_slice(controls_slice);
        let mut items = vec![
            pair("TYPE", Value::Str("ROW".into())),
            pair("CONTROLS", controls),
        ];
        if !align_h_val.is_nil() {
            items.push(pair("ALIGN-H", align_h_val));
        }
        if !align_v_val.is_nil() {
            items.push(pair("ALIGN-V", align_v_val));
        }
        Ok(Value::from_slice(&items))
    });

    // (ocl-column [options] ctrl1 ctrl2 ...)
    env.def_subr("OCL-COLUMN", |args, _| {
        let mut align_h_val = Value::Nil;
        let mut align_v_val = Value::Nil;
        let mut controls_slice = args;

        if !args.is_empty() && matches!(args[0], Value::Cons(_)) && lookup_alist("type", &args[0]).is_none() {
            if let Some(h) = lookup_alist("align-h", &args[0]) {
                align_h_val = h;
            }
            if let Some(v) = lookup_alist("align-v", &args[0]) {
                align_v_val = v;
            }
            controls_slice = &args[1..];
        } else if args.len() > 1 && matches!(args.last().unwrap(), Value::Cons(_)) && lookup_alist("type", args.last().unwrap()).is_none() {
            let last_opt = args.last().unwrap();
            if let Some(h) = lookup_alist("align-h", last_opt) {
                align_h_val = h;
            }
            if let Some(v) = lookup_alist("align-v", last_opt) {
                align_v_val = v;
            }
            controls_slice = &args[..args.len() - 1];
        }

        let controls = Value::from_slice(controls_slice);
        let mut items = vec![
            pair("TYPE", Value::Str("COLUMN".into())),
            pair("CONTROLS", controls),
        ];
        if !align_h_val.is_nil() {
            items.push(pair("ALIGN-H", align_h_val));
        }
        if !align_v_val.is_nil() {
            items.push(pair("ALIGN-V", align_v_val));
        }
        Ok(Value::from_slice(&items))
    });

    // (ocl-text id label [default])
    env.def_subr("OCL-TEXT", |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount {
                expected: "2 or 3 (id label [default])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;
        let default = if args.len() >= 3 {
            extract_str(&args[2]).unwrap_or_else(|| args[2].to_string())
        } else {
            String::new()
        };

        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("TEXT".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("DEFAULT", Value::Str(default)),
        ]))
    });

    // (ocl-number id label [default_or_opts] [opts])
    env.def_subr("OCL-NUMBER", |args, _| {
        if args.len() < 2 || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "2 to 4 (id label [default|options] [options])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;

        let mut default = 0.0;
        let mut min_val = Value::Nil;
        let mut max_val = Value::Nil;
        let mut step_val = Value::Nil;

        if args.len() >= 3 {
            if let Some(num) = extract_num(&args[2]) {
                default = num;
            } else if matches!(args[2], Value::Cons(_)) {
                if let Some(d) = lookup_alist("default", &args[2]).as_ref().and_then(extract_num) {
                    default = d;
                }
                if let Some(m) = lookup_alist("min", &args[2]).as_ref().and_then(extract_num) {
                    min_val = Value::Real(m);
                }
                if let Some(m) = lookup_alist("max", &args[2]).as_ref().and_then(extract_num) {
                    max_val = Value::Real(m);
                }
                if let Some(s) = lookup_alist("step", &args[2]).as_ref().and_then(extract_num) {
                    step_val = Value::Real(s);
                }
            }
        }

        if args.len() >= 4 && matches!(args[3], Value::Cons(_)) {
            if let Some(m) = lookup_alist("min", &args[3]).as_ref().and_then(extract_num) {
                min_val = Value::Real(m);
            }
            if let Some(m) = lookup_alist("max", &args[3]).as_ref().and_then(extract_num) {
                max_val = Value::Real(m);
            }
            if let Some(s) = lookup_alist("step", &args[3]).as_ref().and_then(extract_num) {
                step_val = Value::Real(s);
            }
        }

        let mut items = vec![
            pair("TYPE", Value::Str("NUMBER".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("DEFAULT", Value::Real(default)),
        ];
        if !min_val.is_nil() {
            items.push(pair("MIN", min_val));
        }
        if !max_val.is_nil() {
            items.push(pair("MAX", max_val));
        }
        if !step_val.is_nil() {
            items.push(pair("STEP", step_val));
        }

        Ok(Value::from_slice(&items))
    });

    // (ocl-checkbox id label [default-bool] [opts])
    env.def_subr("OCL-CHECKBOX", |args, _| {
        if args.len() < 2 || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "2, 3, or 4 (id label [default] [opts])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;
        let mut default = Value::Nil;
        let mut opts = None;

        if args.len() >= 3 {
            if matches!(args[2], Value::Cons(_)) && lookup_alist("type", &args[2]).is_none() {
                opts = Some(&args[2]);
            } else if args[2].is_truthy() {
                default = Value::T;
            }
        }
        if args.len() >= 4 && matches!(args[3], Value::Cons(_)) {
            opts = Some(&args[3]);
        }

        let mut items = vec![
            pair("TYPE", Value::Str("CHECKBOX".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("DEFAULT", default),
        ];
        if let Some(opt_val) = opts {
            if let Some(h) = lookup_alist("align-h", opt_val) {
                items.push(pair("ALIGN-H", h));
            }
            if let Some(v) = lookup_alist("align-v", opt_val) {
                items.push(pair("ALIGN-V", v));
            }
        }

        Ok(Value::from_slice(&items))
    });

    // (ocl-choice id label options-list [default_or_opts])
    env.def_subr("OCL-CHOICE", |args, _| {
        if args.len() < 3 || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "3 or 4 (id label options [default|options])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;
        let options = args[2].clone();

        let mut default_val = Value::Int(0);
        if args.len() >= 4 {
            if matches!(args[3], Value::Cons(_)) {
                if let Some(def) = lookup_alist("default", &args[3]) {
                    default_val = def;
                }
            } else {
                default_val = args[3].clone();
            }
        }

        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("CHOICE".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("OPTIONS", options),
            pair("DEFAULT", default_val),
        ]))
    });

    // (ocl-color id label [default_rgb])
    env.def_subr("OCL-COLOR", |args, _| {
        if args.len() < 2 || args.len() > 3 {
            return Err(LispError::WrongArgCount {
                expected: "2 or 3 (id label [default-rgb])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;
        let default_rgb = if args.len() >= 3 {
            args[2].clone()
        } else {
            Value::from_slice(&[Value::Int(0), Value::Int(0), Value::Int(0)])
        };

        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("COLOR".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("DEFAULT", default_rgb),
        ]))
    });

    // (ocl-file id label [default_path] [options])
    env.def_subr("OCL-FILE", |args, _| {
        if args.len() < 2 || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "2 to 4 (id label [default-path] [options])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;

        let mut default_path = String::new();
        let mut filter_val = Value::Str("All Files".into());
        let mut ext_val = Value::Nil;
        let mut save_val = Value::Nil;

        if args.len() >= 3 {
            if let Some(s) = extract_str(&args[2]) {
                default_path = s;
            } else if matches!(args[2], Value::Cons(_)) {
                if let Some(f) = lookup_alist("filter", &args[2]) {
                    filter_val = f;
                }
                if let Some(e) = lookup_alist("extensions", &args[2]) {
                    ext_val = e;
                }
                if let Some(s) = lookup_alist("save", &args[2]) {
                    save_val = s;
                }
            }
        }

        if args.len() >= 4 && matches!(args[3], Value::Cons(_)) {
            if let Some(f) = lookup_alist("filter", &args[3]) {
                filter_val = f;
            }
            if let Some(e) = lookup_alist("extensions", &args[3]) {
                ext_val = e;
            }
            if let Some(s) = lookup_alist("save", &args[3]) {
                save_val = s;
            }
        }

        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("FILE".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("DEFAULT", Value::Str(default_path)),
            pair("FILTER", filter_val),
            pair("EXTENSIONS", ext_val),
            pair("SAVE", save_val),
        ]))
    });

    // (ocl-table id label headers [rows] [options])
    env.def_subr("OCL-TABLE", |args, _| {
        if args.len() < 3 || args.len() > 5 {
            return Err(LispError::WrongArgCount {
                expected: "3 to 5 (id label headers [rows] [options])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;
        let headers = args[2].clone();

        let mut rows = Value::Nil;
        let mut allow_add = Value::T;
        let mut allow_delete = Value::T;

        if args.len() >= 4 {
            if matches!(args[3], Value::Cons(_)) {
                // If it contains allow_add or allow_delete, it's options; otherwise it's rows
                if lookup_alist("allow_add", &args[3]).is_some() || lookup_alist("allow_delete", &args[3]).is_some() {
                    if let Some(a) = lookup_alist("allow_add", &args[3]) {
                        allow_add = if a.is_truthy() { Value::T } else { Value::Nil };
                    }
                    if let Some(d) = lookup_alist("allow_delete", &args[3]) {
                        allow_delete = if d.is_truthy() { Value::T } else { Value::Nil };
                    }
                } else {
                    rows = args[3].clone();
                }
            }
        }

        if args.len() >= 5 && matches!(args[4], Value::Cons(_)) {
            if let Some(a) = lookup_alist("allow_add", &args[4]) {
                allow_add = if a.is_truthy() { Value::T } else { Value::Nil };
            }
            if let Some(d) = lookup_alist("allow_delete", &args[4]) {
                allow_delete = if d.is_truthy() { Value::T } else { Value::Nil };
            }
        }

        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("TABLE".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("HEADERS", headers),
            pair("ROWS", rows),
            pair("ALLOW_ADD", allow_add),
            pair("ALLOW_DELETE", allow_delete),
        ]))
    });

    // (ocl-fields [id] title [options] ctrl1 ctrl2 ...)
    env.def_subr("OCL-FIELDS", |args, _| {
        if args.is_empty() {
            return Err(LispError::WrongArgCount {
                expected: "at least 1 (title [options] ctrl1 ...)",
                actual: args.len(),
            });
        }

        let mut idx = 0;
        let mut id = String::new();
        let mut title = String::new();

        // Check if first two args are id and title strings
        if args.len() >= 2 && extract_str(&args[0]).is_some() && extract_str(&args[1]).is_some() {
            id = extract_str(&args[0]).unwrap();
            title = extract_str(&args[1]).unwrap();
            idx = 2;
        } else if let Some(t) = extract_str(&args[0]) {
            title = t.clone();
            id = t.to_ascii_lowercase().replace(' ', "_");
            idx = 1;
        }

        let mut collapsed = Value::Nil;
        if idx < args.len() && matches!(args[idx], Value::Cons(_)) {
            if let Some(c) = lookup_alist("collapsed", &args[idx]) {
                collapsed = if c.is_truthy() { Value::T } else { Value::Nil };
                idx += 1;
            }
        }

        let controls = Value::from_slice(&args[idx..]);
        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("FIELDS".into())),
            pair("ID", Value::Str(id)),
            pair("TITLE", Value::Str(title)),
            pair("COLLAPSED", collapsed),
            pair("CONTROLS", controls),
        ]))
    });

    // (ocl-button id label [action_or_opts] [opts])
    env.def_subr("OCL-BUTTON", |args, _| {
        if args.len() < 2 || args.len() > 4 {
            return Err(LispError::WrongArgCount {
                expected: "2 to 4 (id label [action|options] [options])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let label = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol label", actual: args[1].to_string() })?;

        let mut action = Value::Nil;
        let mut align_h = Value::Nil;
        let mut width = Value::Nil;

        for arg in &args[2..] {
            if matches!(arg, Value::Cons(_)) {
                if let Some(h) = lookup_alist("align-h", arg) {
                    align_h = h;
                }
                if let Some(w) = lookup_alist("width", arg) {
                    width = w;
                }
                if let Some(act) = lookup_alist("action", arg) {
                    action = act;
                }
            } else if let Some(s) = extract_str(arg) {
                action = Value::Str(s);
            }
        }

        let mut items = vec![
            pair("TYPE", Value::Str("BUTTON".into())),
            pair("ID", Value::Str(id)),
            pair("LABEL", Value::Str(label)),
            pair("ACTION", action),
        ];
        if !align_h.is_nil() {
            items.push(pair("ALIGN-H", align_h));
        }
        if !width.is_nil() {
            items.push(pair("WIDTH", width));
        }

        Ok(Value::from_slice(&items))
    });

    // (ocl-info text [options])
    env.def_subr("OCL-INFO", |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount {
                expected: "1 or 2 (text [options])",
                actual: args.len(),
            });
        }
        let text = extract_str(&args[0]).unwrap_or_else(|| args[0].to_string());
        let mut align_h = Value::Nil;
        if args.len() >= 2 && matches!(args[1], Value::Cons(_)) {
            if let Some(h) = lookup_alist("align-h", &args[1]) {
                align_h = h;
            }
        }

        let mut items = vec![
            pair("TYPE", Value::Str("INFO".into())),
            pair("TEXT", Value::Str(text)),
        ];
        if !align_h.is_nil() {
            items.push(pair("ALIGN-H", align_h));
        }

        Ok(Value::from_slice(&items))
    });

    // (ocl-dialog title [options] row1 row2 ...)
    env.def_subr("OCL-DIALOG", |args, _| {
        if args.is_empty() {
            return Err(LispError::WrongArgCount {
                expected: "at least 1 (title [options] row1 ...)",
                actual: args.len(),
            });
        }
        let title = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol title", actual: args[0].to_string() })?;

        let mut idx = 1;
        let mut width = Value::Nil;
        let mut submit = Value::Str("OK".into());
        let mut cancel = Value::Str("Cancel".into());

        if idx < args.len() && matches!(args[idx], Value::Cons(_)) {
            // Check if it's an options alist rather than a control/row
            let has_type = lookup_alist("type", &args[idx]).is_some();
            if !has_type {
                if let Some(w) = lookup_alist("width", &args[idx]).as_ref().and_then(extract_num) {
                    width = Value::Real(w);
                }
                if let Some(s) = lookup_alist("submit", &args[idx]).as_ref().and_then(extract_str) {
                    submit = Value::Str(s);
                }
                if let Some(c) = lookup_alist("cancel", &args[idx]).as_ref().and_then(extract_str) {
                    cancel = Value::Str(c);
                }
                idx += 1;
            }
        }

        let rows = Value::from_slice(&args[idx..]);
        Ok(Value::from_slice(&[
            pair("TYPE", Value::Str("DIALOG".into())),
            pair("TITLE", Value::Str(title)),
            pair("WIDTH", width),
            pair("SUBMIT", submit),
            pair("CANCEL", cancel),
            pair("ROWS", rows),
        ]))
    });

    // ── 4. Dialog Display: (ocl-dialog-show dlg) ──────────────────────────────
    let b = bridge.clone();
    env.def_subr("OCL-DIALOG-SHOW", move |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount {
                expected: "1 (dialog-def)",
                actual: args.len(),
            });
        }

        let def = lisp_to_dialog_def(&args[0])?;
        let json_str = serde_json::to_string(&def)
            .map_err(|e| LispError::Eval(format!("Failed to serialize dialog: {e}")))?;

        let mut br = b.lock().unwrap();
        let resp_json = br.show_dialog(&json_str).map_err(LispError::Cad)?;

        let response: DialogResponse = serde_json::from_str(&resp_json)
            .map_err(|e| LispError::Eval(format!("Failed to parse dialog response: {e}")))?;

        match response {
            DialogResponse::Cancelled => Ok(Value::Nil),
            DialogResponse::Submitted(values) => {
                let mut pairs = Vec::new();
                for (k, v) in values {
                    pairs.push(Value::cons(Value::Str(k), dialog_value_to_lisp(v)));
                }
                Ok(Value::from_slice(&pairs))
            }
        }
    });

    // ── 5. Association-List Query Helpers ─────────────────────────────────────

    // (ocl-get key alist)
    env.def_subr("OCL-GET", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (key alist)",
                actual: args.len(),
            });
        }
        let key_str = match &args[0] {
            Value::Str(s) => s.clone(),
            Value::Sym(s) => s.as_str().to_string(),
            other => other.to_string(),
        };

        Ok(lookup_alist(&key_str, &args[1]).unwrap_or(Value::Nil))
    });

    // (ocl-get-int key alist)
    env.def_subr("OCL-GET-INT", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (key alist)",
                actual: args.len(),
            });
        }
        let key_str = extract_str(&args[0]).unwrap_or_else(|| args[0].to_string());
        if let Some(val) = lookup_alist(&key_str, &args[1]) {
            match val {
                Value::Int(i) => Ok(Value::Int(i)),
                Value::Real(r) => Ok(Value::Int(r as i64)),
                Value::Str(ref s) => {
                    if let Ok(i) = s.trim().parse::<i64>() {
                        Ok(Value::Int(i))
                    } else if let Ok(f) = s.trim().parse::<f64>() {
                        Ok(Value::Int(f as i64))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                _ => Ok(Value::Nil),
            }
        } else {
            Ok(Value::Nil)
        }
    });

    // (ocl-get-real key alist)
    env.def_subr("OCL-GET-REAL", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (key alist)",
                actual: args.len(),
            });
        }
        let key_str = extract_str(&args[0]).unwrap_or_else(|| args[0].to_string());
        if let Some(val) = lookup_alist(&key_str, &args[1]) {
            match val {
                Value::Real(r) => Ok(Value::Real(r)),
                Value::Int(i) => Ok(Value::Real(i as f64)),
                Value::Str(ref s) => {
                    if let Ok(f) = s.trim().parse::<f64>() {
                        Ok(Value::Real(f))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                _ => Ok(Value::Nil),
            }
        } else {
            Ok(Value::Nil)
        }
    });

    // (ocl-get-str key alist)
    env.def_subr("OCL-GET-STR", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (key alist)",
                actual: args.len(),
            });
        }
        let key_str = extract_str(&args[0]).unwrap_or_else(|| args[0].to_string());
        if let Some(val) = lookup_alist(&key_str, &args[1]) {
            match val {
                Value::Str(s) => Ok(Value::Str(s)),
                Value::Sym(s) => Ok(Value::Str(s.as_str().to_string())),
                Value::Int(i) => Ok(Value::Str(i.to_string())),
                Value::Real(r) => Ok(Value::Str(format!("{:.4}", r))),
                Value::Nil => Ok(Value::Str(String::new())),
                other => Ok(Value::Str(other.to_string())),
            }
        } else {
            Ok(Value::Str(String::new()))
        }
    });

    // (ocl-get-bool key alist)
    env.def_subr("OCL-GET-BOOL", |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (key alist)",
                actual: args.len(),
            });
        }
        let key_str = extract_str(&args[0]).unwrap_or_else(|| args[0].to_string());
        if let Some(val) = lookup_alist(&key_str, &args[1]) {
            match val {
                Value::Nil => Ok(Value::Nil),
                Value::T => Ok(Value::T),
                Value::Int(i) => Ok(if i != 0 { Value::T } else { Value::Nil }),
                Value::Str(ref s) => {
                    if s == "0"
                        || s.eq_ignore_ascii_case("false")
                        || s.eq_ignore_ascii_case("nil")
                        || s.is_empty()
                    {
                        Ok(Value::Nil)
                    } else {
                        Ok(Value::T)
                    }
                }
                _ => Ok(Value::T),
            }
        } else {
            Ok(Value::Nil)
        }
    });

    // ── 6. Dockable Side Panels: (ocl-panel-*) ─────────────────────────────────

    // (ocl-panel-register id title [options] ctrl1 ctrl2 ...)
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-REGISTER", move |args, _| {
        if args.len() < 2 {
            return Err(LispError::WrongArgCount {
                expected: "at least 2 (id title [options] ctrl1 ...)",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;
        let title = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol title", actual: args[1].to_string() })?;

        let mut def = PanelDef::new(id, title);
        let mut idx = 2;

        if idx < args.len() && matches!(args[idx], Value::Cons(_)) {
            // Check if options alist rather than control
            let has_type = lookup_alist("type", &args[idx]).is_some();
            if !has_type {
                if let Some(w) = lookup_alist("width", &args[idx]).as_ref().and_then(extract_num) {
                    def.width = Some(w as f32);
                }
                if let Some(cmd) = lookup_alist("command", &args[idx]).as_ref().and_then(extract_str) {
                    def.command = Some(cmd);
                }
                idx += 1;
            }
        }

        for ctrl_val in &args[idx..] {
            def.controls.push(lisp_to_control(ctrl_val)?);
        }

        let json_str = serde_json::to_string(&def)
            .map_err(|e| LispError::Eval(format!("Failed to serialize panel: {e}")))?;

        let mut br = b.lock().unwrap();
        let ok = br.register_panel(&json_str).map_err(LispError::Cad)?;
        Ok(if ok { Value::T } else { Value::Nil })
    });

    // (ocl-panel-update id alist)
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-UPDATE", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (id alist)",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;

        let pairs = args[1].to_vec().ok_or_else(|| {
            LispError::TypeError { expected: "association list", actual: args[1].to_string() }
        })?;

        let mut map: HashMap<String, DialogValue> = HashMap::new();
        for item in pairs {
            if let Value::Cons(p_rc) = item {
                let p = p_rc.borrow();
                if let Some(k) = extract_str(&p.car) {
                    map.insert(k, lisp_to_dialog_value(&p.cdr));
                }
            }
        }

        let json_str = serde_json::to_string(&map)
            .map_err(|e| LispError::Eval(format!("Failed to serialize panel values: {e}")))?;

        let mut br = b.lock().unwrap();
        let ok = br.update_panel(&id, &json_str).map_err(LispError::Cad)?;
        Ok(if ok { Value::T } else { Value::Nil })
    });

    // (ocl-panel-toggle id [show])
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-TOGGLE", move |args, _| {
        if args.is_empty() || args.len() > 2 {
            return Err(LispError::WrongArgCount {
                expected: "1 or 2 (id [show])",
                actual: args.len(),
            });
        }
        let id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol id", actual: args[0].to_string() })?;

        let show = if args.len() >= 2 {
            Some(args[1].is_truthy())
        } else {
            None
        };

        let mut br = b.lock().unwrap();
        let ok = br.toggle_panel(&id, show).map_err(LispError::Cad)?;
        Ok(if ok { Value::T } else { Value::Nil })
    });

    // (ocl-panel-get panel-id control-id)
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-GET", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (panel-id control-id)",
                actual: args.len(),
            });
        }
        let panel_id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol panel-id", actual: args[0].to_string() })?;
        let control_id = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol control-id", actual: args[1].to_string() })?;

        let br = b.lock().unwrap();
        if let Some(val_json) = br.get_panel_value(&panel_id, &control_id) {
            if let Ok(dv) = serde_json::from_str::<DialogValue>(&val_json) {
                return Ok(dialog_value_to_lisp(dv));
            }
        }
        Ok(Value::Nil)
    });

    // (ocl-panel-get-bool panel-id control-id)
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-GET-BOOL", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (panel-id control-id)",
                actual: args.len(),
            });
        }
        let panel_id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol panel-id", actual: args[0].to_string() })?;
        let control_id = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol control-id", actual: args[1].to_string() })?;

        let br = b.lock().unwrap();
        if let Some(val_json) = br.get_panel_value(&panel_id, &control_id) {
            if let Ok(dv) = serde_json::from_str::<DialogValue>(&val_json) {
                return match dv {
                    DialogValue::Bool(b) => Ok(if b { Value::T } else { Value::Nil }),
                    _ => Ok(Value::Nil),
                };
            }
        }
        Ok(Value::Nil)
    });

    // (ocl-panel-get-str panel-id control-id)
    let b = bridge.clone();
    env.def_subr("OCL-PANEL-GET-STR", move |args, _| {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (panel-id control-id)",
                actual: args.len(),
            });
        }
        let panel_id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol panel-id", actual: args[0].to_string() })?;
        let control_id = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol control-id", actual: args[1].to_string() })?;

        let br = b.lock().unwrap();
        if let Some(val_json) = br.get_panel_value(&panel_id, &control_id) {
            if let Ok(dv) = serde_json::from_str::<DialogValue>(&val_json) {
                return match dv {
                    DialogValue::Text(s) => Ok(Value::Str(s)),
                    DialogValue::Choice { text, .. } => Ok(Value::Str(text)),
                    DialogValue::FilePath(s) => Ok(Value::Str(s)),
                    _ => Ok(Value::Str(String::new())),
                };
            }
        }
        Ok(Value::Str(String::new()))
    });

    // (ocl-panel-get-real panel-id control-id) / (ocl-panel-get-num panel-id control-id)
    let b = bridge.clone();
    let num_fn = move |args: &[Value], _env: &mut _| -> Result<Value, LispError> {
        if args.len() != 2 {
            return Err(LispError::WrongArgCount {
                expected: "2 (panel-id control-id)",
                actual: args.len(),
            });
        }
        let panel_id = extract_str(&args[0])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol panel-id", actual: args[0].to_string() })?;
        let control_id = extract_str(&args[1])
            .ok_or_else(|| LispError::TypeError { expected: "string or symbol control-id", actual: args[1].to_string() })?;

        let br = b.lock().unwrap();
        if let Some(val_json) = br.get_panel_value(&panel_id, &control_id) {
            if let Ok(dv) = serde_json::from_str::<DialogValue>(&val_json) {
                return match dv {
                    DialogValue::Number(n) => Ok(Value::Real(n)),
                    DialogValue::Text(s) => s.trim().parse::<f64>().map(Value::Real).or(Ok(Value::Real(0.0))),
                    _ => Ok(Value::Real(0.0)),
                };
            }
        }
        Ok(Value::Real(0.0))
    };
    env.def_subr("OCL-PANEL-GET-REAL", num_fn.clone());
    env.def_subr("OCL-PANEL-GET-NUM", num_fn);
}
