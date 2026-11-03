pub mod bridge;
pub mod cad_builtins;
pub mod curve;
pub mod dxf;
pub mod mock;
pub mod ui_builtins;
pub mod ui_types;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub use bridge::{CadBridge, DxfValue};
pub use cad_builtins::register_cad_builtins;
pub use dxf::{check_spatial, dxf_to_lisp, dxf_wildcard_match, eval_dxf_filter, get_entity_bbox, lisp_to_dxf};
pub use mock::MockCadBridge;
pub use ui_builtins::register_ui_builtins;
pub use ui_types::*;

use lisp_core::{create_default_env, Env};

/// Creates an AutoLISP environment bound to a live CAD bridge.
pub fn create_cad_env(bridge: Arc<Mutex<dyn CadBridge>>) -> Rc<RefCell<Env>> {
    let env = create_default_env();
    register_cad_builtins(&mut env.borrow_mut(), bridge);
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use lisp_core::{eval_str, Value};

    #[test]
    fn test_cad_command_dispatch() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"(command "_.LINE" '(0 0 0) '(100 100 0) "")"#;
        eval_str(code, &mut env).unwrap();

        let br = mock.lock().unwrap();
        assert_eq!(br.executed_commands.len(), 1);
        assert_eq!(br.executed_commands[0], "_.LINE 0.0000,0.0000,0.0000 100.0000,100.0000,0.0000 ENTER");
        drop(br);

        // Test with PAUSE and null string ""
        let code_pause = r#"(command "_.LINE" '(10 10 0) PAUSE "")"#;
        eval_str(code_pause, &mut env).unwrap();
        let br = mock.lock().unwrap();
        assert_eq!(br.executed_commands.len(), 2);
        assert_eq!(br.executed_commands[1], "_.LINE 10.0000,10.0000,0.0000 \\ ENTER");
    }

    #[test]
    fn test_entmake_entget_entmod() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (entmake '((0 . "LINE") (8 . "WALLS") (10 0.0 0.0 0.0) (11 50.0 50.0 0.0)))
            (setq last_e (entlast))
            (setq elist (entget last_e))
            (cdr (assoc 8 elist))
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Str("WALLS".into()));

        // Now modify the layer via entmod
        let mod_code = r#"
            (setq elist (subst '(8 . "DOORS") '(8 . "WALLS") elist))
            (entmod elist)
            (cdr (assoc 8 (entget last_e)))
        "#;
        let mod_res = eval_str(mod_code, &mut env).unwrap();
        assert_eq!(mod_res, Value::Str("DOORS".into()));
    }

    #[test]
    fn test_regapp_and_xdata() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (regapp "TEST_APP")
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Str("TEST_APP".into()));

        // Second call should return nil since already registered
        let res2 = eval_str(r#"(regapp "TEST_APP")"#, &mut env).unwrap();
        assert_eq!(res2, Value::Nil);

        // Make an entity with XDATA
        eval_str(
            r#"
            (entmake '(
                (0 . "LINE")
                (8 . "0")
                (10 0.0 0.0 0.0)
                (11 10.0 10.0 0.0)
                (-3 ("TEST_APP" (1000 . "MyExtendedData") (1040 . 99.5)))
            ))
            (setq e (entlast))
        "#,
            &mut env,
        )
        .unwrap();

        // Normal entget should not return -3 group
        let normal_has_xdata = eval_str("(assoc -3 (entget e))", &mut env).unwrap();
        assert_eq!(normal_has_xdata, Value::Nil);

        // entget with app list should return the -3 group
        let xdata_val = eval_str(
            r#"(cdr (assoc 1000 (car (cdr (assoc -3 (entget e '("TEST_APP")))))))"#,
            &mut env,
        )
        .unwrap();
        assert_eq!(xdata_val, Value::Str("MyExtendedData".into()));
    }

    #[test]
    fn test_selection_set_ssget_sslength() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        eval_str(
            r#"
            (entmake '((0 . "LINE") (8 . "LAYER1")))
            (entmake '((0 . "CIRCLE") (8 . "LAYER2")))
            (entmake '((0 . "LINE") (8 . "LAYER1")))
            (setq ss (ssget "X" '((0 . "LINE"))))
            (sslength ss)
        "#,
            &mut env,
        )
        .unwrap();

        let count = eval_str("(sslength ss)", &mut env).unwrap();
        assert_eq!(count, Value::Int(2));
    }

    #[test]
    fn test_vlax_activex_emulation() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (entmake '((0 . "CIRCLE") (8 . "FOUNDATION") (40 . 25.0)))
            (setq en (entlast))
            (setq vla (vlax-ename->vla-object en))
            (setq layer (vlax-get-property vla 'Layer))
        "#;
        let layer = eval_str(code, &mut env).unwrap();
        assert_eq!(layer, Value::Str("FOUNDATION".into()));

        // Mutate property via VLAX
        let mut_code = r#"
            (vlax-put-property vla 'Layer "ROOF")
            (vlax-get-property vla 'Layer)
        "#;
        let updated_layer = eval_str(mut_code, &mut env).unwrap();
        assert_eq!(updated_layer, Value::Str("ROOF".into()));
    }

    #[test]
    fn test_c_command_auto_registration() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (defun c:my-special-cmd ()
                (princ "Hello from command")
            )
        "#;
        eval_str(code, &mut env).unwrap();

        let br = mock.lock().unwrap();
        assert!(br.registered_commands.contains_key("MY-SPECIAL-CMD"));
    }

    #[test]
    fn test_vlax_ldata() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (vlax-ldata-put "MY_DICT" "PARAM_A" 42.5)
            (vlax-ldata-get "MY_DICT" "PARAM_A")
        "#;
        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::Real(42.5));

        // Test with list data
        let list_code = r#"
            (vlax-ldata-put "MY_DICT" "CONFIG" '("alpha" 100 T))
            (vlax-ldata-get "MY_DICT" "CONFIG")
        "#;
        let list_res = eval_str(list_code, &mut env).unwrap();
        assert_eq!(
            list_res,
            Value::from_slice(&[Value::Str("alpha".into()), Value::Int(100), Value::T])
        );
    }

    #[test]
    fn test_msgbox_and_ocl_msgbox() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        // Classic msgbox (flags = 0 -> OK, returns 1)
        let res = eval_str(r#"(msgbox "Hello CAD World!" "Greetings")"#, &mut env).unwrap();
        assert_eq!(res, Value::Int(1));

        let br = mock.lock().unwrap();
        assert_eq!(br.msg_boxes.len(), 1);
        assert_eq!(br.msg_boxes[0].0, "Hello CAD World!");
        assert_eq!(br.msg_boxes[0].1, Some("Greetings".to_string()));
        drop(br);

        // dos_msgbox alias
        let res_dos = eval_str(r#"(dos_msgbox "Dos prompt")"#, &mut env).unwrap();
        assert_eq!(res_dos, Value::Int(1));

        // ocl-msgbox with symbol
        mock.lock().unwrap().mock_msgbox_response = Some(6); // IDYES
        let res_ocl = eval_str(r#"(ocl-msgbox "Proceed with operation?" "Confirm" 'yesno)"#, &mut env).unwrap();
        assert_eq!(res_ocl, Value::Sym("YES".into()));
    }

    #[test]
    fn test_ocl_dialog_construction_and_show() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        // Prepare simulated dialog submission in mock bridge
        let mut submitted_vals = std::collections::HashMap::new();
        submitted_vals.insert("txt_layer".to_string(), DialogValue::Text("WALLS".to_string()));
        submitted_vals.insert("num_offset".to_string(), DialogValue::Number(25.4));
        submitted_vals.insert("chk_join".to_string(), DialogValue::Bool(true));
        mock.lock().unwrap().mock_dialog_response =
            Some(serde_json::to_string(&DialogResponse::Submitted(submitted_vals)).unwrap());

        let code = r#"
            (setq dlg (ocl-dialog "Offset Tool"
                        '((width . 400.0) (submit . "Generate"))
                        (ocl-row
                          (ocl-text "txt_layer" "Target Layer" "0")
                          (ocl-number "num_offset" "Offset Distance" 10.0 '((min . 0.1) (max . 500.0))))
                        (ocl-row
                          (ocl-checkbox "chk_join" "Join Segments" T))
                        (ocl-row
                          (ocl-choice "cmb_side" "Side" '("Both" "Left" "Right") "Both"))))

            (setq res (ocl-dialog-show dlg))
            (list (ocl-get "txt_layer" res)
                  (ocl-get-real "num_offset" res)
                  (ocl-get-bool "chk_join" res))
        "#;

        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(
            res,
            Value::from_slice(&[
                Value::Str("WALLS".into()),
                Value::Real(25.4),
                Value::T,
            ])
        );

        // Verify the dialog JSON captured in mock bridge
        let br = mock.lock().unwrap();
        assert_eq!(br.shown_dialogs.len(), 1);
        let def: DialogDef = serde_json::from_str(&br.shown_dialogs[0]).unwrap();
        assert_eq!(def.title, "Offset Tool");
        assert_eq!(def.width, Some(400.0));
        assert_eq!(def.submit_label, "Generate");
        assert_eq!(def.rows.len(), 3);
    }

    #[test]
    fn test_ocl_panel_integration() {
        let mock = Arc::new(Mutex::new(MockCadBridge::new()));
        let mut env = create_cad_env(mock.clone());

        let code = r#"
            (ocl-panel-register "wall_tool" "Wall Parameters"
              '((width . 280) (command . "WALLTOOL"))
              (ocl-fields "Wall Properties"
                (ocl-row
                  (ocl-text "thickness" "Thickness" "6.0")
                  (ocl-number "height" "Height" 96.0)))
              (ocl-button "btn_apply" "Apply Changes" "apply"))

            (ocl-panel-update "wall_tool"
              '((thickness . "8.0")
                (height . 120.0)))

            (ocl-panel-toggle "wall_tool" T)
        "#;

        let res = eval_str(code, &mut env).unwrap();
        assert_eq!(res, Value::T);

        let br = mock.lock().unwrap();
        assert_eq!(br.registered_panels.len(), 1);
        let panel_def: PanelDef = serde_json::from_str(&br.registered_panels[0]).unwrap();
        assert_eq!(panel_def.id, "wall_tool");
        assert_eq!(panel_def.title, "Wall Parameters");
        assert_eq!(panel_def.width, Some(280.0));
        assert_eq!(panel_def.command, Some("WALLTOOL".to_string()));

        assert_eq!(br.updated_panels.len(), 1);
        assert_eq!(br.updated_panels[0].0, "wall_tool");

        assert_eq!(br.toggled_panels.len(), 1);
        assert_eq!(br.toggled_panels[0], ("wall_tool".to_string(), Some(true)));
    }
}

