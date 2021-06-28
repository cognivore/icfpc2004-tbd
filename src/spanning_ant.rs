#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use crate::magic::*;
use crate::phenomenology::Marker;
use crate::geometry::{self, Dir};
use crate::neurology::{LR, SenseDir, SenseCondition};
use crate::{call, var};

fn my_turn(lr: LR, orientation: Dir) -> AntResult<Dir> {
    call!(turn(lr));
    Ok(geometry::turn(lr, orientation))
}

fn random_turn(orientation: Dir) -> AntResult<Dir> {
    if call!(flip(2)) {
        if call!(flip(2)) {
            Ok(call!(my_turn(LR::Left, orientation)))
        } else {
            Ok(call!(my_turn(LR::Right, orientation)))
        }
    } else {
        if call!(flip(2)) {
            let o = call!(my_turn(LR::Left, orientation));
            Ok(call!(my_turn(LR::Left, o)))
        } else {
            let o = call!(my_turn(LR::Right, orientation));
            Ok(call!(my_turn(LR::Right, o)))
        }
    }
}

fn spanning_ant() -> AntResult<()> {
    var!(let mut orientation = Dir::E);
    loop {
        // no food
        loop {
            if !call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(0)))) &&
               !call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(1)))) &&
               !call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(2)))) {
                if (orientation.get() as i32 + 1) % 2 == 1 {
                    call!(mark(Marker(0)));
                }
                if (orientation.get() as i32 + 1) / 2 % 2 == 1 {
                    call!(mark(Marker(1)));
                }
                if (orientation.get() as i32 + 1) / 4 == 1 {
                    call!(mark(Marker(2)));
                }
            }
            if call!(sense(SenseDir::Here, SenseCondition::Food)) {
                if !call!(sense(SenseDir::Here, SenseCondition::Home)) {
                    if call!(pickup()) {                    
                        break;
                    }                    
                }
            }
            if !call!(move_()) {
                orientation.set(call!(random_turn(orientation.get())));
            }        
        }

        // have food
        loop {
            if call!(sense(SenseDir::Here, SenseCondition::Home)) {
                call!(drop());
                orientation.set(call!(my_turn(LR::Left, orientation.get())));
                orientation.set(call!(my_turn(LR::Left, orientation.get())));
                orientation.set(call!(my_turn(LR::Left, orientation.get())));
                break;
            }
            var!(let mut dir = -1 + 3);
            dir.set(dir.get() + call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(0)))) as i32);
            dir.set(dir.get() + call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(1)))) as i32 * 2);
            dir.set(dir.get() + call!(sense(SenseDir::Here, SenseCondition::Marker(Marker(2)))) as i32 * 4);
            let turns_right = (dir.get() + 6 - orientation.get() as i32) % 6;
            for i in 0..turns_right {
                var!(let _i = i);
                orientation.set(call!(my_turn(LR::Right, orientation.get())));
            }
            call!(move_());
        }
    }
}

// ENTRY_POINT
pub fn make_spanning_ant() {
    let brain = compile(spanning_ant);
    brain.save_to_file("outputs/spanning.ant");
    eprintln!("{:?}", brain);
    eprintln!();
}
