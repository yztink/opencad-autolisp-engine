use std::sync::{Arc, Mutex};
use lisp_cad::{create_cad_env, DialogDef, DialogResponse, DialogValue, MockCadBridge};
use lisp_core::{eval_str, Value};

#[test]
fn test_gui_all_controls_roundtrip() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    // Construct a comprehensive dialog with all 11 control types
    let code = r#"
        (setq dlg
          (oca-dialog "Comprehensive CAD Tool"
            '((width . 520.0) (submit . "Execute") (cancel . "Dismiss"))
            (oca-info "Configure all tool parameters below:")
            (oca-row
              (oca-text "txt_name" "Feature Name" "DefaultFeature")
              (oca-number "num_depth" "Cut Depth" 12.5 '((min . 0.0) (max . 100.0) (step . 0.5))))
            (oca-row
              (oca-checkbox "chk_chamfer" "Apply Chamfer" T)
              (oca-choice "cmb_material" "Material" '("Steel" "Aluminum" "Titanium" "Brass") "Aluminum"))
            (oca-row
              (oca-color "col_highlight" "Render Color" '(255 128 0))
              (oca-file "fil_template" "Template Path" "C:/cad/part.dxf"
                '((filter . "Drawing Exchange (*.dxf)") (extensions . ("dxf")) (save . nil))))
            (oca-fields "Advanced Settings" '((collapsed . nil))
              (oca-row
                (oca-table "tbl_offsets" "Tool Offsets"
                  '("Index" "X" "Y")
                  '(("1" "0.0" "0.0") ("2" "10.0" "5.0"))
                  '((allow_add . T) (allow_delete . nil))))
              (oca-row
                (oca-button "btn_preview" "Calculate Preview" "preview_action")))))
    "#;

    let dlg_val = eval_str(code, &mut env).unwrap();
    assert!(dlg_val.is_truthy());

    // Simulate dialog response with submission of all types
    let mut submitted_vals = std::collections::HashMap::new();
    submitted_vals.insert("txt_name".to_string(), DialogValue::Text("MachinedSlot".to_string()));
    submitted_vals.insert("num_depth".to_string(), DialogValue::Number(15.75));
    submitted_vals.insert("chk_chamfer".to_string(), DialogValue::Bool(true));
    submitted_vals.insert("cmb_material".to_string(), DialogValue::Choice { index: 2, text: "Titanium".to_string() });
    submitted_vals.insert("col_highlight".to_string(), DialogValue::Color([255, 128, 0]));
    submitted_vals.insert("fil_template".to_string(), DialogValue::FilePath("C:/cad/custom.dxf".to_string()));
    submitted_vals.insert(
        "tbl_offsets".to_string(),
        DialogValue::Table(vec![
            vec!["1".into(), "0.0".into(), "0.0".into()],
            vec!["2".into(), "10.0".into(), "5.0".into()],
            vec!["3".into(), "20.0".into(), "10.0".into()],
        ]),
    );

    mock.lock().unwrap().mock_dialog_response =
        Some(serde_json::to_string(&DialogResponse::Submitted(submitted_vals)).unwrap());

    let eval_show = r#"
        (setq res (oca-dialog-show dlg))
        (list
          (oca-get-str "txt_name" res)
          (oca-get-real "num_depth" res)
          (oca-get-int "num_depth" res)
          (oca-get-bool "chk_chamfer" res)
          (oca-get "cmb_material" res)
          (oca-get "col_highlight" res)
          (oca-get-str "fil_template" res)
          (length (oca-get "tbl_offsets" res)))
    "#;

    let res = eval_str(eval_show, &mut env).unwrap();
    assert_eq!(
        res,
        Value::from_slice(&[
            Value::Str("MachinedSlot".into()),
            Value::Real(15.75),
            Value::Int(15),
            Value::T,
            Value::Str("Titanium".into()),
            Value::from_slice(&[Value::Int(255), Value::Int(128), Value::Int(0)]),
            Value::Str("C:/cad/custom.dxf".into()),
            Value::Int(3),
        ])
    );

    // Verify dialog serialization structure captured by bridge
    let br = mock.lock().unwrap();
    let def: DialogDef = serde_json::from_str(&br.shown_dialogs[0]).unwrap();
    assert_eq!(def.title, "Comprehensive CAD Tool");
    assert_eq!(def.width, Some(520.0));
    assert_eq!(def.submit_label, "Execute");
    assert_eq!(def.cancel_label, "Dismiss");
    assert_eq!(def.rows.len(), 5);
}

#[test]
fn test_interactive_dialog_callback() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    // Prepare a mock action event (e.g. file picked)
    let mut current_vals = std::collections::HashMap::new();
    current_vals.insert("picker".to_string(), DialogValue::FilePath("C:/scripts/test.lsp".to_string()));
    current_vals.insert("tbl_files".to_string(), DialogValue::Table(vec![vec!["C:/orig.lsp".to_string()]]));

    let event = lisp_cad::ui_types::DialogEvent {
        control_id: "picker".to_string(),
        action: "pick".to_string(),
        value: Some(DialogValue::FilePath("C:/scripts/test.lsp".to_string())),
        current_values: current_vals,
    };
    mock.lock().unwrap().mock_dialog_actions.push(serde_json::to_string(&event).unwrap());

    // Prepare submitted response
    let mut submitted_vals = std::collections::HashMap::new();
    submitted_vals.insert("lbl_status".to_string(), DialogValue::Text("Ready".to_string()));
    mock.lock().unwrap().mock_dialog_response =
        Some(serde_json::to_string(&DialogResponse::Submitted(submitted_vals)).unwrap());

    let code = r#"
        (setq callback-invoked nil)
        (defun my-handler (id act vals)
          (setq callback-invoked (list id act (oca-get "picker" vals)))
          (list
            (cons "lbl_status" "Updated via action")
            (cons "tbl_files" '(("C:/scripts/test.lsp")))
          )
        )
        (setq dlg (oca-dialog "Interactive Test"
                    (oca-info "lbl_status" "Initial")
                    (oca-row (oca-file "picker" "Pick" ""))
                    (oca-row (oca-table "tbl_files" "Files" '("Path") '(("C:/orig.lsp"))))))
        (setq res (oca-dialog-show dlg 'my-handler))
        callback-invoked
    "#;

    let res = eval_str(code, &mut env).unwrap();
    assert_eq!(
        res,
        Value::from_slice(&[
            Value::Str("picker".into()),
            Value::Str("pick".into()),
            Value::Str("C:/scripts/test.lsp".into()),
        ])
    );
}

#[test]
fn test_dialog_cancellation() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    mock.lock().unwrap().mock_dialog_response =
        Some(serde_json::to_string(&DialogResponse::Cancelled).unwrap());

    let code = r#"
        (setq dlg (oca-dialog "Test Cancel" (oca-text "t" "Text" "")))
        (oca-dialog-show dlg)
    "#;
    let res = eval_str(code, &mut env).unwrap();
    assert_eq!(res, Value::Nil);
}

#[test]
fn test_oca_msgbox_conformance() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    // OK -> 1 -> 'OK
    mock.lock().unwrap().mock_msgbox_response = Some(1);
    assert_eq!(eval_str(r#"(oca-msgbox "Operation finished" "Done")"#, &mut env).unwrap(), Value::Sym("OK".into()));

    // Cancel -> 2 -> 'CANCEL
    mock.lock().unwrap().mock_msgbox_response = Some(2);
    assert_eq!(eval_str(r#"(oca-msgbox "Cancel test" "Title" 'okcancel)"#, &mut env).unwrap(), Value::Sym("CANCEL".into()));

    // Yes -> 6 -> 'YES
    mock.lock().unwrap().mock_msgbox_response = Some(6);
    assert_eq!(eval_str(r#"(oca-msgbox "Delete selected?" "Confirm" 'yesno)"#, &mut env).unwrap(), Value::Sym("YES".into()));

    // No -> 7 -> 'NO
    mock.lock().unwrap().mock_msgbox_response = Some(7);
    assert_eq!(eval_str(r#"(oca-msgbox "Delete selected?" "Confirm" 'yesno)"#, &mut env).unwrap(), Value::Sym("NO".into()));

    // Alist button options
    mock.lock().unwrap().mock_msgbox_response = Some(6);
    assert_eq!(eval_str(r#"(oca-msgbox "Overwrite file?" "Save" '((buttons . yesno)))"#, &mut env).unwrap(), Value::Sym("YES".into()));
}

#[test]
fn test_panel_crud_conformance() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let reg_code = r#"
        (oca-panel-register "inspect_pal" "Inspector"
          '((width . 300) (command . "INSPECT"))
          (oca-text "ent_type" "Type" "LINE")
          (oca-number "ent_len" "Length" 100.0)
          (oca-color "ent_col" "Color" '(0 255 0)))
    "#;
    assert_eq!(eval_str(reg_code, &mut env).unwrap(), Value::T);

    let upd_code = r#"
        (oca-panel-update "inspect_pal"
          '((ent_type . "CIRCLE")
            (ent_len . 50.0)
            (ent_col . (255 0 0))))
    "#;
    assert_eq!(eval_str(upd_code, &mut env).unwrap(), Value::T);

    let get_code = r#"(oca-panel-get "inspect_pal" "ent_type")"#;
    assert_eq!(eval_str(get_code, &mut env).unwrap(), Value::Str("CIRCLE".into()));
    let get_len = r#"(oca-panel-get "inspect_pal" "ent_len")"#;
    assert_eq!(eval_str(get_len, &mut env).unwrap(), Value::Real(50.0));

    let toggle_code = r#"(oca-panel-toggle "inspect_pal" nil)"#;
    assert_eq!(eval_str(toggle_code, &mut env).unwrap(), Value::T);

    let br = mock.lock().unwrap();
    assert_eq!(br.toggled_panels[0], ("inspect_pal".to_string(), Some(false)));
}

#[test]
fn test_custom_gui_panel_routine() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (defun c:calc () (oca-panel-toggle "calc_panel" T))
        (defun c:calcdlg () (princ "\nCALCDLG"))
        (defun c:calc-set (val) val)
        (defun calc-key (k) k)
        (oca-panel-register "calc_panel" "CAD Calculator"
          '((width . 200) (command . "CALC"))
          (oca-text "display" "Value" "0"))
    "#;
    eval_str(code, &mut env).expect("panel script failed evaluation");

    {
        let br = mock.lock().unwrap();
        // Verify panel was registered with correct title and command
        assert!(br.registered_panels.iter().any(|p| p.contains("CAD Calculator")));
        assert!(br.registered_commands.contains_key("CALC"));
        assert!(br.registered_commands.contains_key("CALCDLG"));
        assert!(br.registered_commands.contains_key("CALC-SET"));
    }

    // Verify key press evaluation (7 + 5 =)
    eval_str(r#"(calc-key "7")"#, &mut env).expect("Failed pressing 7");
    eval_str(r#"(calc-key "+")"#, &mut env).expect("Failed pressing +");
    eval_str(r#"(calc-key "5")"#, &mut env).expect("Failed pressing 5");
    eval_str(r#"(calc-key "=")"#, &mut env).expect("Failed pressing =");
}

#[test]
fn test_oca_alignment_and_column_conformance() {
    use lisp_cad::ui_types::{DialogControl, DialogDef, HorizontalAlign, VerticalAlign};

    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (setq dlg
          (oca-dialog "Alignment Test"
            (oca-row '((align-h . "center") (align-v . "middle"))
              (oca-info "Centered Title" '((align-h . "center")))
              (oca-button "btn1" "Action" "act1" '((align-h . "right") (width . 150.0))))
            (oca-column '((align-h . "right") (align-v . "top"))
              (oca-text "txt1" "Label" "Val"))))
        (oca-dialog-show dlg)
    "#;
    eval_str(code, &mut env).unwrap();

    let br = mock.lock().unwrap();
    let def: DialogDef = serde_json::from_str(&br.shown_dialogs[0]).unwrap();
    assert_eq!(def.rows.len(), 2);

    // First row
    assert_eq!(def.rows[0].align_h, Some(HorizontalAlign::Center));
    assert_eq!(def.rows[0].align_v, Some(VerticalAlign::Middle));
    match &def.rows[0].controls[0] {
        DialogControl::InfoText { align_h, .. } => {
            assert_eq!(*align_h, Some(HorizontalAlign::Center));
        }
        _ => panic!("Expected InfoText"),
    }
    match &def.rows[0].controls[1] {
        DialogControl::Button { align_h, width, .. } => {
            assert_eq!(*align_h, Some(HorizontalAlign::Right));
            assert_eq!(*width, Some(150.0));
        }
        _ => panic!("Expected Button"),
    }

    // Second row contains single Column control
    match &def.rows[1].controls[0] {
        DialogControl::Column { align_h, align_v, controls } => {
            assert_eq!(*align_h, Some(HorizontalAlign::Right));
            assert_eq!(*align_v, Some(VerticalAlign::Top));
            assert_eq!(controls.len(), 1);
        }
        _ => panic!("Expected Column"),
    }
}



