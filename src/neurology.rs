use crate::geometry::{
    Dir,
    LR,
};

use crate::phenomenology::{
    SenseCondition,
    Marker,
};


#[derive(Copy, Clone)]
pub struct State(pub u16);

pub enum Instruction {
    Sense(Dir, State, State, SenseCondition),
    Mark(Marker, State),
    Unmark(Marker, State),
    PickUp(State, State),
    Drop(State),
    Turn(LR, State),
    Move(State, State),
    Flip(u16, State, State),
}
