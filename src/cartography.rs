use nom::{
    IResult,
    multi::{
        many_m_n
    },
    bytes::{
        complete::{
            tag,
        }
    },
    character::{
        complete::{
            digit1,
        }
    },
    sequence::{
        delimited,
        separated_pair,
        terminated,
    },
};

use std::collections::HashMap;
use std::str::FromStr;

use crate::geography::{
    MapToken,
};

use crate::geometry::{
    Pos
};

pub struct World(pub HashMap<Pos, MapToken>);
impl World {
    pub fn new(x : u8, y : u8) -> World {
        let mut w = World(HashMap::new());
        for cx in 0..x {
            w.0.insert(Pos{ x: cx, y: 0   }, MapToken::Rock);
            w.0.insert(Pos{ x: cx, y: y-1 }, MapToken::Rock);
        }
        for cy in 1..(y - 1) {
            w.0.insert(Pos{ x: 0,   y: cy }, MapToken::Rock);
            w.0.insert(Pos{ x: x-1, y: cy }, MapToken::Rock);
        }
        w
    }

    pub fn parse<'w>(&'w mut self, map : &'w str) -> IResult<&'w str, World> {
        match separated_pair(
            digit1,
            tag("\n"),
            terminated(digit1, tag("\n"))
        )(map) {
            Err(e) => Err(e),
            Ok( (rest, (x, y)) ) => {
                match parse_world(
                    usize::from_str(x).unwrap(),
                    usize::from_str(y).unwrap(),
                    self,
                    rest
                ) {
                    Err(e1) => Err(e1),
                    ok => ok,
                }
            }
        }
    }
}

fn parse_world<'a>(x : usize, y : usize, mut w0 : &'a World, input : &'a str)
-> IResult<&'a str, World>
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
                w0,
                map_tokens
            )
        ))
    }
}

// It really doesn't matter much if we mutate World
// as we build it or copy it, so I left return type,
// but whoever implements can safely make it a method
// of World.
fn fold_map_tokens_y(
    mut a0 : &World,
    xs : Vec<Vec<MapToken>>
) -> World
{
    todo!()
    // fold(fold_map_tokens_x, ....) TODO
}

fn fold_map_tokens_x(
    x : MapToken,
    mut a0 : World
) -> World
{
    todo!()
}

fn rock(input : &str)
-> IResult<&str, MapToken>
{
    todo!()
}

fn map_line(input : &str)
-> IResult<&str, Vec<MapToken>>
{
    todo!()
}
