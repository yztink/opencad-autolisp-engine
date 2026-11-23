use std::sync::{Arc, Mutex};
use lisp_cad::{create_cad_env, DialogControl, DialogDef, DialogResponse, DialogValue, HorizontalAlign, MockCadBridge};
use lisp_core::{eval_str, Value};

#[test]
fn test_gui_all_controls_roundtrip() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    // Construct a comprehensive dialog with all 11 control types
    let code = r#"
        (setq dlg
          (oc-dialog "Comprehensive CAD Tool"
            '((width . 520.0) (submit . "Execute") (cancel . "Dismiss"))
            (oc-info "Configure all tool parameters below:")
            (oc-row
              (oc-text "txt_name" "Feature Name" "DefaultFeature")
              (oc-number "num_depth" "Cut Depth" 12.5 '((min . 0.0) (max . 100.0) (step . 0.5))))
            (oc-row
              (oc-checkbox "chk_chamfer" "Apply Chamfer" T)
              (oc-choice "cmb_material" "Material" '("Steel" "Aluminum" "Titanium" "Brass") "Aluminum"))
            (oc-row
              (oc-color "col_highlight" "Render Color" '(255 128 0))
              (oc-file "fil_template" "Template Path" "C:/cad/part.dxf"
                '((filter . "Drawing Exchange (*.dxf)") (extensions . ("dxf")) (save . nil))))
            (oc-fields "Advanced Settings" '((collapsed . nil))
              (oc-row
                (oc-table "tbl_offsets" "Tool Offsets"
                  '("Index" "X" "Y")
                  '(("1" "0.0" "0.0") ("2" "10.0" "5.0"))
                  '((allow_add . T) (allow_delete . nil))))
              (oc-row
                (oc-button "btn_preview" "Calculate Preview" "preview_action")))))
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
        (setq res (oc-dialog-show dlg))
        (list
          (oc-get-str "txt_name" res)
          (oc-get-real "num_depth" res)
          (oc-get-int "num_depth" res)
          (oc-get-bool "chk_chamfer" res)
          (oc-get "cmb_material" res)
          (oc-get "col_highlight" res)
          (oc-get-str "fil_template" res)
          (length (oc-get "tbl_offsets" res)))
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
          (setq callback-invoked (list id act (oc-get "picker" vals)))
          (list
            (cons "lbl_status" "Updated via action")
            (cons "tbl_files" '(("C:/scripts/test.lsp")))
          )
        )
        (setq dlg (oc-dialog "Interactive Test"
                    (oc-info "lbl_status" "Initial")
                    (oc-row (oc-file "picker" "Pick" ""))
                    (oc-row (oc-table "tbl_files" "Files" '("Path") '(("C:/orig.lsp"))))))
        (setq res (oc-dialog-show dlg 'my-handler))
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
        (setq dlg (oc-dialog "Test Cancel" (oc-text "t" "Text" "")))
        (oc-dialog-show dlg)
    "#;
    let res = eval_str(code, &mut env).unwrap();
    assert_eq!(res, Value::Nil);
}

#[test]
fn test_oc_msgbox_conformance() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    // OK -> 1 -> 'OK
    mock.lock().unwrap().mock_msgbox_response = Some(1);
    assert_eq!(eval_str(r#"(oc-msgbox "Operation finished" "Done")"#, &mut env).unwrap(), Value::Sym("OK".into()));

    // Cancel -> 2 -> 'CANCEL
    mock.lock().unwrap().mock_msgbox_response = Some(2);
    assert_eq!(eval_str(r#"(oc-msgbox "Cancel test" "Title" 'okcancel)"#, &mut env).unwrap(), Value::Sym("CANCEL".into()));

    // Yes -> 6 -> 'YES
    mock.lock().unwrap().mock_msgbox_response = Some(6);
    assert_eq!(eval_str(r#"(oc-msgbox "Delete selected?" "Confirm" 'yesno)"#, &mut env).unwrap(), Value::Sym("YES".into()));

    // No -> 7 -> 'NO
    mock.lock().unwrap().mock_msgbox_response = Some(7);
    assert_eq!(eval_str(r#"(oc-msgbox "Delete selected?" "Confirm" 'yesno)"#, &mut env).unwrap(), Value::Sym("NO".into()));

    // Alist button options
    mock.lock().unwrap().mock_msgbox_response = Some(6);
    assert_eq!(eval_str(r#"(oc-msgbox "Overwrite file?" "Save" '((buttons . yesno)))"#, &mut env).unwrap(), Value::Sym("YES".into()));
}

#[test]
fn test_panel_crud_conformance() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let reg_code = r#"
        (oc-panel-register "inspect_pal" "Inspector"
          '((width . 300) (command . "INSPECT"))
          (oc-text "ent_type" "Type" "LINE")
          (oc-number "ent_len" "Length" 100.0)
          (oc-color "ent_col" "Color" '(0 255 0)))
    "#;
    assert_eq!(eval_str(reg_code, &mut env).unwrap(), Value::T);

    let upd_code = r#"
        (oc-panel-update "inspect_pal"
          '((ent_type . "CIRCLE")
            (ent_len . 50.0)
            (ent_col . (255 0 0))))
    "#;
    assert_eq!(eval_str(upd_code, &mut env).unwrap(), Value::T);

    let get_code = r#"(oc-panel-get "inspect_pal" "ent_type")"#;
    assert_eq!(eval_str(get_code, &mut env).unwrap(), Value::Str("CIRCLE".into()));
    let get_len = r#"(oc-panel-get "inspect_pal" "ent_len")"#;
    assert_eq!(eval_str(get_len, &mut env).unwrap(), Value::Real(50.0));

    let toggle_code = r#"(oc-panel-toggle "inspect_pal" nil)"#;
    assert_eq!(eval_str(toggle_code, &mut env).unwrap(), Value::T);

    let br = mock.lock().unwrap();
    assert_eq!(br.toggled_panels[0], ("inspect_pal".to_string(), Some(false)));
}

#[test]
fn test_custom_gui_panel_routine() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (defun c:calc () (oc-panel-toggle "calc_panel" T))
        (defun c:calcdlg () (princ "\nCALCDLG"))
        (defun c:calc-set (val) val)
        (defun calc-key (k) k)
        (oc-panel-register "calc_panel" "CAD Calculator"
          '((width . 200) (command . "CALC"))
          (oc-text "display" "Value" "0"))
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
fn test_oc_alignment_and_column_conformance() {
    use lisp_cad::ui_types::{DialogControl, DialogDef, HorizontalAlign, VerticalAlign};

    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (setq dlg
          (oc-dialog "Alignment Test"
            (oc-row '((align-h . "center") (align-v . "middle"))
              (oc-info "Centered Title" '((align-h . "center")))
              (oc-button "btn1" "Action" "act1" '((align-h . "right") (width . 150.0))))
            (oc-column '((align-h . "right") (align-v . "top"))
              (oc-text "txt1" "Label" "Val"))))
        (oc-dialog-show dlg)
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

#[test]
fn test_oc_label_and_info_control_conformance() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (setq dlg (oc-dialog "Label Test"
            (oc-row
                (oc-info "Plain Info Without ID")
                (oc-info "info_id" "Info With ID")
                (oc-label "label_id" "Label Text")
                (oc-label "status_id" "Status" "Active" '((align-h . "right"))))))
        (oc-dialog-show dlg)
    "#;
    eval_str(code, &mut env).unwrap();

    let br = mock.lock().unwrap();
    let def: DialogDef = serde_json::from_str(&br.shown_dialogs[0]).unwrap();
    assert_eq!(def.rows.len(), 1);
    let controls = &def.rows[0].controls;
    assert_eq!(controls.len(), 4);

    // 1. Plain info without id -> DialogControl::InfoText
    match &controls[0] {
        DialogControl::InfoText { text, align_h } => {
            assert_eq!(text, "Plain Info Without ID");
            assert_eq!(*align_h, None);
            assert_eq!(controls[0].id(), None);
        }
        _ => panic!("Expected InfoText for plain oc-info"),
    }

    // 2. Info with id -> DialogControl::Label
    match &controls[1] {
        DialogControl::Label { id, text, align_h } => {
            assert_eq!(id, "info_id");
            assert_eq!(text, "Info With ID");
            assert_eq!(*align_h, None);
            assert_eq!(controls[1].id(), Some("info_id"));
        }
        _ => panic!("Expected Label for oc-info with id"),
    }

    // 3. Label with id -> DialogControl::Label
    match &controls[2] {
        DialogControl::Label { id, text, align_h } => {
            assert_eq!(id, "label_id");
            assert_eq!(text, "Label Text");
            assert_eq!(*align_h, None);
            assert_eq!(controls[2].id(), Some("label_id"));
        }
        _ => panic!("Expected Label for oc-label"),
    }

    // 4. Label with id, prefix label, text, and options -> DialogControl::Label
    match &controls[3] {
        DialogControl::Label { id, text, align_h } => {
            assert_eq!(id, "status_id");
            assert_eq!(text, "Status: Active");
            assert_eq!(*align_h, Some(HorizontalAlign::Right));
            assert_eq!(controls[3].id(), Some("status_id"));
        }
        _ => panic!("Expected Label with align_h for oc-label with options"),
    }
}

#[test]
fn test_panel_register_style_option_serialization() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let mut env = create_cad_env(mock.clone());

    let code = r#"
        (oc-panel-register "p1" "Default With Options Look"
          '((width . 280))
          (oc-text "x" "X" "0"))
        (oc-panel-register "p2" "Default No Options Look"
          (oc-text "x" "X" "0"))
        (oc-panel-register "p3" "Palette String Look"
          '(("width" . 280) ("style" . "palette"))
          (oc-text "x" "X" "0"))
        (oc-panel-register "p4" "Palette Symbol Look"
          '((style . palette))
          (oc-text "x" "X" "0"))
        (oc-panel-register "p5" "Dialog Explicit Look"
          '(("style" . "dialog"))
          (oc-text "x" "X" "0"))
        (oc-panel-register "p6" "Form Explicit Look"
          '((style . form))
          (oc-text "x" "X" "0"))
    "#;
    eval_str(code, &mut env).unwrap();

    let br = mock.lock().unwrap();
    assert_eq!(br.registered_panels.len(), 6);

    let p1: serde_json::Value = serde_json::from_str(&br.registered_panels[0]).unwrap();
    assert_eq!(p1["style"], serde_json::json!("form"));

    let p2: serde_json::Value = serde_json::from_str(&br.registered_panels[1]).unwrap();
    assert_eq!(p2["style"], serde_json::json!("form"));

    let p3: serde_json::Value = serde_json::from_str(&br.registered_panels[2]).unwrap();
    assert_eq!(p3["style"], serde_json::json!("palette"));

    let p4: serde_json::Value = serde_json::from_str(&br.registered_panels[3]).unwrap();
    assert_eq!(p4["style"], serde_json::json!("palette"));

    let p5: serde_json::Value = serde_json::from_str(&br.registered_panels[4]).unwrap();
    assert_eq!(p5["style"], serde_json::json!("form"));

    let p6: serde_json::Value = serde_json::from_str(&br.registered_panels[5]).unwrap();
    assert_eq!(p6["style"], serde_json::json!("form"));
}


