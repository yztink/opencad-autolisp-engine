use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::{Symbol, Value};
use crate::error::LispError;

#[derive(Clone)]
pub struct Env {
    pub bindings: HashMap<Symbol, Value>,
    pub parent: Option<Rc<RefCell<Env>>>,
    pub command_hook: Option<Arc<dyn Fn(&str)>>,
}

impl Env {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: None,
            command_hook: None,
        }))
    }

    pub fn with_parent(parent: Rc<RefCell<Env>>) -> Rc<RefCell<Self>> {
        let hook = parent.borrow().command_hook.clone();
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: Some(parent),
            command_hook: hook,
        }))
    }

    pub fn get(&self, sym: &Symbol) -> Option<Value> {
        if let Some(val) = self.bindings.get(sym) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(sym)
        } else {
            None
        }
    }

    pub fn set_local(&mut self, sym: Symbol, val: Value) {
        self.bindings.insert(sym, val);
    }

    /// Sets variable matching AutoLISP `SETQ` semantics:
    /// Walks up the environment chain. If symbol is bound in an ancestor, updates it there;
    /// otherwise in AutoLISP semantics, undeclared symbols are global, so bind it at the root environment!
    pub fn setq(env_rc: &Rc<RefCell<Env>>, sym: Symbol, val: Value) {
        let mut curr = Some(env_rc.clone());
        let mut root = env_rc.clone();
        while let Some(e) = curr {
            if e.borrow().bindings.contains_key(&sym) {
                e.borrow_mut().bindings.insert(sym, val);
                return;
            }
            root = e.clone();
            let next = e.borrow().parent.clone();
            curr = next;
        }

        // Symbol was not found anywhere in ancestor chain.
        // In AutoLISP, variables not declared in the local parameter list are GLOBAL.
        root.borrow_mut().bindings.insert(sym, val);
    }

    pub fn def_subr<F>(&mut self, name: &'static str, func: F)
    where
        F: Fn(&[Value], &mut Rc<RefCell<Env>>) -> Result<Value, LispError> + Send + Sync + 'static,
    {
        let sym = Symbol::new(name);
        let val = Value::Subr {
            name,
            func: Arc::new(func),
        };
        self.bindings.insert(sym, val);
    }

    /// Collects variable bindings `(name, type_name, value_str)` from the current environment
    /// and its ancestors (excluding built-in `Subr`s, `USubr`s, and standard constants).
    pub fn inspect_locals(env_rc: &Rc<RefCell<Env>>) -> Vec<(String, String, String)> {
        let mut seen = std::collections::HashSet::new();
        let mut locals = Vec::new();
        let mut globals = Vec::new();

        let mut curr = Some(env_rc.clone());
        while let Some(e) = curr {
            let borrow = e.borrow();
            let is_root = borrow.parent.is_none();
            let mut frame_vars: Vec<(String, String, String)> = Vec::new();

            for (sym, val) in &borrow.bindings {
                let name = sym.as_str();
                if matches!(val, Value::Subr { .. } | Value::USubr(_)) {
                    continue;
                }
                if is_root && matches!(name, "PI" | "T" | "NIL" | "PAUSE") {
                    continue;
                }
                if seen.insert(name.to_string()) {
                    frame_vars.push((
                        name.to_string(),
                        val.type_name().to_string(),
                        val.to_string(),
                    ));
                }
            }
            frame_vars.sort_by(|a, b| a.0.cmp(&b.0));
            if is_root {
                globals.extend(frame_vars);
            } else {
                locals.extend(frame_vars);
            }
            curr = borrow.parent.clone();
        }

        locals.extend(globals);
        locals
    }
}
