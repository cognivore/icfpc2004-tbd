/*
# Usage

Functions that directly or indirectly call ant IO (like move, sense)
should return AntResult<T> instead of T.
So AntResult is kind of like AntIO monad.

Such functions should always be invoked using call!() macro,
and not directly.
Don't use question mark.

All local variables should be wrapped in var!().
Use v.get() and v.set() to access them.

This is a hack so maybe there are some limitations that didn't occur to me.


# Explanation

The automaton builder invokes ant() multiple times,
each time making sense() return a different sequence of results.
In order for this process to terminate, it need to determine which
execution states are equivalent. We consider states equivant if they
have same call stacks and local variable values. That's why calls
and local variable manipulations need to be wrapped in a thing that
tracks them.

Suppose you have the following code:
    // BAD
    let food_here = call!(sense(Here, Food));
    let food_ahead = call!(sense(Ahead, Food));
    use food_here and food_ahead

The automaton builder can reach the second call!() in two ways
(by making the first call!() return either true or false).
But when it reaches it won't know that the program remembered this outcome
and it can affect its future behavior. So the generated automaton will have
a singe state in which it performs the second sense() instruction.
This is incorrect.

Instead, write
    // GOOD
    var!(let food_here = call!(sense(Here, Food)));
    var!(let food_ahead = call!(sense(Ahead, Food)));
    use food_here.get() and food_ahead.get()
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

fn unbounded_ant() -> AntResult<()> {
    var!(let mut i = 0);
    loop {
        call!(move_());
        i.set(i.get() + 1);
    }
}

// ENTRY_POINT
pub fn make_unbounded_ant() {
    compile(unbounded_ant);
}
