use std::sync::{Arc, Mutex};
use lisp_cad::{create_cad_env, MockCadBridge};
use lisp_core::{eval_str, Value};

fn setup_env() -> (Arc<Mutex<MockCadBridge>>, std::rc::Rc<std::cell::RefCell<lisp_core::Env>>) {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let env = create_cad_env(mock.clone());
    (mock, env)
}

#[test]
fn test_conformance_special_forms() {
    let (_mock, mut env) = setup_env();

    // quote
    assert_eq!(eval_str("'(1 2 3)", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(2), Value::Int(3)]));
    // setq & defun
    eval_str("(defun add2 (x) (+ x 2))", &mut env).unwrap();
    assert_eq!(eval_str("(add2 5)", &mut env).unwrap(), Value::Int(7));
    // if
    assert_eq!(eval_str("(if (= 1 1) \"yes\" \"no\")", &mut env).unwrap(), Value::Str("yes".into()));
    assert_eq!(eval_str("(if (= 1 2) \"yes\" \"no\")", &mut env).unwrap(), Value::Str("no".into()));
    // cond
    let cond_code = "(cond ((= 1 2) \"one\") ((= 2 2) \"two\") (t \"default\"))";
    assert_eq!(eval_str(cond_code, &mut env).unwrap(), Value::Str("two".into()));
    // while
    let while_code = "(setq i 0 acc 0) (while (< i 5) (setq acc (+ acc i)) (setq i (+ i 1))) acc";
    assert_eq!(eval_str(while_code, &mut env).unwrap(), Value::Int(10));
    // repeat
    let rep_code = "(setq r 1) (repeat 4 (setq r (* r 2))) r";
    assert_eq!(eval_str(rep_code, &mut env).unwrap(), Value::Int(16));
    // foreach
    let foreach_code = "(setq sum 0) (foreach x '(10 20 30) (setq sum (+ sum x))) sum";
    assert_eq!(eval_str(foreach_code, &mut env).unwrap(), Value::Int(60));
    // progn
    assert_eq!(eval_str("(progn (setq a 1) (setq b 2) (+ a b))", &mut env).unwrap(), Value::Int(3));
    // and / or
    assert_eq!(eval_str("(and 1 2 3)", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(and 1 nil 3)", &mut env).unwrap(), Value::Nil);
    assert_eq!(eval_str("(or nil nil 42 nil)", &mut env).unwrap(), Value::Int(42));
}

#[test]
fn test_conformance_core_eval() {
    let (_mock, mut env) = setup_env();

    // eval
    assert_eq!(eval_str("(eval '(+ 10 20))", &mut env).unwrap(), Value::Int(30));
    // apply
    assert_eq!(eval_str("(apply '+ '(1 2 3 4))", &mut env).unwrap(), Value::Int(10));
    // mapcar
    assert_eq!(eval_str("(mapcar '1+ '(10 20 30))", &mut env).unwrap(), Value::from_slice(&[Value::Int(11), Value::Int(21), Value::Int(31)]));
    // set & boundp
    eval_str("(set (quote myvar) 99)", &mut env).unwrap();
    assert_eq!(eval_str("myvar", &mut env).unwrap(), Value::Int(99));
    assert_eq!(eval_str("(boundp 'myvar)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(boundp 'nonexistent_xyz)", &mut env).unwrap(), Value::Nil);
    // null & not
    assert_eq!(eval_str("(null nil)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(null 5)", &mut env).unwrap(), Value::Nil);
    assert_eq!(eval_str("(not nil)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(type 42)", &mut env).unwrap(), Value::Sym(lisp_core::Symbol::new("INT")));
}

#[test]
fn test_conformance_math_and_bitwise() {
    let (_mock, mut env) = setup_env();

    assert_eq!(eval_str("(+ 1 2 3)", &mut env).unwrap(), Value::Int(6));
    assert_eq!(eval_str("(- 10 3 2)", &mut env).unwrap(), Value::Int(5));
    assert_eq!(eval_str("(* 2 3 4)", &mut env).unwrap(), Value::Int(24));
    assert_eq!(eval_str("(/ 20 2 2)", &mut env).unwrap(), Value::Int(5));
    assert_eq!(eval_str("(1+ 9)", &mut env).unwrap(), Value::Int(10));
    assert_eq!(eval_str("(1- 10)", &mut env).unwrap(), Value::Int(9));
    assert_eq!(eval_str("(abs -42)", &mut env).unwrap(), Value::Int(42));
    assert_eq!(eval_str("(max 1 5 3)", &mut env).unwrap(), Value::Int(5));
    assert_eq!(eval_str("(min 10 2 8)", &mut env).unwrap(), Value::Int(2));
    assert_eq!(eval_str("(rem 10 3)", &mut env).unwrap(), Value::Int(1));
    assert_eq!(eval_str("(sqrt 16.0)", &mut env).unwrap(), Value::Real(4.0));
    assert_eq!(eval_str("(expt 2 8)", &mut env).unwrap(), Value::Real(256.0));
    assert_eq!(eval_str("(fix 3.8)", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(float 5)", &mut env).unwrap(), Value::Real(5.0));
    assert_eq!(eval_str("(gcd 24 36)", &mut env).unwrap(), Value::Int(12));

    // Log & Exp
    assert_eq!(eval_str("(log (exp 1.0))", &mut env).unwrap(), Value::Real(1.0));

    // Bitwise: logand, logior, ~, lsh, boole
    assert_eq!(eval_str("(logand 7 3)", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(logior 4 2 1)", &mut env).unwrap(), Value::Int(7));
    assert_eq!(eval_str("(~ 0)", &mut env).unwrap(), Value::Int(-1));
    assert_eq!(eval_str("(lsh 1 4)", &mut env).unwrap(), Value::Int(16));
    assert_eq!(eval_str("(lsh 16 -2)", &mut env).unwrap(), Value::Int(4));
    assert_eq!(eval_str("(boole 1 5 3)", &mut env).unwrap(), Value::Int(1)); // AND
    assert_eq!(eval_str("(boole 7 5 3)", &mut env).unwrap(), Value::Int(7)); // OR
    assert_eq!(eval_str("(boole 6 5 3)", &mut env).unwrap(), Value::Int(6)); // XOR
}

#[test]
fn test_conformance_lists() {
    let (_mock, mut env) = setup_env();

    let tree = "'(((1 . 2) . (3 . 4)) . ((5 . 6) . (7 . 8)))";
    eval_str(&format!("(setq t1 {})", tree), &mut env).unwrap();

    assert_eq!(eval_str("(caar t1)", &mut env).unwrap(), Value::cons(Value::Int(1), Value::Int(2)));
    assert_eq!(eval_str("(cadr t1)", &mut env).unwrap(), Value::cons(Value::Int(5), Value::Int(6)));
    assert_eq!(eval_str("(caaar t1)", &mut env).unwrap(), Value::Int(1));
    assert_eq!(eval_str("(cdaar t1)", &mut env).unwrap(), Value::Int(2));
    assert_eq!(eval_str("(cadar t1)", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(cddar t1)", &mut env).unwrap(), Value::Int(4));
    assert_eq!(eval_str("(caadr t1)", &mut env).unwrap(), Value::Int(5));
    assert_eq!(eval_str("(cdadr t1)", &mut env).unwrap(), Value::Int(6));
    assert_eq!(eval_str("(caddr '((1) (2) 3))", &mut env).unwrap(), Value::Int(3));

    // 4th-level permutations
    let tree4 = "'((((1 . 2) . (3 . 4)) . ((5 . 6) . (7 . 8))) . (((9 . 10) . (11 . 12)) . ((13 . 14) . (15 . 16))))";
    eval_str(&format!("(setq t4 {})", tree4), &mut env).unwrap();

    assert_eq!(eval_str("(caaaar t4)", &mut env).unwrap(), Value::Int(1));
    assert_eq!(eval_str("(cdaaar t4)", &mut env).unwrap(), Value::Int(2));
    assert_eq!(eval_str("(cadaar t4)", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(cddaar t4)", &mut env).unwrap(), Value::Int(4));
    assert_eq!(eval_str("(caadar t4)", &mut env).unwrap(), Value::Int(5));
    assert_eq!(eval_str("(cdadar t4)", &mut env).unwrap(), Value::Int(6));
    assert_eq!(eval_str("(caddar t4)", &mut env).unwrap(), Value::Int(7));
    assert_eq!(eval_str("(cdddar t4)", &mut env).unwrap(), Value::Int(8));

    assert_eq!(eval_str("(caaadr t4)", &mut env).unwrap(), Value::Int(9));
    assert_eq!(eval_str("(cdaadr t4)", &mut env).unwrap(), Value::Int(10));
    assert_eq!(eval_str("(cadadr t4)", &mut env).unwrap(), Value::Int(11));
    assert_eq!(eval_str("(cddadr t4)", &mut env).unwrap(), Value::Int(12));
    assert_eq!(eval_str("(caaddr t4)", &mut env).unwrap(), Value::Int(13));
    assert_eq!(eval_str("(cdaddr t4)", &mut env).unwrap(), Value::Int(14));
    assert_eq!(eval_str("(cadddr t4)", &mut env).unwrap(), Value::Int(15));
    assert_eq!(eval_str("(cddddr t4)", &mut env).unwrap(), Value::Int(16));

    // nth, last, reverse, length, append, subst, assoc
    assert_eq!(eval_str("(nth 2 '(10 20 30 40))", &mut env).unwrap(), Value::Int(30));
    assert_eq!(eval_str("(last '(10 20 30))", &mut env).unwrap(), Value::Int(30));
    assert_eq!(eval_str("(reverse '(1 2 3))", &mut env).unwrap(), Value::from_slice(&[Value::Int(3), Value::Int(2), Value::Int(1)]));
    assert_eq!(eval_str("(length '(1 2 3 4))", &mut env).unwrap(), Value::Int(4));
    assert_eq!(eval_str("(append '(1 2) '(3 4))", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    assert_eq!(eval_str("(assoc 10 '((0 . \"LINE\") (10 0 0 0)))", &mut env).unwrap(), Value::from_slice(&[Value::Int(10), Value::Int(0), Value::Int(0), Value::Int(0)]));
    assert_eq!(eval_str("(subst 99 2 '(1 2 3 2))", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(99), Value::Int(3), Value::Int(99)]));
}

#[test]
fn test_conformance_strings_and_chars() {
    let (_mock, mut env) = setup_env();

    assert_eq!(eval_str("(strcat \"Open\" \"CAD\" \" Studio\")", &mut env).unwrap(), Value::Str("OpenCAD Studio".into()));
    assert_eq!(eval_str("(strlen \"AutoLISP\")", &mut env).unwrap(), Value::Int(8));
    assert_eq!(eval_str("(substr \"AutoLISP\" 5 4)", &mut env).unwrap(), Value::Str("LISP".into()));
    assert_eq!(eval_str("(strcase \"Hello World\")", &mut env).unwrap(), Value::Str("HELLO WORLD".into()));
    assert_eq!(eval_str("(strcase \"Hello World\" t)", &mut env).unwrap(), Value::Str("hello world".into()));
    assert_eq!(eval_str("(itoa 123)", &mut env).unwrap(), Value::Str("123".into()));
    assert_eq!(eval_str("(atoi \"456\")", &mut env).unwrap(), Value::Int(456));
    assert_eq!(eval_str("(atof \"3.14\")", &mut env).unwrap(), Value::Real(3.14));
    assert_eq!(eval_str("(rtos 12.3456 2 2)", &mut env).unwrap(), Value::Str("12.35".into()));
    assert_eq!(eval_str("(ascii \"A\")", &mut env).unwrap(), Value::Int(65));
    assert_eq!(eval_str("(chr 65)", &mut env).unwrap(), Value::Str("A".into()));
    assert_eq!(eval_str("(wcmatch \"WALL-100\" \"WALL-*\")", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(wcmatch \"DOOR-100\" \"WALL-*\")", &mut env).unwrap(), Value::Nil);

    // angtof & distof
    let ang = eval_str("(angtof \"180\")", &mut env).unwrap();
    assert_eq!(ang, Value::Real(std::f64::consts::PI));
    assert_eq!(eval_str("(distof \"123.45\")", &mut env).unwrap(), Value::Real(123.45));
}

#[test]
fn test_conformance_geometry() {
    let (_mock, mut env) = setup_env();

    // distance
    assert_eq!(eval_str("(distance '(0 0 0) '(3 4 0))", &mut env).unwrap(), Value::Real(5.0));
    // angle
    let ang = eval_str("(angle '(0 0 0) '(1 1 0))", &mut env).unwrap();
    assert_eq!(ang, Value::Real(std::f64::consts::FRAC_PI_4));
    // polar
    let pt = eval_str("(polar '(0 0 0) 0.0 10.0)", &mut env).unwrap();
    let pt_vec = pt.to_vec().unwrap();
    assert_eq!(pt_vec[0], Value::Real(10.0));
    assert_eq!(pt_vec[1], Value::Real(0.0));

    // inters: intersecting cross at (5, 5, 0)
    let isect = eval_str("(inters '(0 5 0) '(10 5 0) '(5 0 0) '(5 10 0))", &mut env).unwrap();
    assert_eq!(isect, Value::from_slice(&[Value::Real(5.0), Value::Real(5.0), Value::Real(0.0)]));
    // parallel lines
    assert_eq!(eval_str("(inters '(0 0 0) '(10 0 0) '(0 5 0) '(10 5 0))", &mut env).unwrap(), Value::Nil);
}

#[test]
fn test_conformance_visual_lisp_vl() {
    let (_mock, mut env) = setup_env();

    // vl-string-*
    assert_eq!(eval_str("(vl-string-search \"cad\" \"opencad\")", &mut env).unwrap(), Value::Int(4));
    assert_eq!(eval_str("(vl-string-subst \"CAD\" \"cad\" \"opencad\")", &mut env).unwrap(), Value::Str("openCAD".into()));
    assert_eq!(eval_str("(vl-string-trim \"*\" \"***hello***\")", &mut env).unwrap(), Value::Str("hello".into()));
    assert_eq!(eval_str("(vl-string-translate \"abc\" \"123\" \"a-b-c-d\")", &mut env).unwrap(), Value::Str("1-2-3-d".into()));

    // vl-list-*
    assert_eq!(eval_str("(vl-list-length '(1 2 3))", &mut env).unwrap(), Value::Int(3));
    assert_eq!(eval_str("(vl-sort '(3 1 4 1 5) '<)", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(1), Value::Int(3), Value::Int(4), Value::Int(5)]));
    assert_eq!(eval_str("(vl-sort-i '(30 10 20) '<)", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(2), Value::Int(0)]));
    assert_eq!(eval_str("(vl-remove 2 '(1 2 3 2 4))", &mut env).unwrap(), Value::from_slice(&[Value::Int(1), Value::Int(3), Value::Int(4)]));
    assert_eq!(eval_str("(vl-every 'numberp '(1 2 3))", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vl-every 'numberp '(1 \"two\" 3))", &mut env).unwrap(), Value::Nil);
    assert_eq!(eval_str("(vl-some 'numberp '(\"a\" 2 \"c\"))", &mut env).unwrap(), Value::T);

    // vl-member-if
    let m = eval_str("(vl-member-if 'numberp '(\"a\" \"b\" 42 \"c\"))", &mut env).unwrap();
    assert_eq!(m, Value::from_slice(&[Value::Int(42), Value::Str("c".into())]));

    // vl-filename-*
    assert_eq!(eval_str("(vl-filename-base \"C:/drawings/floorplan.dwg\")", &mut env).unwrap(), Value::Str("floorplan".into()));
    assert_eq!(eval_str("(vl-filename-extension \"C:/drawings/floorplan.dwg\")", &mut env).unwrap(), Value::Str(".dwg".into()));

    // vl-catch-all-*
    eval_str("(setq err (vl-catch-all-apply '/ '(10 0)))", &mut env).unwrap();
    assert_eq!(eval_str("(vl-catch-all-error-p err)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vl-catch-all-error-p \"not an error\")", &mut env).unwrap(), Value::Nil);
}

#[test]
fn test_conformance_cad_and_vlax() {
    let (_mock, mut env) = setup_env();

    // Database: entmake, entlast, entget, entmod, entupd, handent
    eval_str("(entmake '((0 . \"LINE\") (8 . \"WALL\") (10 0.0 0.0 0.0) (11 100.0 0.0 0.0)))", &mut env).unwrap();
    let last_e = eval_str("(setq e1 (entlast))", &mut env).unwrap();
    assert!(matches!(last_e, Value::Ename(_)));

    let elist = eval_str("(entget e1)", &mut env).unwrap();
    assert_eq!(eval_str("(cdr (assoc 8 (entget e1)))", &mut env).unwrap(), Value::Str("WALL".into()));

    eval_str("(entmod (subst '(8 . \"PARTITION\") '(8 . \"WALL\") (entget e1)))", &mut env).unwrap();
    assert_eq!(eval_str("(cdr (assoc 8 (entget e1)))", &mut env).unwrap(), Value::Str("PARTITION".into()));
    assert_eq!(eval_str("(entupd e1)", &mut env).unwrap(), last_e);

    // handent
    assert_eq!(eval_str("(handent \"1\")", &mut env).unwrap(), last_e);

    // Selection Sets
    eval_str("(entmake '((0 . \"LINE\") (8 . \"WALL\")))", &mut env).unwrap();
    let ss = eval_str("(setq ss1 (ssget \"X\"))", &mut env).unwrap();
    assert!(matches!(ss, Value::PickSet(_)));
    assert_eq!(eval_str("(sslength ss1)", &mut env).unwrap(), Value::Int(2));
    assert_eq!(eval_str("(ssname ss1 0)", &mut env).unwrap(), last_e);
    assert_eq!(eval_str("(ssmemb e1 ss1)", &mut env).unwrap(), Value::T);

    // Interactive inputs
    let pt = eval_str("(getpoint \"Pick a point: \")", &mut env).unwrap();
    assert_eq!(pt, Value::from_slice(&[Value::Real(10.0), Value::Real(20.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(getdist '(0 0 0) \"Pick dist: \")", &mut env).unwrap(), Value::Real(50.0));
    assert_eq!(eval_str("(getcorner '(0 0 0) \"Corner: \")", &mut env).unwrap(), Value::from_slice(&[Value::Real(10.0), Value::Real(20.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(getint \"Enter count: \")", &mut env).unwrap(), Value::Int(42));
    assert_eq!(eval_str("(getreal \"Enter float: \")", &mut env).unwrap(), Value::Real(3.14159));
    assert_eq!(eval_str("(getstring \"Enter text: \")", &mut env).unwrap(), Value::Str("TEST_STRING".into()));
    assert_eq!(eval_str("(getkword \"Option [Yes/No]: \")", &mut env).unwrap(), Value::Str("YES".into()));
    assert_eq!(eval_str("(initget 1 \"Yes No\")", &mut env).unwrap(), Value::Nil);
    assert_eq!(eval_str("(getorient '(0 0 0) \"Direction: \")", &mut env).unwrap(), Value::Real(1.1071487177940905));

    // VLAX ActiveX emulation
    eval_str("(setq vla (vlax-ename->vla-object e1))", &mut env).unwrap();
    assert_eq!(eval_str("(vlax-vla-object->ename vla)", &mut env).unwrap(), last_e);
    assert_eq!(eval_str("(vlax-get-property vla 'Layer)", &mut env).unwrap(), Value::Str("PARTITION".into()));
    assert_eq!(eval_str("(vlax-get vla 'Layer)", &mut env).unwrap(), Value::Str("PARTITION".into()));
    eval_str("(vlax-put-property vla 'Layer \"EXTERIOR\")", &mut env).unwrap();
    assert_eq!(eval_str("(vlax-get vla 'Layer)", &mut env).unwrap(), Value::Str("EXTERIOR".into()));
    assert_eq!(eval_str("(vlax-method-applicable-p vla 'Delete)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vlax-property-available-p vla 'Layer)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vlax-variant-value (vlax-make-variant 42))", &mut env).unwrap(), Value::Int(42));

    // VLAX Curves
    assert_eq!(eval_str("(vlax-curve-getstartpoint e1)", &mut env).unwrap(), Value::from_slice(&[Value::Real(0.0), Value::Real(0.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getendpoint e1)", &mut env).unwrap(), Value::from_slice(&[Value::Real(100.0), Value::Real(0.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getdistatparam e1 0.25)", &mut env).unwrap(), Value::Real(25.0));
    assert_eq!(eval_str("(vlax-curve-getparamatdist e1 75.0)", &mut env).unwrap(), Value::Real(0.75));

    // VLAX LData
    eval_str("(vlax-ldata-put \"PROJECT\" \"REV\" \"1.0.4\")", &mut env).unwrap();
    assert_eq!(eval_str("(vlax-ldata-get \"PROJECT\" \"REV\")", &mut env).unwrap(), Value::Str("1.0.4".into()));
    let ldata_list = eval_str("(vlax-ldata-list \"PROJECT\")", &mut env).unwrap();
    assert_eq!(ldata_list, Value::from_slice(&[Value::cons(Value::Str("REV".into()), Value::Str("1.0.4".into()))]));
    assert_eq!(eval_str("(vlax-ldata-delete \"PROJECT\" \"REV\")", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vlax-ldata-get \"PROJECT\" \"REV\")", &mut env).unwrap(), Value::Nil);

    // ── Clipboard Operations (get_clipboard, set_clipboard, dos_clipboard) ──
    assert_eq!(eval_str("(set_clipboard \"OpenCAD Clipboard Text 123\")", &mut env).unwrap(), Value::Str("OpenCAD Clipboard Text 123".into()));
    assert_eq!(eval_str("(get_clipboard)", &mut env).unwrap(), Value::Str("OpenCAD Clipboard Text 123".into()));

    // Polymorphic dos_clipboard
    assert_eq!(eval_str("(dos_clipboard \"DOSLib Polymo Copy\")", &mut env).unwrap(), Value::Str("DOSLib Polymo Copy".into()));
    assert_eq!(eval_str("(dos_clipboard)", &mut env).unwrap(), Value::Str("DOSLib Polymo Copy".into()));
    assert_eq!(eval_str("(get_clipboard)", &mut env).unwrap(), Value::Str("DOSLib Polymo Copy".into()));
}

#[test]
fn test_table_and_curve_features() {
    let (mock, mut env) = setup_env();

    // 1. Symbol table search (tblsearch) for all standard tables
    assert!(eval_str("(tblsearch \"LAYER\" \"0\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"LTYPE\" \"CONTINUOUS\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"STYLE\" \"STANDARD\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"BLOCK\" \"*MODEL_SPACE\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"DIMSTYLE\" \"STANDARD\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"APPID\" \"ACAD\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"VIEW\" \"TOP\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"VPORT\" \"*ACTIVE\")", &mut env).unwrap() != Value::Nil);
    assert!(eval_str("(tblsearch \"UCS\" \"WORLD\")", &mut env).unwrap() != Value::Nil);
    assert_eq!(eval_str("(tblsearch \"LAYER\" \"NON_EXISTENT\")", &mut env).unwrap(), Value::Nil);

    // 2. Symbol table next (tblnext)
    let next_layer = eval_str("(tblnext \"LAYER\" T)", &mut env).unwrap();
    assert!(next_layer != Value::Nil);
    let next_layer2 = eval_str("(tblnext \"LAYER\")", &mut env).unwrap();
    assert!(next_layer2 != Value::Nil);

    // 3. Selection (entsel, nentsel)
    eval_str("(entmake '((0 . \"LINE\") (8 . \"0\") (10 0.0 0.0 0.0) (11 50.0 0.0 0.0)))", &mut env).unwrap();
    let es = eval_str("(entsel \"Pick: \")", &mut env).unwrap();
    assert!(matches!(es, Value::Cons(_)));
    let nes = eval_str("(nentsel)", &mut env).unwrap();
    assert!(matches!(nes, Value::Cons(_)));

    // 4. Undo Marks
    assert_eq!(mock.lock().unwrap().undo_marks, 0);
    eval_str("(vla-StartUndoMark (vla-get-ActiveDocument (vlax-get-acad-object)))", &mut env).unwrap();
    assert_eq!(mock.lock().unwrap().undo_marks, 1);
    eval_str("(vla-EndUndoMark (vla-get-ActiveDocument (vlax-get-acad-object)))", &mut env).unwrap();
    assert_eq!(mock.lock().unwrap().undo_marks, 0);

    // 5. Lightweight Polyline Curve Geometry & Coordinates
    eval_str(r#"(entmake '(
        (0 . "LWPOLYLINE")
        (100 . "AcDbEntity")
        (100 . "AcDbPolyline")
        (90 . 4)
        (70 . 1)
        (10 0.0 0.0 0.0)
        (10 100.0 0.0 0.0)
        (10 100.0 50.0 0.0)
        (10 0.0 50.0 0.0)
    ))"#, &mut env).unwrap();
    eval_str("(setq pline (entlast))", &mut env).unwrap();

    let coords = eval_str("(vlax-safearray->list (vlax-variant-value (vla-get-Coordinates (vlax-ename->vla-object pline))))", &mut env).unwrap();
    assert_eq!(coords, Value::from_slice(&[
        Value::Real(0.0), Value::Real(0.0),
        Value::Real(100.0), Value::Real(0.0),
        Value::Real(100.0), Value::Real(50.0),
        Value::Real(0.0), Value::Real(50.0),
    ]));

    assert_eq!(eval_str("(vlax-curve-isClosed pline)", &mut env).unwrap(), Value::T);
    assert_eq!(eval_str("(vlax-curve-getEndParam pline)", &mut env).unwrap(), Value::Real(4.0));
    assert_eq!(eval_str("(vlax-curve-getPointAtParam pline 0.5)", &mut env).unwrap(), Value::from_slice(&[Value::Real(50.0), Value::Real(0.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getPointAtParam pline 1.5)", &mut env).unwrap(), Value::from_slice(&[Value::Real(100.0), Value::Real(25.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getFirstDeriv pline 0.5)", &mut env).unwrap(), Value::from_slice(&[Value::Real(1.0), Value::Real(0.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getClosestPointTo pline '(50.0 10.0 0.0))", &mut env).unwrap(), Value::from_slice(&[Value::Real(50.0), Value::Real(0.0), Value::Real(0.0)]));
    assert_eq!(eval_str("(vlax-curve-getDistAtParam pline 4.0)", &mut env).unwrap(), Value::Real(300.0));
    assert_eq!(eval_str("(vlax-curve-getPointAtDist pline 150.0)", &mut env).unwrap(), Value::from_slice(&[Value::Real(100.0), Value::Real(50.0), Value::Real(0.0)]));

    // 6. Special form LET
    let let_res = eval_str("(let ((x 10) (y 25)) (+ x y))", &mut env).unwrap();
    assert_eq!(let_res, Value::Int(35));
}
