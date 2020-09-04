use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

use std::collections::HashMap;

use crate::geometry::{
    LR,
    Dir,
    Pos,
};

// TODO: Abstract away
pub enum MapToken {}
pub enum Ant {}
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
    -> Result<(MapToken, Option<Ant>), LookupError>
{
    todo!()
}

pub fn adj_features(Pos{x, y} : Pos)
    -> HashMap<Dir, Result<(MapToken, Option<Ant>), LookupError>>
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

// ENTRY_POINT
pub fn prelude_entry_point() {
    println!("Hello from prelude")
}
