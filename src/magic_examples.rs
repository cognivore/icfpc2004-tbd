use crate::magic::*;
use crate::{call, var};

fn looping_ant() -> AntResult<()> {
    loop {
        for i in 0..3 {
            var!(let _i = i);
            call!(drop());
        }
        call!(drop());
    }
}

// ENTRY_POINT
pub fn magic_example() {
    for (i, (insn, comment)) in traverse(looping_ant).into_iter().enumerate() {
        println!("{:>4}   {:?}    # {}", i, insn, comment);
    }
}
