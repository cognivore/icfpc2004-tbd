use crate::geometry::{
    Dir
};

use crate::neurology::{
    State
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    Red,
    Black,
}

#[derive(Clone, Debug)]
pub struct Ant {
    pub id : u8, // Every map contains just one anthill, so there should be up to 91 ants
    pub color : Color,
    pub state : State,
    pub resting : u8,
    pub direction : Dir,
    pub has_food : bool,
}

// Biology functions
pub fn other_color(c : Color) -> Color {
    match c {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    }
}

impl Ant {
    pub fn with_counter_new(next_id : &mut u8, color : Color)
    -> Ant
    {
        let ant = Ant{
            id: next_id.clone(),
            color,
            state: State(0),
            resting: 0,
            direction: Dir::E,
            has_food: false,
        };
        *next_id = *next_id + 1;
        ant
    }
    pub fn new(color : Color) -> Ant {
        Ant{
            id: 0,
            color,
            state: State(0),
            resting: 0,
            direction: Dir::E,
            has_food: false,
        }
    }
}
