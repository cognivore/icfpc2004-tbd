#![allow(dead_code)]  // TODO

use super::tokenizer::{TokenKind, Token};
use super::vm::Value;

#[derive(Debug)]
struct Span {
    first_token: usize,
    last_token: usize,
}

impl Span {
    fn new(first_token: usize, last_token: usize) -> Span {
        Span { first_token, last_token }
    }
}

#[derive(Debug)]
pub enum Expr {
    Const(Value),
    Var(String),
    UnaryOp {
        op: &'static str,
        arg: Box<Expr>,
    },
    BinaryOp {
        op: &'static str,
        op_token_idx: usize,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        f: Box<Expr>,
        open_paren_token_idx: usize,
        args: Vec<Expr>,
    },
}

impl Expr {
    fn to_sexpr(&self) -> String {
        match self {
            Expr::Const(c) => format!("{:?}", c),
            Expr::Var(v) => v.to_string(),
            Expr::UnaryOp { op, arg } => format!("({} {})", op, arg.to_sexpr()),
            Expr::BinaryOp { op, left, right, .. } => format!(
                "({} {} {})",
                op, left.to_sexpr(), right.to_sexpr()),
            Expr::Call { f, args, .. } => {
                let mut res = format!("(call {}", f.to_sexpr());
                for arg in args {
                    res.push(' ');
                    res += &arg.to_sexpr();
                }
                res.push(')');
                res
            }
        }
    }
}

pub type Block = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    Pass,
    Break { token_idx: usize },
    Continue { token_idx: usize },
    Return {
        token_idx: usize,
        expr: Option<Box<Expr>>,
    },
    Expr(Expr),
    Assign {
        left: String,
        right: Expr,
        op: Option<&'static str>,
    },
    If {
        cond: Expr,
        then: Block,
        els: Option<Block>,
    },
    While {
        cond: Expr,
        body: Block,
    },
    Def {
        token_idx: usize,
        name: String,
        args: Vec<String>,
        body: Block,
    },
    Global {
        token_idx: usize,
        vars: Vec<String>,
    },
}

impl Stmt {
    fn to_sexpr(&self) -> String {
        match self {
            Stmt::Pass => "pass".to_string(),
            Stmt::Break {..} => "break".to_string(),
            Stmt::Continue {..} => "continue".to_string(),
            Stmt::Return { expr: None, .. } => "return".to_string(),
            Stmt::Return { expr: Some(e), .. } => format!("(return {})", e.to_sexpr()),
            Stmt::Expr(e) => e.to_sexpr(),
            Stmt::Assign { left, right, op } =>
                format!("({}= {} {})", op.unwrap_or(""), left, right.to_sexpr()),
            Stmt::If { cond, then, els } => {
                let mut res = "(if ".to_string();
                res.push_str(&cond.to_sexpr());
                res.push(' ');
                res.push_str(&block_to_sexpr(then));
                if let Some(els) = els {
                    res.push(' ');
                    res.push_str(&block_to_sexpr(els));
                }
                res.push(')');
                res
            }
            Stmt::While { cond, body } =>
                format!("(while {} {})", cond.to_sexpr(), block_to_sexpr(body)),
            Stmt::Def { name, args, body, .. } => {
                let mut res = "(def ".to_string();
                res += name;
                res += " (";
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        res.push(' ');
                    }
                    res += arg;
                }
                res += ") ";
                res += &block_to_sexpr(body);
                res.push(')');
                res
            }
            Stmt::Global { vars, .. } => {
                let mut res = "(global".to_string();
                for v in vars {
                    res.push(' ');
                    res += v;
                }
                res.push(')');
                res
            }
        }
    }
}

fn block_to_sexpr(block: &[Stmt]) -> String {
    assert!(!block.is_empty());
    if block.len() == 1 {
        block[0].to_sexpr()
    } else {
        let mut res = "(block".to_string();
        for s in block {
            res.push(' ');
            res.push_str(&s.to_sexpr());
        }
        res.push(')');
        res
    }
}

pub struct Parser<'a> {
    text: &'a str,
    tokens: &'a [Token],
    i: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub token_idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str, tokens: &'a [Token]) -> Self {
        Parser { text, tokens, i: 0 }
    }

    fn peek_token(&self) -> (TokenKind, &str) {
        let t = &self.tokens[self.i];
        (t.kind, &self.text[t.start..t.end])
    }

    fn peek_token_idx(&self) -> usize { self.i }

    fn consume(&mut self) {
        self.i += 1;
    }

    fn consume_expected_kind(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        let t = self.peek_token();
        if t.0 == kind {
            self.consume();
            Ok(())
        } else {
            Err(ParseError {
                msg: format!(
                    "{:?} expected, got {:?} {:?}",
                    kind, t.0, t.1),
                token_idx: self.peek_token_idx(),
            })
        }
    }

    fn consume_expected(&mut self, kind: TokenKind, value: &str) -> Result<(), ParseError> {
        let t = self.peek_token();
        if t == (kind, value) {
            self.consume();
            Ok(())
        } else {
            Err(ParseError {
                msg: format!(
                    "{:?} {:?} expected, got {:?} {:?}",
                    kind, value, t.0, t.1),
                token_idx: self.peek_token_idx(),
            })
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        let mut block = vec![];
        loop {
            match self.peek_token() {
                (TokenKind::Dedent, _) | (TokenKind::End, _) => break,
                (TokenKind::Keyword, "pass") => {
                    self.consume();
                    self.consume_expected_kind(TokenKind::Newline)?;
                    block.push(Stmt::Pass);
                }
                (TokenKind::Keyword, "break") => {
                    let token_idx = self.peek_token_idx();
                    self.consume();
                    self.consume_expected_kind(TokenKind::Newline)?;
                    block.push(Stmt::Break { token_idx });
                }
                (TokenKind::Keyword, "continue") => {
                    let token_idx = self.peek_token_idx();
                    self.consume();
                    self.consume_expected_kind(TokenKind::Newline)?;
                    block.push(Stmt::Continue { token_idx });
                }
                (TokenKind::Keyword, "return") => {
                    let token_idx = self.peek_token_idx();
                    self.consume();
                    let expr;
                    if self.peek_token().0 == TokenKind::Newline {
                        expr = None;
                        self.consume();
                    } else {
                        expr = Some(Box::new(self.parse_expr()?));
                        self.consume_expected_kind(TokenKind::Newline)?;
                    }
                    block.push(Stmt::Return {
                        token_idx,
                        expr,
                    });
                }
                (TokenKind::Keyword, "if") => {
                    self.consume();
                    let cond = self.parse_expr()?;
                    self.consume_expected(TokenKind::Punct, ":")?;
                    self.consume_expected_kind(TokenKind::Newline)?;
                    self.consume_expected_kind(TokenKind::Indent)?;
                    let then = self.parse_block()?;
                    self.consume_expected_kind(TokenKind::Dedent)?;

                    let els;
                    if self.peek_token() == (TokenKind::Keyword, "else") {
                        self.consume();
                        self.consume_expected(TokenKind::Punct, ":")?;
                        self.consume_expected_kind(TokenKind::Newline)?;
                        self.consume_expected_kind(TokenKind::Indent)?;
                        els = Some(self.parse_block()?);
                        self.consume_expected_kind(TokenKind::Dedent)?;
                    } else {
                        els = None;
                    };
                    
                    block.push(Stmt::If {
                        cond, then, els
                    });
                }
                (TokenKind::Keyword, "while") => {
                    self.consume();
                    let cond = self.parse_expr()?;
                    self.consume_expected(TokenKind::Punct, ":")?;
                    self.consume_expected_kind(TokenKind::Newline)?;
                    self.consume_expected_kind(TokenKind::Indent)?;
                    let body = self.parse_block()?;
                    self.consume_expected_kind(TokenKind::Dedent)?;
                    block.push(Stmt::While {
                        cond,
                        body
                    });
                }
                (TokenKind::Keyword, "def") => block.push(self.parse_def()?),
                (TokenKind::Keyword, "global") => block.push(self.parse_global()?),
                _ => {
                    block.push(self.parse_assign_or_expr()?);
                    self.consume_expected_kind(TokenKind::Newline)?;
                }
            }
        }
        Ok(block)
    }

    fn parse_global(&mut self) -> Result<Stmt, ParseError> {
        let token_idx = self.peek_token_idx();
        self.consume_expected(TokenKind::Keyword, "global")?;

        let mut vars = vec![];
        loop {
            if let (TokenKind::Iden, name) = self.peek_token() {
                vars.push(name.to_string());
                self.consume();
            } else {
                return Err(ParseError {
                    msg: "expected var name".to_string(),
                    token_idx: self.peek_token_idx(),
                });
            };
            match self.peek_token() {
                (TokenKind::Newline, _) => {
                    self.consume();
                    break;
                }
                (TokenKind::Punct, ",") => {
                    self.consume();
                }
                _ => return Err(ParseError {
                    msg: "expected newline or ','".to_string(),
                    token_idx: self.peek_token_idx(),
                })
            }
        }

        Ok(Stmt::Global {
            token_idx,
            vars,
        })
    }

    fn parse_def(&mut self) -> Result<Stmt, ParseError> {
        let token_idx = self.peek_token_idx();
        self.consume_expected(TokenKind::Keyword, "def").unwrap();

        let name = if let (TokenKind::Iden, name) = self.peek_token() {
            name.to_string()
        } else {
            return Err(ParseError {
                msg: "expected function name".to_string(),
                token_idx: self.peek_token_idx(),
            });
        };
        self.consume();

        self.consume_expected(TokenKind::Punct, "(")?;
        let mut args = vec![];
        loop {
            match self.peek_token() {
                (TokenKind::Punct, ")") => {
                    self.consume();
                    break;
                }
                (TokenKind::Iden, name) => {
                    args.push(name.to_string());
                    self.consume();
                }
                _ => return Err(ParseError {
                    msg: "expected formal argument name".to_string(),
                    token_idx: self.peek_token_idx(),
                }),
            }

            match self.peek_token() {
                (TokenKind::Punct, ")") => {
                    self.consume();
                    break;
                }
                (TokenKind::Punct, ",") => {
                    self.consume();    
                }
                _ => return Err(ParseError {
                    msg: "expected ',' or ')'".to_string(),
                    token_idx: self.peek_token_idx(),
                }),
            }
        }
        self.consume_expected(TokenKind::Punct, ":")?;
        self.consume_expected_kind(TokenKind::Newline)?;
        self.consume_expected_kind(TokenKind::Indent)?;
        let body = self.parse_block()?;
        self.consume_expected_kind(TokenKind::Dedent)?;
        Ok(Stmt::Def {
            token_idx,
            name,
            args,
            body,
        })
    }

    fn parse_assign_or_expr(&mut self) -> Result<Stmt, ParseError> {
        let left = self.parse_expr()?;
        let op = match self.peek_token() {
            (TokenKind::Punct, "=") => None,
            (TokenKind::Punct, "+=") => Some("+"),
            (TokenKind::Punct, "*=") => Some("*"),
            _ => return Ok(Stmt::Expr(left)),
        };
        
        let left = match left {
            Expr::Var(v) => v,
            _ => return Err(ParseError {
                msg: "can only assign to variables".to_string(),
                token_idx: self.peek_token_idx(),
            }),
        };

        self.consume();
        let right = self.parse_expr()?;

        Ok(Stmt::Assign {
            left, right, op
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_bp(i32::MIN)
    }

    // https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    fn parse_expr_bp(&mut self, min_bp: i32) -> Result<Expr, ParseError> {
        let mut lhs = match self.peek_token() {
            (TokenKind::Numeral, s) => {
                let token_idx = self.peek_token_idx();
                let c: i32 = s.parse()
                    .map_err(|e| ParseError {
                        msg: format!("{:?}", e),
                        token_idx,
                    })?;
                self.consume();
                Expr::Const(Value::Int(c))
            }
            (TokenKind::Literal, s) => {
                let value = s
                    .strip_prefix('\'').unwrap()
                    .strip_suffix('\'').unwrap()
                    .to_string();
                self.consume();
                Expr::Const(Value::String(value))
            }
            (TokenKind::Keyword, "None") => {
                self.consume();
                Expr::Const(Value::None)
            }
            (TokenKind::Keyword, "True") => {
                self.consume();
                Expr::Const(Value::Bool(true))
            }
            (TokenKind::Keyword, "False") => {
                self.consume();
                Expr::Const(Value::Bool(false))
            }
            (TokenKind::Iden, name) => {
                let name = name.to_owned();
                self.consume();
                Expr::Var(name)
            }
            (TokenKind::Punct, "(") => {
                self.consume();
                let e = self.parse_expr()?;
                self.consume_expected(TokenKind::Punct, ")")?;
                e
            }
            (TokenKind::Keyword, "not") => {
                self.consume();
                let r_bp = 301;
                let arg = Box::new(self.parse_expr_bp(r_bp)?);
                Expr::UnaryOp { op: "not", arg }
            }
            _ => return Err(ParseError {
                msg: "expected expression".to_string(),
                token_idx: self.peek_token_idx(),
            })
        };
        loop {
            let token_idx = self.peek_token_idx();

            if self.peek_token() == (TokenKind::Punct, "(") {
                let open_paren_token_idx = self.peek_token_idx();
                let l_bp = 700;
                assert_ne!(l_bp, min_bp, "ambiguous binding power");
                if l_bp < min_bp {
                    break;
                }                    
                self.consume();
                let mut args = vec![];
                loop {
                    if self.peek_token() == (TokenKind::Punct, ")") {
                        self.consume();
                        break;
                    }
                    args.push(self.parse_expr()?);
                    match self.peek_token() {
                        (TokenKind::Punct, ")") => {
                            self.consume();
                            break;
                        }
                        (TokenKind::Punct, ",") => {
                            self.consume();
                            continue;
                        }
                        _ => return Err(ParseError {
                            msg: "expected ',' or ')'".to_string(),
                            token_idx: self.peek_token_idx(),
                        }),
                    }
                }
                lhs = Expr::Call { f: Box::new(lhs), args, open_paren_token_idx };
                continue;
            }

            let (op, l_bp, r_bp) = match self.peek_token() {
                (TokenKind::Keyword, "or") => ("or", 100, 101),
                (TokenKind::Keyword, "and") => ("and", 200, 201),
                (TokenKind::Punct, "<") => ("<", 400, 401),
                (TokenKind::Punct, "<=") => ("<=", 400, 401),
                (TokenKind::Punct, "==") => ("==", 400, 401),
                (TokenKind::Punct, "+") => ("+", 500, 501),
                (TokenKind::Punct, "-") => ("-", 500, 501),
                (TokenKind::Punct, "*") => ("*", 600, 601),
                _ => break,
            };
            assert_ne!(l_bp, min_bp, "ambiguous binding power");
            if l_bp < min_bp {
                break;
            }
            self.consume();
            let rhs = self.parse_expr_bp(r_bp)?;
            lhs = Expr::BinaryOp {
                op,
                op_token_idx: token_idx,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tokenizer::tokenize;

    fn check_parse_expr(text: &str, expected: &str) {
        println!("{}", text);
        let tokens = tokenize(text).unwrap();
        let mut parser = Parser::new(text, &tokens);
        let e = parser.parse_expr().unwrap();
        parser.consume_expected_kind(TokenKind::Newline).unwrap();
        parser.consume_expected_kind(TokenKind::End).unwrap();

        dbg!(&e);
        assert_eq!(e.to_sexpr(), expected);
    }

    fn check_parse_block(text: &str, expected: &str) {
        println!("{}", text);
        let tokens = tokenize(text).unwrap();
        let mut parser = Parser::new(text, &tokens);
        let b = parser.parse_block().unwrap();
        parser.consume_expected_kind(TokenKind::End).unwrap();

        dbg!(&b);
        assert_eq!(block_to_sexpr(&b), expected);
    }

    #[test]
    fn parse_expr() {
        check_parse_expr("1", "1");
        check_parse_expr("'hello'", r#""hello""#);
        check_parse_expr("x", "x");

        check_parse_expr("True", "true");
        check_parse_expr("False", "false");
        check_parse_expr("None", "None");

        check_parse_expr("1 + 2", "(+ 1 2)");
        check_parse_expr("1 + 2 + 3", "(+ (+ 1 2) 3)");
        check_parse_expr("1 + 2 * 3", "(+ 1 (* 2 3))");
        check_parse_expr("1 * 2 + 3", "(+ (* 1 2) 3)");

        check_parse_expr("(1)", "1");
        check_parse_expr("((1))", "1");
        check_parse_expr("(1 + (2 + 3))", "(+ 1 (+ 2 3))");
        check_parse_expr("(1 + 2) * 3", "(* (+ 1 2) 3)");

        check_parse_expr("not 1 + 2", "(not (+ 1 2))");

        check_parse_expr("f()", "(call f)");
        check_parse_expr("f(x)", "(call f x)");
        check_parse_expr("f(x,)", "(call f x)");
        check_parse_expr("f(x, y)", "(call f x y)");
    }

    #[test]
    fn assignments() {
        check_parse_block("7", "7");
        check_parse_block("a = 7", "(= a 7)");
        check_parse_block("a += 1", "(+= a 1)");
    }

    #[test]
    fn control_flow() {
        check_parse_block("return", "return");
        check_parse_block("return 42", "(return 42)");
        check_parse_block("
            pass
            break
            ",
            "(block pass break)");
        check_parse_block("
            if 1:
                pass
            ",
            "(if 1 pass)");
        check_parse_block("
            if 1:
                continue
            else:
                break
            ",
            "(if 1 continue break)");
        check_parse_block("
            1
            if 2:
                3
                4
            5
            ",
            "(block 1 (if 2 (block 3 4)) 5)");

        check_parse_block("
            while True:
                pass
            ",
            "(while true pass)")
    }

    #[test]
    fn def() {
        check_parse_block("
            def f():
                pass
            ",
            "(def f () pass)");
        check_parse_block("
            def f(x):
                pass
            ",
            "(def f (x) pass)");
        check_parse_block("
            def f(x,):
                pass
            ",
            "(def f (x) pass)");
        check_parse_block("
            def f(x, y):
                pass
            ",
            "(def f (x y) pass)");
    }

    #[test]
    fn global() {
        check_parse_block("global x", "(global x)");
        check_parse_block("global x, y", "(global x y)");
    }
}
