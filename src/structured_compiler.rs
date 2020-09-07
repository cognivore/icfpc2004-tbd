use crate::neurology::{ Instruction, SenseDir, SenseCondition, State, Marker, LR };
use std::{cell::RefCell, rc::Rc};

const INVALID_STATE: State = State(0xFFFF);

type Fixup = Rc<RefCell<State>>;

fn make_fixup() -> Fixup {
    Rc::new(RefCell::new(INVALID_STATE))
}

impl From<&Fixup> for State {
    fn from(it: &Fixup) -> State {
        let val = State(it.borrow().0);
        assert_ne!(val, INVALID_STATE); // it's all about this line actually
        val
    }
}


// We need to be able to fixup next instruction address(es) after inserting an instruction, and
// moreover refer to a specific patch location within an instruction with several addresses.
#[derive(Debug)]
enum FixableInstruction {
    Sense(SenseDir, Fixup, Fixup, SenseCondition),
    Mark(Marker, Fixup),
    Unmark(Marker, Fixup),
    PickUp(Fixup, Fixup),
    Drop(Fixup),
    Turn(LR, Fixup),
    Move(Fixup, Fixup),
    Flip(u16, Fixup, Fixup),
}

use FixableInstruction as FI;


struct CompilerCtx {
    insns: Vec<FixableInstruction>,

    // The list of jump addresses that should be set to the address of the logically next
    // instruction as the destination. We might need several because both branches of an "if/else"
    // jump to the first instruction after the if/else statement and this can compound.
    fixups: Vec<Fixup>,
}


impl CompilerCtx {
    fn new() -> Self {
        Self {
            insns: Vec::new(),
            fixups: Vec::new(),
        }
    }

    // fn dump(&self) {
    //     for (i, insn) in self.insns.iter().enumerate() {
    //         println!("{:04} {:?}", i, insn);
    //     }
    // }


    fn to_instructions(&self) -> Vec<Instruction> {
        self.insns.iter().map(|it| match it {
            FI::Sense(dir, state1, state2, cond) => Instruction::Sense(*dir, State::from(state1), State::from(state2), *cond),
            FI::Mark(marker, state) => Instruction::Mark(*marker, State::from(state)),
            FI::Unmark(marker, state) => Instruction::Unmark(*marker, State::from(state)),
            FI::PickUp(state1, state2) => Instruction::PickUp(State::from(state1), State::from(state2)),
            FI::Drop(state) => Instruction::Drop(State::from(state)),
            FI::Turn(lr, state) => Instruction::Turn(*lr, State::from(state)),
            FI::Move(state1, state2) => Instruction::Move(State::from(state1), State::from(state2)),
            FI::Flip(n, state1, state2) => Instruction::Flip(*n, State::from(state1), State::from(state2)),
        }).collect()
    }


    fn _fixup_state(&mut self, state: State) {
        for it in self.fixups.iter() {
            it.replace(state);
        }
        self.fixups.clear()
    }


    // set all fixup addresses to the address of the next instruction to be inserted.
    fn fixup(&mut self) {
        self._fixup_state(State(self.insns.len() as u16));
    }


    // implement an implicit main loop by calling this at the end.
    fn fixup_mainloop(&mut self) {
        self._fixup_state(State(0));
    }


    // convenience method for 1-address instructions.
    fn set_fixup(&mut self) -> Fixup {
        // convenience method for 1-address instructions.
        assert!(self.fixups.is_empty());
        let fx = make_fixup();
        self.fixups.push(fx.clone());
        fx
    }
}


// global variable go BRRRR. Prefer convenience to slight unsafety.
thread_local! {
    static CTX : RefCell<Option<CompilerCtx>> = RefCell::new(None);
}

fn with_ctx<T>(f: impl FnOnce(&mut CompilerCtx) -> T) -> T {
    CTX.with(|ctx| -> T {
        f(&mut ctx.borrow_mut().as_mut().expect("Call compile()"))
    })
}


// does all the ceremonies, pushes instruction.
fn with_fixup(f: impl FnOnce(Fixup) -> FixableInstruction) {
    with_ctx(|ctx| {
        ctx.fixup();
        let fixup = ctx.set_fixup();
        ctx.insns.push(f(fixup));
    })
}


fn generic_2branch(branch1: impl FnOnce(), branch2: impl FnOnce(),
        f: impl FnOnce(Fixup, Fixup) -> FixableInstruction) {
    let fixup1 = make_fixup();
    let fixup2 = make_fixup();
    with_ctx(|ctx| {
        ctx.fixup();
        ctx.insns.push(f(fixup1.clone(), fixup2.clone()));
        ctx.fixups.push(fixup1.clone());
    });

    // ctx.fixups points to the first state argument, which branch1 would either fix up with the
    // address of its first instruction and replace with its own last instruction destination or
    // leave intact if empty
    branch1();

    // save fixups for later and set ctx.fixups to the second argument, so that branch2 can fix
    // it up and replace it or also leave intact
    let fixups_after_branch1: Vec<Fixup> = with_ctx(|ctx| {
        std::mem::replace(&mut ctx.fixups, vec![fixup2.clone()])
    });

    branch2();

    // join the fixups that should be pointing at the first instruction after the entire "if/else"
    // statement
    with_ctx(|ctx| {
        ctx.fixups.extend(fixups_after_branch1);
    });
}


// public interface.

pub fn mark(m: Marker) {
    with_fixup(|fixup| FI::Mark(m, fixup))
}


pub fn sense(dir: SenseDir, cond: SenseCondition, branch1: impl FnOnce(), branch2: impl FnOnce()) {
    generic_2branch(branch1, branch2,
        |fixup1, fixup2| FI::Sense(dir, fixup1, fixup2, cond));
}


pub fn unmark(m: Marker) {
    with_fixup(|fixup| FI::Unmark(m, fixup));
}


pub fn pickup() {
    // currently doesn't allow checking for error.
    with_fixup(|fixup| FI::PickUp(fixup.clone(), fixup));
}


pub fn drop() {
    with_fixup(FI::Drop);
}


pub fn turn(dir: LR) {
    with_fixup(|fixup| FI::Turn(dir, fixup));
}


pub fn move_() {
    with_fixup(|fixup| FI::Move(fixup.clone(), fixup));
}


pub fn flip(n: u16, branch1: impl FnOnce(), branch2: impl FnOnce()) {
    generic_2branch(branch1, branch2,
        |fixup1, fixup2| FI::Flip(n, fixup1, fixup2));
}


pub fn compile(ant: impl FnOnce()) -> Vec<Instruction> {
    CTX.with(|ctx| {
        ctx.replace(Some(CompilerCtx::new()));
    });

    ant();

    let ctx = CTX.with(|ctx| {
        ctx.replace(None)
    });

    let mut ctx = ctx.unwrap();
    ctx.fixup_mainloop();
    ctx.to_instructions()
}


fn test_ant() {
    sense(SenseDir::Ahead, SenseCondition::Marker(Marker::new(1)), || {
       turn(LR::Left);
       move_();
       pickup();
    }, || {
       flip(3, || {
           // do nothing
       }, || {
           drop()
       });
    });
}


// ENTRY_POINT
pub fn structured_compiler() {
    println!("{:?}", compile(test_ant));
}
