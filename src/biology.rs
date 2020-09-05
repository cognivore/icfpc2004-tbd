use crate::geometry::{
    Dir
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
    pub state : u16,
    pub resting : u8,
    pub direction : Dir,
    pub has_food : bool,
}

impl Ant {
    pub fn with_counter_new(next_id : &mut u8, color : Color)
    -> Ant
    {
        let ant = Ant{
            id: next_id.clone(),
            color,
            state: 0,
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
            state: 0,
            resting: 0,
            direction: Dir::E,
            has_food: false,
        }
    }
}
