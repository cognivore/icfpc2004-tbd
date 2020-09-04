// TODO: remove when it's implemented
#![allow(unused_imports, unused_variables, unused_mut)]

pub enum Dir {
    E,
    SE,
    SW,
    W,
    NW,
    NE,
}

pub enum SenseDir {
    Here,
    Ahead,
    LeftAhead,
    RightAhead,
}

pub enum LR {
    Left,
    Right,
}

pub struct Pos {
    pub x : u8,
    pub y : u8,
}

impl Pos {
    pub fn distance(self, them : Pos) -> u8 {
        todo!()
    }
}

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
    -> std::fmt::Result
    {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl std::ops::SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Pos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
