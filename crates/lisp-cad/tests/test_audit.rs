use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use lisp_cad::{create_cad_env, MockCadBridge};
use lisp_core::{eval_str, Value};

#[test]
fn test_audit_function_inventory() {
    let mock = Arc::new(Mutex::new(MockCadBridge::new()));
    let env = create_cad_env(mock);

    // List of standard special forms evaluated directly by evaluator (eval.rs)
    let special_forms: HashSet<&'static str> = [
        "QUOTE", "SETQ", "DEFUN", "LAMBDA", "IF", "COND",
        "WHILE", "REPEAT", "FOREACH", "PROGN", "AND", "OR",
        "EVAL", "SET", "BOUNDP",
    ].into_iter().collect();

    let bindings: HashSet<String> = env.borrow().bindings.keys().map(|k| k.as_str().to_ascii_uppercase()).collect();

    // Standard catalog of functions grouped by category
    let catalog: &[(&str, &[&str])] = &[
        ("Special Forms & Flow Control", &[
            "QUOTE", "SETQ", "DEFUN", "LAMBDA", "IF", "COND", "WHILE", "REPEAT", "FOREACH", "PROGN", "AND", "OR"
        ]),
        ("Core Execution & Evaluation", &[
            "EVAL", "APPLY", "MAPCAR", "SET", "BOUNDP", "NULL", "NOT", "TYPE"
        ]),
        ("Math, Trigonometry & Bitwise", &[
            "+", "-", "*", "/", "1+", "1-", "ABS", "MAX", "MIN", "REM", "SQRT", "EXPT",
            "SIN", "COS", "ATAN", "LOG", "EXP", "FLOAT", "FIX", "GCD",
            "LOGAND", "LOGIOR", "BOOLE", "LSH", "~"
        ]),
        ("Predicates & Logic", &[
            "=", "/=", "<", "<=", ">", ">=", "EQ", "EQUAL", "ATOM", "LISTP", "NUMBERP", "MINUSP", "ZEROP"
        ]),
        ("List Manipulation", &[
            "CAR", "CDR", "CAAR", "CADR", "CDAR", "CDDR", "CAAAR", "CAADR", "CADAR", "CADDR",
            "CDAAR", "CDADR", "CDDAR", "CDDDR",
            "CAAAAR", "CAAADR", "CAADAR", "CAADDR", "CADAAR", "CADADR", "CADDAR", "CADDDR",
            "CDAAAR", "CDAADR", "CDADAR", "CDADDR", "CDDAAR", "CDDADR", "CDDDAR", "CDDDDR",
            "CONS", "LIST", "APPEND", "REVERSE", "LENGTH", "NTH", "LAST", "ASSOC", "MEMBER", "SUBST"
        ]),
        ("String & Character Handling", &[
            "STRCAT", "STRLEN", "SUBSTR", "STRCASE", "ITOA", "ATOI", "ATOF", "RTOS",
            "ANGTOS", "ANGTOF", "DISTOF", "ASCII", "CHR", "READ", "WCMATCH"
        ]),
        ("Geometry & Coordinate Math", &[
            "DISTANCE", "ANGLE", "POLAR", "INTERS"
        ]),
        ("Console & File I/O", &[
            "PRINC", "PRIN1", "PRINT", "OPEN", "CLOSE", "READ-LINE", "WRITE-LINE", "READ-CHAR", "WRITE-CHAR",
            "FINDFILE", "LOAD", "GETENV", "SETENV"
        ]),
        ("CAD Database & Entity Management", &[
            "ENTGET", "ENTMOD", "ENTMAKE", "ENTMAKEX", "ENTDEL", "ENTNEXT", "ENTLAST", "ENTUPD", "HANDENT"
        ]),
        ("Selection Sets", &[
            "SSGET", "SSLENGTH", "SSNAME", "SSADD", "SSDEL", "SSMEMB"
        ]),
        ("Interactive User Input", &[
            "GETPOINT", "GETCORNER", "GETDIST", "GETANGLE", "GETORIENT", "GETSTRING", "GETINT", "GETREAL", "GETKWORD", "INITGET"
        ]),
        ("CAD Command & System", &[
            "COMMAND", "ALERT", "VER", "VL-VERSION"
        ]),
        ("Visual LISP Extensions (vl-*)", &[
            "VL-LOAD-COM", "VL-STRING-TRIM", "VL-STRING-LEFT-TRIM", "VL-STRING-RIGHT-TRIM", "VL-STRING-SEARCH",
            "VL-STRING-SUBST", "VL-STRING-TRANSLATE", "VL-STRING->LIST", "VL-LIST->STRING",
            "VL-LIST-EXPORT", "VL-LIST-LENGTH", "VL-SORT", "VL-SORT-I", "VL-EVERY", "VL-SOME",
            "VL-MEMBER-IF", "VL-MEMBER-IF-NOT", "VL-REMOVE", "VL-REMOVE-IF", "VL-REMOVE-IF-NOT",
            "VL-CONSP", "VL-POSITION", "VL-CATCH-ALL-APPLY", "VL-CATCH-ALL-ERROR-P", "VL-CATCH-ALL-ERROR-MESSAGE",
            "VL-FILENAME-BASE", "VL-FILENAME-DIRECTORY", "VL-FILENAME-EXTENSION",
            "VL-FILE-COPY", "VL-FILE-DELETE", "VL-FILE-RENAME", "VL-FILE-SIZE", "VL-FILE-SYSTIME", "VL-DIRECTORY-FILES"
        ]),
        ("ActiveX & Object Extensions (vlax-*)", &[
            "VLAX-ENAME->VLA-OBJECT", "VLAX-VLA-OBJECT->ENAME", "VLAX-GET-PROPERTY", "VLAX-PUT-PROPERTY",
            "VLAX-GET", "VLAX-PUT", "VLAX-INVOKE", "VLAX-INVOKE-METHOD",
            "VLAX-LDATA-GET", "VLAX-LDATA-PUT", "VLAX-LDATA-DELETE", "VLAX-LDATA-LIST",
            "VLAX-METHOD-APPLICABLE-P", "VLAX-PROPERTY-AVAILABLE-P",
            "VLAX-VARIANT-VALUE", "VLAX-MAKE-VARIANT", "VLAX-SAFEARRAY-VALUE", "VLAX-MAKE-SAFEARRAY",
            "VLAX-CURVE-GETSTARTPOINT", "VLAX-CURVE-GETENDPOINT", "VLAX-CURVE-GETDISTATPARAM", "VLAX-CURVE-GETPARAMATDIST"
        ]),
    ];

    println!("\n========================================================");
    println!("     AUTOLISP & VISUAL LISP CONFORMANCE AUDIT REPORT     ");
    println!("========================================================");

    let mut total_funcs = 0;
    let mut implemented_count = 0;
    let mut missing_by_cat: Vec<(&str, Vec<&str>)> = Vec::new();

    for (cat_name, funcs) in catalog {
        let mut cat_missing = Vec::new();
        let mut cat_present = 0;

        for f in *funcs {
            total_funcs += 1;
            let upper = f.to_ascii_uppercase();
            if special_forms.contains(upper.as_str()) || bindings.contains(&upper) {
                cat_present += 1;
                implemented_count += 1;
            } else {
                cat_missing.push(*f);
            }
        }

        println!("[{}] {} / {} supported ({:.1}%)", cat_name, cat_present, funcs.len(), (cat_present as f64 / funcs.len() as f64) * 100.0);
        if !cat_missing.is_empty() {
            println!("   Missing: {}", cat_missing.join(", "));
        }
        missing_by_cat.push((cat_name, cat_missing));
    }

    println!("--------------------------------------------------------");
    println!("Total CAD functions cataloged: {}", total_funcs);
    println!("Implemented: {} ({:.1}%)", implemented_count, (implemented_count as f64 / total_funcs as f64) * 100.0);
    println!("Missing / Pending: {}", total_funcs - implemented_count);
    println!("========================================================\n");
}
