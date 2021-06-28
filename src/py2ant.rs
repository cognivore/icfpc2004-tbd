use std::path::Path;
use std::convert::TryInto;
use std::collections::HashMap;
use crate::neurology::{State, Instruction};
use crate::neurology;
use crate::py::compiler::*;
use crate::py::vm::{Status, Value};
use crate::py::vm;

struct BrainWithSymbols {
    insns: Vec<Instruction>,
}

fn outputs_to_insn_and_transitions(outputs: &[Value]) -> (Instruction, Vec<(State, Value)>) {
    let cmd = match &outputs[0] {
        Value::String(s) => s,
        _ => panic!("{:?}", outputs),
    };
    match cmd.as_str() {
        "flip" => {
            let n = match outputs[1..] {
                [Value::Int(n)] => n,
                _ => panic!("{:?}", outputs),
            };
            (Instruction::Flip(n.try_into().unwrap(), State(1), State(0)),
                vec![(State(1), Value::Bool(true)),
                     (State(0), Value::Bool(false))])
        }
        _ => panic!("{:?}", outputs),
    }
}

fn run_to_input(vm_state: &mut vm::State, cp: &CompiledProgram) -> Vec<Value> {
    let mut outputs = vec![];
    loop {
        let res = vm_state.step(cp, &mut outputs).unwrap();
        match res {
            Status::Terminated => panic!("ant should be an infinite loop"),
            Status::Running => continue,
            Status::BlockedOnInput => return outputs,
        }
    }
}

#[derive(Debug)]
struct Node {
    insn: Instruction,
    edges: Vec<(State, usize)>,
}

impl Node {
    fn new(insn: Instruction) -> Self {
        Node { insn, edges: vec![] }
    }

    fn patched_insn(&self) -> Instruction {
        let mut insn = self.insn;
        let mut cnt = 0;
        for s in insn.transitions_mut() {
            cnt += 1;
            let idx = self.edges.iter().find_map(|(ss, idx)| {
                if s == ss {
                    Some(*idx)
                } else {
                    None
                }
            }).unwrap();
            *s = State(idx.try_into().unwrap());
        }
        assert_eq!(cnt, self.edges.len());
        insn
    }
}

fn unroll_dfa(_lfs: &LoadedFiles, cp: &CompiledProgram) -> BrainWithSymbols {
    let mut vm_state = vm::State::new();
    let mut nodes = vec![];
    #[allow(clippy::type_complexity)]
    let mut idx_map: HashMap<(vm::State, Instruction, Vec<(State, Value)>), usize> = HashMap::new();

    let outputs = run_to_input(&mut vm_state, cp);
    let (insn, transitions) = outputs_to_insn_and_transitions(&outputs);

    let idx = nodes.len();
    idx_map.insert((vm_state.clone(), insn, transitions.clone()), idx);
    nodes.push(Node::new(insn));

    let mut worklist = vec![(idx, vm_state, transitions)];
    while let Some((idx, vm_state, transitions)) = worklist.pop() {
        assert!(nodes[idx].edges.is_empty());
        for (state, input) in transitions {
            let mut vm_state2 = vm_state.clone();
            vm_state2.give_input(input);
            let outputs = run_to_input(&mut vm_state2, cp);
            let (insn2, transitions2) = outputs_to_insn_and_transitions(&outputs);

            idx_map.entry((vm_state2, insn2, transitions2)).and_modify(|&mut idx2| {
                nodes[idx].edges.push((state, idx2));
            }).or_insert_with_key(|(vm_state2, insn2, transitions2)| {
                let idx2 = nodes.len();
                nodes.push(Node::new(*insn2));
                worklist.push((idx2, vm_state2.clone(), transitions2.clone()));
                nodes[idx].edges.push((state, idx2));
                idx2
            });
        }
    }

    BrainWithSymbols {
        insns: nodes.iter().map(|node| node.patched_insn()).collect(),
    }
}

// ENTRY_POINT
pub fn py2ant() {
    let args: Vec<String> = std::env::args().collect();
    let input_filename = match &args[..] {
        [_, _, input] => input,
        _ => {
            println!("Usage:");
            println!("    py2ant data/py/example.py");
            std::process::exit(1);
        }
    };
    println!("hello, {}", input_filename);

    let prelude_text = std::fs::read_to_string("data/py/_prelude.py").unwrap();
    let input_text = std::fs::read_to_string(input_filename).unwrap();

    let mut lfs = LoadedFiles::new(&[
        ("prelude.py", &prelude_text),
        (input_filename, &input_text),
    ]);

    let cp = CompiledProgram::new(&mut lfs);
    let cp = match cp {
        Ok(cp) => cp,
        Err(e) => {
            println!("{}", lfs.render_error(e));
            std::process::exit(1);
        }
    };

    println!("{}", cp);

    let output_path = Path::new(input_filename).file_name().unwrap();
    let output_path = Path::new("outputs").join(output_path);
    let output_path = format!("{}.ant", output_path.to_str().unwrap());

    println!();
    let b = unroll_dfa(&lfs, &cp);
    for (i, insn) in b.insns.iter().enumerate() {
        println!("{:>4}:  {}", i, insn);
    }

    println!();
    std::fs::write(&output_path, neurology::dumps(&b.insns)).unwrap();
    println!("saved to {}", output_path);
}
