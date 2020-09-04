use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

use std::collections::HashMap;

use crate::geometry::{
    LR,
    Dir,
    Pos,
};

use crate::geography::{
    MapToken,
};

use crate::biology::{
    Color,
};

// TODO: Abstract away
// ...
// TODO

pub fn simple_enum_iter<T: FromPrimitive>(n : i8) -> impl Iterator<Item=T> {
    (0..n).map(|x| FromPrimitive::from_i8(x).unwrap())
}

pub fn turn(lr : LR, dir : Dir) -> Dir {
    todo!()
}

pub fn adj_unsafe(Pos{x, y} : Pos, d : Dir) -> Pos {
    todo!()
}

pub fn adj(p : Pos, d : Dir) -> Option<Pos> {
    todo!()
}

pub fn adjs_unsafe(Pos{x, y} : Pos) -> HashMap<Dir, Pos> {
    todo!()
}

pub fn adjs(Pos{x, y} : Pos) -> HashMap<Dir, Option<Pos>> {
    todo!()
}

pub fn adj_feature(Pos{x, y} : Pos, d : Dir)
    -> Result<MapToken, LookupError>
{
    todo!()
}

pub fn adj_features(Pos{x, y} : Pos)
    -> HashMap<Dir, Result<MapToken, LookupError>>
{
    todo!()
}

pub fn even< I : std::ops::BitAnd<Output = I> +
                 PartialEq >
           (x : I) -> bool {
    todo!()
}

pub enum LookupError {
    HexOutOfBounds,
    NotFound,
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
}
