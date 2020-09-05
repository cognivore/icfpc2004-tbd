use num_traits::FromPrimitive;

use std::collections::HashMap;

use crate::geometry::{
    LR,
    Dir,
    SenseDir,
    Pos,
};

use crate::biology::{
    Color,
};

use crate::utils::{
    even,
};

// TODO: Abstract away
// ...
// TODO

pub fn simple_enum_iter<T: FromPrimitive>(n : i8) -> impl Iterator<Item=T> {
    (0..n).map(|x| FromPrimitive::from_i8(x).unwrap())
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

// Biology functions

pub fn other_color(c : Color) -> Color {
    match c {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    }
}

// ENTRY_POINT
pub fn prelude_entry_point() {
    println!("Hello from prelude")
}


pub struct Random(u32);

impl Random {
    pub fn next(&mut self, modulo: u32) -> u32 {
        self.0 = (self.0 as u64 * 22695477 + 1) as u32;
        ((self.0 >> 16) & 0x3FFF) % modulo
    }

    pub fn new(seed: u32) -> Self {
        let mut res = Self(seed);
        for _ in 0..3 {
            res.next(1);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng() {
        let expected = [7193, 2932, 10386, 5575, 100, 15976, 430, 9740, 9449, 1636, 11030, 9848, 13965, 16051, 14483, 6708,
        5184, 15931, 7014, 461, 11371, 5856, 2136, 9139, 1684, 15900, 10236, 13297, 1364, 6876, 15687,
        14127, 11387, 13469, 11860, 15589, 14209, 16327, 7024, 3297, 3120, 842, 12397, 9212, 5520, 4983,
        7205, 7193, 4883, 7712, 6732, 7006, 10241, 1012, 15227, 9910, 14119, 15124, 6010, 13191, 5820,
        14074, 5582, 5297, 10387, 4492, 14468, 7879, 8839, 12668, 5436, 8081, 4900, 10723, 10360, 1218,
        11923, 3870, 12071, 3574, 12232, 15592, 12909, 9711, 6638, 2488, 12725, 16145, 9746, 9053, 5881,
        3867, 10512, 4312, 8529, 1576, 15803, 5498, 12730, 7397];
        let mut rnd = Random::new(12345);
        for i in expected.iter() {
            assert_eq!(*i, rnd.next(0x3FFF));
        }
    }

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
