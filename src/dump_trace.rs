use crate::cartography::{
    World,
};

use crate::geography::{
    MapToken,
};

use crate::geometry::{
    Pos
};

use crate::biology::Color::*;
use crate::geography::MapToken::*;

pub fn dump_world(world : World, count : usize) {
    println!("After round {}...", count);
    let mut v: Vec<_> = world.data.into_iter().collect();
    v.sort_by(|(Pos{x,y},_),(Pos{x : a,y : b},_)| (Pos{x : *y,y : *x}).cmp(&(Pos{x : *b,y : *a})));
    for (Pos{x,y},v) in v {
        println!("cell ({}, {}): {}", x,y, pp(v));
    }
    println!();
}

fn pp(t : MapToken) -> String {
    match t {
        Rock => "rock".to_string(),
        Clear(cont) => {
            let mut res = String::new();
            if cont.food.0 > 0 {
                res = format!("{} food; ",cont.food.0);
            }
            match cont.anthill {
                Some(Black) => {res.push_str("black hill; ");}
                Some(Red) => {res.push_str("red hill; ");}
                _ => {}
            }
            if let Some(rm) = cont.markers.0.get(&Red) {
                if !rm.is_empty() {
                    res.push_str("red marks: ");
                    let bits : Vec<usize> = rm.into_iter().collect();
                    for b in bits {
                        res.push_str(&b.to_string());
                    }
                    res.push_str("; ");
                }
            }
            if let Some(rm) = cont.markers.0.get(&Black) {
                if !rm.is_empty() {
                    res.push_str("black marks: ");
                    let bits : Vec<usize> = rm.into_iter().collect();
                    for b in bits {
                        res.push_str(&b.to_string());
                    }
                    res.push_str("; ");
                }
            }
            if let Some(ant) = cont.ant {
                match ant.color {
                    Red => {res.push_str("red");}
                    Black => {res.push_str("black");}
                }
                let antfood = if ant.has_food { 1 } else { 0 };
                res = format!("{} ant of id {}, dir {}, food {}, state {}, resting {}",
                                res, ant.id, ant.direction as usize, antfood, ant.state.0, ant.resting);
            }
            res
        },
    }
}

// ENTRY_POINT
pub fn dump_ep() {
    println!("hello world");
}
