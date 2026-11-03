use crate::error::LispError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    Dot,
    Str(String),
    Int(i64),
    Real(f64),
    Sym(String),
}

/// 1-based source line and column location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

fn offset_to_span(source: &str, offset: usize) -> Span {
    let prefix = &source.as_bytes()[..offset.min(source.len())];
    let mut line = 1usize;
    let mut col = 1usize;
    for &b in prefix {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else if b != b'\r' {
            col += 1;
        }
    }
    Span { line, col }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, LispError> {
    Ok(tokenize_spanned(source)?
        .into_iter()
        .map(|st| st.token)
        .collect())
}

pub fn tokenize_spanned(source: &str) -> Result<Vec<SpannedToken>, LispError> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        // Whitespace
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // Line comment: ';' to end of line
        if b == b';' && (i + 1 >= len || bytes[i + 1] != b'|') {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Block comment: ';|' to '|;' (supports nesting)
        if b == b';' && i + 1 < len && bytes[i + 1] == b'|' {
            let comment_start = i;
            let mut depth = 1usize;
            i += 2;
            while i < len && depth > 0 {
                if i + 1 < len && bytes[i] == b';' && bytes[i + 1] == b'|' {
                    depth += 1;
                    i += 2;
                } else if i + 1 < len && bytes[i] == b'|' && bytes[i + 1] == b';' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if depth > 0 {
                let sp = offset_to_span(source, comment_start);
                return Err(LispError::Syntax(format!(
                    "Unterminated block comment ;| ... |; (started at line {}, column {})",
                    sp.line, sp.col
                )));
            }
            continue;
        }

        let token_start = i;

        // Parentheses
        if b == b'(' {
            tokens.push(SpannedToken {
                token: Token::LParen,
                span: offset_to_span(source, token_start),
            });
            i += 1;
            continue;
        }
        if b == b')' {
            tokens.push(SpannedToken {
                token: Token::RParen,
                span: offset_to_span(source, token_start),
            });
            i += 1;
            continue;
        }

        // Quote & Quasiquote
        if b == b'\'' {
            tokens.push(SpannedToken {
                token: Token::Quote,
                span: offset_to_span(source, token_start),
            });
            i += 1;
            continue;
        }
        if b == b'`' {
            tokens.push(SpannedToken {
                token: Token::Quasiquote,
                span: offset_to_span(source, token_start),
            });
            i += 1;
            continue;
        }

        // Unquote / Unquote-splicing
        if b == b',' {
            if i + 1 < len && bytes[i + 1] == b'@' {
                tokens.push(SpannedToken {
                    token: Token::UnquoteSplicing,
                    span: offset_to_span(source, token_start),
                });
                i += 2;
            } else {
                tokens.push(SpannedToken {
                    token: Token::Unquote,
                    span: offset_to_span(source, token_start),
                });
                i += 1;
            }
            continue;
        }

        // String literal
        if b == b'"' {
            i += 1;
            let mut s = String::new();
            let mut terminated = false;

            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 1;
                    match bytes[i] {
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'r' => s.push('\r'),
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        other => s.push(other as char),
                    }
                } else if bytes[i] == b'"' {
                    i += 1;
                    terminated = true;
                    break;
                } else {
                    s.push(bytes[i] as char);
                }
                i += 1;
            }

            if !terminated {
                let sp = offset_to_span(source, token_start);
                return Err(LispError::Syntax(format!(
                    "Unterminated string literal (started at line {}, column {})",
                    sp.line, sp.col
                )));
            }
            tokens.push(SpannedToken {
                token: Token::Str(s),
                span: offset_to_span(source, token_start),
            });
            continue;
        }

        // Atom: symbol, number, or dot
        let start = i;
        while i < len
            && !bytes[i].is_ascii_whitespace()
            && !matches!(bytes[i], b'(' | b')' | b'"' | b';' | b'\'' | b'`')
        {
            i += 1;
        }

        let atom_str = &source[start..i];
        let span = offset_to_span(source, start);

        let tok = if atom_str == "." {
            Token::Dot
        } else if let Ok(val) = atom_str.parse::<i64>() {
            Token::Int(val)
        } else if let Ok(val) = atom_str.parse::<f64>() {
            Token::Real(val)
        } else {
            Token::Sym(atom_str.to_string())
        };
        tokens.push(SpannedToken { token: tok, span });
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let code = "(setq a 10 b 25.5 msg \"hello \\\"world\\\"\") ; line comment";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Sym("setq".into()),
                Token::Sym("a".into()),
                Token::Int(10),
                Token::Sym("b".into()),
                Token::Real(25.5),
                Token::Sym("msg".into()),
                Token::Str("hello \"world\"".into()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_tokenize_block_comment() {
        let code = "(setq a ;| nested ;| block |; comment |; 42)";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Sym("setq".into()),
                Token::Sym("a".into()),
                Token::Int(42),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_tokenize_quote_and_dotted() {
        let code = "'(0 . \"LINE\")";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Quote,
                Token::LParen,
                Token::Int(0),
                Token::Dot,
                Token::Str("LINE".into()),
                Token::RParen,
            ]
        );
    }
}
