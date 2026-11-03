use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use thiserror::Error;

use crate::ast::{ExprSpan, UserFunction, Value};
use crate::env::Env;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LispError {
    #[error("Syntax error: {0}")]
    Syntax(String),

    #[error("Evaluation error: {0}")]
    Eval(String),

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError {
        expected: &'static str,
        actual: String,
    },

    #[error("Wrong number of arguments: expected {expected}, got {actual}")]
    WrongArgCount {
        expected: &'static str,
        actual: usize,
    },

    #[error("Unbound symbol: {0}")]
    UnboundSymbol(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("IO error: {0}")]
    Io(String),

    #[error("CAD error: {0}")]
    Cad(String),

    #[error("Stack overflow: maximum call depth exceeded")]
    StackOverflow,

    #[error("User error: {0}")]
    UserError(String),

    #[error("Quit / Exit")]
    UserExit,
}

/// Represents a single user-function (`defun` / `lambda`) frame in an AutoLISP call stack.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorFrame {
    pub func_name: String,
    pub call_expr: String,
    pub defun_source: String,
}

/// Rich diagnostic context captured when an AutoLISP error occurs.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorDiagnostic {
    pub error: LispError,
    pub primitive: Option<String>,
    pub failing_expr: Option<String>,
    pub frames: Vec<ErrorFrame>,
}

impl ErrorDiagnostic {
    pub fn new(error: LispError) -> Self {
        Self {
            error,
            primitive: None,
            failing_expr: None,
            frames: Vec::new(),
        }
    }

    /// Formats the primary error headline, incorporating the built-in or special-form name if known.
    pub fn headline(&self) -> String {
        if let Some(ref prim) = self.primitive {
            match &self.error {
                LispError::TypeError { expected, actual } => {
                    format!("Type error in {prim}: expected {expected}, got {actual}")
                }
                LispError::WrongArgCount { expected, actual } => {
                    format!("Wrong number of arguments in {prim}: expected {expected}, got {actual}")
                }
                LispError::DivisionByZero => {
                    format!("Division by zero in {prim}")
                }
                LispError::Cad(msg) => {
                    format!("CAD error in {prim}: {msg}")
                }
                LispError::Eval(msg) => {
                    format!("Evaluation error in {prim}: {msg}")
                }
                LispError::Io(msg) => {
                    format!("IO error in {prim}: {msg}")
                }
                other => other.to_string(),
            }
        } else {
            self.error.to_string()
        }
    }

    /// Formats the diagnostic detail lines (failing expression, offending function definition, and call stack).
    pub fn detail_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(ref expr) = self.failing_expr {
            lines.push(format!("; in expression: {expr}"));
        }
        if !self.frames.is_empty() {
            let f0 = &self.frames[0];
            lines.push(format!("; in function:   {}", f0.call_expr));
            lines.push(format!("; definition:    {}", f0.defun_source));

            let max_extra = 10;
            for f in self.frames.iter().skip(1).take(max_extra) {
                lines.push(format!("; called from:   {}", f.call_expr));
            }
            if self.frames.len() > 1 + max_extra {
                lines.push(format!(
                    "; ... ({} more stack frames)",
                    self.frames.len() - (1 + max_extra)
                ));
            }
        }
        lines
    }

    /// Formats a concise single-line summary for the CAD command line showing only the function that errored.
    pub fn format_cmdline(&self) -> String {
        let head = self.headline();
        match (self.frames.first(), self.failing_expr.as_deref()) {
            (Some(frame), Some(expr)) => {
                format!("in {}: {} [{}]", frame.func_name, head, expr)
            }
            (Some(frame), None) => {
                format!("in {}: {}", frame.func_name, head)
            }
            (None, Some(expr)) => {
                format!("{} [{}]", head, expr)
            }
            (None, None) => head,
        }
    }

    /// Formats the complete multi-line diagnostic report.
    pub fn format_full(&self) -> String {
        let mut out = self.headline();
        for line in self.detail_lines() {
            out.push('\n');
            out.push_str(&line);
        }
        out
    }
}

#[derive(Default)]
struct UnwindState {
    primitive: Option<String>,
    failing_expr: Option<String>,
    frames: Vec<ErrorFrame>,
}

#[derive(Clone)]
struct ActiveCall {
    user_fn: Arc<UserFunction>,
    args: Vec<Value>,
}

thread_local! {
    static UNWIND_STATE: RefCell<UnwindState> = RefCell::new(UnwindState::default());
    static LAST_DIAGNOSTIC: RefCell<Option<ErrorDiagnostic>> = const { RefCell::new(None) };
    static ACTIVE_CALL_STACK: RefCell<Vec<ActiveCall>> = const { RefCell::new(Vec::new()) };
    static TRACED_FUNCTIONS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static DEBUG_SEXPR_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static THREAD_DEBUG_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Clears any in-flight unwind state before starting a new evaluation or after catching an error.
pub fn clear_unwind_state() {
    UNWIND_STATE.with(|s| {
        *s.borrow_mut() = UnwindState::default();
    });
}

/// Records the built-in or special form name if none has been recorded for the current unwinding error.
#[inline(never)]
pub fn record_error_primitive(name: &str) {
    UNWIND_STATE.with(|s| {
        let mut st = s.borrow_mut();
        if st.primitive.is_none() {
            st.primitive = Some(name.to_string());
        }
    });
}

/// Records the innermost failing S-expression if none has been recorded for the current unwinding error.
#[inline(never)]
pub fn record_failing_expr(expr: &Value) {
    UNWIND_STATE.with(|s| {
        let mut st = s.borrow_mut();
        if st.failing_expr.is_none() {
            st.failing_expr = Some(expr.to_string());
        }
    });
}

/// Appends a user-function call frame to the unwinding error backtrace.
#[inline(never)]
pub fn record_error_frame(user_fn: &UserFunction, args: &[Value]) {
    let frame = ErrorFrame {
        func_name: user_fn.display_name().to_string(),
        call_expr: user_fn.format_call(args),
        defun_source: user_fn.format_defun(),
    };
    UNWIND_STATE.with(|s| {
        s.borrow_mut().frames.push(frame);
    });
}

/// Finalizes the unwinding error state into `LAST_DIAGNOSTIC` and returns it.
pub fn finalize_error_diagnostic(err: &LispError) -> ErrorDiagnostic {
    let diag = UNWIND_STATE.with(|s| {
        let mut st = s.borrow_mut();
        ErrorDiagnostic {
            error: err.clone(),
            primitive: st.primitive.take(),
            failing_expr: st.failing_expr.take(),
            frames: std::mem::take(&mut st.frames),
        }
    });
    LAST_DIAGNOSTIC.with(|d| {
        *d.borrow_mut() = Some(diag.clone());
    });
    diag
}

/// Retrieves the most recently finalized `ErrorDiagnostic` on this thread.
pub fn get_last_error_diagnostic() -> Option<ErrorDiagnostic> {
    LAST_DIAGNOSTIC.with(|d| d.borrow().clone())
}

/// Pushes an active call frame onto the live execution stack (used by `vl-bt` and `trace`).
pub fn push_active_call(user_fn: Arc<UserFunction>, args: &[Value]) {
    ACTIVE_CALL_STACK.with(|stack| {
        stack.borrow_mut().push(ActiveCall {
            user_fn,
            args: args.to_vec(),
        });
    });
}

/// Pops the top call frame from the live execution stack.
pub fn pop_active_frame() {
    ACTIVE_CALL_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

/// Returns a snapshot of the current active call stack (from innermost to outermost).
pub fn get_active_call_stack() -> Vec<ErrorFrame> {
    ACTIVE_CALL_STACK.with(|stack| {
        stack
            .borrow()
            .iter()
            .rev()
            .map(|ac| ErrorFrame {
                func_name: ac.user_fn.display_name().to_string(),
                call_expr: ac.user_fn.format_call(&ac.args),
                defun_source: ac.user_fn.format_defun(),
            })
            .collect()
    })
}

/// Returns the current active call depth.
pub fn active_call_depth() -> usize {
    ACTIVE_CALL_STACK.with(|stack| stack.borrow().len())
}

/// Enables tracing for a function symbol (normalized uppercase).
pub fn trace_function(name: &str) {
    TRACED_FUNCTIONS.with(|set| {
        set.borrow_mut().insert(name.to_ascii_uppercase());
    });
}

/// Disables tracing for a function symbol (normalized uppercase).
pub fn untrace_function(name: &str) {
    TRACED_FUNCTIONS.with(|set| {
        set.borrow_mut().remove(&name.to_ascii_uppercase());
    });
}

/// Disables tracing for all functions and returns the list of previously traced names.
pub fn untrace_all() -> Vec<String> {
    TRACED_FUNCTIONS.with(|set| {
        let mut v: Vec<String> = set.borrow_mut().drain().collect();
        v.sort();
        v
    })
}

/// Checks whether a function symbol is currently traced.
pub fn is_function_traced(name: &str) -> bool {
    TRACED_FUNCTIONS.with(|set| {
        let guard = set.borrow();
        !guard.is_empty() && guard.contains(&name.to_ascii_uppercase())
    })
}

/// Stepping mode for the AutoLISP S-expression debugger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugStepMode {
    #[default]
    None,
    StepInto,
    StepOver {
        max_depth: usize,
    },
    StepOut {
        max_depth: usize,
    },
}

/// Action returned by the IDE debugger when resuming from a pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugResumeAction {
    Continue,
    StepInto,
    StepOver,
    StepOut,
    Stop,
}

/// Distinguishes between entering an S-expression and returning from an evaluated S-expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEventKind {
    Entry,
    Return,
}

/// Information passed to the debugger callback when evaluation pauses at an S-expression.
#[derive(Debug, Clone)]
pub struct DebugPauseInfo {
    pub span: ExprSpan,
    pub expr_preview: String,
    pub eval_depth: usize,
    pub locals: Vec<(String, String, String)>,
    pub stack: Vec<(String, usize)>,
    pub event_kind: DebugEventKind,
    pub return_value: Option<String>,
}

pub type DebugPauseCallback = Arc<dyn Fn(&DebugPauseInfo) -> DebugResumeAction + Send + Sync>;

#[derive(Default)]
struct DebuggerState {
    breakpoints: HashSet<(u32, u32)>,
    step_mode: DebugStepMode,
    callback: Option<DebugPauseCallback>,
}

static DEBUG_HOOK_ENABLED: AtomicBool = AtomicBool::new(false);
static DEBUGGER_STATE: OnceLock<Mutex<DebuggerState>> = OnceLock::new();

fn debugger_state() -> &'static Mutex<DebuggerState> {
    DEBUGGER_STATE.get_or_init(|| Mutex::new(DebuggerState::default()))
}

fn update_debug_enabled_flag(st: &DebuggerState) {
    let active = st.callback.is_some()
        && (!st.breakpoints.is_empty() || st.step_mode != DebugStepMode::None);
    DEBUG_HOOK_ENABLED.store(active, Ordering::Relaxed);
}

/// Fast check for whether the S-expression debugger hook is active on the current thread.
#[inline(always)]
pub fn is_debug_hook_enabled() -> bool {
    DEBUG_HOOK_ENABLED.load(Ordering::Relaxed) && THREAD_DEBUG_ENABLED.with(|c| c.get())
}

/// Registers or clears the callback invoked when the evaluator pauses at a breakpoint or step,
/// and enables/disables debugging on the calling evaluator thread.
pub fn set_debug_pause_callback(cb: Option<DebugPauseCallback>) {
    THREAD_DEBUG_ENABLED.with(|c| c.set(cb.is_some()));
    let mut st = debugger_state().lock().unwrap();
    st.callback = cb;
    update_debug_enabled_flag(&st);
}

/// Replaces the active set of parenthesis breakpoints `(1-based line, 1-based col)`.
pub fn set_debug_breakpoints(bps: Vec<(u32, u32)>) {
    let mut st = debugger_state().lock().unwrap();
    st.breakpoints = bps.into_iter().collect();
    update_debug_enabled_flag(&st);
}

/// Sets the current S-expression stepping mode (`None`, `StepInto`, `StepOver`, `StepOut`).
pub fn set_debug_step_mode(mode: DebugStepMode) {
    let mut st = debugger_state().lock().unwrap();
    st.step_mode = mode;
    update_debug_enabled_flag(&st);
}

/// Enters an S-expression during debugging, incrementing the thread-local S-expression depth
/// and pausing if a breakpoint or step condition matches.
#[inline(never)]
pub fn debug_enter_cons(expr: &Value, env: &Rc<RefCell<Env>>) -> Result<bool, LispError> {
    let Some(span) = expr.expr_span() else {
        return Ok(false);
    };
    if span.start_line == 0 || span.start_col == 0 {
        return Ok(false);
    }
    let depth = DEBUG_SEXPR_DEPTH.with(|d| {
        let cur = d.get();
        d.set(cur + 1);
        cur
    });
    if let Err(e) = check_debug_pause(expr, span, env, depth, DebugEventKind::Entry, None) {
        DEBUG_SEXPR_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        return Err(e);
    }
    Ok(true)
}

/// Decrements the thread-local S-expression depth without pausing (e.g. on error).
#[inline(never)]
pub fn debug_pop_depth() {
    DEBUG_SEXPR_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
}

/// Invoked when an evaluated S-expression completes successfully, pausing if stepping.
#[inline(never)]
pub fn debug_exit_cons(
    expr: &Value,
    result: &Value,
    env: &Rc<RefCell<Env>>,
) -> Result<(), LispError> {
    let depth = DEBUG_SEXPR_DEPTH.with(|d| {
        let cur = d.get().saturating_sub(1);
        d.set(cur);
        cur
    });
    let Some(span) = expr.expr_span() else {
        return Ok(());
    };
    if span.start_line == 0 || span.start_col == 0 {
        return Ok(());
    }
    check_debug_pause(expr, span, env, depth, DebugEventKind::Return, Some(result))
}

/// Checks if the evaluator should pause before or after evaluating `expr` at `span` and `depth`.
#[inline(never)]
pub fn check_debug_pause(
    expr: &Value,
    span: ExprSpan,
    env: &Rc<RefCell<Env>>,
    depth: usize,
    event_kind: DebugEventKind,
    return_val: Option<&Value>,
) -> Result<(), LispError> {
    if span.start_line == 0 || span.start_col == 0 {
        return Ok(());
    }

    // Check if the expression is a top-level (DEFUN ...) or (DEFUN-Q ...) definition
    let is_defun_form = match expr.car() {
        Value::Sym(s) => s.as_str() == "DEFUN" || s.as_str() == "DEFUN-Q",
        _ => false,
    };

    let cb_opt = {
        let st = debugger_state().lock().unwrap();
        let Some(ref cb) = st.callback else {
            return Ok(());
        };
        let is_bp = matches!(event_kind, DebugEventKind::Entry)
            && st.breakpoints.contains(&(span.start_line, span.start_col));
        let should_step = !is_defun_form
            && match st.step_mode {
                DebugStepMode::None => false,
                DebugStepMode::StepInto => true,
                DebugStepMode::StepOver { max_depth } => match event_kind {
                    DebugEventKind::Entry => depth <= max_depth,
                    DebugEventKind::Return => depth < max_depth,
                },
                DebugStepMode::StepOut { max_depth } => match event_kind {
                    DebugEventKind::Entry => depth < max_depth,
                    DebugEventKind::Return => depth <= max_depth,
                },
            };
        if is_bp || should_step {
            Some(cb.clone())
        } else {
            None
        }
    };

    if let Some(cb) = cb_opt {
        let locals = Env::inspect_locals(env);
        let stack: Vec<(String, usize)> = ACTIVE_CALL_STACK.with(|s| {
            s.borrow()
                .iter()
                .rev()
                .map(|ac| (ac.user_fn.display_name().to_string(), span.start_line as usize))
                .collect()
        });
        let preview = if let Some(ret) = return_val {
            format!("{} => {}", expr, ret)
        } else {
            expr.to_string()
        };
        let info = DebugPauseInfo {
            span,
            expr_preview: preview,
            eval_depth: depth,
            locals,
            stack,
            event_kind,
            return_value: return_val.map(|v| v.to_string()),
        };
        let action = cb(&info);
        let mut st = debugger_state().lock().unwrap();
        match action {
            DebugResumeAction::Continue => {
                st.step_mode = DebugStepMode::None;
                update_debug_enabled_flag(&st);
            }
            DebugResumeAction::StepInto => {
                st.step_mode = DebugStepMode::StepInto;
                update_debug_enabled_flag(&st);
            }
            DebugResumeAction::StepOver => {
                st.step_mode = DebugStepMode::StepOver { max_depth: depth };
                update_debug_enabled_flag(&st);
            }
            DebugResumeAction::StepOut => {
                st.step_mode = DebugStepMode::StepOut { max_depth: depth };
                update_debug_enabled_flag(&st);
            }
            DebugResumeAction::Stop => {
                st.step_mode = DebugStepMode::None;
                update_debug_enabled_flag(&st);
                return Err(LispError::UserExit);
            }
        }
    }

    Ok(())
}
