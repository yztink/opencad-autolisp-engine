use std::cell::RefCell;
use std::rc::Rc;

use lisp_core::{create_default_env, eval_str, Env, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplInputStatus {
    Complete,
    Incomplete,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplOutput {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

pub struct ReplSession {
    pub env: Rc<RefCell<Env>>,
    buffer: String,
}

impl ReplSession {
    pub fn new() -> Self {
        Self {
            env: create_default_env(),
            buffer: String::new(),
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn reset_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Checks whether the current buffer + new line has balanced parentheses.
    pub fn feed_line(&mut self, line: &str) -> (ReplInputStatus, Option<ReplOutput>) {
        let trimmed = line.trim();
        if self.buffer.is_empty() && trimmed.is_empty() {
            return (ReplInputStatus::Empty, None);
        }

        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        if is_paren_balanced(&self.buffer) {
            let full_code = self.buffer.clone();
            self.buffer.clear();

            match eval_str(&full_code, &mut self.env) {
                Ok(val) => {
                    let out = if val != Value::Nil {
                        Some(format!("{}", val))
                    } else {
                        Some("nil".to_string())
                    };
                    (
                        ReplInputStatus::Complete,
                        Some(ReplOutput {
                            success: true,
                            result: out,
                            error: None,
                        }),
                    )
                }
                Err(err) => (
                    ReplInputStatus::Complete,
                    Some(ReplOutput {
                        success: false,
                        result: None,
                        error: Some(err.to_string()),
                    }),
                ),
            }
        } else {
            (ReplInputStatus::Incomplete, None)
        }
    }
}

impl Default for ReplSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks whether parentheses, quotes, and block comments are balanced in the source.
pub fn is_paren_balanced(source: &str) -> bool {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut in_comment = false;
    let mut in_block_comment = 0i32;
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        if in_string {
            if b == b'\\' && i + 1 < len {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if in_comment {
            if b == b'\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }

        if in_block_comment > 0 {
            if i + 1 < len && b == b';' && bytes[i + 1] == b'|' {
                in_block_comment += 1;
                i += 2;
                continue;
            } else if i + 1 < len && b == b'|' && bytes[i + 1] == b';' {
                in_block_comment -= 1;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if b == b';' {
            if i + 1 < len && bytes[i + 1] == b'|' {
                in_block_comment += 1;
                i += 2;
                continue;
            } else {
                in_comment = true;
                i += 1;
                continue;
            }
        }

        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }

        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
        }

        i += 1;
    }

    !in_string && in_block_comment == 0 && depth <= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paren_balance() {
        assert!(is_paren_balanced("(+ 1 2)"));
        assert!(!is_paren_balanced("(+ 1 2"));
        assert!(is_paren_balanced("(setq str \"(not a paren)\")"));
        assert!(!is_paren_balanced("(setq str \"unterminated string"));
        assert!(is_paren_balanced("(+ 1 ;| comment with ) paren |; 2)"));
    }

    #[test]
    fn test_multiline_repl_feed() {
        let mut session = ReplSession::new();
        let (status1, out1) = session.feed_line("(setq total");
        assert!(matches!(status1, ReplInputStatus::Incomplete));
        assert!(out1.is_none());

        let (status2, out2) = session.feed_line("      (+ 10 20))");
        assert!(matches!(status2, ReplInputStatus::Complete));
        assert_eq!(out2.unwrap().result.unwrap(), "30");
    }
}
