use std::collections::HashMap;

use crate::bridge::{CadBridge, DxfValue};

#[derive(Default)]
pub struct MockCadBridge {
    pub entities: HashMap<u64, Vec<(i16, DxfValue)>>,
    pub entity_order: Vec<u64>,
    pub next_handle: u64,
    pub selection_sets: HashMap<u64, Vec<u64>>,
    pub next_ss_handle: u64,
    pub previous_selection: Vec<u64>,
    pub executed_commands: Vec<String>,
    pub registered_commands: HashMap<String, String>,
    pub system_variables: HashMap<String, DxfValue>,
    pub alerts: Vec<String>,
    pub ldata: HashMap<(String, String), String>,
    pub mock_dialog_response: Option<String>,
    pub shown_dialogs: Vec<String>,
    pub registered_panels: Vec<String>,
    pub updated_panels: Vec<(String, String)>,
    pub toggled_panels: Vec<(String, Option<bool>)>,
    pub msg_boxes: Vec<(String, Option<String>, i32)>,
    pub mock_msgbox_response: Option<i32>,
    pub panel_values: HashMap<(String, String), String>,
    pub registered_apps: std::collections::HashSet<String>,
    pub tables: HashMap<String, Vec<Vec<(i16, DxfValue)>>>,
    pub table_cursors: HashMap<String, usize>,
    pub undo_marks: usize,
}

impl MockCadBridge {
    pub fn new() -> Self {
        let mut bridge = Self {
            next_handle: 1,
            next_ss_handle: 1,
            ..Default::default()
        };
        // Standard default system variables
        bridge.system_variables.insert("CLAYER".into(), DxfValue::Str("0".into()));
        bridge.system_variables.insert("CIRCLERAD".into(), DxfValue::Real(10.0));

        // Standard symbol tables
        bridge.tables.insert(
            "LAYER".into(),
            vec![
                vec![
                    (0, DxfValue::Str("LAYER".into())),
                    (2, DxfValue::Str("0".into())),
                    (70, DxfValue::Int(0)),
                    (62, DxfValue::Int(7)),
                    (6, DxfValue::Str("CONTINUOUS".into())),
                ],
                vec![
                    (0, DxfValue::Str("LAYER".into())),
                    (2, DxfValue::Str("DEFPOINTS".into())),
                    (70, DxfValue::Int(0)),
                    (62, DxfValue::Int(7)),
                    (6, DxfValue::Str("CONTINUOUS".into())),
                ],
            ],
        );
        bridge.tables.insert(
            "LTYPE".into(),
            vec![
                vec![
                    (0, DxfValue::Str("LTYPE".into())),
                    (2, DxfValue::Str("CONTINUOUS".into())),
                    (70, DxfValue::Int(0)),
                    (3, DxfValue::Str("Solid line".into())),
                    (72, DxfValue::Int(65)),
                    (73, DxfValue::Int(0)),
                    (40, DxfValue::Real(0.0)),
                ],
                vec![
                    (0, DxfValue::Str("LTYPE".into())),
                    (2, DxfValue::Str("BYLAYER".into())),
                    (70, DxfValue::Int(0)),
                    (3, DxfValue::Str("".into())),
                    (72, DxfValue::Int(65)),
                    (73, DxfValue::Int(0)),
                    (40, DxfValue::Real(0.0)),
                ],
                vec![
                    (0, DxfValue::Str("LTYPE".into())),
                    (2, DxfValue::Str("BYBLOCK".into())),
                    (70, DxfValue::Int(0)),
                    (3, DxfValue::Str("".into())),
                    (72, DxfValue::Int(65)),
                    (73, DxfValue::Int(0)),
                    (40, DxfValue::Real(0.0)),
                ],
            ],
        );
        bridge.tables.insert(
            "STYLE".into(),
            vec![
                vec![
                    (0, DxfValue::Str("STYLE".into())),
                    (2, DxfValue::Str("STANDARD".into())),
                    (70, DxfValue::Int(0)),
                    (40, DxfValue::Real(0.0)),
                    (41, DxfValue::Real(1.0)),
                    (50, DxfValue::Real(0.0)),
                    (71, DxfValue::Int(0)),
                    (42, DxfValue::Real(0.2)),
                    (3, DxfValue::Str("txt".into())),
                    (4, DxfValue::Str("".into())),
                ],
            ],
        );
        bridge.tables.insert(
            "BLOCK".into(),
            vec![
                vec![
                    (0, DxfValue::Str("BLOCK".into())),
                    (2, DxfValue::Str("*MODEL_SPACE".into())),
                    (70, DxfValue::Int(0)),
                    (10, DxfValue::Point([0.0, 0.0, 0.0])),
                    (1, DxfValue::Str("".into())),
                ],
                vec![
                    (0, DxfValue::Str("BLOCK".into())),
                    (2, DxfValue::Str("*PAPER_SPACE".into())),
                    (70, DxfValue::Int(0)),
                    (10, DxfValue::Point([0.0, 0.0, 0.0])),
                    (1, DxfValue::Str("".into())),
                ],
            ],
        );
        bridge.tables.insert(
            "DIMSTYLE".into(),
            vec![
                vec![
                    (0, DxfValue::Str("DIMSTYLE".into())),
                    (2, DxfValue::Str("STANDARD".into())),
                    (70, DxfValue::Int(0)),
                    (40, DxfValue::Real(1.0)),
                    (41, DxfValue::Real(0.18)),
                ],
            ],
        );
        bridge.tables.insert(
            "APPID".into(),
            vec![
                vec![
                    (0, DxfValue::Str("APPID".into())),
                    (2, DxfValue::Str("ACAD".into())),
                    (70, DxfValue::Int(0)),
                ],
            ],
        );
        bridge.tables.insert(
            "VIEW".into(),
            vec![
                vec![
                    (0, DxfValue::Str("VIEW".into())),
                    (2, DxfValue::Str("TOP".into())),
                    (70, DxfValue::Int(0)),
                    (40, DxfValue::Real(100.0)),
                    (10, DxfValue::Point([0.0, 0.0, 0.0])),
                    (41, DxfValue::Real(150.0)),
                ],
            ],
        );
        bridge.tables.insert(
            "VPORT".into(),
            vec![
                vec![
                    (0, DxfValue::Str("VPORT".into())),
                    (2, DxfValue::Str("*ACTIVE".into())),
                    (70, DxfValue::Int(0)),
                    (10, DxfValue::Point([0.0, 0.0, 0.0])),
                    (11, DxfValue::Point([1.0, 1.0, 0.0])),
                ],
            ],
        );
        bridge.tables.insert(
            "UCS".into(),
            vec![
                vec![
                    (0, DxfValue::Str("UCS".into())),
                    (2, DxfValue::Str("WORLD".into())),
                    (70, DxfValue::Int(0)),
                    (10, DxfValue::Point([0.0, 0.0, 0.0])),
                    (11, DxfValue::Point([1.0, 0.0, 0.0])),
                    (12, DxfValue::Point([0.0, 1.0, 0.0])),
                ],
            ],
        );

        bridge
    }
}

impl CadBridge for MockCadBridge {
    fn execute_command(&mut self, cmd: &str) -> Result<(), String> {
        self.executed_commands.push(cmd.to_string());
        Ok(())
    }

    fn register_command(&mut self, name: &str, description: &str) -> Result<(), String> {
        self.registered_commands.insert(name.to_ascii_uppercase(), description.to_string());
        Ok(())
    }

    fn reg_app(&mut self, name: &str) -> Result<bool, String> {
        Ok(self.registered_apps.insert(name.to_ascii_uppercase()))
    }

    fn ent_get(&self, handle: u64, app_list: Option<&[String]>) -> Result<Vec<(i16, DxfValue)>, String> {
        let elist = self
            .entities
            .get(&handle)
            .cloned()
            .ok_or_else(|| format!("Entity handle {:X} not found", handle))?;

        match app_list {
            None => Ok(elist.into_iter().filter(|(code, _)| *code != -3).collect()),
            Some(apps) => {
                let match_all = apps.iter().any(|a| a == "*");
                let app_names: std::collections::HashSet<String> =
                    apps.iter().map(|a| a.to_ascii_uppercase()).collect();
                let filtered = elist
                    .into_iter()
                    .filter_map(|(code, val)| {
                        if code == -3 {
                            if let DxfValue::XData(xapps) = val {
                                let kept: Vec<_> = xapps
                                    .into_iter()
                                    .filter(|(name, _)| {
                                        match_all || app_names.contains(&name.to_ascii_uppercase())
                                    })
                                    .collect();
                                if !kept.is_empty() {
                                    Some((-3, DxfValue::XData(kept)))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            Some((code, val))
                        }
                    })
                    .collect();
                Ok(filtered)
            }
        }
    }

    fn ent_mod(&mut self, elist: &[(i16, DxfValue)]) -> Result<(), String> {
        // Find handle in elist (group -1 or 5)
        let handle = elist
            .iter()
            .find(|(code, _)| *code == 5 || *code == -1)
            .and_then(|(_, val)| match val {
                DxfValue::Handle(h) => Some(*h),
                DxfValue::Str(s) => u64::from_str_radix(s, 16).ok(),
                _ => None,
            })
            .ok_or_else(|| "No entity handle specified in entmod list".to_string())?;

        if self.entities.contains_key(&handle) {
            self.entities.insert(handle, elist.to_vec());
            Ok(())
        } else {
            Err(format!("Entity {:X} not found", handle))
        }
    }

    fn ent_make(&mut self, elist: &[(i16, DxfValue)]) -> Result<u64, String> {
        let handle = self.next_handle;
        self.next_handle += 1;

        let mut record = elist.to_vec();
        record.push((5, DxfValue::Handle(handle)));

        self.entities.insert(handle, record);
        self.entity_order.push(handle);
        Ok(handle)
    }

    fn ent_del(&mut self, handle: u64) -> Result<(), String> {
        self.entities.remove(&handle);
        self.entity_order.retain(|&h| h != handle);
        Ok(())
    }

    fn ent_next(&self, handle: Option<u64>) -> Option<u64> {
        match handle {
            None => self.entity_order.first().copied(),
            Some(h) => {
                let idx = self.entity_order.iter().position(|&x| x == h)?;
                self.entity_order.get(idx + 1).copied()
            }
        }
    }

    fn ent_last(&self) -> Option<u64> {
        self.entity_order.last().copied()
    }

    fn ss_get(
        &mut self,
        mode: &str,
        points: Option<&[[f64; 3]]>,
        filter: Option<&[(i16, DxfValue)]>,
    ) -> Result<Option<u64>, String> {
        let mode_upper = mode.to_uppercase();
        let single_select = mode_upper.contains(":S");
        let allow_duplicates = mode_upper.contains(":D");

        let candidate_handles: Vec<u64> = if mode_upper.contains('P') && !mode_upper.contains("WP") && !mode_upper.contains("CP") {
            self.previous_selection.clone()
        } else if mode_upper.contains('L') && !mode_upper.contains(":L") {
            self.ent_last().into_iter().collect()
        } else {
            self.entity_order.clone()
        };

        let mut matches = Vec::new();
        for handle in candidate_handles {
            if let Some(elist) = self.entities.get(&handle) {
                // 1. Spatial check
                let bbox = crate::dxf::get_entity_bbox(elist);
                if !crate::dxf::check_spatial(&mode_upper, points, bbox) {
                    continue;
                }

                // 2. DXF Filter check
                if let Some(filt) = filter {
                    if !crate::dxf::eval_dxf_filter(elist, filt) {
                        continue;
                    }
                }

                if allow_duplicates || !matches.contains(&handle) {
                    matches.push(handle);
                }

                if single_select {
                    break;
                }
            }
        }

        if matches.is_empty() {
            return Ok(None);
        }

        self.previous_selection = matches.clone();
        let ss_handle = self.next_ss_handle;
        self.next_ss_handle += 1;
        self.selection_sets.insert(ss_handle, matches);
        Ok(Some(ss_handle))
    }

    fn ss_length(&self, ss_handle: u64) -> usize {
        self.selection_sets
            .get(&ss_handle)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    fn ss_name(&self, ss_handle: u64, index: usize) -> Option<u64> {
        self.selection_sets
            .get(&ss_handle)
            .and_then(|s| s.get(index).copied())
    }

    fn ss_add(&mut self, ename: u64, ss_handle: Option<u64>) -> Result<u64, String> {
        match ss_handle {
            Some(h) => {
                let set = self.selection_sets.entry(h).or_default();
                if ename != 0 && !set.contains(&ename) {
                    set.push(ename);
                }
                Ok(h)
            }
            None => {
                let new_h = self.next_ss_handle;
                self.next_ss_handle += 1;
                let initial = if ename == 0 { Vec::new() } else { vec![ename] };
                self.selection_sets.insert(new_h, initial);
                Ok(new_h)
            }
        }
    }

    fn ss_del(&mut self, ename: u64, ss_handle: u64) -> Result<(), String> {
        if let Some(set) = self.selection_sets.get_mut(&ss_handle) {
            set.retain(|&h| h != ename);
        }
        Ok(())
    }

    fn ss_memb(&self, ename: u64, ss_handle: u64) -> bool {
        self.selection_sets
            .get(&ss_handle)
            .map(|set| set.contains(&ename))
            .unwrap_or(false)
    }

    fn get_point(&mut self, _base_pt: Option<[f64; 3]>, _prompt: &str) -> Result<Option<[f64; 3]>, String> {
        Ok(Some([10.0, 20.0, 0.0]))
    }

    fn get_dist(&mut self, _base_pt: Option<[f64; 3]>, _prompt: &str) -> Result<Option<f64>, String> {
        Ok(Some(50.0))
    }

    fn get_string(&mut self, _has_spaces: bool, _prompt: &str) -> Result<Option<String>, String> {
        Ok(Some("TEST_STRING".to_string()))
    }

    fn get_int(&mut self, _prompt: &str) -> Result<Option<i64>, String> {
        Ok(Some(42))
    }

    fn get_real(&mut self, _prompt: &str) -> Result<Option<f64>, String> {
        Ok(Some(3.14159))
    }

    fn get_kword(&mut self, _prompt: &str) -> Result<Option<String>, String> {
        Ok(Some("YES".to_string()))
    }

    fn init_get(&mut self, _bits: i32, _keywords: &str) {}

    fn get_var(&self, name: &str) -> Result<DxfValue, String> {
        self.system_variables
            .get(&name.to_ascii_uppercase())
            .cloned()
            .ok_or_else(|| format!("System variable {} not found", name))
    }

    fn set_var(&mut self, name: &str, val: DxfValue) -> Result<(), String> {
        self.system_variables.insert(name.to_ascii_uppercase(), val);
        Ok(())
    }

    fn alert(&mut self, message: &str) {
        self.alerts.push(message.to_string());
    }

    fn msg_box(
        &mut self,
        prompt: &str,
        title: Option<&str>,
        buttons: i32,
    ) -> Result<i32, String> {
        self.alerts.push(prompt.to_string());
        self.msg_boxes.push((prompt.to_string(), title.map(|s| s.to_string()), buttons));
        Ok(self.mock_msgbox_response.unwrap_or(1))
    }

    fn show_dialog(&mut self, dialog_json: &str) -> Result<String, String> {
        self.shown_dialogs.push(dialog_json.to_string());
        Ok(self.mock_dialog_response.clone().unwrap_or_else(|| "\"Cancelled\"".to_string()))
    }

    fn register_panel(&mut self, panel_json: &str) -> Result<bool, String> {
        self.registered_panels.push(panel_json.to_string());
        if let Ok(def) = serde_json::from_str::<crate::ui_types::PanelDef>(panel_json) {
            seed_mock_panel_defaults(&def.id, &def.controls, &mut self.panel_values);
        }
        Ok(true)
    }

    fn update_panel(&mut self, panel_id: &str, values_json: &str) -> Result<bool, String> {
        self.updated_panels.push((panel_id.to_string(), values_json.to_string()));
        if let Ok(values) = serde_json::from_str::<HashMap<String, crate::ui_types::DialogValue>>(values_json) {
            for (k, v) in values {
                if let Ok(v_json) = serde_json::to_string(&v) {
                    self.panel_values.insert((panel_id.to_lowercase(), k.to_lowercase()), v_json);
                }
            }
        }
        Ok(true)
    }

    fn toggle_panel(&mut self, panel_id: &str, show: Option<bool>) -> Result<bool, String> {
        self.toggled_panels.push((panel_id.to_string(), show));
        Ok(true)
    }

    fn get_panel_value(&self, panel_id: &str, control_id: &str) -> Option<String> {
        self.panel_values.get(&(panel_id.to_lowercase(), control_id.to_lowercase())).cloned()
    }

    fn ldata_put(&mut self, dict: &str, key: &str, val: &str) -> Result<(), String> {
        self.ldata.insert((dict.to_string(), key.to_string()), val.to_string());
        Ok(())
    }

    fn ldata_get(&self, dict: &str, key: &str) -> Option<String> {
        self.ldata.get(&(dict.to_string(), key.to_string())).cloned()
    }

    fn ldata_list(&self, dict: &str) -> Vec<(String, String)> {
        self.ldata
            .iter()
            .filter(|((d, _), _)| d == dict)
            .map(|((_, k), v)| (k.clone(), v.clone()))
            .collect()
    }

    fn ldata_delete(&mut self, dict: &str, key: &str) -> Result<(), String> {
        self.ldata.remove(&(dict.to_string(), key.to_string()));
        Ok(())
    }

    fn ent_sel(&mut self, _prompt: Option<&str>) -> Result<Option<(u64, [f64; 3])>, String> {
        if let Some(h) = self.ent_last() {
            Ok(Some((h, [10.0, 20.0, 0.0])))
        } else {
            Ok(None)
        }
    }

    fn tbl_search(&self, table: &str, symbol: &str) -> Option<Vec<(i16, DxfValue)>> {
        let tbl_key = table.to_ascii_uppercase();
        let sym_key = symbol.to_ascii_uppercase();
        if let Some(records) = self.tables.get(&tbl_key) {
            for rec in records {
                if let Some((_, DxfValue::Str(name))) = rec.iter().find(|(c, _)| *c == 2) {
                    if name.to_ascii_uppercase() == sym_key {
                        return Some(rec.clone());
                    }
                }
            }
        }
        None
    }

    fn tbl_next(&mut self, table: &str, rewind: bool) -> Option<Vec<(i16, DxfValue)>> {
        let tbl_key = table.to_ascii_uppercase();
        let cursor = self.table_cursors.entry(tbl_key.clone()).or_insert(0);
        if rewind {
            *cursor = 0;
        }
        if let Some(records) = self.tables.get(&tbl_key) {
            if *cursor < records.len() {
                let item = records[*cursor].clone();
                *cursor += 1;
                return Some(item);
            }
        }
        None
    }

    fn start_undo_mark(&mut self) -> Result<(), String> {
        self.undo_marks += 1;
        Ok(())
    }

    fn end_undo_mark(&mut self) -> Result<(), String> {
        self.undo_marks = self.undo_marks.saturating_sub(1);
        Ok(())
    }
}

fn seed_mock_panel_defaults(
    panel_id: &str,
    controls: &[crate::ui_types::DialogControl],
    map: &mut HashMap<(String, String), String>,
) {
    use crate::ui_types::{DialogControl, DialogValue};
    let pid = panel_id.to_lowercase();
    for ctrl in controls {
        match ctrl {
            DialogControl::Checkbox { id, default, .. } => {
                if let Ok(s) = serde_json::to_string(&DialogValue::Bool(*default)) {
                    map.insert((pid.clone(), id.to_lowercase()), s);
                }
            }
            DialogControl::Choice { id, options, default_index, .. } => {
                let text = options.get(*default_index).cloned().unwrap_or_default();
                if let Ok(s) = serde_json::to_string(&DialogValue::Choice { index: *default_index, text }) {
                    map.insert((pid.clone(), id.to_lowercase()), s);
                }
            }
            DialogControl::Text { id, default, .. } => {
                if let Ok(s) = serde_json::to_string(&DialogValue::Text(default.clone())) {
                    map.insert((pid.clone(), id.to_lowercase()), s);
                }
            }
            DialogControl::Number { id, default, .. } => {
                if let Ok(s) = serde_json::to_string(&DialogValue::Number(*default)) {
                    map.insert((pid.clone(), id.to_lowercase()), s);
                }
            }
            DialogControl::Color { id, default, .. } => {
                if let Ok(s) = serde_json::to_string(&DialogValue::Color(*default)) {
                    map.insert((pid.clone(), id.to_lowercase()), s);
                }
            }
            DialogControl::Row { controls: sub, .. }
            | DialogControl::Column { controls: sub, .. }
            | DialogControl::Fields { controls: sub, .. } => {
                seed_mock_panel_defaults(panel_id, sub, map);
            }
            _ => {}
        }
    }
}
