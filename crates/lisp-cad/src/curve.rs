use crate::bridge::DxfValue;

#[derive(Debug, Clone)]
pub struct CurveInfo {
    pub points: Vec<[f64; 3]>,
    pub is_closed: bool,
    pub entity_type: String,
}

pub fn extract_curve_info(records: &[(i16, DxfValue)]) -> CurveInfo {
    let mut entity_type = String::new();
    let mut points = Vec::new();
    let mut is_closed = false;
    let mut group_10_pt = None;
    let mut group_11_pt = None;

    for (code, val) in records {
        match *code {
            0 => {
                if let DxfValue::Str(s) = val {
                    entity_type = s.to_ascii_uppercase();
                }
            }
            70 => {
                if let DxfValue::Int(flags) = val {
                    // For LWPOLYLINE / POLYLINE: bit 1 (mask 1) is closed
                    if (flags & 1) != 0 {
                        is_closed = true;
                    }
                }
            }
            10 => {
                if let DxfValue::Point(pt) = val {
                    points.push(*pt);
                    if group_10_pt.is_none() {
                        group_10_pt = Some(*pt);
                    }
                }
            }
            11 => {
                if let DxfValue::Point(pt) = val {
                    group_11_pt = Some(*pt);
                }
            }
            _ => {}
        }
    }

    if entity_type == "LINE" {
        points.clear();
        if let Some(p10) = group_10_pt {
            points.push(p10);
        }
        if let Some(p11) = group_11_pt {
            points.push(p11);
        }
    }

    // If points are closed by start == end (and length > 2), treat as closed
    if points.len() > 2 {
        let first = points[0];
        let last = points[points.len() - 1];
        let dist = ((first[0] - last[0]).powi(2) + (first[1] - last[1]).powi(2) + (first[2] - last[2]).powi(2)).sqrt();
        if dist < 1e-6 {
            is_closed = true;
        }
    }

    CurveInfo {
        points,
        is_closed,
        entity_type,
    }
}

pub fn curve_end_param(info: &CurveInfo) -> f64 {
    if info.points.is_empty() {
        return 0.0;
    }
    let n = info.points.len();
    if info.entity_type == "LINE" {
        return 1.0;
    }
    if info.is_closed {
        n as f64
    } else {
        (n.saturating_sub(1)) as f64
    }
}

pub fn curve_point_at_param(info: &CurveInfo, param: f64) -> [f64; 3] {
    if info.points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = info.points.len();
    if n == 1 {
        return info.points[0];
    }
    let max_p = curve_end_param(info);
    if max_p <= 0.0 {
        return info.points[0];
    }

    let p = if info.is_closed {
        let rem = param % max_p;
        if rem < 0.0 { rem + max_p } else { rem }
    } else {
        param.clamp(0.0, max_p)
    };

    let idx = p.floor() as usize;
    let frac = p - (idx as f64);

    let p1 = info.points[idx % n];
    let p2 = info.points[(idx + 1) % n];

    [
        p1[0] + frac * (p2[0] - p1[0]),
        p1[1] + frac * (p2[1] - p1[1]),
        p1[2] + frac * (p2[2] - p1[2]),
    ]
}

pub fn curve_first_deriv(info: &CurveInfo, param: f64) -> [f64; 3] {
    if info.points.len() < 2 {
        return [1.0, 0.0, 0.0];
    }
    let n = info.points.len();
    let max_p = curve_end_param(info);
    let p = if info.is_closed {
        let rem = param % max_p;
        if rem < 0.0 { rem + max_p } else { rem }
    } else {
        param.clamp(0.0, max_p)
    };

    let idx = (p.floor() as usize).min(if info.is_closed { n - 1 } else { n.saturating_sub(2) });
    let p1 = info.points[idx % n];
    let p2 = info.points[(idx + 1) % n];

    let dx = p2[0] - p1[0];
    let dy = p2[1] - p1[1];
    let dz = p2[2] - p1[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-9 {
        [1.0, 0.0, 0.0]
    } else {
        [dx / len, dy / len, dz / len]
    }
}

pub fn curve_total_length(info: &CurveInfo) -> f64 {
    if info.points.len() < 2 {
        return 0.0;
    }
    let n = info.points.len();
    let mut total = 0.0;
    let num_segs = if info.is_closed { n } else { n - 1 };
    for i in 0..num_segs {
        let p1 = info.points[i % n];
        let p2 = info.points[(i + 1) % n];
        total += ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2) + (p2[2] - p1[2]).powi(2)).sqrt();
    }
    total
}

pub fn curve_point_at_dist(info: &CurveInfo, dist: f64) -> [f64; 3] {
    if info.points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = info.points.len();
    if n == 1 || dist <= 0.0 {
        return info.points[0];
    }
    let num_segs = if info.is_closed { n } else { n - 1 };
    let mut accumulated = 0.0;

    for i in 0..num_segs {
        let p1 = info.points[i % n];
        let p2 = info.points[(i + 1) % n];
        let seg_len = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2) + (p2[2] - p1[2]).powi(2)).sqrt();
        if accumulated + seg_len >= dist || i == num_segs - 1 {
            let remain = dist - accumulated;
            let frac = if seg_len > 1e-9 { (remain / seg_len).clamp(0.0, 1.0) } else { 0.0 };
            return [
                p1[0] + frac * (p2[0] - p1[0]),
                p1[1] + frac * (p2[1] - p1[1]),
                p1[2] + frac * (p2[2] - p1[2]),
            ];
        }
        accumulated += seg_len;
    }

    info.points[info.points.len() - 1]
}

pub fn curve_dist_at_param(info: &CurveInfo, param: f64) -> f64 {
    if info.points.len() < 2 || param <= 0.0 {
        return 0.0;
    }
    let n = info.points.len();
    let max_p = curve_end_param(info);
    let p = param.clamp(0.0, max_p);
    let full_segs = p.floor() as usize;
    let frac = p - (full_segs as f64);

    let mut accumulated = 0.0;
    for i in 0..full_segs {
        let p1 = info.points[i % n];
        let p2 = info.points[(i + 1) % n];
        accumulated += ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2) + (p2[2] - p1[2]).powi(2)).sqrt();
    }
    if frac > 0.0 && full_segs < n {
        let p1 = info.points[full_segs % n];
        let p2 = info.points[(full_segs + 1) % n];
        let seg_len = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2) + (p2[2] - p1[2]).powi(2)).sqrt();
        accumulated += frac * seg_len;
    }
    accumulated
}

pub fn curve_param_at_dist(info: &CurveInfo, dist: f64) -> f64 {
    if info.points.len() < 2 || dist <= 0.0 {
        return 0.0;
    }
    let n = info.points.len();
    let num_segs = if info.is_closed { n } else { n - 1 };
    let mut accumulated = 0.0;

    for i in 0..num_segs {
        let p1 = info.points[i % n];
        let p2 = info.points[(i + 1) % n];
        let seg_len = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2) + (p2[2] - p1[2]).powi(2)).sqrt();
        if accumulated + seg_len >= dist {
            let remain = dist - accumulated;
            let frac = if seg_len > 1e-9 { (remain / seg_len).clamp(0.0, 1.0) } else { 0.0 };
            return (i as f64) + frac;
        }
        accumulated += seg_len;
    }

    curve_end_param(info)
}

pub fn curve_closest_point_to(info: &CurveInfo, pt: [f64; 3]) -> [f64; 3] {
    if info.points.is_empty() {
        return pt;
    }
    let n = info.points.len();
    if n == 1 {
        return info.points[0];
    }
    let num_segs = if info.is_closed { n } else { n - 1 };
    let mut best_pt = info.points[0];
    let mut best_dist_sq = f64::MAX;

    for i in 0..num_segs {
        let p1 = info.points[i % n];
        let p2 = info.points[(i + 1) % n];

        let vx = p2[0] - p1[0];
        let vy = p2[1] - p1[1];
        let vz = p2[2] - p1[2];
        let len_sq = vx * vx + vy * vy + vz * vz;

        let proj = if len_sq < 1e-12 {
            p1
        } else {
            let u = (((pt[0] - p1[0]) * vx + (pt[1] - p1[1]) * vy + (pt[2] - p1[2]) * vz) / len_sq).clamp(0.0, 1.0);
            [p1[0] + u * vx, p1[1] + u * vy, p1[2] + u * vz]
        };

        let dist_sq = (pt[0] - proj[0]).powi(2) + (pt[1] - proj[1]).powi(2) + (pt[2] - proj[2]).powi(2);
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_pt = proj;
        }
    }

    best_pt
}

pub fn curve_area(info: &CurveInfo) -> f64 {
    if info.points.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    let n = info.points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += info.points[i][0] * info.points[j][1];
        area -= info.points[j][0] * info.points[i][1];
    }
    (area.abs()) / 2.0
}
