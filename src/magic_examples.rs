/*
Functions that directly or indirectly call ant IO (like move, sense)
should return AntResult<T> instead of T.
So AntResult is kind of like AntIO monad.

Such functions should always be invoked using call!() macro,
and not directly.
Don't use question mark.

All local variables should be wrapped in var!().
Use v.get() and v.set() to access them.

This is a hack so maybe there are some limitations that didn't occur to me.
*/

use crate::magic::*;
use crate::neurology::LR;
use crate::{call, var};

fn move_n(n: i32) -> AntResult<()> {
    for i in 0..n {
        var!(let _i = i);
        call!(move_());
    }
    Ok(())
}

fn looping_ant() -> AntResult<()> {
    if call!(flip(3)) {
        call!(pickup());
    } else {
        call!(move_());
    }
    loop {
        call!(move_n(3));
        call!(drop());
    }
}

fn spiral_ant() -> AntResult<()> {
    var!(let mut k = 1);

    while k.get() <= 4 {
        call!(move_n(k.get()));
        call!(turn(LR::Left));
        k.set(k.get() * 2);
    }

    loop {
        call!(drop());
    }
}

// ENTRY_POINT
pub fn magic_example() {
    eprintln!("looping ant");
    let brain = compile(looping_ant);
    brain.save_to_file("outputs/looping.ant");
    eprintln!("{:?}", brain);
    eprintln!();

    eprintln!("spiral ant");
    let brain = compile(spiral_ant);
    brain.save_to_file("outputs/spiral.ant");
    eprintln!("{:?}", brain);
    eprintln!();
}
