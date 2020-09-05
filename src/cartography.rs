// TODO: remove when it's implemented
#![allow(unused_imports, unused_variables, unused_mut)]

use nom::{
    IResult,
    branch::alt,
    multi::{
        many_m_n,
        many0,
    },
    bytes::{
        complete::{
            tag,
        }
    },
    character::{
        complete::{
            digit1,
            one_of,
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
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::geography::{
    MapToken,
    Contents,
    Food,
    Markers,
};

use crate::utils::*;

use crate::geography::MapToken::*;

use crate::biology::{
    Color,
    Ant,
};

use crate::biology::Color::*;

use crate::geometry::{
    Dir,
    Pos,
};

use crate::prelude::{
    adj,
    simple_enum_iter,
};

pub enum LookupError {
    HexOutOfBounds,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct World{
    pub x : u8, // usize is better here
    pub y : u8, // but I couldn't be fucked
    pub data : HashMap<Pos, MapToken>,
}
impl World {
    pub fn new() -> World {
        World{ x: 0, y: 0, data: HashMap::new() }
    }

    pub fn framed(x : u8, y : u8) -> World {
        let mut h : HashMap<Pos, MapToken> =
            HashMap::new();
        for cx in 0..x {
            h.insert(Pos{ x: cx, y: 0     }, Rock);
            h.insert(Pos{ x: cx, y: y - 1 }, Rock);
        }
        for cy in 1..y-1 {
            h.insert(Pos{ x: 0    , y: cy}, Rock);
            h.insert(Pos{ x: x - 1, y: cy}, Rock);
        }
        World{ x, y, data: h }
    }

    pub fn parse<'w>(map : &'w str) -> IResult<&'w str, World> {
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
                    World::framed(
                        u8::from_str(x).unwrap(),
                        u8::from_str(y).unwrap()
                    ),
                    rest
                ) {
                    Err(e1) => Err(e1),
                    Ok((rest1, w1)) => Ok((rest1, w1)),
                }
            }
        }
    }

    pub fn adj_feature(self, p : Pos, d : Dir)
        -> Result<MapToken, LookupError>
    {
        if let Some(a) = adj(p,d) {
            if let Some(token) = self.data.get(&a) {
                Ok(token.clone())
            } else {
                Err(LookupError::NotFound)
            }
        } else {
            Err(LookupError::HexOutOfBounds)
        }
    }

    pub fn adj_features(self, p : Pos)
        -> HashMap<Dir, Result<MapToken, LookupError>>
    {
        let mut res = HashMap::new();
        for d in simple_enum_iter::<Dir>(6) {
            res.insert(d, self.clone().adj_feature(p, d));
        }
        res
    }

}

impl Display for World {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let mut buff = "".to_string();
        for cy in 0..self.y {
            if !even(cy) {
                buff = format!("{} ", buff);
            }
            for cx in 0..self.x {
                let mt = self.data.get(&Pos{ x: cx, y: cy });
                buff = format!(
                    "{} {}",
                    buff,
                    p(mt)
                );
            }
            buff = format!("{}\n", buff);
        }
        write!(f, "{}", buff)
    }
}

fn parse_world<'a>(x : usize, y : usize, w0 : World, input : &'a str)
-> IResult<&'a str, World>
{
    //println!("{:?}", w0);
    match delimited(
        terminated(many_m_n(x, x, _rock_), tag("\n")),
        many_m_n(y-2, y-2, map_line(x)),
        many_m_n(x, x, _rock_)
    )(input) {
        Err(e) => Err(e),
        Ok((rest, map_tokens)) => {
            //println!("{:?}", map_tokens);
            Ok((
                rest,
                fold_map_tokens_y(
                    w0,
                    map_tokens
                )
            ))
        }
    }
}

//TODO!!
fn fold_map_tokens_y(
    a0 : World,
    xss : Vec<Vec<MapToken>>
) -> World
{
    let mut a1 = a0.clone();
    let mut cy = 0;
    for xs in xss {
        cy += 1;
        fold_map_tokens_x(&mut a1, xs, &cy);
    }
    a1
}

fn fold_map_tokens_x(
    a0 : &mut World,
    xs : Vec<MapToken>,
    y : &u8
) -> World
{
    let mut cx = 0;
    for x in xs {
        cx += 1;
        let p = Pos{ x: cx, y: y.clone() };
        a0.data.insert(p, x);
    }
    a0.clone()
}

fn map_line(x : usize)
-> impl Fn(&str)
-> IResult<&str, Vec<MapToken>>
{
    move |input| {
        terminated(
            delimited(
                _rock_,
                many_m_n(
                    x-2,
                    x-2,
                    alt_token
                ),
                _rock_
            ),
            tag("\n")
        )(input)
    }
}

fn alt_token(input : &str)
-> IResult<&str, MapToken>
{
    //let mut ants_counter = 0;
    delimited(
        many0(ws),
        alt((
            rock,
            clear,
            anthill(),
            food
        )),
        many0(ws)
    )(input)
}

fn _rock_(input : &str)
-> IResult<&str, MapToken>
{
    //println!("{}", input);
    delimited(many0(ws), rock, many0(ws))(input)
}
fn rock(input : &str)
-> IResult<&str, MapToken>
{
    //println!("{}", input);
    match tag("#")(input) {
        Err(e) => Err(e),
        Ok( (rest, _) ) => Ok( (rest, MapToken::Rock) ),
    }
}

fn clear(input : &str)
-> IResult<&str, MapToken>
{
    match tag(".")(input) {
        Err(e) => Err(e),
        Ok( (rest, _) ) => Ok ((
            rest,
            Clear(Contents::empty())
        ))
    }
}

fn anthill(/*ants_counter : &mut u8*/)
//-> impl FnMut(&str)
-> impl Fn(&str)
-> IResult<&str, MapToken>
{
    move |input| {
        match alt( (tag("+"), tag("-")) )(input) {
            Err(e) => Err(e),
            Ok( (rest, "+") ) => Ok ((
                rest,
                Clear(Contents{
                    /*
                    ant: Some(Ant::with_counter_new(
                        ants_counter,
                        Red
                    )),
                    */
                    ant: Some(Ant::new(Red)),
                    anthill: Some(Red),
                    food: Food(0),
                    markers: Markers::empty(),
                })
            )),
            Ok( (rest, "-") ) => Ok ((
                rest,
                Clear(Contents{
                    /*
                    ant: Some(Ant::with_counter_new(
                        ants_counter,
                        Black
                    )),
                    */
                    ant: Some(Ant::new(Black)),
                    anthill: Some(Black),
                    food: Food(0),
                    markers: Markers::empty(),
                })
            )),
            Ok(_) => unreachable!(),
        }
    }
}

fn food(input : &str)
-> IResult<&str, MapToken>
{
    match digit1(input) {
        Err(e) => Err(e),
        Ok( (rest, ds) ) => Ok ((
            rest,
            Clear(Contents{
                ant: None,
                anthill: None,
                food: Food(u8::from_str(ds).unwrap()),
                markers: Markers::empty(),
            })
        ))
    }
}

fn ws(input : &str) -> IResult<&str, char> {
    one_of(" \t")(input)
}

fn p(x : Option<&MapToken>) -> String {
    match x {
        None => "x".to_string(),
        Some(Rock) => "#".to_string(),
        Some(Clear(contents)) => pc(contents.clone()),
    }
}

fn pc(Contents{anthill, food: Food(fq), ..} : Contents) -> String {
    if fq > 0 {
        return fq.to_string();
    }
    match anthill {
        None => ".".to_string(),
        Some(Red) => "+".to_string(),
        Some(Black) => "-".to_string(),
    }
}

// ENTRY_POINT
pub fn cartography_manual_testing_entry_point() {
    use std::env;
    use std::fs;
    let world_txt = fs::read_to_string("data/tiny.world")
        .expect("File not found or is broken");
    match World::parse(world_txt.as_str()) {
        Err(e) => println!("Well, fuck"),
        Ok((_, w)) => println!("``Tiny world''\n{}", w),
    }
}

