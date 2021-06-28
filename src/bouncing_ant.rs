#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use crate::magic::*;
use crate::neurology::{LR, SenseDir, SenseCondition};
use crate::{call, var};

fn turn_around() -> AntResult<()> {
    call!(turn(LR::Left));
    call!(turn(LR::Left));
    call!(turn(LR::Left));
    Ok(())
}

fn random_turn() -> AntResult<()> {
    if call!(flip(2)) {
        if call!(flip(2)) {
            call!(turn(LR::Left));
        } else {
            call!(turn(LR::Right));
        }
    } else {
        if call!(flip(2)) {
            call!(turn(LR::Left));
            call!(turn(LR::Left));
        } else {
            call!(turn(LR::Right));
            call!(turn(LR::Right));
        }
    }
    Ok(())
}

// Moves in straight line until it reaches an obstacle.
// Then rotates randomly.
// Picks up and drops off food when the chance arises,
// similar to the ant from spec.
fn bouncing_ant() -> AntResult<()> {
    var!(let mut has_food = false);
    loop {
        if has_food.get() {
            if call!(sense(SenseDir::Here, SenseCondition::Home)) {
                call!(drop());
                has_food.set(false);
                call!(turn_around());
            }
        } else {
            if call!(sense(SenseDir::Here, SenseCondition::Food)) {
                if call!(pickup()) {
                    has_food.set(true);
                    call!(turn_around());
                }
            }
        }
        if !call!(move_()) {
            call!(random_turn());
        }
    }
}

// ENTRY_POINT
pub fn make_bouncing_ant() {
    let brain = compile(bouncing_ant);
    brain.save_to_file("outputs/bouncing.ant");
    eprintln!("{:?}", brain);
    eprintln!();
}
