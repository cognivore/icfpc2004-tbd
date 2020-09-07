use crate::biology::Color;
use crate::biology::Color::*;

use crate::cartography::{
    World,
};

use crate::neurology::{
    parse_ant,
    Instruction,
};

use crate::number_theory::{
    Random,
};

pub struct MatchScore(usize, usize);

impl MatchScore {

    pub fn new() -> Self {
        Self(0,0)
    }

}

pub fn full_match(world : &mut World, ant_brains : &[Vec<Instruction>; 2], rng : &mut Random) -> Option<Color> {
    for _ in 0..100000 {
        world.round(ant_brains, rng);
    }
    let red_score = world.food_at_anthill(Red).0;
    let black_score = world.food_at_anthill(Black).0;
    if red_score > black_score {
        Some(Red)
    } else if red_score < black_score {
        Some(Black)
    } else {
        None
    }
}

// returns (red score, black score)
pub fn match_pair(score : &mut MatchScore, world : &mut World,
                  ant_brains : &mut [Vec<Instruction>; 2], rng : &mut Random) {

    let mut world_copy = world.clone();
    let mut rng_copy = rng.clone();

    let first_res = full_match(world, ant_brains, rng);
    match first_res {
        Some(Red) => { score.0 += 2; },
        Some(Black) => { score.1 += 2; },
        None => { score.0 += 1; score.1 += 1; },
    }
    ant_brains.reverse();
    let second_res = full_match(&mut world_copy, ant_brains, &mut rng_copy);
    match second_res {
        Some(Red) => { score.1 += 2; },
        Some(Black) => { score.0 += 2; },
        None => { score.0 += 1; score.1 += 1; },
    }


}

// ENTRY_POINT
pub fn tournament_ep() {
    use std::fs;
    let mut worlds = Vec::new();
    let mut seeds = vec![12345, 98765, 3566235, 375688, 864532, 42, 563845, 2071995, 8673, 35481];
    for p in std::fs::read_dir("data").unwrap() {
        let p = p.unwrap().path();
        //println!("{:?}", p);
        let pathstr = p.to_str().unwrap().to_string();
        if p.extension().unwrap() == "world" && p.file_stem().unwrap().to_str().unwrap() != "tiny" {
            worlds.push(pathstr);
        }
    }

    let mut sum_score = MatchScore::new();

    let ant1 = std::env::args().nth(2).unwrap_or("example_from_spec".to_string());
    let ant2 = std::env::args().nth(3).unwrap_or("example_from_spec".to_string());

    for wpath in worlds {
        let w = fs::read_to_string(wpath.as_str())
            .expect("File not found or is broken");
        let mut w = World::from_map_string(&w);
        let mut ant_brains = [
            parse_ant(&std::fs::read_to_string(format!("data/{}.ant", ant1)).unwrap()),
            parse_ant(&std::fs::read_to_string(format!("data/{}.ant", ant2)).unwrap()),
        ];
        let mut rng = Random::new(seeds.pop().unwrap());

        let mut wscore = MatchScore::new();

        match_pair(&mut wscore, &mut w, &mut ant_brains, &mut rng);

        println!("Score on {} - {} {} : {} {}", wpath, ant1, wscore.0, wscore.1, ant2);

        sum_score.0 += wscore.0;
        sum_score.1 += wscore.1;
    }

    println!("Final scores: {} {}, {} {}", ant1, sum_score.0, ant2, sum_score.1);

}
