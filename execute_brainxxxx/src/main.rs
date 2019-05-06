use std::fs::File;
use std::io::Read;
use std::num::Wrapping;
use std::str::Chars;

const MEMORY_SIZE: usize = 30000;

#[derive(Debug)]
enum Instruction {
    IncrementPointer,
    DecrementPointer,
    IncrementValue,
    DecrementValue,
    Output,
    Input,
    BranchIfZero(usize),
    BranchIfNotZero(usize),
}

fn compile(src: Chars) -> Vec<Instruction> {
    let mut program = Vec::new();
    let mut stack = Vec::new();
    for c in src {
        match c {
            '>' => {
                program.push(Instruction::IncrementPointer);
            }
            '<' => {
                program.push(Instruction::DecrementPointer);
            }
            '+' => {
                program.push(Instruction::IncrementValue);
            }
            '-' => {
                program.push(Instruction::DecrementValue);
            }
            '.' => {
                program.push(Instruction::Output);
            }
            ',' => {
                program.push(Instruction::Input);
            }
            '[' => {
                stack.push(program.len());
                program.push(Instruction::BranchIfZero(0));
            }
            ']' => {
                let dst = stack.pop().expect("']' appeared before '['");
                program[dst] = Instruction::BranchIfZero(program.len());
                program.push(Instruction::BranchIfNotZero(dst));
            }
            // コメントは読み飛ばす
            _ => {}
        }
    }
    if stack.len() > 0 {
        panic!("there are '[' whithout matching ']'");
    }

    program
}

fn execute(program: &Vec<Instruction>) {
    let mut pc = 0;
    let mut ptr = 0;
    let mut mem = vec![Wrapping(0); MEMORY_SIZE];
    let stdin = std::io::stdin();
    let mut input = stdin.lock().bytes();
    while pc < program.len() {
        match &program[pc] {
            Instruction::IncrementPointer => {
                ptr += 1;
            }
            Instruction::DecrementPointer => {
                ptr -= 1;
            }
            Instruction::IncrementValue => {
                mem[ptr] += Wrapping(1);
            }
            Instruction::DecrementValue => {
                mem[ptr] -= Wrapping(1);
            }
            Instruction::Output => {
                print!("{}", mem[ptr].0 as char);
            }
            Instruction::Input => {
                let v = input.next().unwrap().unwrap();
                mem[ptr] = Wrapping(v);
            }
            Instruction::BranchIfZero(dst) => {
                if mem[ptr].0 == 0 {
                    pc = *dst;
                }
            }
            Instruction::BranchIfNotZero(dst) => {
                if mem[ptr].0 != 0 {
                    pc = *dst;
                }
            }
        }
        pc += 1;
    }
}

fn print_usage(name: &str) {
    println!("Usage: {} filename", name);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        debug_assert!(args.len() == 1);
        print_usage(&args[0]);
        return;
    }

    let mut buf = String::new();
    match File::open(&args[1]) {
        Ok(mut f) => {
            f.read_to_string(&mut buf).expect("cannot read file.");
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    execute(&compile(buf.chars()));
}
