use crate::ast::{ExprSpan, Symbol, Value};
use crate::error::LispError;
use crate::lexer::{tokenize_spanned, Span, SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let spanned = tokens
            .into_iter()
            .map(|token| SpannedToken {
                token,
                span: Span { line: 1, col: 1 },
            })
            .collect();
        Self {
            tokens: spanned,
            cursor: 0,
        }
    }

    pub fn from_spanned(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_all(&mut self) -> Result<Vec<Value>, LispError> {
        let mut exprs = Vec::new();
        while self.cursor < self.tokens.len() {
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    pub fn parse_expr(&mut self) -> Result<Value, LispError> {
        let SpannedToken { token, span } = self.next_spanned_token()?;

        match token {
            Token::Quote => {
                let sub = self.parse_expr()?;
                Ok(Value::from_slice(&[Value::Sym(Symbol::new("QUOTE")), sub]))
            }
            Token::Quasiquote => {
                let sub = self.parse_expr()?;
                Ok(Value::from_slice(&[Value::Sym(Symbol::new("QUASIQUOTE")), sub]))
            }
            Token::Unquote => {
                let sub = self.parse_expr()?;
                Ok(Value::from_slice(&[Value::Sym(Symbol::new("UNQUOTE")), sub]))
            }
            Token::UnquoteSplicing => {
                let sub = self.parse_expr()?;
                Ok(Value::from_slice(&[Value::Sym(Symbol::new("UNQUOTE-SPLICING")), sub]))
            }
            Token::LParen => self.parse_list(span),
            Token::RParen => Err(LispError::Syntax(format!(
                "Unexpected ')' at line {}, column {}",
                span.line, span.col
            ))),
            Token::Dot => Err(LispError::Syntax(format!(
                "Unexpected '.' at line {}, column {}",
                span.line, span.col
            ))),
            Token::Int(n) => Ok(Value::Int(n)),
            Token::Real(r) => Ok(Value::Real(r)),
            Token::Str(s) => Ok(Value::Str(s)),
            Token::Sym(s) => {
                let upper = s.to_ascii_uppercase();
                if upper == "NIL" {
                    Ok(Value::Nil)
                } else if upper == "T" {
                    Ok(Value::T)
                } else {
                    Ok(Value::Sym(Symbol(upper)))
                }
            }
        }
    }

    fn parse_list(&mut self, open_span: Span) -> Result<Value, LispError> {
        let mut items = Vec::new();

        while self.cursor < self.tokens.len() {
            if self.peek_token() == Some(&Token::RParen) {
                let close_span = self.tokens[self.cursor].span;
                self.cursor += 1; // consume ')'
                let span = ExprSpan {
                    start_line: open_span.line as u32,
                    start_col: open_span.col as u32,
                    end_line: close_span.line as u32,
                    end_col: close_span.col as u32,
                };
                return Ok(Value::from_slice_spanned(&items, span));
            }

            if self.peek_token() == Some(&Token::Dot) {
                let dot_span = self.tokens[self.cursor].span;
                self.cursor += 1; // consume '.'
                if items.is_empty() {
                    return Err(LispError::Syntax(format!(
                        "Cannot create dotted pair with no car at line {}, column {}",
                        dot_span.line, dot_span.col
                    )));
                }
                let dotted_cdr = self.parse_expr()?;
                let next = self.next_spanned_token()?;
                if next.token != Token::RParen {
                    return Err(LispError::Syntax(format!(
                        "Expected ')' after cdr in dotted pair at line {}, column {}",
                        next.span.line, next.span.col
                    )));
                }
                let span = ExprSpan {
                    start_line: open_span.line as u32,
                    start_col: open_span.col as u32,
                    end_line: next.span.line as u32,
                    end_col: next.span.col as u32,
                };
                let mut res = dotted_cdr;
                for item in items.into_iter().rev() {
                    res = Value::cons(item, res);
                }
                res.set_expr_span(span);
                return Ok(res);
            }

            items.push(self.parse_expr()?);
        }

        Err(LispError::Syntax(format!(
            "Missing ')' at end of list (opened at line {}, column {})",
            open_span.line, open_span.col
        )))
    }

    fn next_spanned_token(&mut self) -> Result<SpannedToken, LispError> {
        if self.cursor >= self.tokens.len() {
            Err(LispError::Syntax(
                "Unexpected EOF while parsing AutoLISP".to_string(),
            ))
        } else {
            let t = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Ok(t)
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.cursor).map(|st| &st.token)
    }
}

pub fn parse(source: &str) -> Result<Vec<Value>, LispError> {
    let tokens = tokenize_spanned(source)?;
    let mut parser = Parser::from_spanned(tokens);
    parser.parse_all()
}

pub fn parse_one(source: &str) -> Result<Value, LispError> {
    let tokens = tokenize_spanned(source)?;
    let mut parser = Parser::from_spanned(tokens);
    parser.parse_expr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        assert_eq!(parse_one("123").unwrap(), Value::Int(123));
        assert_eq!(parse_one("3.14").unwrap(), Value::Real(3.14));
        assert_eq!(parse_one("\"hello\"").unwrap(), Value::Str("hello".into()));
        assert_eq!(parse_one("nil").unwrap(), Value::Nil);
        assert_eq!(parse_one("T").unwrap(), Value::T);
        assert_eq!(parse_one("my_var").unwrap(), Value::Sym(Symbol::new("MY_VAR")));
    }

    #[test]
    fn test_parse_list_and_quote() {
        let expr = parse_one("'(1 2 3)").unwrap();
        let expected = Value::from_slice(&[
            Value::Sym(Symbol::new("QUOTE")),
            Value::from_slice(&[Value::Int(1), Value::Int(2), Value::Int(3)]),
        ]);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_dotted_pair() {
        let expr = parse_one("(0 . \"LINE\")").unwrap();
        let expected = Value::cons(Value::Int(0), Value::Str("LINE".into()));
        assert_eq!(expr, expected);
        assert_eq!(format!("{}", expr), "(0 . \"LINE\")");
    }

    #[test]
    fn test_parse_syntax_error_line_col() {
        let err = parse("(setq a 10\n  (setq b 20\n)").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Missing ')'"), "Got: {msg}");
        assert!(msg.contains("line 1, column 1"), "Got: {msg}");

        let err2 = parse("(setq a 10)\n  )").unwrap_err();
        let msg2 = err2.to_string();
        assert!(msg2.contains("Unexpected ')' at line 2, column 3"), "Got: {msg2}");
    }
}
