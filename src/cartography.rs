use nom::{
    IResult,
    multi::{many_m_n},
    sequence::{pair, preceded, delimited},
};

use std::collections::HashMap;

use crate::geography::{
    MapToken,
};

use crate::geometry::{
    Pos
};

pub struct World(pub HashMap<Pos, MapToken>);
impl World {
    pub fn new() -> World {
        World(HashMap::new())
    }
}

pub fn parse_world(x : usize, y : usize, input : &str)
-> IResult<&str, World>
{
    match delimited(
        many_m_n(x, x, rock),
        many_m_n(y, y, map_line),
        many_m_n(x, x, rock)
    )(input) {
        Err(e) => Err(e),
        Ok((rest, map_tokens)) => Ok((
            rest,
            fold_map_tokens_y(
                World::new(),
                map_tokens
            )
        ))
    }
}

// It really doesn't matter much if we mutate World
// as we build it or copy it, so I left return type,
// but whoever implements can safely make it a method
// of World.
pub fn fold_map_tokens_y(
    mut a0 : World,
    xs : Vec<Vec<MapToken>>
) -> World
{
    todo!()
    // fold(fold_map_tokens_x, ....) TODO
}

pub fn fold_map_tokens_x(
    x : MapToken,
    mut a0 : World
) -> World
{
    todo!()
}

pub fn rock(input : &str)
-> IResult<&str, MapToken>
{
    todo!()
}

pub fn map_line(input : &str)
-> IResult<&str, Vec<MapToken>>
{
    todo!()
}
