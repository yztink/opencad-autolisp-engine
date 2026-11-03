use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use lisp_core::{eval_str, Env};
use lisp_cad::bridge::{CadBridge, DxfValue};
use lisp_cad::{create_cad_env, MockCadBridge};

pub type CommandCallback = extern "C" fn(cmd_str: *const c_char) -> c_int;

pub struct CallbackBridge {
    pub inner: MockCadBridge,
    pub cmd_callback: Option<CommandCallback>,
}

impl CallbackBridge {
    pub fn new() -> Self {
        Self {
            inner: MockCadBridge::new(),
            cmd_callback: None,
        }
    }
}

impl CadBridge for CallbackBridge {
    fn execute_command(&mut self, cmd: &str) -> Result<(), String> {
        if let Some(cb) = self.cmd_callback {
            if let Ok(c_str) = CString::new(cmd) {
                let res = cb(c_str.as_ptr());
                if res != 0 {
                    return Err(format!("Command execution failed with code {}", res));
                }
                return Ok(());
            }
        }
        self.inner.execute_command(cmd)
    }

    fn register_command(&mut self, name: &str, description: &str) -> Result<(), String> {
        self.inner.register_command(name, description)
    }

    fn ent_get(&self, handle: u64, app_list: Option<&[String]>) -> Result<Vec<(i16, DxfValue)>, String> {
        self.inner.ent_get(handle, app_list)
    }

    fn ent_mod(&mut self, elist: &[(i16, DxfValue)]) -> Result<(), String> {
        self.inner.ent_mod(elist)
    }

    fn ent_make(&mut self, elist: &[(i16, DxfValue)]) -> Result<u64, String> {
        let handle = self.inner.ent_make(elist)?;
        if let Some(cb) = self.cmd_callback {
            let mut etype = "";
            let mut p1 = None;
            let mut p2 = None;
            let mut radius = None;
            for (code, val) in elist {
                if *code == 0 {
                    if let DxfValue::Str(s) = val {
                        etype = s.as_str();
                    }
                } else if *code == 10 {
                    if let DxfValue::Point(pt) = val {
                        p1 = Some(*pt);
                    }
                } else if *code == 11 {
                    if let DxfValue::Point(pt) = val {
                        p2 = Some(*pt);
                    }
                } else if *code == 40 {
                    if let DxfValue::Real(r) = val {
                        radius = Some(*r);
                    } else if let DxfValue::Int(n) = val {
                        radius = Some(*n as f64);
                    }
                }
            }
            if etype.eq_ignore_ascii_case("LINE") {
                if let (Some(pt1), Some(pt2)) = (p1, p2) {
                    let cmd = format!("LINE {:.4},{:.4},{:.4} {:.4},{:.4},{:.4}", pt1[0], pt1[1], pt1[2], pt2[0], pt2[1], pt2[2]);
                    if let Ok(c_str) = CString::new(cmd) {
                        let _ = cb(c_str.as_ptr());
                    }
                }
            } else if etype.eq_ignore_ascii_case("CIRCLE") {
                if let (Some(pt1), Some(r)) = (p1, radius) {
                    let cmd = format!("CIRCLE {:.4},{:.4},{:.4} {:.4}", pt1[0], pt1[1], pt1[2], r);
                    if let Ok(c_str) = CString::new(cmd) {
                        let _ = cb(c_str.as_ptr());
                    }
                }
            }
        }
        Ok(handle)
    }

    fn ent_del(&mut self, handle: u64) -> Result<(), String> {
        self.inner.ent_del(handle)
    }

    fn ent_next(&self, handle: Option<u64>) -> Option<u64> {
        self.inner.ent_next(handle)
    }

    fn ent_last(&self) -> Option<u64> {
        self.inner.ent_last()
    }

    fn ss_get(
        &mut self,
        mode: &str,
        points: Option<&[[f64; 3]]>,
        filter: Option<&[(i16, DxfValue)]>,
    ) -> Result<Option<u64>, String> {
        self.inner.ss_get(mode, points, filter)
    }

    fn ss_length(&self, ss_handle: u64) -> usize {
        self.inner.ss_length(ss_handle)
    }

    fn ss_name(&self, ss_handle: u64, index: usize) -> Option<u64> {
        self.inner.ss_name(ss_handle, index)
    }

    fn ss_add(&mut self, ename: u64, ss_handle: Option<u64>) -> Result<u64, String> {
        self.inner.ss_add(ename, ss_handle)
    }

    fn ss_del(&mut self, ename: u64, ss_handle: u64) -> Result<(), String> {
        self.inner.ss_del(ename, ss_handle)
    }

    fn ss_memb(&self, ename: u64, ss_handle: u64) -> bool {
        self.inner.ss_memb(ename, ss_handle)
    }

    fn get_point(&mut self, base_pt: Option<[f64; 3]>, prompt: &str) -> Result<Option<[f64; 3]>, String> {
        self.inner.get_point(base_pt, prompt)
    }

    fn get_dist(&mut self, base_pt: Option<[f64; 3]>, prompt: &str) -> Result<Option<f64>, String> {
        self.inner.get_dist(base_pt, prompt)
    }

    fn get_string(&mut self, has_spaces: bool, prompt: &str) -> Result<Option<String>, String> {
        self.inner.get_string(has_spaces, prompt)
    }

    fn get_int(&mut self, prompt: &str) -> Result<Option<i64>, String> {
        self.inner.get_int(prompt)
    }

    fn get_real(&mut self, prompt: &str) -> Result<Option<f64>, String> {
        self.inner.get_real(prompt)
    }

    fn get_kword(&mut self, prompt: &str) -> Result<Option<String>, String> {
        self.inner.get_kword(prompt)
    }

    fn init_get(&mut self, bits: i32, keywords: &str) {
        self.inner.init_get(bits, keywords)
    }

    fn get_var(&self, name: &str) -> Result<DxfValue, String> {
        self.inner.get_var(name)
    }

    fn set_var(&mut self, name: &str, val: DxfValue) -> Result<(), String> {
        self.inner.set_var(name, val)
    }

    fn alert(&mut self, message: &str) {
        self.inner.alert(message)
    }

    fn msg_box(&mut self, prompt: &str, title: Option<&str>, buttons: i32) -> Result<i32, String> {
        self.inner.msg_box(prompt, title, buttons)
    }

    fn show_dialog(&mut self, dialog_json: &str) -> Result<String, String> {
        self.inner.show_dialog(dialog_json)
    }

    fn register_panel(&mut self, panel_json: &str) -> Result<bool, String> {
        self.inner.register_panel(panel_json)
    }

    fn update_panel(&mut self, panel_id: &str, values_json: &str) -> Result<bool, String> {
        self.inner.update_panel(panel_id, values_json)
    }

    fn toggle_panel(&mut self, panel_id: &str, show: Option<bool>) -> Result<bool, String> {
        self.inner.toggle_panel(panel_id, show)
    }

    fn ldata_put(&mut self, dict: &str, key: &str, val: &str) -> Result<(), String> {
        self.inner.ldata_put(dict, key, val)
    }

    fn ldata_get(&self, dict: &str, key: &str) -> Option<String> {
        self.inner.ldata_get(dict, key)
    }

    fn ldata_list(&self, dict: &str) -> Vec<(String, String)> {
        self.inner.ldata_list(dict)
    }

    fn ldata_delete(&mut self, dict: &str, key: &str) -> Result<(), String> {
        self.inner.ldata_delete(dict, key)
    }
}

pub struct LispEngine {
    pub env: Rc<RefCell<Env>>,
    pub bridge: Arc<Mutex<CallbackBridge>>,
    pub last_error: String,
    pub last_result: String,
}

impl LispEngine {
    pub fn new() -> Self {
        let bridge = Arc::new(Mutex::new(CallbackBridge::new()));
        let env = create_cad_env(bridge.clone());
        Self {
            env,
            bridge,
            last_error: String::new(),
            last_result: String::new(),
        }
    }
}

impl Default for LispEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new AutoLISP engine instance.
/// Returns a raw pointer that must be freed using `lisp_engine_destroy`.
#[no_mangle]
pub extern "C" fn lisp_engine_create() -> *mut LispEngine {
    Box::into_raw(Box::new(LispEngine::new()))
}

/// Destroys an AutoLISP engine instance and frees its memory.
#[no_mangle]
pub extern "C" fn lisp_engine_destroy(engine: *mut LispEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Registers a CAD command execution callback: `int callback(const char* cmd_str)`.
/// This enables `(command ...)` in AutoLISP to drive the CAD host directly.
#[no_mangle]
pub extern "C" fn lisp_engine_set_command_callback(
    engine: *mut LispEngine,
    callback: Option<CommandCallback>,
) {
    if engine.is_null() {
        return;
    }
    let engine_ref = unsafe { &mut *engine };
    engine_ref.bridge.lock().unwrap().cmd_callback = callback;
}

/// Evaluates an AutoLISP code string.
/// Returns 0 on success, -1 on evaluation error, -2 on null input.
#[no_mangle]
pub extern "C" fn lisp_eval_string(
    engine: *mut LispEngine,
    code_ptr: *const c_char,
    out_buf: *mut c_char,
    out_buf_len: usize,
) -> c_int {
    if engine.is_null() || code_ptr.is_null() {
        return -2;
    }

    let engine = unsafe { &mut *engine };
    let c_str = unsafe { CStr::from_ptr(code_ptr) };
    let code = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            engine.last_error = format!("UTF-8 decoding error: {}", e);
            return -1;
        }
    };

    match eval_str(code, &mut engine.env) {
        Ok(val) => {
            engine.last_result = format!("{}", val);
            engine.last_error.clear();

            if !out_buf.is_null() && out_buf_len > 0 {
                copy_to_c_buffer(&engine.last_result, out_buf, out_buf_len);
            }
            0
        }
        Err(err) => {
            engine.last_error = err.to_string();
            engine.last_result.clear();

            if !out_buf.is_null() && out_buf_len > 0 {
                copy_to_c_buffer(&engine.last_error, out_buf, out_buf_len);
            }
            -1
        }
    }
}

/// Copies the last error message into the provided buffer.
#[no_mangle]
pub extern "C" fn lisp_get_last_error(
    engine: *mut LispEngine,
    out_buf: *mut c_char,
    out_buf_len: usize,
) -> c_int {
    if engine.is_null() || out_buf.is_null() || out_buf_len == 0 {
        return -1;
    }

    let engine = unsafe { &*engine };
    copy_to_c_buffer(&engine.last_error, out_buf, out_buf_len);
    0
}

fn copy_to_c_buffer(src: &str, dest: *mut c_char, dest_len: usize) {
    let c_string = CString::new(src).unwrap_or_else(|_| CString::new("").unwrap());
    let bytes = c_string.as_bytes_with_nul();
    let to_copy = bytes.len().min(dest_len);

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, dest, to_copy);
        if to_copy > 0 {
            *dest.add(dest_len - 1) = 0;
        }
    }
}
