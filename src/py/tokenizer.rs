#![allow(clippy::or_fun_call)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Newline,
    Indent,
    Dedent,
    End,
    Punct,
    Keyword,
    Iden,
    Numeral,
    Literal,
}
use TokenKind::*;

pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

// First match is used, so '+=' should be before '+', etc.
const PUNCTUATION: &[&str] = &[
    "(", ")", "[", "]", "{", "}",
    ":", ",",
    "<=", ">=",
    "<", ">",
    "==", "!=",
    "+=", "-=", "*=", "/=", "%=",
    "+", "-", "*", "/", "%",
    "=",
];

const KEYWORDS: &[&str] = &[
    "def", "global",
    "return", "pass", "break", "continue",
    "if", "else", "while",
    "True", "False",
    "None",
    "and", "or", "not",
];

#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    UnrecognizedToken { pos: usize },
    InconsistentIndent { pos: usize },
    MismatchedQuote { pos: usize },
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, TokenizerError> {
    let mut tokens = vec![];

    let mut round_balance = 0;
    let mut square_balance = 0;
    let mut curly_balance = 0;

    let mut indent_stack = vec![];

    let mut line_start = 0;
    while line_start < s.len() {
        let rest = &s[line_start..];
        let line_len = rest.find('\n').unwrap_or(rest.len());
        let line = &rest[..line_len];

        let trimmed_line = line.trim_start();
        let mut pos = line.len() - trimmed_line.len();
        let is_empty_line = trimmed_line.is_empty() || trimmed_line.starts_with('#');

        if round_balance == 0 &&
           square_balance == 0 &&
           curly_balance == 0 &&
           !is_empty_line {
            let indent = line.len() - line.trim_start_matches(' ').len();
            match indent_stack.last() {
                None => indent_stack.push(indent),
                Some(&last_indent) => {
                    if indent > last_indent {
                        indent_stack.push(indent);
                        tokens.push(Token {
                            kind: Indent,
                            start: line_start + last_indent,
                            end: line_start + indent,
                        });                            
                    } else {
                        let q = indent_stack.iter().position(|&i| i == indent);
                        match q {
                            None => return Err(TokenizerError::InconsistentIndent {
                                pos: line_start + indent,
                            }),
                            Some(q) => {
                                for _ in q + 1..indent_stack.len() {
                                    tokens.push(Token {
                                        kind: Dedent,
                                        start: line_start + indent,
                                        end: line_start + indent,
                                    });
                                }
                                indent_stack.truncate(q + 1);
                            }
                        }
                    }
                }
            }
        }

        loop {
            match line[pos..].chars().next() {
                None => break,
                Some('#') => break,
                Some(c) if c.is_whitespace() => {
                    pos += c.len_utf8();
                }
                Some('\'') => {
                    let start = pos;
                    pos += 1;
                    let mut closed = false;
                    for c in line[pos..].chars() {
                        pos += c.len_utf8();
                        if c == '\'' {
                            closed = true;
                            break;
                        }
                    }
                    if closed {
                        tokens.push(Token {
                            kind: Literal,
                            start: line_start + start,
                            end: line_start + pos,
                        });
                    } else {
                        return Err(TokenizerError::MismatchedQuote {
                            pos: line_start + start,
                        });
                    }
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let start = pos;
                    for c in line[start..].chars() {
                        if c.is_alphanumeric() || c == '_' {
                            pos += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token {
                        kind: if KEYWORDS.contains(&&line[start..pos]) {
                            Keyword
                        } else {
                            Iden
                        },
                        start: line_start + start,
                        end: line_start + pos,
                    });
                }
                Some(c) if c.is_numeric() => {
                    let start = pos;
                    for c in line[start..].chars() {
                        if c.is_numeric() {
                            pos += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token {
                        kind: Numeral,
                        start: line_start + start,
                        end: line_start + pos,
                    })
                }
                Some(_) => {
                    match PUNCTUATION.iter().find(|&&p| line[pos..].starts_with(p)) {
                        Some(&p) => {
                            match p {
                                "(" => round_balance += 1,
                                ")" => round_balance -= 1,
                                "[" => square_balance += 1,
                                "]" => square_balance -= 1,
                                "{" => curly_balance += 1,
                                "}" => curly_balance -= 1,
                                _ => {}
                            }
                            tokens.push(Token {
                                kind: Punct,
                                start: line_start + pos,
                                end: line_start + pos + p.len(),
                            });
                            pos += p.len();
                        }
                        None => return Err(TokenizerError::UnrecognizedToken {
                            pos: line_start + pos,
                        }),
                    }
                }
            }
        }

        if round_balance == 0 &&
           square_balance == 0 &&
           curly_balance == 0 &&
           !is_empty_line {
            tokens.push(Token {
                kind: Newline,
                start: line_start + line_len,
                end: (line_start + line_len + 1).min(s.len()),
            });
        }

        line_start += line_len + 1;
    }

    if round_balance == 0 &&
       square_balance == 0 &&
       curly_balance == 0 {
        while indent_stack.len() > 1 {
            tokens.push(Token {
                kind: Dedent,
                start: s.len(),
                end: s.len(),
            });
            indent_stack.pop();
        }
    }

    tokens.push(Token {
        kind: End,
        start: s.len(),
        end: s.len(),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_to_pairs(s: &str) -> Vec<(TokenKind, &str)> {
        tokenize(s).unwrap().into_iter().map(|t| {
            (t.kind, &s[t.start..t.end])
        }).collect()
    }

    #[test]
    fn hw() {
        assert_eq!(
            tokenize_to_pairs("hello 'world'"),
            &[(Iden, "hello"), (Literal, "'world'"), (Newline, ""), (End, "")]);
    }

    #[test]
    fn indent() {
        assert_eq!(
            tokenize_to_pairs("a\n b\nc"),
            &[(Iden, "a"), (Newline, "\n"),
              (Indent, " "),
              (Iden, "b"), (Newline, "\n"),
              (Dedent, ""),
              (Iden, "c"), (Newline, ""),
              (End, ""),
             ]);
    }

    #[test]
    fn underscores() {
        assert_eq!(
            tokenize_to_pairs("_a a_2"),
            &[(Iden, "_a"),
              (Iden, "a_2"),
              (Newline, ""),
              (End, ""),
            ]);
    }

    #[test]
    fn imbalanced() {
        assert_eq!(
            tokenize_to_pairs("(1 + #zzz\n2)"),
            &[(Punct, "("), (Numeral, "1"), (Punct, "+"), (Numeral, "2"), (Punct, ")"),
              (Newline, ""), (End, "")]);
    }

    #[test]
    fn double_dedent() {
        assert_eq!(
            tokenize_to_pairs("
            a

              b
        # comment
                c
            d"),
            &[(Iden, "a"), (Newline, "\n"),
              (Indent, "  "),
              (Iden, "b"), (Newline, "\n"),
              (Indent, "  "),
              (Iden, "c"), (Newline, "\n"),
              (Dedent, ""),
              (Dedent, ""),
              (Iden, "d"), (Newline, ""),
              (End, ""),
            ]);
    }

    #[test]
    fn fib() {
        assert_eq!(
            tokenize_to_pairs("
            def fib(n):
                return n
            "),
            &[
                (Keyword, "def"), (Iden, "fib"),
                (Punct, "("), (Iden, "n"), (Punct, ")"), (Punct, ":"), (Newline, "\n"),
                (Indent, "    "),
                (Keyword, "return"), (Iden, "n"), (Newline, "\n"),
                (Dedent, ""),
                (End, ""),
            ]);
    }

    #[test]
    fn unrecognized_token() {
        assert_eq!(
            tokenize("$").err().unwrap(),
            TokenizerError::UnrecognizedToken { pos: 0 });
    }

    #[test]
    fn inconsisent_indent() {
        assert_eq!(
            tokenize(" a\nb").err().unwrap(),
            TokenizerError::InconsistentIndent { pos: 3 });
    }

    #[test]
    fn mismatched_quote() {
        assert_eq!(
            tokenize("'").err().unwrap(),
            TokenizerError::MismatchedQuote { pos: 0 });
    }
}
