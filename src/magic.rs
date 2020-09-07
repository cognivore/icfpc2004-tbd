use std::cell::RefCell;
use std::fmt::Debug;
use std::convert::TryInto;
use std::collections::HashMap;
use crate::{phenomenology::{Marker, SenseCondition}, neurology::{State, Instruction, SenseDir, LR}};

#[macro_export]
macro_rules! call {
    ($fn_name:ident($($arg:tt)*)) => {{
        let loc = crate::magic::Loc {
            file: file!(),
            line: line!(),
            column: column!(),
        };
        let _callret = crate::magic::CallRet::new(loc, stringify!($fn_name));
        $fn_name($($arg)*)?
    }};
}

#[macro_export]
macro_rules! var {
    (let $name:ident = $value:tt) => {
        let $name = crate::magic::Var::new(stringify!($name), $value);
    };
    (let $name:ident: $tp:ty = $value:tt) => {
        let $name: Var<$tp> = crate::magic::Var::new(stringify!($name), $value);
    };
    (let mut $name:ident = $value:tt) => {
        let mut $name = crate::magic::Var::new(stringify!($name), $value);
    };
    (let mut $name:ident: $tp:ty = $value:tt) => {
        let mut $name: Var<$tp> = crate::magic::Var::new(stringify!($name), $value);
    };
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

pub fn sense(sense_dir: SenseDir, cond: SenseCondition) -> AntResult<bool> {
    let st_true = State(1);
    let st_false = State(0);
    let res = suspend(Instruction::Sense(sense_dir, st_true, st_false, cond))?;
    Ok(if res == st_true {
        true
    } else if res == st_false {
        false
    } else {
        panic!()
    })
}

pub fn mark(m: Marker) -> AntResult<()> {
    let st = State(0);
    let res = suspend(Instruction::Mark(m, st))?;
    assert_eq!(res, st);
    Ok(())
}

pub fn unmark(m: Marker) -> AntResult<()> {
    let st = State(0);
    let res = suspend(Instruction::Unmark(m, st))?;
    assert_eq!(res, st);
    Ok(())
}

pub fn pickup() -> AntResult<bool> {
    let st_true = State(1);
    let st_false = State(0);
    let res = suspend(Instruction::PickUp(st_true, st_false))?;
    Ok(if res == st_true {
        true
    } else if res == st_false {
        false
    } else {
        panic!()
    })
}

pub fn drop() -> AntResult<()> {
    let st = State(0);
    let res = suspend(Instruction::Drop(st))?;
    assert_eq!(res, st);
    Ok(())
}

pub fn turn(lr: LR) -> AntResult<()> {
    let st = State(0);
    let res = suspend(Instruction::Turn(lr, st))?;
    assert_eq!(res, st);
    Ok(())
}

pub fn move_() -> AntResult<bool> {
    let st_true = State(1);
    let st_false = State(0);
    let res = suspend(Instruction::Move(st_true, st_false))?;
    Ok(if res == st_true {
        true
    } else if res == st_false {
        false
    } else {
        panic!()
    })
}

/// true with probability 1/p
pub fn flip(p: u16) -> AntResult<bool> {
    let st_true = State(1);
    let st_false = State(0);
    let res = suspend(Instruction::Flip(p, st_true, st_false))?;
    Ok(if res == st_true {
        true
    } else if res == st_false {
        false
    } else {
        panic!()
    })
}

pub fn traverse(ant: fn() -> AntResult<()>) -> Vec<(Instruction, String)> {
    let mut exe_to_state: HashMap<ExeState, State> = HashMap::new();
    let mut path_to_state: HashMap<Vec<(Instruction, Branch)>, usize> = HashMap::new();
    
    let mut brain: Vec<(Instruction, String)> = Vec::new();
    let mut branch_to_state: Vec<HashMap<Branch, State>> = Vec::new();

    let mut paths: Vec<Vec<(Instruction, Branch)>> = vec![vec![]];

    while let Some(path) = paths.pop() {
        // eprintln!("{:?}", path);
        let mut rev_path = path.clone();
        rev_path.reverse();
        CTX.with(|ctx| *ctx.borrow_mut() = Ctx {
            stack: vec![],
            prerecorded: rev_path,
        });
        let _callret = CallRet::new(Loc { file: "compiler", line: 0, column: 0 }, "ant");
        match ant() {
            Ok(_) => panic!("ant shouldn't terminate"),
            Err(SuspensionPoint {
                insn,
                exe_state,
            }) => {
                let state = *exe_to_state.entry(exe_state.clone()).or_insert_with(|| {
                    let idx = brain.len();

                    let loc = &exe_state.last().unwrap().caller;
                    brain.push((insn, format!("{}:{}", loc.file, loc.line)));
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
pub struct Var<T: Debug> {
    name: &'static str,
    idx: usize,
    value: T,
}

impl<T: Debug> Var<T> {
    pub fn new(name: &'static str, value: T) -> Self {
        // eprintln!("var {:?} created", name);
        let idx = CTX.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let f = ctx.stack.last_mut().unwrap();
            f.vars.push((name, format!("{:?}", value)));
            f.vars.len() - 1
        });
        Self { name, idx, value }
    }

    pub fn as_ref(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, new_value: T) {
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
    pub fn get(&self) -> T {
        self.value
    }
}

impl<T: Debug> Drop for Var<T> {
    fn drop(&mut self) {
        // eprintln!("var {:?} dropped", self.name);
        CTX.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let f = ctx.stack.last_mut().unwrap();
            let (name, _) = f.vars.pop().unwrap();
            assert_eq!(name, self.name);
            assert_eq!(f.vars.len(), self.idx);
        })
    }
}

pub struct CallRet;

impl CallRet {
    pub fn new(loc: Loc, fn_name: &'static str) -> Self {
        // eprintln!("call {}", fn_name);
        CTX.with(|ctx| {
            ctx.borrow_mut().stack.push(StackFrame {
                caller: loc,
                fn_name,
                vars: vec![],
            });
        });
        Self
    }
}

impl Drop for CallRet {
    fn drop(&mut self) {
        // eprintln!("ret");
        CTX.with(|ctx| {
            let f = ctx.borrow_mut().stack.pop().unwrap();
            assert!(f.vars.is_empty());
        })        
    }
}
