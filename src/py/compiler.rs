#![allow(dead_code)]

use std::collections::HashSet;
use super::tokenizer::{tokenize, Token, TokenizerError};
use super::ast::{Parser, Expr, Stmt};
use super::vm::Value;

struct LoadedFile<'a> {
    filename: &'a str,
    text: &'a str,
    tokens: Option<Vec<Token>>,
}

pub struct LoadedFiles<'a> {
    files: Vec<LoadedFile<'a>>,
}

impl<'a> LoadedFiles<'a> {
    pub fn new(files: &[(&'a str, &'a str)]) -> Self {
        LoadedFiles {
            files: files.iter().map(|&(filename, text)| LoadedFile {
                filename,
                text,
                tokens: None,
            }).collect(),
        }
    }

    fn render_loc(&self, loc: Loc) -> String {
        let file = &self.files[loc.file_idx];
        let (row, col) = byte_pos_to_row_col(loc.pos, file.text);
        format!("{}:{}:{}", file.filename, row + 1, col + 1)
    }

    pub fn render_error(&self, e: AnyError) -> String {
        format!("{}  {}", self.render_loc(e.loc), e.msg)
    }
}

#[derive(Clone, Copy, Debug)]
struct Loc {
    file_idx: usize,
    pos: usize,
}

impl Loc {
    fn from_token(lfs: &LoadedFiles, file_idx: usize, token_idx: usize) -> Loc {
        Loc {
            file_idx,
            pos: lfs.files[file_idx].tokens.as_ref().unwrap()[token_idx].start,
        }
    }
}

pub fn byte_pos_to_row_col(pos: usize, s: &str) -> (usize, usize) {
    let row = s[..pos].matches('\n').count();
    let line_start = s[..pos].rfind('\n').map_or(0, |p| p + 1);
    let col = s[line_start..pos].chars().count();
    (row, col)
}

#[derive(Debug)]
pub struct CompileError {
    msg: String,
    token_idx: usize,
}

// Lexer, parser, or compiler error.
#[derive(Debug)]
pub struct AnyError {
    msg: String,
    loc: Loc,
}

#[derive(Debug)]
pub enum Insn {
    Pop,
    PushConst(Value),
    BinOp(&'static str),
    PushGlobal(String),
    PopGlobal(String),
    PushLocal(usize),
    PopLocal(usize),
    Jump(isize),
    JumpIfFalse(isize),
    Call {
        f_idx: usize,
        num_args: usize,
    },
    Input,
    Output,
}

impl Insn {
    fn jump_target(&self, addr: usize) -> Option<usize> {
        match self {
            Insn::Jump(d) | Insn::JumpIfFalse(d) =>
                Some((addr as isize + 1 + d) as usize),
            _ => None,
        }
    }

    fn display(&self, addr: usize, cp: &CompiledProgram, cf: Option<&CompiledFunction>) -> String {
        match self {
            Insn::Pop => "pop".to_string(),
            Insn::PushConst(c) => format!("push {:?}", c),
            Insn::BinOp(op) => format!("binop {:?}", op),
            Insn::PushGlobal(v) => format!("push global {}", v),
            Insn::PopGlobal(v) => format!("pop global {}", v),
            &Insn::PushLocal(i) => {
                let (al, name) = cf.unwrap().arg_or_local_name_by_idx(i);
                format!("push {} {}", al, name)
            }
            &Insn::PopLocal(i) => {
                let (al, name) = cf.unwrap().arg_or_local_name_by_idx(i);
                format!("pop {} {}", al, name)
            }
            Insn::Jump(_) =>
                format!("jump label_{}", self.jump_target(addr).unwrap()),
            Insn::JumpIfFalse(_) =>
                format!("pop and jump if false label_{}", self.jump_target(addr).unwrap()),
            &Insn::Call { f_idx, num_args } =>
                format!("call {} with {} args", cp.functions[f_idx].name, num_args),
            Insn::Input => "input".to_string(),
            Insn::Output => "output".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CompiledFunction {
    pub name: String,
    pub arg_names: Vec<String>,
    pub local_names: Vec<String>,
    pub insns: Vec<Insn>,
}

impl CompiledFunction {
    pub fn arg_or_local_name_by_idx(&self, i: usize) -> (&'static str, &str) {
        if let Some(name) = self.arg_names.get(i) {
            ("arg", name)
        } else {
            ("local", &self.local_names[i - self.arg_names.len()])
        }
    }
}

#[derive(Debug)]
pub struct CompiledProgram {
    pub functions: Vec<CompiledFunction>,
    pub insns: Vec<Insn>,
}

impl CompiledProgram {
    pub fn new(lfs: &mut LoadedFiles) -> Result<Self, AnyError> {
        let mut functions: Vec<CompiledFunction> = vec![];

        let mut asts = vec![];
        for (file_idx, file) in lfs.files.iter_mut().enumerate() {
            let tokens = tokenize(file.text).map_err(|e| {
                let (msg, pos) = match e {
                    TokenizerError::UnrecognizedToken { pos } => ("unrecognized token", pos),
                    TokenizerError::InconsistentIndent { pos } => ("inconsistent indentation", pos),
                    TokenizerError::MismatchedQuote { pos } => ("mismatched quote", pos),
                };
                AnyError {
                    msg: format!("lexical error: {}", msg),
                    loc: Loc { file_idx, pos },
                }
            })?;
            let mut parser = Parser::new(file.text, &tokens);
            let ast = parser.parse_block().map_err(|e| {
                let pos = tokens[e.token_idx].start;
                AnyError {
                    msg: format!("syntax error: {}", e.msg),
                    loc: Loc { file_idx, pos }
                }
            })?;
            file.tokens = Some(tokens);
            asts.push(ast);
        }

        for (file_idx, ast) in asts.iter().enumerate() {
            for stmt in ast {
                if let Stmt::Def { token_idx, .. } = stmt {
                    let cf = precompile_def(stmt).map_err(
                        |CompileError { msg, token_idx }| AnyError {
                            msg,
                            loc: Loc::from_token(lfs, file_idx, token_idx),
                        }
                    )?;
                    if functions.iter().any(|f| f.name == cf.name) {
                        return Err(AnyError {
                            msg: format!("duplicate function {:?}", cf.name),
                            loc: Loc::from_token(lfs, file_idx, *token_idx),
                        });
                    }
                    functions.push(cf);
                }
            }
        }

        for (file_idx, ast) in asts.iter().enumerate() {
            for stmt in ast {
                let (name, body) = match stmt {
                    Stmt::Def { name, body, .. } => (name, body),
                    _ => continue
                };
                let cf = functions.iter().find(|cf| cf.name == *name).unwrap();
                let mut ctx = Ctx {
                    functions: &functions,
                    current_function: Some(cf),
                    insns: vec![],
                    break_label: None,
                    continue_label: None,
                };

                if let Some(Stmt::Return { .. }) = body.last() {
                } else {
                    // implicit return
                    ctx.insns.push(Insn::PushConst(Value::None));
                }
                for stmt in body.iter().rev() {
                    match stmt {
                        Stmt::Global { .. } => {},
                        _ => compile_stmt(stmt, &mut ctx).map_err(
                            |CompileError { msg, token_idx }| AnyError {
                                msg,
                                loc: Loc::from_token(lfs, file_idx, token_idx),
                            }
                        )?,
                    }
                }
                ctx.insns.reverse();
                let insns = ctx.insns;
                let cf = functions.iter_mut().find(|cf| cf.name == *name).unwrap();
                cf.insns = insns;
            }
        }

        let mut insns = vec![];
        for (file_idx, ast) in asts.iter().enumerate() {
            let mut ctx = Ctx {
                functions: &functions,
                current_function: None,
                insns: vec![],
                break_label: None,
                continue_label: None,
            };
            for stmt in ast.iter().rev() {
                match stmt {
                    Stmt::Def { .. } => {}
                    Stmt::Global { token_idx, ..} => {
                        return Err(AnyError {
                            msg: "global outside of function".to_string(),
                            loc: Loc::from_token(lfs, file_idx, *token_idx),
                        })
                    }
                    _ => compile_stmt(stmt, &mut ctx).map_err(
                        |CompileError { msg, token_idx }| AnyError {
                            msg,
                            loc: Loc::from_token(lfs, file_idx, token_idx),
                        }
                    )?,
                }
            }
            insns.extend(ctx.insns);
        }
        insns.reverse();

        Ok(CompiledProgram {
            functions,
            insns,
        })
    }
}

impl std::fmt::Display for CompiledProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cf in &self.functions {
            writeln!(f, "function {}, args: {:?}, locals: {:?}",
                cf.name, cf.arg_names, cf.local_names)?;
            display_insns(&cf.insns, self, Some(cf), f)?;
            writeln!(f, "----------------------")?;
        }

        writeln!(f, "entry point:")?;
        display_insns(&self.insns, self, None, f)
    }
}

fn display_insns(
    insns: &[Insn],
    cp: &CompiledProgram,
    cf: Option<&CompiledFunction>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let mut jump_targets = vec![false; insns.len() + 1];
    for (i, insn) in insns.iter().enumerate() {
        if let Some(t) = insn.jump_target(i) {
            jump_targets[t] = true;
        }
    }

    for (i, insn) in insns.iter().enumerate() {
        if jump_targets[i] {
            writeln!(f, "label_{}:", i)?;
        }
        writeln!(f, "        {}", insn.display(i, cp, cf))?;
    }
    if jump_targets[insns.len()] {
        writeln!(f, "label_{}:", insns.len())?;
    }
    Ok(())
}

fn precompile_def(stmt: &Stmt)
-> Result<CompiledFunction, CompileError> {
    let (name, args, body) = match stmt {
        Stmt::Def { name, args, body, .. } => (name, args, body),
        _ => panic!(),
    };
    let mut cf = CompiledFunction {
        name: name.clone(),
        arg_names: args.clone(),
        local_names: vec![],
        insns: vec![],
    };

    let mut globals = HashSet::new();
    for stmt in body {
        if let Stmt::Global { vars, token_idx } = stmt {
            for v in vars {
                // TODO: quadratic
                if cf.arg_names.contains(v) {
                    return Err(CompileError {
                        msg: format!("name {:?} is parameter and global", v),
                        token_idx: *token_idx,
                    })
                }
                globals.insert(v.clone());
            }
        }
    }

    fn visit(stmt: &Stmt, globals: &HashSet<String>, cf: &mut CompiledFunction) {
        match stmt {
            Stmt::Pass => {}
            Stmt::Break { .. } => {}
            Stmt::Continue { .. } => {}
            Stmt::Return { .. } => {}
            Stmt::Expr(_) => {}
            Stmt::Assign { left, .. } => {
                if !globals.contains(left) {
                    // TODO: quadratic
                    if !cf.arg_names.contains(left) &&
                       !cf.local_names.contains(left) {
                        cf.local_names.push(left.clone());
                    }
                }
            }
            Stmt::If { cond: _, then, els } => {
                for stmt in then {
                    visit(stmt, globals, cf);
                }
                if let Some(els) = els {
                    for stmt in els {
                        visit(stmt, globals, cf);
                    }
                }
            }
            Stmt::While { cond: _, body } => {
                for stmt in body {
                    visit(stmt, globals, cf);
                }
            }
            Stmt::Def { .. } => {}
            Stmt::Global { .. } => {}
        }
    }

    for stmt in body {
        visit(stmt, &globals, &mut cf);
    }

    Ok(cf)
}

struct Ctx<'a> {
    functions: &'a [CompiledFunction],
    current_function: Option<&'a CompiledFunction>,

    insns: Vec<Insn>,  // reversed
    break_label: Option<usize>,
    continue_label: Option<usize>,
}

impl<'a> Ctx<'a> {
    fn label(&self) -> usize {
        self.insns.len()
    }

    fn offset_to(&self, label: usize) -> isize {
        self.label() as isize - label as isize
    }

    fn emit_jump(&mut self, label: usize) {
        let d = self.offset_to(label);
        if d != 0 {
            self.insns.push(Insn::Jump(d));
        }
    }

    fn emit_push_var(&mut self, var: &str) {
        if let Some(cf) = self.current_function {
            // TODO: quadratic
            if let Some(i) = cf.arg_names.iter().position(|a| a == var) {
                self.insns.push(Insn::PushLocal(i));
                return;
            }
            if let Some(i) = cf.local_names.iter().position(|a| a == var) {
                self.insns.push(Insn::PushLocal(cf.arg_names.len() + i));
                return;
            }
        }
        self.insns.push(Insn::PushGlobal(var.to_string()));
    }

    fn emit_pop_var(&mut self, var: &str) {
        if let Some(cf) = self.current_function {
            // TODO: quadratic
            if let Some(i) = cf.arg_names.iter().position(|a| a == var) {
                self.insns.push(Insn::PopLocal(i));
                return;
            }
            if let Some(i) = cf.local_names.iter().position(|a| a == var) {
                self.insns.push(Insn::PopLocal(cf.arg_names.len() + i));
                return;
            }
        }
        self.insns.push(Insn::PopGlobal(var.to_string()));
    }
}

fn compile_expr(e: &Expr, ctx: &mut Ctx) -> Result<(), CompileError> {
    match e {
        Expr::Const(c) => ctx.insns.push(Insn::PushConst(c.clone())),
        Expr::Var(v) => ctx.emit_push_var(v),
        Expr::UnaryOp { op: "not", .. } |
        Expr::BinaryOp { op: "and", .. } |
        Expr::BinaryOp { op: "or", .. } => {
            let end_label = ctx.label();
            ctx.insns.push(Insn::PushConst(Value::Bool(false)));
            let false_label = ctx.label();
            ctx.emit_jump(end_label);
            ctx.insns.push(Insn::PushConst(Value::Bool(true)));
            let true_label = ctx.label();
            compile_cond(e, true_label, false_label, ctx)?;
        }
        Expr::BinaryOp { op, left, right, .. } => {
            ctx.insns.push(Insn::BinOp(op));
            compile_expr(right, ctx)?;
            compile_expr(left, ctx)?;
        }
        Expr::Call { f, args, open_paren_token_idx } => {
            let name = match &**f {
                Expr::Var(name) => name,
                _ => return Err(CompileError {
                    msg: "calling expr that is not a function".to_string(),
                    token_idx: *open_paren_token_idx,
                }),
            };

            match name.as_str() {
                "_output" => {
                    assert_eq!(args.len(), 1);
                    ctx.insns.push(Insn::PushConst(Value::None));
                    ctx.insns.push(Insn::Output);
                    compile_expr(&args[0], ctx)?;
                    return Ok(());
                }
                "_input" => {
                    assert_eq!(args.len(), 0);
                    ctx.insns.push(Insn::Input);
                    return Ok(());
                }
                _ => {}
            }

            let f_idx = ctx.functions.iter().position(|cf| cf.name == *name);
            let f_idx = match f_idx {
                Some(it) => it,
                None => return Err(CompileError {
                    msg: format!("function {:?} not defined", name),
                    token_idx: *open_paren_token_idx,
                }),
            };

            if ctx.functions[f_idx].arg_names.len() != args.len() {
                return Err(CompileError {
                    msg: format!("function {:?} takes {} arguments ({} given)",
                        name,
                        ctx.functions[f_idx].arg_names.len(),
                        args.len(),
                    ),
                    token_idx: *open_paren_token_idx,
                })
            }

            ctx.insns.push(Insn::Call { f_idx, num_args: args.len() });
            for arg in args.iter().rev() {
                compile_expr(arg, ctx)?;
            }
        }
        _ => todo!(),
    }
    Ok(())
}

fn compile_cond(e: &Expr, true_label: usize, false_label: usize, ctx: &mut Ctx) -> Result<(), CompileError> {
    match e {
        Expr::Const(Value::Bool(true)) => {
            ctx.emit_jump(true_label);
        }
        Expr::UnaryOp { op: "not", arg } => {
            compile_cond(arg, false_label, true_label, ctx)?;
        }
        Expr::BinaryOp { op: "and", left, right, .. } => {
            compile_cond(right, true_label, false_label, ctx)?;
            let right_label = ctx.label();
            compile_cond(left, right_label, false_label, ctx)?;
        }
        Expr::BinaryOp { op: "or", left, right, .. } => {
            compile_cond(right, true_label, false_label, ctx)?;
            let right_label = ctx.label();
            compile_cond(left, true_label, right_label, ctx)?;
        }
        _ => {
            ctx.emit_jump(true_label);
            ctx.insns.push(Insn::JumpIfFalse(ctx.offset_to(false_label)));
            compile_expr(e, ctx)?;
        }
    }
    Ok(())
}

fn compile_stmt(stmt: &Stmt, ctx: &mut Ctx) -> Result<(), CompileError> {
    match stmt {
        Stmt::Pass => {}
        Stmt::Expr(e) => {
            ctx.insns.push(Insn::Pop);
            compile_expr(e, ctx)?;
        }
        Stmt::Assign { left, right, op: None } => {
            ctx.emit_pop_var(left);
            compile_expr(right, ctx)?;
        }
        Stmt::Assign { left, right, op: Some(op) } => {
            ctx.emit_pop_var(left);
            ctx.insns.push(Insn::BinOp(op));
            compile_expr(right, ctx)?;
            ctx.emit_push_var(left);
        }
        Stmt::If { cond, then, els } => {
            let end_label = ctx.label();
            if let Some(els) = els {
                compile_block(els, ctx)?;
            }
            let else_label = ctx.label();
            ctx.emit_jump(end_label);
            compile_block(then, ctx)?;
            let then_label = ctx.label();
            compile_cond(cond, then_label, else_label, ctx)?;
        }
        Stmt::While { cond, body } => {
            let end_label = ctx.label();
            let jump_back_idx = ctx.insns.len();
            ctx.insns.push(Insn::Jump(0));
            let continue_label = ctx.label();

            let old_break_label = std::mem::replace(&mut ctx.break_label, Some(end_label));
            let old_continue_label = std::mem::replace(&mut ctx.continue_label, Some(continue_label));
            compile_block(body, ctx)?;
            ctx.break_label = old_break_label;
            ctx.continue_label = old_continue_label;

            let body_label = ctx.label();
            compile_cond(cond, body_label, end_label, ctx)?;
            ctx.insns[jump_back_idx] = Insn::Jump(
                jump_back_idx as isize - ctx.insns.len() as isize);
        }
        Stmt::Break { token_idx } => {
            let break_label = ctx.break_label.ok_or_else(|| {
                CompileError {
                    msg: "break outside of loop".to_string(),
                    token_idx: *token_idx,
                }
            })?;
            ctx.emit_jump(break_label);
        }
        Stmt::Continue { token_idx } => {
            let continue_label = ctx.continue_label.ok_or_else(|| {
                CompileError {
                    msg: "continue outside of loop".to_string(),
                    token_idx: *token_idx,
                }
            })?;
            ctx.emit_jump(continue_label);
        }
        Stmt::Return { token_idx, expr } => {
            if ctx.current_function.is_none() {
                return Err(CompileError {
                    msg: "'return' outside function".to_string(),
                    token_idx: *token_idx,
                });
            }
            ctx.emit_jump(0);
            match expr {
                Some(expr) => compile_expr(expr, ctx)?,
                None => ctx.insns.push(Insn::PushConst(Value::None)),
            }
        }
        Stmt::Global { token_idx, .. } => return Err(CompileError {
            msg: "global is not allowed here".to_string(),
            token_idx: *token_idx,
        }),
        _ => todo!(),
    }
    Ok(())
}

fn compile_block(block: &[Stmt], ctx: &mut Ctx) -> Result<(), CompileError> {
    for stmt in block.iter().rev() {
        compile_stmt(stmt, ctx)?;
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn compile_single_file(filename: &str, text: &str)
    -> Result<CompiledProgram, AnyError> {
        let mut lfs = LoadedFiles::new(&[(filename, text)]);
        CompiledProgram::new(&mut lfs)
    }

    fn expect_error(filename: &str, text: &str, error: &str) {
        let mut lfs = LoadedFiles::new(&[(filename, text)]);
        let e = CompiledProgram::new(&mut lfs).err().unwrap();
        assert_eq!(lfs.render_error(e), error);
    }

    #[test]
    fn syntax_errors() {
        expect_error(
            "a.py",
            "1@",
            "a.py:1:2  lexical error: unrecognized token");
        expect_error(
            "a.py",
            "(1 + ",
            "a.py:1:6  syntax error: expected expression");
    }

    #[test]
    fn loops() {
        expect_error(
            "a.py",
            "  break",
            "a.py:1:3  break outside of loop");
        expect_error(
            "a.py",
            "\ncontinue",
            "a.py:2:1  continue outside of loop");
        let cp = compile_single_file("example.py",
            "
            while True:
                break
                continue
                x = 1
            ").unwrap();
        eprintln!("{}", cp);
    }

    #[test]
    fn fns() {
        expect_error(
            "a.py",
            "global x, y",
            "a.py:1:1  global outside of function");

        expect_error(
            "a.py", "
            def f():
                pass
            def f():
                pass
            ",
            r#"a.py:4:13  duplicate function "f""#);

        expect_error(
            "a.py", "
            def f(x):
                global x
            ",
            r#"a.py:3:17  name "x" is parameter and global"#);

        expect_error(
            "a.py", "
            def f():
                if True:
                    global x
            ",
            r#"a.py:4:21  global is not allowed here"#);

        let cp = compile_single_file("example.py", "
            def f(x, y):
                global g
                i = 1
                j += 1
                g = 1
            ").unwrap();
        eprintln!("{}", cp);
        let cf = &cp.functions[0];
        assert_eq!(cf.arg_names, ["x", "y"]);
        assert_eq!(cf.local_names, ["i", "j"]);
    }

    #[test]
    fn return_() {
        expect_error(
            "a.py",
            "return 42",
            "a.py:1:1  'return' outside function");

        let cp = compile_single_file("example.py", "
            def f1():
                pass
            def f2():
                return 42
            def f3():
                return
                return 42
                g = g
            def f4():
                if x:
                    return y
                else:
                    return z
            ").unwrap();
        eprintln!("{}", cp);        
    }

    #[test]
    fn calls() {
        expect_error(
            "a.py",
            "f()",
            r#"a.py:1:2  function "f" not defined"#);
        expect_error(
            "a.py", "
            def f(x):
                pass
            f(1, 2)
            ",
            r#"a.py:4:14  function "f" takes 1 arguments (2 given)"#);

        let cp = compile_single_file("example.py", "
            def f(x, y):
                pass
            z = f(1, 2)
            ").unwrap();
        eprintln!("{}", cp);
    }

    #[test]
    fn locals() {
        let cp = compile_single_file("example.py", "
            def f(a):
                global g
                a = a
                i = i
                g = g
            ").unwrap();
        eprintln!("{}", cp);        
    }

    #[test]
    fn zzz() {
        let cp = compile_single_file("example.py", "
            def f(x):
                t = x
            g = 42
            ").unwrap();
        eprintln!("{}", cp);
    }
}
