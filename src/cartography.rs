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
use std::time::Instant;

use crate::biology::{
    Color,
    Ant,
    other_color,
};

use crate::biology::Color::*;

use crate::geography::{
    MapToken,
    Contents,
    Food,
    Markers,
};

use crate::geography::MapToken::{
    Clear
};

use crate::geometry::{
    Dir,
    Pos,
    adj,
    sensed_cell,
    turn,
};

use crate::neurology::{
    Instruction,
    parse_ant
};

use crate::neurology::Instruction::*;

use crate::number_theory::{
    Random,
};

use crate::phenomenology::{
    Marker,
    SenseCondition,
};

use crate::phenomenology::SenseCondition::*;

use crate::prelude::{
    simple_enum_iter,
    even,
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
    pub ant_positions : HashMap<u8, Pos>,
}
impl World {
    pub fn round(&mut self, ant_brains : &[Vec<Instruction>; 2], rng : &mut Random) {
        // there are two anthills, 91 ants max each
        for id in 0..182 {
            self.step(id, ant_brains, rng);
        }
    }

    pub fn step(&mut self, id : u8, ant_brains : &[Vec<Instruction>; 2], rng : &mut Random) {
        if let Some(pos) = self.find_ant(id) {
            if let Some(ant) = self.ant_at(pos) {
                if ant.resting > 0 {
                    self.set_ant_at(pos, Ant { resting : ant.resting-1, ..ant })
                } else {
                    match ant_brains[ant.color as usize][ant.state.0 as usize] {
                        Sense(sdir, s1, s2, cond) => {
                            if let Some(sensed_pos) = sensed_cell(pos, ant.direction, sdir) {
                                let state = if self.cell_matches(sensed_pos, cond, ant.color) { s1 } else { s2 };
                                self.set_ant_at(pos, Ant { state, ..ant });
                            }
                        },
                        Mark(i, state) => {
                            self.set_marker_at(pos, ant.color, i);
                            self.set_ant_at(pos, Ant { state, ..ant });
                        },
                        Unmark(i, state) => {
                            self.clear_marker_at(pos, ant.color, i);
                            self.set_ant_at(pos, Ant { state, ..ant });
                        },
                        PickUp(s1, s2) => {
                            let food_amount = self.food_at(pos).0;
                            if ant.has_food || food_amount == 0 {
                                self.set_ant_at(pos, Ant { state : s2, ..ant });
                            } else {
                                self.set_food_at(pos, Food(food_amount-1));
                                self.set_ant_at(pos, Ant { state : s1, has_food : true, ..ant });
                            }
                        },
                        Drop(state) => {
                            if ant.has_food {
                                let food_amount = self.food_at(pos).0;
                                self.set_food_at(pos, Food(food_amount + 1));
                            }
                            self.set_ant_at(pos, Ant { state, has_food : false, ..ant });
                        },
                        Turn(lr, state) => {
                            self.set_ant_at(pos, Ant { state, direction : turn(lr, ant.direction), ..ant });
                        },
                        Move(s1,s2) => {
                            if let Some(new_pos) = adj(pos, ant.direction) {
                                if self.rocky(new_pos) || self.some_ant_is_at(new_pos) {
                                    self.set_ant_at(pos, Ant { state : s2, ..ant });
                                } else {
                                    self.clear_ant_at(pos);
                                    self.set_ant_at(new_pos, Ant { state : s1, resting : 14, ..ant });
                                    self.ant_positions.insert(ant.id, new_pos);
                                    self.check_for_surrounded_ants(new_pos);
                                }
                            } else {
                                self.set_ant_at(pos, Ant { state : s2, ..ant });
                            }
                        },
                        Flip(n, s1, s2) => {
                            let state = if rng.next(n.into()) == 0 { s1 } else { s2 };
                            self.set_ant_at(pos, Ant { state, ..ant });
                        },
                    }
                }
            }
        }
    }

    pub fn cell_matches(&self, p : Pos, cond : SenseCondition, c : Color) -> bool {
        match cond {
            Friend => {
                if let Some(ant) = self.ant_at(p) {
                    return ant.color == c
                }
            },
            Foe => {
                if let Some(ant) = self.ant_at(p) {
                    return ant.color != c
                }
            },
            FriendWithFood => {
                if let Some(ant) = self.ant_at(p) {
                    return ant.color == c && ant.has_food
                }
            },
            FoeWithFood => {
                if let Some(ant) = self.ant_at(p) {
                    return ant.color != c && ant.has_food
                }
            },
            SenseCondition::Food => { return self.food_at(p).0 > 0 },
            Rock => { return self.rocky(p) },
            SenseCondition::Marker(i) => { return self.check_marker_at(p, c, i) },
            FoeMarker => { return self.check_any_marker_at(p, other_color(c)) },
            Home => { return self.anthill_at(p,c) },
            FoeHome => { return self.anthill_at(p,other_color(c)) },
        }
        false
    }

    pub fn new() -> World {
        World{ x: 0, y: 0, data: HashMap::new(), ant_positions: HashMap::new() }
    }

    pub fn framed(x : u8, y : u8) -> World {
        let mut h : HashMap<Pos, MapToken> =
            HashMap::new();
        for cx in 0..x {
            h.insert(Pos{ x: cx, y: 0     }, MapToken::Rock);
            h.insert(Pos{ x: cx, y: y - 1 }, MapToken::Rock);
        }
        for cy in 1..y-1 {
            h.insert(Pos{ x: 0    , y: cy}, MapToken::Rock);
            h.insert(Pos{ x: x - 1, y: cy}, MapToken::Rock);
        }
        World{ x, y, data: h, ant_positions: HashMap::new() }
    }

    pub fn from_map_string(map: &str) -> World {
        let (rest, world) = World::parse(map).unwrap();
        assert_eq!(rest, "\n");
        world
    }

    fn parse<'w>(map : &'w str) -> IResult<&'w str, World> {
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

    pub fn feature(self, p : Pos) -> Result<MapToken, LookupError> {
        if let Some(token) = self.data.get(&p) {
            Ok(token.clone())
        } else {
            Err(LookupError::NotFound)
        }
    }

    pub fn adj_feature(&self, p : Pos, d : Dir)
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

    pub fn adj_features(&self, p : Pos)
        -> HashMap<Dir, Result<MapToken, LookupError>>
    {
        let mut res = HashMap::new();
        for d in simple_enum_iter::<Dir>(6) {
            res.insert(d, self.adj_feature(p, d));
        }
        res
    }

    pub fn surrounding_ants_amount(&self, p : Pos, c : Color) -> u8 {
        let mut count = 0;
        let adj_cells = self.adj_features(p);
        for a in adj_cells.values() {
            if let Ok(Clear(Contents { ant : Some(ant), .. } )) = a {
                if ant.color == c {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn check_for_surrounded_ant_at(&mut self, p : Pos) {
        if let Some(ant) = self.ant_at(p) {
            if self.surrounding_ants_amount(p, other_color(ant.color)) >= 5 {
                self.clear_ant_at(p);
                self.ant_positions.remove(&ant.id);
                let food_amount = self.food_at(p).0;
                self.set_food_at(p, Food(food_amount + 3 + if ant.has_food { 1 } else { 0 } ));
            }
        }
    }

    pub fn check_for_surrounded_ants(&mut self, p : Pos) {
        self.check_for_surrounded_ant_at(p);
        for d in simple_enum_iter::<Dir>(6) {
            if let Some(a) = adj(p,d) {
                self.check_for_surrounded_ant_at(a);
            }
        }
    }

    //Accessor functions
    pub fn rocky(&self, p : Pos) -> bool {
        if let Some(MapToken::Rock) = self.data.get(&p) {
            return true;
        }
        false
    }

    pub fn anthill_at(&self, p : Pos, c : Color) -> bool {
        if let Some(Clear(cont)) = self.data.get(&p) {
            return cont.anthill == Some(c);
        }
        false
    }

    pub fn check_marker_at(&self, p : Pos, c : Color, m : Marker) -> bool {
        if let Some(Clear(Contents { markers, .. } )) = self.data.get(&p) {
            if let Some(cms) = markers.0.get(&c) {
                return cms.get(m.0);
            }
        }
        false
    }

    pub fn check_any_marker_at(&self, p : Pos, c : Color) -> bool {
        if let Some(Clear(Contents { markers, .. } )) = self.data.get(&p) {
            if let Some(cms) = markers.0.get(&c) {
                return !cms.is_empty();
            }
        }
        false
    }

    pub fn some_ant_is_at(&self, p : Pos) -> bool {
        if let Some(Clear(Contents { ant : Some(_), .. } )) = self.data.get(&p) {
            return true;
        }
        false
    }

    pub fn ant_at(&self, p : Pos) -> Option<Ant> {
        if let Some(Clear(cont)) = self.data.get(&p) {
            return cont.ant.clone();
        }
        None
    }

    pub fn food_at(&self, p : Pos) -> Food {
        if let Some(Clear(cont)) = self.data.get(&p) {
            return cont.food;
        }
        Food(0)
    }

    pub fn food_at_anthill(&self, c : Color) -> Food {
        let mut foodn = 0;
        for v in self.data.values() {
            if let Clear(Contents { anthill : Some(color), food, .. }) = v {
                if c == *color {
                    foodn += food.0;
                }
            }
        }
        Food(foodn)
    }

    pub fn set_ant_at(&mut self, p : Pos, a : Ant) {
        if let Some(Clear(t)) = self.data.get_mut(&p) {
            t.ant = Some(a);
        }
    }

    pub fn clear_ant_at(&mut self, p : Pos) {
        if let Some(Clear(t)) = self.data.get_mut(&p) {
            t.ant = None;
        }
    }

    pub fn set_food_at(&mut self, p : Pos, f : Food) {
        if let Some(Clear(t)) = self.data.get_mut(&p) {
            t.food = f;
        }
    }

    pub fn set_marker_at(&mut self, p : Pos, c : Color, m : Marker) {
        if let Some(Clear(Contents { markers, .. } )) = self.data.get_mut(&p) {
            if let Some(cms) = markers.0.get_mut(&c) {
                cms.set(m.0, true);
            }
        }
    }

    pub fn clear_marker_at(&mut self, p : Pos, c : Color, m : Marker) {
        if let Some(Clear(Contents { markers, .. } )) = self.data.get_mut(&p) {
            if let Some(cms) = markers.0.get_mut(&c) {
                cms.set(m.0, false);
            }
        }
    }

    pub fn ant_is_alive(&self, id : u8) -> bool {
        self.find_ant(id).is_some()
    }

    pub fn find_ant(&self, id : u8) -> Option<Pos> {
        if let Some(pos) = self.ant_positions.get(&id) {
            Some(*pos)
        } else {
            None
        }
    }

    pub fn ant_by_id(self, id : u8) -> Option<Ant> {
        if let Some(pos) = self.find_ant(id) {
            self.ant_at(pos)
        } else {
            None
        }
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

// Parse functions
//------------------------------------------------------------------

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
            let mut ants_counter : u8 = 0;
            Ok((
                rest,
                fold_map_tokens_y(
                    w0,
                    map_tokens,
                    &mut ants_counter
                )
            ))
        }
    }
}

fn fold_map_tokens_y(
    a0 : World,
    xss : Vec<Vec<MapToken>>,
    ants_counter : &mut u8
) -> World
{
    let mut a1 = a0;
    let mut cy = 0;
    for xs in xss {
        cy += 1;
        fold_map_tokens_x(&mut a1, xs, &cy, ants_counter);
    }
    a1
}

fn fold_map_tokens_x(
    a0 : &mut World,
    xs : Vec<MapToken>,
    y : &u8,
    ants_counter : &mut u8
)
{
    let mut cx = 0;
    for x in xs {
        cx += 1;
        let p = Pos{ x: cx, y: *y };
        if let Clear(xx) = x {
            if let Contents{ant : Some(a), ..} = xx {
                let ant_with_id = Contents { ant : Some( Ant { id : *ants_counter, ..a } ),
                                             ..xx };
                a0.data.insert(p, Clear(ant_with_id));
                a0.ant_positions.insert(*ants_counter, p);
                *ants_counter += 1;
            } else {
                a0.data.insert(p, Clear(xx));
            }
        } else {
            a0.data.insert(p, x);
        }
    }
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
                food: Food(ds.parse().unwrap()),
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
        Some(MapToken::Rock) => "#".to_string(),
        Some(Clear(contents)) => pc(contents),
    }
}

fn pc(Contents{anthill, food: Food(fq), ..} : &Contents) -> String {
    if *fq > 0 {
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
    use std::fs;
    let w = fs::read_to_string("data/tiny.world")
        .expect("File not found or is broken");
    let w = World::from_map_string(&w);
    println!("``Tiny world``:\n{}", w);
}

// ENTRY_POINT
pub fn cartography_time_workload() {
    use std::fs;
    let w = fs::read_to_string("data/sample0.world")
        .expect("File not found or is broken");
    let mut w = World::from_map_string(&w);
    let ant_brains = [
        parse_ant(&std::fs::read_to_string("data/sample.ant").unwrap()),
        parse_ant(&std::fs::read_to_string("data/example_from_spec.ant").unwrap()),
    ];
    let mut rng = Random::new(12345);
    // full match
    let time = Instant::now();
    for _ in 0..100000 {
        w.round(&ant_brains, &mut rng);
    }
    println!("Elapsed time for full match: {:?}", time.elapsed());
    //println!("``Sample0 world after 1 round``:\n{}", w);
}
