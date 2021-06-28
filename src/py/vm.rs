#![allow(dead_code)]  // TODO

use std::collections::BTreeMap;
use super::compiler::{Insn, CompiledProgram};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Value {
    None,
    Bool(bool),
    Int(i32),
    String(String),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{:?}", s),
        }
    }
}

impl Value {
    fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a + *b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError(
                format!("can't add {:?} and {:?}", self, other)
            )),
        }
    }

    fn sub(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a - *b)),
            _ => Err(RuntimeError(
                format!("can't subtract {:?} and {:?}", self, other)
            )),
        }
    }

    fn mul(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a * *b)),
            _ => Err(RuntimeError(
                format!("can't multiply {:?} and {:?}", self, other)
            )),
        }
    }

    fn div(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => if *b == 0 {
                Err(RuntimeError("division by zero".to_owned()))
            } else {
                Ok(Value::Int(a.div_euclid(*b)))
            }
            _ => Err(RuntimeError(
                format!("can't divide {:?} by {:?}", self, other)
            )),
        }
    }

    fn rem(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => if *b == 0 {
                Err(RuntimeError("division by zero".to_owned()))
            } else {
                Ok(Value::Int(a.rem_euclid(*b)))
            }
            _ => Err(RuntimeError(
                format!("can't divide {:?} by {:?}", self, other)
            )),
        }
    }

    fn str(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Int(a) => Ok(Value::String(format!("{}", a))),
            _ => Err(RuntimeError(
                format!("can't call str() on {:?}", self)
            )),
        }
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct StackFrame {
    ip: usize,
    locals: Vec<Option<Value>>,
    fn_idx: Option<usize>,  // None when running global
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub globals: BTreeMap<String, Value>,  // for hashability
    call_stack: Vec<StackFrame>,
    value_stack: Vec<Value>,

    input: Option<Value>,
}

pub enum Status {
    Terminated,
    Running,
    BlockedOnInput,
}

#[derive(Debug)]
pub struct RuntimeError(String);

impl State {
    pub fn new() -> Self {
        State {
            globals: BTreeMap::new(),
            call_stack: vec![StackFrame {
                ip: 0,
                locals: vec![],
                fn_idx: None,
            }],
            value_stack: vec![],
            input: None,
        }
    }

    pub fn give_input(&mut self, input: Value) {
        assert!(self.input.is_none());
        self.input = Some(input);
    }

    pub fn step(
        &mut self,
        cp: &CompiledProgram,
        output: &mut Vec<Value>,
    ) -> Result<Status, RuntimeError> {
        let frame = self.call_stack.last_mut().unwrap();
        let insns = match frame.fn_idx {
            None => &cp.insns,
            Some(fn_idx) => &cp.functions[fn_idx].insns,
        };

        if frame.ip == insns.len() {
            assert!(self.input.is_none());
            if frame.fn_idx.is_none() {
                assert!(self.value_stack.is_empty());
                return Ok(Status::Terminated);
            } else {
                self.call_stack.pop();
            }
        } else {
            let insn = &insns[frame.ip];
            frame.ip += 1;
            match insn {
                Insn::Pop => {
                    self.value_stack.pop().unwrap();
                }
                Insn::PushConst(c) =>
                    self.value_stack.push(c.clone()),
                &Insn::UnOp(op) => {
                    let a = self.value_stack.pop().unwrap();
                    let res = match op {
                        "str" => a.str()?,
                        _ => panic!("{:?}", op),
                    };
                    self.value_stack.push(res);
                }
                &Insn::BinOp(op) => {
                    let b = self.value_stack.pop().unwrap();
                    let a = self.value_stack.pop().unwrap();
                    let res = match op {
                        "+" => a.add(&b)?,
                        "-" => a.sub(&b)?,
                        "*" => a.mul(&b)?,
                        "/" => a.div(&b)?,
                        "%" => a.rem(&b)?,
                        "==" => Value::Bool(a.eq(&b)),
                        "!=" => Value::Bool(!a.eq(&b)),
                        "<" => Value::Bool(a.partial_cmp(&b).expect("TODO") == std::cmp::Ordering::Less),
                        "<=" => Value::Bool(a.partial_cmp(&b).expect("TODO") != std::cmp::Ordering::Greater),
                        ">" => Value::Bool(a.partial_cmp(&b).expect("TODO") == std::cmp::Ordering::Greater),
                        ">=" => Value::Bool(a.partial_cmp(&b).expect("TODO") != std::cmp::Ordering::Less),
                        _ => panic!("{:?}", op),
                    };
                    self.value_stack.push(res);
                }
                Insn::PushGlobal(name) => {
                    self.value_stack.push(self.globals[name].clone());
                }
                Insn::PopGlobal(name) => {
                    let v = self.value_stack.pop().unwrap();
                    self.globals.insert(name.clone(), v);
                }
                &Insn::PushLocal(i) => match frame.locals[i].clone() {
                    Some(v) => self.value_stack.push(v),
                    None => {
                        let (al, name) = cp.functions[frame.fn_idx.unwrap()]
                            .arg_or_local_name_by_idx(i);
                        return Err(RuntimeError(
                            format!("{} {} referenced before assignment", al, name)
                        ));
                    }
                }
                &Insn::PopLocal(i) => {
                    let v = self.value_stack.pop().unwrap();
                    frame.locals[i] = Some(v);
                }
                Insn::Jump(delta) => {
                    frame.ip = (frame.ip as isize + delta) as usize;
                }
                Insn::JumpIfFalse(delta) => {
                    let v = self.value_stack.pop().unwrap();
                    match v {
                        Value::Bool(true) => {}
                        Value::Bool(false) => frame.ip = (frame.ip as isize + delta) as usize,
                        _ => return Err(RuntimeError(
                            format!("can't use value {:?} in boolean context", v)
                        )),
                    }
                }
                &Insn::Call { f_idx, num_args } => {
                    let cf = &cp.functions[f_idx];
                    assert_eq!(cf.arg_names.len(), num_args);

                    let mut locals = vec![None; cf.arg_names.len() + cf.local_names.len()];
                    for i in (0..num_args).rev() {
                        let v = self.value_stack.pop().unwrap();
                        locals[i] = Some(v);
                    }
                    self.call_stack.push(StackFrame {
                        ip: 0,
                        locals,
                        fn_idx: Some(f_idx),
                    });
                }
                Insn::Output => {
                    let v = self.value_stack.pop().unwrap();
                    output.push(v);
                }
                Insn::Input => {
                    match self.input.take() {
                        None => {
                            frame.ip -= 1;
                            return Ok(Status::BlockedOnInput);
                        }
                        Some(v) => {
                            self.value_stack.push(v);
                            return Ok(Status::Running);
                        }
                    }
                }
            }
            assert!(self.input.is_none());
        }

        Ok(Status::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::compiler::tests::compile_single_file;

    fn run_to_termination(filename: &str, text: &str) -> Result<State, RuntimeError> {
        let cp = compile_single_file(filename, text).unwrap();
        eprintln!("{}", cp);
        let mut state = State::new();
        let mut output = vec![];
        loop {
            match state.step(&cp, &mut output)? {
                Status::Terminated => return Ok(state),
                Status::Running => {}
                Status::BlockedOnInput => panic!(),
            }
        }
    }

    fn expect_runtime_error(filename: &str, text: &str, expected: &str) {
        let e = run_to_termination(filename, text).err().unwrap();
        assert_eq!(e.0, expected);
    }

    #[test]
    fn smoke() {
        expect_runtime_error("a.py", "1 + True", "can't add 1 and true");

        let state = run_to_termination("a.py", "g = 1 + 2").unwrap();
        assert_eq!(state.globals["g"], Value::Int(3));
    }

    #[test]
    fn factorial() {
        let state = run_to_termination("a.py", "
            def factorial(n):
                result = 1
                i = 1
                while i <= n:
                    result *= i
                    i += 1
                return result

            g = factorial(4)
            ").unwrap();
        assert_eq!(state.globals["g"], Value::Int(24));
    }

    #[test]
    fn fib() {
        let state = run_to_termination("a.py", "
            def fib(n):
                if n < 2:
                    return n
                return fib(n - 2) + fib(n - 1)

            f0 = fib(0)
            f1 = fib(1)
            f2 = fib(2)
            f3 = fib(3)
            f8 = fib(8)
            ").unwrap();
        assert_eq!(state.globals["f0"], Value::Int(0));
        assert_eq!(state.globals["f1"], Value::Int(1));
        assert_eq!(state.globals["f2"], Value::Int(1));
        assert_eq!(state.globals["f3"], Value::Int(2));
        assert_eq!(state.globals["f8"], Value::Int(21));
    }

    #[test]
    fn binomial() {
        let state = run_to_termination("a.py", "
            def binomial(n, k):
                if k == 0 or k == n:
                    return 1
                return binomial(n - 1, k - 1) + binomial(n - 1, k)

            c_5_2 = binomial(5, 2)
            ").unwrap();
        assert_eq!(state.globals["c_5_2"], Value::Int(10));
    }

    #[test]
    fn io() {
        let cp = compile_single_file("a.py", "
            _output(2 * _input())
            ").unwrap();
        eprintln!("{}", cp);
        let mut state = State::new();
        let mut output = vec![];
        let mut input = vec![Value::Int(42)];
        loop {
            match state.step(&cp, &mut output).unwrap() {
                Status::Terminated => break,
                Status::Running => {}
                Status::BlockedOnInput => {
                    state.give_input(input.pop().unwrap());
                }
            }
        }
        assert_eq!(input, []);
        assert_eq!(output, [Value::Int(84)]);
    }
}
