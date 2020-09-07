use std::cell::RefCell;
use std::fmt::Debug;
use std::convert::TryInto;
use std::collections::{HashMap, HashSet};
use crate::neurology::{State, Instruction};

#[macro_export]
macro_rules! call {
    ($fn_name:ident($($arg:tt)*)) => {{
        let loc = crate::magic::Loc {
            file: file!(),
            line: line!(),
            column: column!(),
        };
        crate::magic::call_internal(loc, stringify!($fn_name));
        let res = $fn_name($($arg)*)?;
        crate::magic::ret_internal();
        res
    }};
}

macro_rules! var {
    (let $name:ident = $value:tt) => {
        let $name = Var::new(stringify!($name), $value);
    };
    (let $name:ident: $tp:ty = $value:tt) => {
        let $name: Var<$tp> = Var::new(stringify!($name), $value);
    };
    (let mut $name:ident = $value:tt) => {
        let mut $name = Var::new(stringify!($name), $value);
    };
    (let mut $name:ident: $tp:ty = $value:tt) => {
        let mut $name: Var<$tp> = Var::new(stringify!($name), $value);
    };
}

fn looping_ant() -> AntResult<()> {
    loop {
        call!(drop());
        call!(drop());
    }
}

thread_local!(static CTX: RefCell<Ctx> = RefCell::new(Ctx::default()));

type Branch = State;

#[derive(Debug, Default)]
struct Ctx {
    stack: Vec<StackFrame>,
    prerecorded: Vec<(Instruction, Branch)>,
}

type ExeState = Vec<StackFrame>;

pub struct SuspensionPoint {
    insn: Instruction,
    exe_state: ExeState,
}

fn suspend(insn: Instruction) -> AntResult<Branch> {
    CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        match ctx.prerecorded.pop() {
            Some((i, x)) => {
                assert_eq!(i, insn, "ant is nondeterministic maybe");
                Ok(x)
            }
            _ => {
                Err(SuspensionPoint {
                    insn,
                    exe_state: ctx.stack.clone(),
                })
            }
        }
    })
}

pub fn drop() -> AntResult<()> {
    let st = State(0);
    let res = suspend(Instruction::Drop(st))?;
    assert_eq!(res, st);
    Ok(())
}

pub fn traverse(ant: fn() -> AntResult<()>) -> Vec<(Instruction, String)> {
    let mut exe_to_state: HashMap<ExeState, State> = HashMap::new();
    let mut path_to_state: HashMap<Vec<(Instruction, Branch)>, usize> = HashMap::new();
    
    let mut brain: Vec<(Instruction, String)> = Vec::new();
    let mut branch_to_state: Vec<HashMap<Branch, State>> = Vec::new();

    let mut paths: Vec<Vec<(Instruction, Branch)>> = vec![vec![]];

    while let Some(path) = paths.pop() {
        dbg!(&path);
        CTX.with(|ctx| *ctx.borrow_mut() = Ctx {
            stack: vec![],
            prerecorded: path.clone(),
        });
        match ant() {
            Ok(_) => panic!("ant shouldn't terminate"),
            Err(SuspensionPoint {
                insn,
                exe_state,
            }) => {
                let state = *exe_to_state.entry(exe_state.clone()).or_insert_with(|| {
                    let idx = brain.len();

                    // exe_state.iter().rev().find(|f| f.caller )
                    let loc = &exe_state.last().unwrap().caller;
                    // let file = frame.caller.file.strip_prefix("src/").unwrap();
                    brain.push((insn, format!("{}:{}", loc.file, loc.line)));  // TODO: source location
                    branch_to_state.push(HashMap::new());
                    let state = State(idx.try_into().unwrap());

                    let old = path_to_state.insert(path.clone(), idx);
                    assert!(old.is_none());
                    for tr in insn.transitions() {
                        let mut new_path = path.clone();
                        new_path.push((insn, *tr));
                        paths.push(new_path);
                    }

                    state
                });

                if let Some(((_, last_branch), prev_path)) = path.split_last() {
                    let prev_state = path_to_state[prev_path];
                    let old = branch_to_state[prev_state].insert(*last_branch, state);
                    assert!(old.is_none());
                } else {
                    assert_eq!(state.0, 0);
                }
            }
        }
    }
    // dbg!(brain);
    // dbg!(branch_to_state);
    assert_eq!(brain.len(), branch_to_state.len());
    brain.into_iter().zip(branch_to_state)
        .map(|((mut insn, comment), branch_to_state)| {
            assert_eq!(insn.transitions().count(), branch_to_state.len());
            for branch in insn.transitions_mut() {
                *branch = branch_to_state[branch];
            }
            (insn, comment)
        })
        .collect()
}

pub type AntResult<T> = Result<T, SuspensionPoint>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loc {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StackFrame {
    caller: Loc,
    fn_name: &'static str,
    vars: Vec<(&'static str, String)>,
}

#[derive(Debug)]
struct Var<T: Debug> {
    name: &'static str,
    idx: usize,
    value: T,
}

impl<T: Debug> Var<T> {
    fn new(name: &'static str, value: T) -> Self {
        let idx = CTX.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let f = ctx.stack.last_mut().unwrap();
            f.vars.push((name, format!("{:?}", value)));
            f.vars.len() - 1
        });
        Self { name, idx, value }
    }

    fn as_ref(&self) -> &T {
        &self.value
    }

    fn set(&mut self, new_value: T) {
        self.value = new_value;
        CTX.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let f = ctx.stack.last_mut().unwrap();
            assert_eq!(self.name, f.vars[self.idx].0);
            f.vars[self.idx].1 = format!("{:?}", self.value);
        })
    }
}

impl<T: Debug + Copy> Var<T> {
    fn get(&self) -> T {
        self.value
    }
}

impl<T: Debug> Drop for Var<T> {
    fn drop(&mut self) {
        CTX.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let f = ctx.stack.last_mut().unwrap();
            let (name, _) = f.vars.pop().unwrap();
            assert_eq!(name, self.name);
        })
    }
}

pub fn call_internal(loc: Loc, fn_name: &'static str) {
    CTX.with(|ctx| {
        ctx.borrow_mut().stack.push(StackFrame {
            caller: loc,
            fn_name,
            vars: vec![],
        });
    })
}

pub fn ret_internal() {
    CTX.with(|ctx| {
        let f = ctx.borrow_mut().stack.pop().unwrap();
        assert!(f.vars.is_empty());
    })
}
