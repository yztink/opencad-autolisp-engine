use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::env::Env;
use crate::error::LispError;

/// Case-insensitive AutoLISP symbol (stored normalized as uppercase).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(name: &str) -> Self {
        Self(name.to_ascii_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

/// 1-based source span of an S-expression `( ... )` from its opening `(` to closing `)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ExprSpan {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// A standard Lisp cons cell (car . cdr) with optional source span metadata.
#[derive(Clone, Debug)]
pub struct Cons {
    pub car: Value,
    pub cdr: Value,
    pub span: Option<ExprSpan>,
}

impl Cons {
    pub fn new(car: Value, cdr: Value) -> Self {
        Self {
            car,
            cdr,
            span: None,
        }
    }

    pub fn with_span(car: Value, cdr: Value, span: ExprSpan) -> Self {
        Self {
            car,
            cdr,
            span: Some(span),
        }
    }
}

impl PartialEq for Cons {
    fn eq(&self, other: &Self) -> bool {
        self.car == other.car && self.cdr == other.cdr
    }
}

/// A user-defined function closure created by `(defun ...)` or `(lambda ...)`.
#[derive(Clone)]
pub struct UserFunction {
    pub name: Option<Symbol>,
    pub params: Vec<Symbol>,
    pub locals: Vec<Symbol>,
    pub body: Vec<Value>,
    pub env: Rc<RefCell<Env>>,
}

impl UserFunction {
    pub fn display_name(&self) -> &str {
        self.name.as_ref().map(|s| s.as_str()).unwrap_or("LAMBDA")
    }

    /// Reconstructs the AutoLISP `(DEFUN ...)` or `(LAMBDA ...)` source representation.
    pub fn format_defun(&self) -> String {
        let params_str = self.params.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(" ");
        let locals_str = self.locals.iter().map(|l| l.as_str()).collect::<Vec<_>>().join(" ");
        let arg_list = match (params_str.is_empty(), locals_str.is_empty()) {
            (true, true) => "()".to_string(),
            (false, true) => format!("({})", params_str),
            (true, false) => format!("(/ {})", locals_str),
            (false, false) => format!("({} / {})", params_str, locals_str),
        };
        let body_str = self.body.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ");
        if let Some(ref name) = self.name {
            if body_str.is_empty() {
                format!("(DEFUN {} {})", name.as_str(), arg_list)
            } else {
                format!("(DEFUN {} {} {})", name.as_str(), arg_list, body_str)
            }
        } else if body_str.is_empty() {
            format!("(LAMBDA {})", arg_list)
        } else {
            format!("(LAMBDA {} {})", arg_list, body_str)
        }
    }

    /// Formats a call invocation with evaluated arguments: `(FUNC arg1 arg2 ...)`.
    pub fn format_call(&self, args: &[Value]) -> String {
        let name = self.display_name();
        if args.is_empty() {
            format!("({})", name)
        } else {
            let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ");
            format!("({} {})", name, args_str)
        }
    }
}

impl fmt::Debug for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#<USUBR :{} ({} / {})>",
            self.name.as_ref().map(|s| s.as_str()).unwrap_or("LAMBDA"),
            self.params.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(" "),
            self.locals.iter().map(|l| l.as_str()).collect::<Vec<_>>().join(" ")
        )
    }
}

impl PartialEq for UserFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params && self.locals == other.locals
    }
}

/// Native built-in primitive function pointer.
pub type NativeFn = Arc<dyn Fn(&[Value], &mut Rc<RefCell<Env>>) -> Result<Value, LispError> + Send + Sync>;

/// All dynamically typed values in AutoLISP.
#[derive(Clone)]
pub enum Value {
    Nil,
    T,
    Int(i64),
    Real(f64),
    Str(String),
    Sym(Symbol),
    Cons(Rc<RefCell<Cons>>),
    Subr {
        name: &'static str,
        func: NativeFn,
    },
    USubr(Arc<UserFunction>),
    Ename(u64),
    PickSet(u64),
    VlaObject(u64),
    File(usize),
    CatchError(String),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil)
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn car(&self) -> Value {
        match self {
            Value::Cons(c) => c.borrow().car.clone(),
            _ => Value::Nil,
        }
    }

    pub fn cdr(&self) -> Value {
        match self {
            Value::Cons(c) => c.borrow().cdr.clone(),
            _ => Value::Nil,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "NIL",
            Value::T => "SYM",
            Value::Int(_) => "INT",
            Value::Real(_) => "REAL",
            Value::Str(_) => "STR",
            Value::Sym(_) => "SYM",
            Value::Cons(_) => "LIST",
            Value::Subr { .. } => "SUBR",
            Value::USubr(_) => "USUBR",
            Value::Ename(_) => "ENAME",
            Value::PickSet(_) => "PICKSET",
            Value::VlaObject(_) => "VLA-OBJECT",
            Value::File(_) => "FILE",
            Value::CatchError(_) => "VL-CATCH-ALL-APPLY-ERROR",
        }
    }

    /// Converts a cons chain into a vector of items if it is a proper list.
    pub fn to_vec(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut curr = self.clone();

        while let Value::Cons(cons_rc) = curr {
            let cons = cons_rc.borrow();
            result.push(cons.car.clone());
            curr = cons.cdr.clone();
        }

        if curr.is_nil() {
            Some(result)
        } else {
            None // Dotted pair, not a proper list
        }
    }

    /// Constructs a proper list of Cons cells ending in Nil.
    pub fn from_slice(items: &[Value]) -> Self {
        let mut list = Value::Nil;
        for item in items.iter().rev() {
            list = Value::Cons(Rc::new(RefCell::new(Cons::new(item.clone(), list))));
        }
        list
    }

    /// Constructs a proper list of Cons cells ending in Nil and attaches `span` to the head cell.
    pub fn from_slice_spanned(items: &[Value], span: ExprSpan) -> Self {
        let list = Self::from_slice(items);
        list.set_expr_span(span);
        list
    }

    /// Returns the source `ExprSpan` attached to the head of this `Cons` list, if any.
    pub fn expr_span(&self) -> Option<ExprSpan> {
        match self {
            Value::Cons(c) => c.borrow().span,
            _ => None,
        }
    }

    /// Attaches a source `ExprSpan` to the head `Cons` cell if this value is a `Cons`.
    pub fn set_expr_span(&self, span: ExprSpan) {
        if let Value::Cons(c) = self {
            c.borrow_mut().span = Some(span);
        }
    }

    /// Creates a cons cell: (cons car cdr)
    pub fn cons(car: Value, cdr: Value) -> Self {
        Value::Cons(Rc::new(RefCell::new(Cons::new(car, cdr))))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::T, Value::T) => true,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Real(a), Value::Real(b)) => a == b,
            (Value::Int(a), Value::Real(b)) => (*a as f64) == *b,
            (Value::Real(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Sym(a), Value::Sym(b)) => a == b,
            (Value::Cons(a), Value::Cons(b)) => *a.borrow() == *b.borrow(),
            (Value::Subr { name: a, .. }, Value::Subr { name: b, .. }) => a == b,
            (Value::USubr(a), Value::USubr(b)) => a == b,
            (Value::Ename(a), Value::Ename(b)) => a == b,
            (Value::PickSet(a), Value::PickSet(b)) => a == b,
            (Value::VlaObject(a), Value::VlaObject(b)) => a == b,
            (Value::File(a), Value::File(b)) => a == b,
            (Value::CatchError(a), Value::CatchError(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::T => write!(f, "T"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Real(r) => {
                if r.fract() == 0.0 && !r.is_infinite() && !r.is_nan() {
                    write!(f, "{:.1}", r)
                } else {
                    write!(f, "{}", r)
                }
            }
            Value::Str(s) => write!(f, "\"{}\"", s.replace('\\', "\\\\").replace('\"', "\\\"")),
            Value::Sym(s) => write!(f, "{}", s),
            Value::Cons(cons_rc) => {
                write!(f, "(")?;
                let mut curr = cons_rc.clone();
                let mut first = true;

                loop {
                    let (car, cdr) = {
                        let c = curr.borrow();
                        (c.car.clone(), c.cdr.clone())
                    };

                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", car)?;
                    first = false;

                    match cdr {
                        Value::Nil => break,
                        Value::Cons(next) => curr = next,
                        other => {
                            write!(f, " . {}", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            Value::Subr { name, .. } => write!(f, "#<SUBR :{{{}}}>", name),
            Value::USubr(u) => write!(f, "{:?}", u),
            Value::Ename(h) => write!(f, "<Entity name: {:08X}>", h),
            Value::PickSet(h) => write!(f, "<Selection set: {:08X}>", h),
            Value::VlaObject(h) => write!(f, "#<VLA-OBJECT {:08X}>", h),
            Value::File(h) => write!(f, "<File: {}>", h),
            Value::CatchError(msg) => write!(f, "#<VL-CATCH-ALL-APPLY-ERROR: {}>", msg),
        }
    }
}
