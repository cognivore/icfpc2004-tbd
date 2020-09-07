pub use crate::geometry::{
    SenseDir,
    LR,
};

pub use crate::phenomenology::{
    SenseCondition,
    Marker,
};


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State(pub u16);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    Sense(SenseDir, State, State, SenseCondition),
    Mark(Marker, State),
    Unmark(Marker, State),
    PickUp(State, State),
    Drop(State),
    Turn(LR, State),
    Move(State, State),
    Flip(u16, State, State),
}

impl Instruction {
    pub fn parse(s: &str) -> Self {
        let end = s.find(';').unwrap_or_else(|| s.len());
        let s = &s[..end];
        let mut it = s.split_whitespace().map(str::to_lowercase);
        let cmd = it.next().unwrap();
        let res = match cmd.as_str() {
            "sense" => {
                let sense_dir = match it.next().unwrap().as_str() {
                    "here" => SenseDir::Here,
                    "ahead" => SenseDir::Ahead,
                    "leftahead" => SenseDir::LeftAhead,
                    "rightahead" => SenseDir::RightAhead,
                    _ => panic!("{:?}", s),
                };
                let st1 = State(it.next().unwrap().parse().unwrap());
                let st2 = State(it.next().unwrap().parse().unwrap());
                let cond = match it.next().unwrap().as_str() {
                    "friend" => SenseCondition::Friend,
                    "foe" => SenseCondition::Foe,
                    "friendwithfood" => SenseCondition::FriendWithFood,
                    "foewithfood" => SenseCondition::FoeWithFood,
                    "food" => SenseCondition::Food,
                    "rock" => SenseCondition::Rock,
                    "marker" => SenseCondition::Marker(
                        Marker::new(it.next().unwrap().parse().unwrap())),
                    "foemarker" => SenseCondition::FoeMarker,
                    "home" => SenseCondition::Home,
                    "foehome" => SenseCondition::FoeHome,
                    _ => panic!("{:?}", s),
                };
                Instruction::Sense(sense_dir, st1, st2, cond)
            }
            "mark" | "unmark" => {
                let marker = Marker::new(it.next().unwrap().parse().unwrap());
                let st = State(it.next().unwrap().parse().unwrap());
                if cmd == "mark" {
                    Instruction::Mark(marker, st)
                } else {
                    Instruction::Unmark(marker, st)
                }
            }
            "pickup" => {
                let st1 = State(it.next().unwrap().parse().unwrap());
                let st2 = State(it.next().unwrap().parse().unwrap());
                Instruction::PickUp(st1, st2)
            }
            "drop" => Instruction::Drop(State(it.next().unwrap().parse().unwrap())),
            "turn" => {
                let lr = match it.next().unwrap().as_str() {
                    "left" => LR::Left,
                    "right" => LR::Right,
                    _ => panic!("{:?}", s),
                };
                let st = State(it.next().unwrap().parse().unwrap());
                Instruction::Turn(lr, st)
            }
            "move" => {
                let st1 = State(it.next().unwrap().parse().unwrap());
                let st2 = State(it.next().unwrap().parse().unwrap());
                Instruction::Move(st1, st2)
            }
            "flip" => {
                let p = it.next().unwrap().parse().unwrap();
                let st1 = State(it.next().unwrap().parse().unwrap());
                let st2 = State(it.next().unwrap().parse().unwrap());
                Instruction::Flip(p, st1, st2)
            }
            _ => panic!("{:?}", s),
        };
        assert!(it.next().is_none(), "{:?}", s);
        res
    }
}

pub fn parse_ant(s: &str) -> Vec<Instruction> {
    s.split_terminator('\n').map(Instruction::parse).collect()
}

#[cfg(test)]
#[test]
fn parse_insn_test() {
    assert_eq!(
        Instruction::parse(" droP  42  ; zzz"),
        Instruction::Drop(State(42)))
}

#[cfg(test)]
#[test]
fn parse_ant_test() {
    let s = std::fs::read_to_string("data/example_from_spec.ant").unwrap();
    let ant = parse_ant(&s);
    for insn in ant {
        eprintln!("{:?}", insn);
    }
    eprintln!("---");

    let s = std::fs::read_to_string("data/sample.ant").unwrap();
    let ant = parse_ant(&s);
    for insn in ant {
        eprintln!("{:?}", insn);
    }
}
