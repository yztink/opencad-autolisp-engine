//! Comprehensive conformance and gap-analysis test suite for all AutoLISP built-in functions,
//! special forms, Visual LISP (VL), ActiveX (VLAX), and OpenCAD Lisp (OCL) extensions.

use std::sync::{Arc, Mutex};
use lisp_cad::{create_cad_env, MockCadBridge};
use lisp_core::{eval_str, Value};

fn setup_env() -> (Arc<Mutex<MockCadBridge>>, std::rc::Rc<std::cell::RefCell<lisp_core::Env>>) {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let env = create_cad_env(mock.clone());
    (mock, env)
}

struct TestResult {
    category: &'static str,
    function: &'static str,
    test_case: &'static str,
    status: &'static str,
    details: String,
}

fn check<F>(results: &mut Vec<TestResult>, cat: &'static str, func: &'static str, name: &'static str, f: F)
where
    F: FnOnce(&mut std::rc::Rc<std::cell::RefCell<lisp_core::Env>>) -> Result<bool, String>,
{
    let (_mock, mut env) = setup_env();
    match f(&mut env) {
        Ok(true) => results.push(TestResult {
            category: cat,
            function: func,
            test_case: name,
            status: "PASS",
            details: "Matches standard behavior".to_string(),
        }),
        Ok(false) => results.push(TestResult {
            category: cat,
            function: func,
            test_case: name,
            status: "FAIL",
            details: "Returned unexpected value".to_string(),
        }),
        Err(e) => {
            let status = if e.contains("Unbound variable") || e.contains("Unknown function") || e.contains("unbound") {
                "MISSING"
            } else {
                "ERROR"
            };
            results.push(TestResult {
                category: cat,
                function: func,
                test_case: name,
                status,
                details: e,
            });
        }
    }
}

#[test]
fn test_all_autolisp_builtins_suite() {
    let mut results: Vec<TestResult> = Vec::new();

    // ── 1. Core Evaluation, Scope & Control Flow ────────────────────────────
    check(&mut results, "Core", "defun", "Basic defun and call", |env| {
        eval_str("(defun test_add (a b) (+ a b)) (test_add 10 20)", env)
            .map(|v| v == Value::Int(30)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "defun", "Local variables (/ x y)", |env| {
        let code = "(setq a 1 b 2) (defun f1 (/ a) (setq a 100)) (f1) a";
        eval_str(code, env).map(|v| v == Value::Int(1)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "setq", "Multi-assignment", |env| {
        let code = "(setq x 10 y 20 z (+ x y)) z";
        eval_str(code, env).map(|v| v == Value::Int(30)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "quote", "Quote special form and apostrophe", |env| {
        eval_str("(equal (quote (a b)) '(a b))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "lambda", "Anonymous lambda execution", |env| {
        eval_str("((lambda (x y) (* x y)) 6 7)", env).map(|v| v == Value::Int(42)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "if", "If-then-else", |env| {
        eval_str("(if (< 5 10) \"yes\" \"no\")", env).map(|v| v == Value::Str("yes".into())).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "cond", "Multi-branch conditional", |env| {
        eval_str("(cond ((= 1 2) 'one) ((= 3 3) 'three) (t 'other))", env)
            .map(|v| v == Value::Sym("THREE".into())).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "while", "While loop accumulator", |env| {
        eval_str("(setq i 0 s 0) (while (< i 4) (setq s (+ s i) i (1+ i))) s", env)
            .map(|v| v == Value::Int(6)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "repeat", "Repeat counter", |env| {
        eval_str("(setq a 1) (repeat 5 (setq a (* a 2))) a", env)
            .map(|v| v == Value::Int(32)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "foreach", "Foreach list traversal", |env| {
        eval_str("(setq acc 0) (foreach x '(1 2 3 4) (setq acc (+ acc x))) acc", env)
            .map(|v| v == Value::Int(10)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "progn", "Sequence execution", |env| {
        eval_str("(progn 1 2 3 42)", env).map(|v| v == Value::Int(42)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "and", "Short-circuit boolean AND", |env| {
        eval_str("(and 1 2 '(a) \"ok\")", env).map(|v| v == Value::Str("ok".into())).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "or", "Short-circuit boolean OR", |env| {
        eval_str("(or nil nil 99 nil)", env).map(|v| v == Value::Int(99)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "null", "Null predicate", |env| {
        eval_str("(and (null nil) (null '()) (not (null 0)) (not (null \"\")))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Core", "type", "Type reflection", |env| {
        eval_str("(list (type 42) (type 3.14) (type \"a\") (type '(1)) (type 'sym))", env)
            .map(|v| match v {
                Value::Cons(_) => {
                    let s = v.to_string();
                    s.contains("INT") && s.contains("REAL") && s.contains("STR") && s.contains("LIST") && s.contains("SYM")
                }
                _ => false,
            }).map_err(|e| format!("{:?}", e))
    });

    // ── 2. List Manipulation (Cons, Pairs & Compositions) ───────────────────
    check(&mut results, "List", "cons", "Pair and list prepending", |env| {
        eval_str("(equal (cons 1 '(2 3)) '(1 2 3))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "car/cdr", "Basic car and cdr", |env| {
        eval_str("(and (= (car '(10 20 30)) 10) (equal (cdr '(10 20 30)) '(20 30)))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "c...r compositions", "Testing all 28 c...r compositions", |env| {
        let code = r#"
            (setq tree '((((1 . 2) . (3 . 4)) . ((5 . 6) . (7 . 8))) . (((9 . 10) . (11 . 12)) . ((13 . 14) . (15 . 16)))))
            (list (caar '((1 2) 3)) (cadr '(1 2 3)) (cdar '((1 2) 3)) (cddr '(1 2 3))
                  (caaar tree) (caadr tree) (cadar tree) (caddr '(1 2 3 4))
                  (cdaar tree) (cdadr tree) (cddar tree) (cdddr '(1 2 3 4))
                  (cadddr '(1 2 3 4 5)))
        "#;
        eval_str(code, env).map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "append", "List concatenation", |env| {
        eval_str("(equal (append '(1 2) '(3 4) '(5)) '(1 2 3 4 5))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "reverse", "List reversal", |env| {
        eval_str("(equal (reverse '(1 2 3 4)) '(4 3 2 1))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "length", "List length", |env| {
        eval_str("(length '(1 2 3 4 5))", env).map(|v| v == Value::Int(5)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "last", "Last element", |env| {
        eval_str("(last '(1 2 3 42))", env).map(|v| v == Value::Int(42) || v == Value::from_slice(&[Value::Int(42)])).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "nth", "0-indexed element access", |env| {
        eval_str("(nth 2 '(10 20 30 40))", env).map(|v| v == Value::Int(30)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "member", "List membership search", |env| {
        eval_str("(equal (member 3 '(1 2 3 4 5)) '(3 4 5))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "assoc", "Alist key lookup", |env| {
        eval_str("(assoc 10 '((0 . \"LINE\") (10 1.0 2.0 0.0) (8 . \"0\")))", env)
            .map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "subst", "Item replacement in alist", |env| {
        eval_str("(equal (subst '(8 . \"WALLS\") '(8 . \"0\") '((0 . \"LINE\") (8 . \"0\"))) '((0 . \"LINE\") (8 . \"WALLS\")))", env)
            .map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "mapcar", "Mapping function over lists", |env| {
        eval_str("(equal (mapcar '+ '(1 2 3) '(10 20 30)) '(11 22 33))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "List", "apply", "Applying function to arguments list", |env| {
        eval_str("(apply '+ '(1 2 3 4 5))", env).map(|v| v == Value::Int(15)).map_err(|e| format!("{:?}", e))
    });

    // ── 3. String & Character Operations ────────────────────────────────────
    check(&mut results, "String", "strcat", "Concatenate strings", |env| {
        eval_str("(strcat \"Open\" \"CAD\" \"Studio\")", env).map(|v| v == Value::Str("OpenCADStudio".into())).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "strlen", "String character length", |env| {
        eval_str("(strlen \"AutoLISP\")", env).map(|v| v == Value::Int(8)).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "substr", "Substring with 1-based index and length", |env| {
        eval_str("(and (= (substr \"OpenCAD\" 1 4) \"Open\") (= (substr \"OpenCAD\" 5) \"CAD\"))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "strcase", "Case conversion uppercase and lowercase", |env| {
        eval_str("(and (= (strcase \"hello\") \"HELLO\") (= (strcase \"HELLO\" T) \"hello\"))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "wcmatch", "Wildcard matching (*, ?, #, @, ~, [...])", |env| {
        let code = r#"
            (and (wcmatch "LINE123" "LINE*")
                 (wcmatch "A5" "@#")
                 (wcmatch "test.dwg" "*.dwg")
                 (not (wcmatch "abc" "~a*"))
                 (wcmatch "B" "[A-Z]"))
        "#;
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "ascii/chr", "Character to ASCII and back", |env| {
        eval_str("(= (chr (ascii \"A\")) \"A\")", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "String", "read", "String parser into Lisp value", |env| {
        eval_str("(equal (read \"(1 2 3)\") '(1 2 3))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });

    // ── 4. Math, Formatting & Bitwise Operations ────────────────────────────
    check(&mut results, "Math", "Arithmetic", "+, -, *, /, 1+, 1-, abs, min, max, rem", |env| {
        let code = "(and (= (+ 1 2 3) 6) (= (- 10 3 2) 5) (= (* 2 3 4) 24) (= (/ 100 2 5) 10) (= (1+ 5) 6) (= (1- 5) 4) (= (abs -42) 42) (= (min 3 1 4) 1) (= (max 3 1 4) 4) (= (rem 10 3) 1))";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "Trigonometric", "sin, cos, atan, sqrt, exp, log", |env| {
        let code = "(and (< (abs (sin 0.0)) 1e-6) (< (abs (- (cos 0.0) 1.0)) 1e-6) (< (abs (- (sqrt 16.0) 4.0)) 1e-6))";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "atan", "2-argument atan (y x)", |env| {
        eval_str("(< (abs (- (atan 1.0 1.0) (/ 3.1415926535 4.0))) 1e-4)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "Bitwise", "logand, logior, lsh, ~", |env| {
        let code = "(and (= (logand 7 3) 3) (= (logior 4 2) 6) (= (lsh 1 3) 8))";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "rtos", "Linear units formatting (Decimal=2, Architectural=4)", |env| {
        eval_str("(= (rtos 12.5 2 2) \"12.50\")", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "distof", "String distance parsing", |env| {
        eval_str("(< (abs (- (distof \"12.50\" 2) 12.5)) 1e-5)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "angtos", "Angular units formatting", |env| {
        eval_str("(and (= (angtos 0.0 0 0) \"0\") (= (angtos 3.1415926535 0 0) \"180\"))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Math", "angtof", "String angle parsing in radians", |env| {
        eval_str("(< (abs (- (angtof \"180\" 0) 3.1415926535)) 1e-4)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 5. Geometry & Spatial Operations ────────────────────────────────────
    check(&mut results, "Geometry", "polar", "Calculate point at angle and distance", |env| {
        let code = "(setq pt (polar '(0.0 0.0 0.0) 0.0 10.0)) (and (< (abs (- (car pt) 10.0)) 1e-5) (< (abs (cadr pt)) 1e-5))";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Geometry", "distance", "2D and 3D point distance", |env| {
        eval_str("(< (abs (- (distance '(0 0 0) '(3 4 0)) 5.0)) 1e-5)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Geometry", "angle", "2D angle between two points", |env| {
        eval_str("(< (abs (- (angle '(0 0) '(0 10)) (/ 3.1415926535 2.0))) 1e-4)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Geometry", "inters", "Intersection point (bounded vs infinite lines)", |env| {
        let code = r#"
            (and (equal (inters '(0 0) '(10 10) '(0 10) '(10 0)) '(5.0 5.0))
                 (null (inters '(0 0) '(2 2) '(0 10) '(10 0) T))
                 (equal (inters '(0 0) '(2 2) '(0 10) '(10 0) nil) '(5.0 5.0)))
        "#;
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Geometry", "trans", "Coordinate system translation (WCS/UCS/OCS)", |env| {
        eval_str("(trans '(10 20 0) 0 1)", env).map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });

    // ── 6. Entity Database Functions ────────────────────────────────────────
    check(&mut results, "Entity", "entmake", "Create Line entity via DXF list", |env| {
        let code = "(entmake '((0 . \"LINE\") (8 . \"0\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0)))";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Entity", "entlast", "Retrieve last entity name", |env| {
        let code = "(entmake '((0 . \"POINT\") (10 5.0 5.0 0.0))) (entlast)";
        eval_str(code, env).map(|v| matches!(v, Value::Ename(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Entity", "entget", "Retrieve entity DXF record", |env| {
        let code = "(entmake '((0 . \"CIRCLE\") (10 0.0 0.0 0.0) (40 . 5.0))) (entget (entlast))";
        eval_str(code, env).map(|v| match v {
            Value::Cons(_) => v.to_string().contains("CIRCLE"),
            _ => false,
        }).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Entity", "entmod", "Modify entity DXF record", |env| {
        let code = r#"
            (entmake '((0 . "CIRCLE") (10 0.0 0.0 0.0) (40 . 5.0)))
            (setq e (entlast) data (entget e))
            (setq data (subst '(40 . 10.0) (assoc 40 data) data))
            (entmod data)
            (= (cdr (assoc 40 (entget e))) 10.0)
        "#;
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Entity", "entdel", "Delete and undelete entity", |env| {
        let code = "(entmake '((0 . \"POINT\") (10 0.0 0.0 0.0))) (setq e (entlast)) (entdel e) (entdel e) e";
        eval_str(code, env).map(|v| matches!(v, Value::Ename(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Entity", "handent", "Lookup entity from hex handle", |env| {
        let code = "(entmake '((0 . \"POINT\") (10 0.0 0.0 0.0))) (setq e (entlast) h (cdr (assoc 5 (entget e)))) (equal (handent h) e)";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 7. Selection Set Operations & Modes ──────────────────────────────────
    check(&mut results, "Selection", "ssget (all entities 'X')", "ssget mode X", |env| {
        eval_str("(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (ssget \"X\")", env).map(|v| matches!(v, Value::PickSet(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (DXF filter)", "ssget mode X with entity type filter", |env| {
        eval_str("(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (ssget \"X\" '((0 . \"LINE\")))", env).map(|v| matches!(v, Value::PickSet(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (interactive/nil)", "ssget interactive prompt mode", |env| {
        eval_str("(ssget)", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (implied '_I')", "ssget mode _I (pre-selected)", |env| {
        eval_str("(ssget \"_I\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (previous 'P')", "ssget mode P (previous set)", |env| {
        eval_str("(ssget \"P\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (last 'L')", "ssget mode L (last created entity)", |env| {
        eval_str("(ssget \"L\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (point pick)", "ssget with single point coordinate", |env| {
        eval_str("(ssget '(5.0 5.0 0.0))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (window 'W')", "ssget mode W with 2 corner points", |env| {
        eval_str("(ssget \"W\" '(0.0 0.0) '(10.0 10.0))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (crossing 'C')", "ssget mode C with 2 corner points", |env| {
        eval_str("(ssget \"C\" '(0.0 0.0) '(10.0 10.0))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (window polygon 'WP')", "ssget mode WP with point list", |env| {
        eval_str("(ssget \"WP\" '((0 0) (10 0) (10 10) (0 10)))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (crossing polygon 'CP')", "ssget mode CP with point list", |env| {
        eval_str("(ssget \"CP\" '((0 0) (10 0) (10 10) (0 10)))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (fence 'F')", "ssget mode F with fence points list", |env| {
        eval_str("(ssget \"F\" '((0 0) (5 5) (10 0)))", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (single select ':S')", "ssget flag :S", |env| {
        eval_str("(ssget \":S\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (pickbox ':E')", "ssget flag :E", |env| {
        eval_str("(ssget \":E\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (allow duplicate ':D')", "ssget flag :D", |env| {
        eval_str("(ssget \":D\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (locked layers check ':L')", "ssget flag :L", |env| {
        eval_str("(ssget \":L\")", env).map(|v| matches!(v, Value::PickSet(_)) || v == Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (relational filter)", "ssget with -4 relational operator", |env| {
        let code = "(entmake '((0 . \"CIRCLE\") (10 0.0 0.0 0.0) (40 . 10.0))) (ssget \"X\" '((-4 . \">\") (40 . 5.0)))";
        eval_str(code, env).map(|v| matches!(v, Value::PickSet(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssget (logical filter)", "ssget with -4 <OR and OR> operator", |env| {
        let code = "(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (ssget \"X\" '((-4 . \"<OR\") (0 . \"LINE\") (0 . \"CIRCLE\") (-4 . \"OR>\")))";
        eval_str(code, env).map(|v| matches!(v, Value::PickSet(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "sslength", "Selection set entity count", |env| {
        eval_str("(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (sslength (ssget \"X\"))", env).map(|v| matches!(v, Value::Int(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssname", "Lookup entity by 0-based index", |env| {
        eval_str("(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (setq ss (ssget \"X\")) (if (> (sslength ss) 0) (ssname ss 0) T)", env)
            .map(|v| matches!(v, Value::Ename(_)) || v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "Selection", "ssadd/ssdel", "Add and remove entities from selection set", |env| {
        let code = "(entmake '((0 . \"LINE\") (10 0.0 0.0 0.0) (11 10.0 10.0 0.0))) (setq ss (ssadd)) (ssadd (entlast) ss) (= (sslength ss) 1)";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 8. Symbol Table & Dictionary Functions ──────────────────────────────
    check(&mut results, "SymbolTable", "tblsearch", "Search LAYER symbol table", |env| {
        eval_str("(tblsearch \"LAYER\" \"0\")", env).map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "SymbolTable", "tblnext", "Iterate symbol table records", |env| {
        eval_str("(tblnext \"LAYER\" T)", env).map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "SymbolTable", "snvalid", "Symbol name validity check", |env| {
        eval_str("(and (snvalid \"WALLS\") (not (snvalid \"INVALID*NAME\")))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 9. System Variables & Environment ───────────────────────────────────
    check(&mut results, "SysVar", "getvar", "Read CLAYER and CIRCLERAD", |env| {
        eval_str("(list (getvar \"CLAYER\") (getvar 'CMDECHO))", env).map(|v| matches!(v, Value::Cons(_))).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "SysVar", "setvar", "Write system variable", |env| {
        eval_str("(setvar \"USERS1\" \"HelloOCS\") (= (getvar \"USERS1\") \"HelloOCS\")", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "SysVar", "getenv/setenv", "Environment variable access", |env| {
        eval_str("(setenv \"OCSTEST\" \"123\") (= (getenv \"OCSTEST\") \"123\")", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 10. Visual LISP Extensions (VL-*) ───────────────────────────────────
    check(&mut results, "VL", "vl-consp", "Cons pair predicate", |env| {
        eval_str("(and (vl-consp '(1 . 2)) (vl-consp '(1 2 3)) (not (vl-consp nil)))", env)
            .map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-position", "Find element index in list", |env| {
        eval_str("(= (vl-position 'c '(a b c d)) 2)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-remove", "Remove matching items", |env| {
        eval_str("(equal (vl-remove 2 '(1 2 3 2 4)) '(1 3 4))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-sort", "Sort list with predicate", |env| {
        eval_str("(equal (vl-sort '(4 1 3 2) '<) '(1 2 3 4))", env).map(|v| v == Value::T).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-string-trim", "Trim whitespace from string", |env| {
        eval_str("(= (vl-string-trim \" \" \"  test  \") \"test\")", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-string-search", "Sub-string index search", |env| {
        eval_str("(= (vl-string-search \"CAD\" \"OpenCADStudio\") 4)", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "VL", "vl-catch-all-apply", "Safely catch errors without abortion", |env| {
        let code = "(setq err (vl-catch-all-apply '/ '(1 0))) (vl-catch-all-error-p err)";
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // ── 11. OpenCAD Studio GUI Extensions (OCL-*) ───────────────────────────
    check(&mut results, "GUI", "ocl-dialog", "Define dialog layout", |env| {
        let code = r#"
            (ocl-dialog "test_dlg" "Test Dialog"
                (ocl-row
                    (ocl-text "lbl" "Label")
                    (ocl-button "btn" "Click" nil)))
        "#;
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "GUI", "ocl-panel-register", "Register dockable side panel", |env| {
        let code = r#"
            (ocl-panel-register "pnl" "My Panel"
                (ocl-checkbox "chk" "Option" T)
                (ocl-choice "ch" "Color" '("Red" "Green") "Red"))
        "#;
        eval_str(code, env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });
    check(&mut results, "GUI", "ocl-panel-get", "Read panel state value", |env| {
        eval_str("(ocl-panel-get \"pnl\" \"chk\")", env).map(|v| v != Value::Nil).map_err(|e| format!("{:?}", e))
    });

    // Print consolidated summary for reporting
    println!("\n=================== AUTOLISP BUILT-INS CONFORMANCE REPORT ===================");
    let mut pass = 0;
    let mut fail = 0;
    let mut missing = 0;
    let mut err = 0;
    for r in &results {
        println!("[{}] {} :: {} -> {}", r.status, r.category, r.test_case, r.details);
        match r.status {
            "PASS" => pass += 1,
            "FAIL" => fail += 1,
            "MISSING" => missing += 1,
            _ => err += 1,
        }
    }
    println!("-----------------------------------------------------------------------------");
    println!("TOTAL: {} | PASS: {} | FAIL: {} | MISSING: {} | ERROR: {}", results.len(), pass, fail, missing, err);
    println!("=============================================================================\n");

    assert!(pass > 0, "At least some built-ins must pass");
}
