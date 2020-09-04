use num_traits::FromPrimitive;
use num_derive::FromPrimitive;


use crate::utils::{
    even,
};

#[derive(Debug, FromPrimitive, Eq, PartialEq, Hash, Copy, Clone)]
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x : u8,
    pub y : u8,
}

impl Pos {
    pub fn distance(self, them : Pos) -> u8 {
        // max(|dy|, |dx| + floor(|dy|/2) + offset)
        // offset = ( (even(y1) && odd(y2) && (x1 < x2)) || (even(y2) && odd(y1) && (x2 < x1)) ) ? 1 : 0
        let dx = i16::from(them.x - self.x).abs() as u8;
        let dy = i16::from(them.y - self.y).abs() as u8;
        let offset =
            if (even(self.y) && !even(them.y) && self.x < them.x) ||
               (even(them.y) && !even(self.y) && them.x < self.x) { 1 } else { 0 };
        std::cmp::max(dy, dx + dy/2 + offset)
    }

    // assuming that the world is always 100x100, as per spec
    pub fn out_of_bounds(self) -> bool {
        self.x > 99 || self.y > 99
    }

    pub fn inbound(self) -> Option<Pos> {
        if !self.out_of_bounds() {
            Some(self)
        } else {
            None
        }
    }
}

impl Dir {

    pub fn cw(self, offset : u8) -> Dir {
        match FromPrimitive::from_u8(((self as u8) + offset) % 6) {
            Some(dir) => dir,
            None      => unreachable!()
        }
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
