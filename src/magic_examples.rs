use crate::magic::*;
use crate::{call, var};

fn looping_ant() -> AntResult<()> {
    if call!(flip(3)) {
        call!(pickup());
    } else {
        call!(move_());
    }
    loop {
        for i in 0..3 {
            var!(let _i = i);
            call!(move_());
        }
        call!(drop());
    }
}

// ENTRY_POINT
pub fn magic_example() {
    eprintln!("{:?}", compile(looping_ant));
}
