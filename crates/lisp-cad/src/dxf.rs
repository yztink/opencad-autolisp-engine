use lisp_core::{LispError, Value};

use crate::bridge::DxfValue;

/// Converts internal DXF records into an AutoLISP DXF association list:
/// `((0 . "LINE") (8 . "0") (10 0.0 0.0 0.0) ...)`
pub fn dxf_to_lisp(records: &[(i16, DxfValue)]) -> Value {
    let mut pairs = Vec::with_capacity(records.len());

    for (code, val) in records {
        let code_val = Value::Int(*code as i64);
        match val {
            DxfValue::Point(pt) => {
                // Points in AutoLISP DXF are lists: (10 x y z)
                let item = Value::from_slice(&[
                    code_val,
                    Value::Real(pt[0]),
                    Value::Real(pt[1]),
                    Value::Real(pt[2]),
                ]);
                pairs.push(item);
            }
            DxfValue::Str(s) => {
                pairs.push(Value::cons(code_val, Value::Str(s.clone())));
            }
            DxfValue::Int(n) => {
                pairs.push(Value::cons(code_val, Value::Int(*n)));
            }
            DxfValue::Real(r) => {
                pairs.push(Value::cons(code_val, Value::Real(*r)));
            }
            DxfValue::Handle(h) => {
                pairs.push(Value::cons(code_val, Value::Str(format!("{:X}", h))));
            }
            DxfValue::XData(apps) => {
                let mut app_entries = Vec::with_capacity(apps.len() + 1);
                app_entries.push(code_val);
                for (app_name, records) in apps {
                    let mut app_items = Vec::with_capacity(records.len() + 1);
                    app_items.push(Value::Str(app_name.clone()));
                    for (sub_code, sub_val) in records {
                        let sub_code_val = Value::Int(*sub_code as i64);
                        match sub_val {
                            DxfValue::Point(pt) => {
                                app_items.push(Value::from_slice(&[
                                    sub_code_val,
                                    Value::Real(pt[0]),
                                    Value::Real(pt[1]),
                                    Value::Real(pt[2]),
                                ]));
                            }
                            DxfValue::Str(s) => {
                                app_items.push(Value::cons(sub_code_val, Value::Str(s.clone())));
                            }
                            DxfValue::Int(n) => {
                                app_items.push(Value::cons(sub_code_val, Value::Int(*n)));
                            }
                            DxfValue::Real(r) => {
                                app_items.push(Value::cons(sub_code_val, Value::Real(*r)));
                            }
                            DxfValue::Handle(h) => {
                                app_items.push(Value::cons(sub_code_val, Value::Str(format!("{:X}", h))));
                            }
                            _ => {}
                        }
                    }
                    app_entries.push(Value::from_slice(&app_items));
                }
                pairs.push(Value::from_slice(&app_entries));
            }
        }
    }

    Value::from_slice(&pairs)
}

/// Parses an AutoLISP DXF association list back into internal DXF records.
pub fn lisp_to_dxf(val: &Value) -> Result<Vec<(i16, DxfValue)>, LispError> {
    let items = val.to_vec().ok_or_else(|| {
        LispError::TypeError {
            expected: "DXF association list",
            actual: val.to_string(),
        }
    })?;

    let mut records = Vec::with_capacity(items.len());

    for item in items {
        match item {
            // Point list format: (10 x y [z])
            Value::Cons(c_rc) => {
                let car = c_rc.borrow().car.clone();
                let cdr = c_rc.borrow().cdr.clone();

                let code = match car {
                    Value::Int(n) => n as i16,
                    other => {
                        return Err(LispError::TypeError {
                            expected: "integer DXF group code",
                            actual: other.to_string(),
                        })
                    }
                };

                if code == -3 {
                    let mut apps = Vec::new();
                    if let Some(app_nodes) = cdr.to_vec() {
                        for app_node in app_nodes {
                            if let Some(app_elements) = app_node.to_vec() {
                                if app_elements.is_empty() {
                                    continue;
                                }
                                let app_name = match &app_elements[0] {
                                    Value::Str(s) => s.clone(),
                                    other => other.to_string(),
                                };
                                let mut app_records = Vec::new();
                                for elem in &app_elements[1..] {
                                    if let Value::Cons(elem_rc) = elem {
                                        let elem_car = elem_rc.borrow().car.clone();
                                        let elem_cdr = elem_rc.borrow().cdr.clone();
                                        let elem_code = match elem_car {
                                            Value::Int(n) => n as i16,
                                            _ => continue,
                                        };
                                        if let Some(pt_coords) = elem_cdr.to_vec() {
                                            if pt_coords.len() >= 2 {
                                                let x = match pt_coords.first() {
                                                    Some(Value::Real(r)) => *r,
                                                    Some(Value::Int(n)) => *n as f64,
                                                    _ => 0.0,
                                                };
                                                let y = match pt_coords.get(1) {
                                                    Some(Value::Real(r)) => *r,
                                                    Some(Value::Int(n)) => *n as f64,
                                                    _ => 0.0,
                                                };
                                                let z = match pt_coords.get(2) {
                                                    Some(Value::Real(r)) => *r,
                                                    Some(Value::Int(n)) => *n as f64,
                                                    _ => 0.0,
                                                };
                                                app_records.push((elem_code, DxfValue::Point([x, y, z])));
                                                continue;
                                            }
                                        }
                                        let sub_val = match elem_cdr {
                                            Value::Str(s) => DxfValue::Str(s),
                                            Value::Int(n) => DxfValue::Int(n),
                                            Value::Real(r) => DxfValue::Real(r),
                                            other => DxfValue::Str(other.to_string()),
                                        };
                                        app_records.push((elem_code, sub_val));
                                    }
                                }
                                apps.push((app_name, app_records));
                            }
                        }
                    }
                    records.push((-3, DxfValue::XData(apps)));
                    continue;
                }

                // Check if cdr is a proper list (point) or a scalar (dotted pair)
                if let Some(coords) = cdr.to_vec() {
                    if coords.len() >= 2 {
                        let x = match coords.first() {
                            Some(Value::Real(r)) => *r,
                            Some(Value::Int(n)) => *n as f64,
                            _ => 0.0,
                        };
                        let y = match coords.get(1) {
                            Some(Value::Real(r)) => *r,
                            Some(Value::Int(n)) => *n as f64,
                            _ => 0.0,
                        };
                        let z = match coords.get(2) {
                            Some(Value::Real(r)) => *r,
                            Some(Value::Int(n)) => *n as f64,
                            _ => 0.0,
                        };
                        records.push((code, DxfValue::Point([x, y, z])));
                        continue;
                    }
                }

                // Scalar dotted pair: (code . value)
                let dxf_val = match cdr {
                    Value::Str(s) => DxfValue::Str(s),
                    Value::Int(n) => DxfValue::Int(n),
                    Value::Real(r) => DxfValue::Real(r),
                    other => DxfValue::Str(other.to_string()),
                };

                records.push((code, dxf_val));
            }
            other => {
                return Err(LispError::TypeError {
                    expected: "DXF group pair or list",
                    actual: other.to_string(),
                })
            }
        }
    }

    Ok(records)
}

pub fn dxf_wildcard_match(actual: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "*" {
        return true;
    }
    for sub_pat in pattern.split(',') {
        let sub = sub_pat.trim();
        if sub.is_empty() {
            continue;
        }
        let (negated, pat_body) = if sub.starts_with('~') {
            (true, &sub[1..])
        } else {
            (false, sub)
        };
        let matched = glob_match(actual, pat_body);
        if negated {
            if matched {
                return false;
            }
        } else if matched {
            return true;
        }
    }
    false
}

fn glob_match(actual: &str, pattern: &str) -> bool {
    let a_chars: Vec<char> = actual.to_uppercase().chars().collect();
    let p_chars: Vec<char> = pattern.to_uppercase().chars().collect();
    glob_match_slices(&a_chars, &p_chars)
}

fn glob_match_slices(a: &[char], p: &[char]) -> bool {
    let mut i = 0;
    let mut j = 0;
    let mut star_j = None;
    let mut match_i = 0;
    while i < a.len() {
        if j < p.len() && (p[j] == '?' || p[j] == a[i]) {
            i += 1;
            j += 1;
        } else if j < p.len() && p[j] == '*' {
            star_j = Some(j);
            j += 1;
            match_i = i;
        } else if let Some(sj) = star_j {
            j = sj + 1;
            match_i += 1;
            i = match_i;
        } else {
            return false;
        }
    }
    while j < p.len() && p[j] == '*' {
        j += 1;
    }
    j == p.len()
}

fn compare_point_coords(a: [f64; 3], op: &str, t: [f64; 3]) -> bool {
    if op.contains(',') {
        let parts: Vec<&str> = op.split(',').map(|s| s.trim()).collect();
        let ops = [
            parts.first().copied().unwrap_or("*"),
            parts.get(1).copied().unwrap_or("*"),
            parts.get(2).copied().unwrap_or("*"),
        ];
        for i in 0..3 {
            if ops[i] != "*" && !compare_nums(a[i], ops[i], t[i]) {
                return false;
            }
        }
        true
    } else {
        match op {
            "=" => (a[0] - t[0]).abs() < 1e-6 && (a[1] - t[1]).abs() < 1e-6 && (a[2] - t[2]).abs() < 1e-6,
            "!=" | "/=" | "<>" => (a[0] - t[0]).abs() >= 1e-6 || (a[1] - t[1]).abs() >= 1e-6 || (a[2] - t[2]).abs() >= 1e-6,
            "<" => a[0] < t[0] && a[1] < t[1] && a[2] < t[2],
            "<=" => a[0] <= t[0] + 1e-6 && a[1] <= t[1] + 1e-6 && a[2] <= t[2] + 1e-6,
            ">" => a[0] > t[0] && a[1] > t[1] && a[2] > t[2],
            ">=" => a[0] >= t[0] - 1e-6 && a[1] >= t[1] - 1e-6 && a[2] >= t[2] - 1e-6,
            _ => (a[0] - t[0]).abs() < 1e-6 && (a[1] - t[1]).abs() < 1e-6,
        }
    }
}

fn compare_dxf_values(actual: &DxfValue, op: &str, target: &DxfValue) -> bool {
    match (actual, target) {
        (DxfValue::Str(a), DxfValue::Str(t)) => match op {
            "=" => dxf_wildcard_match(a, t),
            "!=" | "/=" | "<>" => !dxf_wildcard_match(a, t),
            "<" => a < t,
            "<=" => a <= t,
            ">" => a > t,
            ">=" => a >= t,
            _ => dxf_wildcard_match(a, t),
        },
        (DxfValue::Int(a), DxfValue::Int(t)) => compare_nums(*a as f64, op, *t as f64),
        (DxfValue::Real(a), DxfValue::Real(t)) => compare_nums(*a, op, *t),
        (DxfValue::Int(a), DxfValue::Real(t)) => compare_nums(*a as f64, op, *t),
        (DxfValue::Real(a), DxfValue::Int(t)) => compare_nums(*a, op, *t as f64),
        (DxfValue::Point(a), DxfValue::Point(t)) => compare_point_coords(*a, op, *t),
        _ => actual == target,
    }
}

fn compare_nums(a: f64, op: &str, t: f64) -> bool {
    match op {
        "=" => (a - t).abs() < 1e-6,
        "!=" | "/=" | "<>" => (a - t).abs() >= 1e-6,
        "<" => a < t,
        "<=" => a <= t + 1e-6,
        ">" => a > t,
        ">=" => a >= t - 1e-6,
        "&" => ((a as i64) & (t as i64)) != 0,
        "&=" => ((a as i64) & (t as i64)) == (t as i64),
        "*" => true,
        _ => (a - t).abs() < 1e-6,
    }
}

pub fn eval_dxf_filter(elist: &[(i16, DxfValue)], filter: &[(i16, DxfValue)]) -> bool {
    if filter.is_empty() {
        return true;
    }
    eval_filter_slice(elist, filter, 0).0
}

fn eval_filter_slice(
    elist: &[(i16, DxfValue)],
    filter: &[(i16, DxfValue)],
    mut idx: usize,
) -> (bool, usize) {
    let mut results: Vec<bool> = Vec::new();
    let mut current_op = "=".to_string();

    while idx < filter.len() {
        let (code, val) = &filter[idx];
        if *code == -4 {
            let s = match val {
                DxfValue::Str(s) => s.trim().to_uppercase(),
                _ => String::new(),
            };
            let is_logical_open = matches!(s.as_str(), "<AND" | "<OR" | "<NOT" | "<XOR");
            let is_logical_close = matches!(s.as_str(), "AND>" | "OR>" | "NOT>" | "XOR>");
            if is_logical_open {
                let block_type = s.trim_start_matches('<').to_string();
                let (sub_res, next_idx) = eval_filter_block(elist, filter, idx + 1, &block_type);
                results.push(sub_res);
                idx = next_idx;
                continue;
            } else if is_logical_close {
                idx += 1;
                break;
            } else {
                current_op = s;
                idx += 1;
                continue;
            }
        }

        let op = current_op.clone();
        current_op = "=".to_string();

        let matched = elist
            .iter()
            .any(|(c, v)| *c == *code && compare_dxf_values(v, &op, val));
        results.push(matched);
        idx += 1;
    }

    let combined = combine_results(&results, "AND");
    (combined, idx)
}

fn eval_filter_block(
    elist: &[(i16, DxfValue)],
    filter: &[(i16, DxfValue)],
    mut idx: usize,
    block_type: &str,
) -> (bool, usize) {
    let mut results: Vec<bool> = Vec::new();
    let mut current_op = "=".to_string();

    while idx < filter.len() {
        let (code, val) = &filter[idx];
        if *code == -4 {
            let s = match val {
                DxfValue::Str(s) => s.trim().to_uppercase(),
                _ => String::new(),
            };
            let is_logical_open = matches!(s.as_str(), "<AND" | "<OR" | "<NOT" | "<XOR");
            let is_logical_close = matches!(s.as_str(), "AND>" | "OR>" | "NOT>" | "XOR>");
            if is_logical_open {
                let inner_block = s.trim_start_matches('<').to_string();
                let (sub_res, next_idx) = eval_filter_block(elist, filter, idx + 1, &inner_block);
                results.push(sub_res);
                idx = next_idx;
                continue;
            } else if is_logical_close {
                idx += 1;
                break;
            } else {
                current_op = s;
                idx += 1;
                continue;
            }
        }

        let op = current_op.clone();
        current_op = "=".to_string();

        let matched = elist
            .iter()
            .any(|(c, v)| *c == *code && compare_dxf_values(v, &op, val));
        results.push(matched);
        idx += 1;
    }

    let combined = combine_results(&results, block_type);
    (combined, idx)
}

fn combine_results(results: &[bool], mode: &str) -> bool {
    if results.is_empty() {
        return true;
    }
    match mode {
        "OR" => results.iter().any(|&r| r),
        "NOT" => !results.iter().all(|&r| r),
        "XOR" => results.iter().filter(|&&r| r).count() == 1,
        _ => results.iter().all(|&r| r),
    }
}

pub fn get_entity_bbox(elist: &[(i16, DxfValue)]) -> Option<[f64; 4]> {
    let mut p10 = None;
    let mut p11 = None;
    let mut rad = None;

    for (code, val) in elist {
        match (*code, val) {
            (10, DxfValue::Point(p)) => p10 = Some(*p),
            (11, DxfValue::Point(p)) => p11 = Some(*p),
            (40, DxfValue::Real(r)) => rad = Some(*r),
            (40, DxfValue::Int(n)) => rad = Some(*n as f64),
            _ => {}
        }
    }

    if let (Some(p1), Some(p2)) = (p10, p11) {
        Some([
            p1[0].min(p2[0]),
            p1[1].min(p2[1]),
            p1[0].max(p2[0]),
            p1[1].max(p2[1]),
        ])
    } else if let (Some(c), Some(r)) = (p10, rad) {
        Some([c[0] - r, c[1] - r, c[0] + r, c[1] + r])
    } else if let Some(p) = p10 {
        Some([p[0], p[1], p[0], p[1]])
    } else {
        None
    }
}

pub fn check_spatial(
    mode_upper: &str,
    points: Option<&[[f64; 3]]>,
    bbox: Option<[f64; 4]>,
) -> bool {
    let pts = match points {
        Some(p) if !p.is_empty() => p,
        _ => return true,
    };

    let b = match bbox {
        Some(b) => b,
        None => return true,
    };

    if mode_upper.contains('W') && !mode_upper.contains("WP") {
        if pts.len() >= 2 {
            let (p1, p2) = (pts[0], pts[1]);
            let min_x = p1[0].min(p2[0]);
            let max_x = p1[0].max(p2[0]);
            let min_y = p1[1].min(p2[1]);
            let max_y = p1[1].max(p2[1]);
            return b[0] >= min_x - 1e-4 && b[2] <= max_x + 1e-4 && b[1] >= min_y - 1e-4 && b[3] <= max_y + 1e-4;
        }
    } else if mode_upper.contains('C') && !mode_upper.contains("CP") {
        if pts.len() >= 2 {
            let (p1, p2) = (pts[0], pts[1]);
            let min_x = p1[0].min(p2[0]);
            let max_x = p1[0].max(p2[0]);
            let min_y = p1[1].min(p2[1]);
            let max_y = p1[1].max(p2[1]);
            return b[2] >= min_x && b[0] <= max_x && b[3] >= min_y && b[1] <= max_y;
        }
    } else if mode_upper.contains("WP") {
        let min_x = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let max_y = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        return b[0] >= min_x - 1e-4 && b[2] <= max_x + 1e-4 && b[1] >= min_y - 1e-4 && b[3] <= max_y + 1e-4;
    } else if mode_upper.contains("CP") {
        let min_x = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let max_y = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        return b[2] >= min_x && b[0] <= max_x && b[3] >= min_y && b[1] <= max_y;
    } else if mode_upper.contains('F') {
        let min_x = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let max_y = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        return b[2] >= min_x && b[0] <= max_x && b[3] >= min_y && b[1] <= max_y;
    } else if pts.len() >= 2 {
        let (p1, p2) = (pts[0], pts[1]);
        let min_x = p1[0].min(p2[0]);
        let max_x = p1[0].max(p2[0]);
        let min_y = p1[1].min(p2[1]);
        let max_y = p1[1].max(p2[1]);
        if p1[0] <= p2[0] {
            // Window selection (left-to-right box): entity entirely inside box
            return b[0] >= min_x - 1e-4 && b[2] <= max_x + 1e-4 && b[1] >= min_y - 1e-4 && b[3] <= max_y + 1e-4;
        } else {
            // Crossing selection (right-to-left box): entity inside or intersecting box
            return b[2] >= min_x && b[0] <= max_x && b[3] >= min_y && b[1] <= max_y;
        }
    } else if pts.len() == 1 {
        let p = pts[0];
        let tol = 2.0;
        return p[0] >= b[0] - tol && p[0] <= b[2] + tol && p[1] >= b[1] - tol && p[1] <= b[3] + tol;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxf_roundtrip() {
        let records = vec![
            (0, DxfValue::Str("LINE".into())),
            (8, DxfValue::Str("WALLS".into())),
            (10, DxfValue::Point([0.0, 0.0, 0.0])),
            (11, DxfValue::Point([100.0, 200.0, 0.0])),
            (40, DxfValue::Real(2.5)),
            (
                -3,
                DxfValue::XData(vec![(
                    "MYAPP".into(),
                    vec![
                        (1000, DxfValue::Str("DesignData".into())),
                        (1040, DxfValue::Real(12.5)),
                        (1070, DxfValue::Int(42)),
                        (1010, DxfValue::Point([1.0, 2.0, 3.0])),
                    ],
                )]),
            ),
        ];

        let lisp_val = dxf_to_lisp(&records);
        let parsed = lisp_to_dxf(&lisp_val).unwrap();

        assert_eq!(records, parsed);
    }

    #[test]
    fn test_two_point_box_spatial_check() {
        let line_bbox = Some([10.0, 10.0, 20.0, 20.0]);

        // Left-to-right window: (0,0) to (30,30) contains the line
        assert!(check_spatial("", Some(&[[0.0, 0.0, 0.0], [30.0, 30.0, 0.0]]), line_bbox));
        // Left-to-right window: (0,0) to (15,15) only partially covers it -> should not match window
        assert!(!check_spatial("", Some(&[[0.0, 0.0, 0.0], [15.0, 15.0, 0.0]]), line_bbox));

        // Right-to-left crossing: (15,15) to (0,0) intersects the line -> should match crossing
        assert!(check_spatial("", Some(&[[15.0, 15.0, 0.0], [0.0, 0.0, 0.0]]), line_bbox));
        // Right-to-left crossing: (100,100) to (50,50) does not touch the line
        assert!(!check_spatial("", Some(&[[100.0, 100.0, 0.0], [50.0, 50.0, 0.0]]), line_bbox));
    }

    #[test]
    fn test_dxf_filter_relational_and_bitwise() {
        let elist = vec![
            (0, DxfValue::Str("LINE".into())),
            (8, DxfValue::Str("WALLS".into())),
            (70, DxfValue::Int(5)), // binary 101: bit 1 and bit 4 set
            (10, DxfValue::Point([10.0, 20.0, 0.0])),
            (40, DxfValue::Real(12.5)),
        ];

        // 1. Inequality <>
        let filt_neq = vec![(-4, DxfValue::Str("<>".into())), (0, DxfValue::Str("CIRCLE".into()))];
        assert!(eval_dxf_filter(&elist, &filt_neq));

        let filt_neq_fail = vec![(-4, DxfValue::Str("<>".into())), (0, DxfValue::Str("LINE".into()))];
        assert!(!eval_dxf_filter(&elist, &filt_neq_fail));

        // 2. Bitwise & (mask test: 5 & 1 != 0, 5 & 2 == 0)
        let filt_bit1 = vec![(-4, DxfValue::Str("&".into())), (70, DxfValue::Int(1))];
        assert!(eval_dxf_filter(&elist, &filt_bit1));

        let filt_bit2 = vec![(-4, DxfValue::Str("&".into())), (70, DxfValue::Int(2))];
        assert!(!eval_dxf_filter(&elist, &filt_bit2));

        // 3. Bitwise &= (exact mask: 5 & 5 == 5, 5 & 7 != 7)
        let filt_mask_exact = vec![(-4, DxfValue::Str("&=".into())), (70, DxfValue::Int(5))];
        assert!(eval_dxf_filter(&elist, &filt_mask_exact));

        let filt_mask_over = vec![(-4, DxfValue::Str("&=".into())), (70, DxfValue::Int(7))];
        assert!(!eval_dxf_filter(&elist, &filt_mask_over));

        // 4. Multi-component 3D coordinate operator ">,>,*"
        let filt_pt_gt = vec![(-4, DxfValue::Str(">,>,*".into())), (10, DxfValue::Point([5.0, 15.0, 100.0]))];
        assert!(eval_dxf_filter(&elist, &filt_pt_gt));

        let filt_pt_fail = vec![(-4, DxfValue::Str(">,>,*".into())), (10, DxfValue::Point([15.0, 15.0, 100.0]))];
        assert!(!eval_dxf_filter(&elist, &filt_pt_fail));
    }
}
