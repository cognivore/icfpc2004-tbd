use std::fmt;

pub use crate::geometry::{
    SenseDir,
    LR,
};

pub use crate::phenomenology::{
    SenseCondition,
    Marker,
};


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct State(pub u16);


impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
    pub fn transitions_mut(&mut self) -> impl Iterator<Item=&mut State> {
        let (st1, st2) = match self {
            Instruction::Sense(_, st1, st2, _) |
            Instruction::PickUp(st1, st2) |
            Instruction::Move(st1, st2) |
            Instruction::Flip(_, st1, st2)
                => (st1, Some(st2)),

            Instruction::Mark(_, st) |
            Instruction::Unmark(_, st) |
            Instruction::Drop(st) |
            Instruction::Turn(_, st)
                => (st, None),
        };
        std::iter::once(st1).chain(st2)
    }

    pub fn transitions(&self) -> impl Iterator<Item=&State> {
        let (st1, st2) = match self {
            Instruction::Sense(_, st1, st2, _) |
            Instruction::PickUp(st1, st2) |
            Instruction::Move(st1, st2) |
            Instruction::Flip(_, st1, st2)
                => (st1, Some(st2)),

            Instruction::Mark(_, st) |
            Instruction::Unmark(_, st) |
            Instruction::Drop(st) |
            Instruction::Turn(_, st)
                => (st, None),
        };
        std::iter::once(st1).chain(st2)
    }

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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            Sense(dir, state1, state2, cond) => write!(f, "Sense {} {} {} {}", dir, state1, state2, cond),
            Mark(marker, state) => write!(f, "Mark {} {}", marker, state),
            Unmark(marker, state) => write!(f, "Unmark {} {}", marker, state),
            PickUp(state1, state2) => write!(f, "PickUp {} {}", state1, state2),
            Drop(state) => write!(f, "Drop {}", state),
            Turn(lr, state) => write!(f, "Turn {} {}", lr, state),
            Move(state1, state2) => write!(f, "Move {} {}", state1, state2),
            Flip(n, state1, state2) => write!(f, "Flip {} {} {}", n, state1, state2),
        }
    }
}


pub fn parse_ant(s: &str) -> Vec<Instruction> {
    s.split_terminator('\n').map(Instruction::parse).collect()
}


pub fn dumps(insns: &Vec<Instruction>) -> String {
    let mut res: String = String::new();
    for insn in insns.iter() {
        res.push_str(&insn.to_string());
        res.push_str("\n")
    }
    res
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_insn_test() {
        assert_eq!(
            Instruction::parse(" droP  42  ; zzz"),
            Instruction::Drop(State(42)))
    }

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

    #[test]
    fn dumps_roundtrip_test() {
        let s = std::fs::read_to_string("data/sample.ant").unwrap();
        let ant = parse_ant(&s);
        let roundtrip = dumps(&ant);
        for (a, b) in s.split("\n").zip(roundtrip.split("\n")) {
            assert_eq!(a, b);
        }
    }
}
