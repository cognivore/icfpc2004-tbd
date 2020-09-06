use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

use std::collections::HashMap;

use crate::prelude::{
    even,
    simple_enum_iter
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SenseDir {
    Here,
    Ahead,
    LeftAhead,
    RightAhead,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LR {
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Pos {
    pub x : u8,
    pub y : u8,
}

pub fn turn(lr : LR, dir : Dir) -> Dir {
    match lr {
        LR::Left => dir.cw(5),
        LR::Right => dir.cw(1),
    }
}

pub fn adj_unsafe(p : Pos, d : Dir) -> Pos {
    if let Some(a) = adj(p,d) {
        return a
    } else {
        panic!("No adjacent cell in that direction!");
    }

}

pub fn adj(Pos{x,y} : Pos, d : Dir) -> Option<Pos> {
    match d {
        Dir::E  => Pos{x: x+1, y: y}.inbound(),
        Dir::SE => if even(y) {Pos{x:x,y:y+1}.inbound()} else {Pos{x:x+1,y:y+1}.inbound()},
        Dir::SW => if even(y) {
            if x == 0 { None } else { Pos{x:x-1,y:y+1}.inbound() }
        } else { Pos{x:x,y:y+1}.inbound() },
        Dir::W  => if x == 0 { None } else { Pos{x:x-1,y:y}.inbound() },
        Dir::NW => if y == 0 { None } else {
            if even(y) {
                if x == 0 { None } else { Pos{x:x-1,y:y-1}.inbound() }
            } else { Pos{x:x,y:y-1}.inbound() }
        },
        Dir::NE => if y == 0 { None } else {
            if even(y) {
                Pos{x:x,y:y-1}.inbound()
            } else { Pos{x:x+1,y:y-1}.inbound() }
        },
    }
}

pub fn adjs_unsafe(p : Pos) -> HashMap<Dir, Pos> {
    let mut adjs_unsafe = HashMap::new();

    for (k,v) in adjs(p).iter() {
        if let Some(pos) = v {
            adjs_unsafe.insert(*k,*pos);
        } else {
            continue;
        }
    }

    adjs_unsafe
}

pub fn adjs(p : Pos) -> HashMap<Dir, Option<Pos>> {
    let mut adjs = HashMap::new();

    for d in simple_enum_iter::<Dir>(6) {
        adjs.insert(d,adj(p,d));
    }
    adjs
}

pub fn sensed_cell(p : Pos, d : Dir, sd : SenseDir) -> Option<Pos> {
    match sd {
        SenseDir::Here => Some(p),
        SenseDir::Ahead => adj(p,d),
        SenseDir::LeftAhead => adj(p,turn(LR::Left, d)),
        SenseDir::RightAhead => adj(p,turn(LR::Right, d)),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn() {
        assert_eq!(Dir::NE, turn(LR::Left,  Dir::E));
        assert_eq!(Dir::SE, turn(LR::Right, Dir::E));
        assert_ne!(Dir::W,  turn(LR::Right, Dir::E));
    }

    #[test]
    fn test_adj() {
        assert_eq!(Some(Pos{x:0,y:1}), adj(Pos{x:0,y:0}, Dir::SE));
        assert_ne!(Some(Pos{x:2,y:2}), adj(Pos{x:2,y:1}, Dir::NE));
        assert_eq!(None, adj(Pos{x:0,y:0}, Dir::NW));
    }

}

// ENTRY_POINT
pub fn geometry_entry_point() {
    println!("Hello from prelude")
}

