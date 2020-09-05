use crate::geometry::{
    Dir
};

#[derive(Clone)]
pub enum Color {
    Red,
    Black,
}

#[derive(Clone)]
pub struct Ant {
    pub id : u8, // Every map contains just one anthill, so there should be up to 91 ants
    pub color : Color,
    pub state : u16,
    pub resting : u8,
    pub direction : Dir,
    pub has_food : bool,
}
