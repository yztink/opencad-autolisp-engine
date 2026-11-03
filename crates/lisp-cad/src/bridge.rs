#[derive(Clone, Debug, PartialEq)]
pub enum DxfValue {
    Str(String),
    Int(i64),
    Real(f64),
    Point([f64; 3]),
    Handle(u64),
    XData(Vec<(String, Vec<(i16, DxfValue)>)>),
}

pub trait CadBridge: Send + Sync {
    fn execute_command(&mut self, cmd: &str) -> Result<(), String>;
    fn execute_command_args(&mut self, args: &[String]) -> Result<(), String> {
        self.execute_command(&args.join(" "))
    }
    fn register_command(&mut self, name: &str, description: &str) -> Result<(), String>;
    fn reg_app(&mut self, _name: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn ent_get(&self, handle: u64, app_list: Option<&[String]>) -> Result<Vec<(i16, DxfValue)>, String>;
    fn ent_mod(&mut self, elist: &[(i16, DxfValue)]) -> Result<(), String>;
    fn ent_make(&mut self, elist: &[(i16, DxfValue)]) -> Result<u64, String>;
    fn ent_del(&mut self, handle: u64) -> Result<(), String>;
    fn ent_next(&self, handle: Option<u64>) -> Option<u64>;
    fn ent_last(&self) -> Option<u64>;

    fn ss_get(
        &mut self,
        mode: &str,
        points: Option<&[[f64; 3]]>,
        filter: Option<&[(i16, DxfValue)]>,
    ) -> Result<Option<u64>, String>;
    fn ss_length(&self, ss_handle: u64) -> usize;
    fn ss_name(&self, ss_handle: u64, index: usize) -> Option<u64>;
    fn ss_add(&mut self, ename: u64, ss_handle: Option<u64>) -> Result<u64, String>;
    fn ss_del(&mut self, ename: u64, ss_handle: u64) -> Result<(), String>;
    fn ss_memb(&self, ename: u64, ss_handle: u64) -> bool;

    fn get_point(&mut self, base_pt: Option<[f64; 3]>, prompt: &str) -> Result<Option<[f64; 3]>, String>;
    fn get_dist(&mut self, base_pt: Option<[f64; 3]>, prompt: &str) -> Result<Option<f64>, String>;
    fn get_string(&mut self, has_spaces: bool, prompt: &str) -> Result<Option<String>, String>;
    fn get_int(&mut self, prompt: &str) -> Result<Option<i64>, String>;
    fn get_real(&mut self, prompt: &str) -> Result<Option<f64>, String>;
    fn get_kword(&mut self, prompt: &str) -> Result<Option<String>, String>;
    fn init_get(&mut self, bits: i32, keywords: &str);

    fn get_var(&self, name: &str) -> Result<DxfValue, String>;
    fn set_var(&mut self, name: &str, val: DxfValue) -> Result<(), String>;

    fn alert(&mut self, message: &str);

    // UI Dialogs, Message Boxes & Panels
    fn msg_box(
        &mut self,
        prompt: &str,
        title: Option<&str>,
        buttons: i32,
    ) -> Result<i32, String> {
        let _ = title;
        let _ = buttons;
        self.alert(prompt);
        Ok(1) // IDOK
    }

    fn show_dialog(&mut self, _dialog_json: &str) -> Result<String, String> {
        Err("Dialogs not supported by this CAD host".to_string())
    }

    fn register_panel(&mut self, _panel_json: &str) -> Result<bool, String> {
        Err("Dockable panels not supported by this CAD host".to_string())
    }

    fn update_panel(&mut self, _panel_id: &str, _values_json: &str) -> Result<bool, String> {
        Err("Dockable panels not supported by this CAD host".to_string())
    }

    fn toggle_panel(&mut self, _panel_id: &str, _show: Option<bool>) -> Result<bool, String> {
        Err("Dockable panels not supported by this CAD host".to_string())
    }

    fn get_panel_value(&self, _panel_id: &str, _control_id: &str) -> Option<String> {
        None
    }

    // LData dictionary storage
    fn ldata_put(&mut self, _dict: &str, _key: &str, _val: &str) -> Result<(), String> {
        Ok(())
    }
    fn ldata_get(&self, _dict: &str, _key: &str) -> Option<String> {
        None
    }
    fn ldata_list(&self, _dict: &str) -> Vec<(String, String)> {
        Vec::new()
    }
    fn ldata_delete(&mut self, _dict: &str, _key: &str) -> Result<(), String> {
        Ok(())
    }

    // Entity Selection & Inspection
    fn ent_sel(&mut self, _prompt: Option<&str>) -> Result<Option<(u64, [f64; 3])>, String> {
        if let Some(h) = self.ent_last() {
            Ok(Some((h, [0.0, 0.0, 0.0])))
        } else {
            Ok(None)
        }
    }

    // Symbol Tables (tblsearch / tblnext)
    fn tbl_search(&self, table: &str, symbol: &str) -> Option<Vec<(i16, DxfValue)>> {
        let _ = table;
        let _ = symbol;
        None
    }

    fn tbl_next(&mut self, table: &str, rewind: bool) -> Option<Vec<(i16, DxfValue)>> {
        let _ = table;
        let _ = rewind;
        None
    }

    // Transaction & Undo Marks
    fn start_undo_mark(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn end_undo_mark(&mut self) -> Result<(), String> {
        Ok(())
    }
}
