use crate::magic::*;
use crate::call;

fn looping_ant() -> AntResult<()> {
    loop {
        call!(drop());
        call!(drop());
    }
}

// ENTRY_POINT
pub fn magic_example() {
    for (i, (insn, comment)) in traverse(looping_ant).into_iter().enumerate() {
        println!("{:>4}   {:?}    # {}", i, insn, comment);
    }
}
