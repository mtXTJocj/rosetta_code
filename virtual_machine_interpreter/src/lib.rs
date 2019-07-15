use lexical_analyzer::error::{CompileError, ErrorKind, Result};
use std::convert::TryInto;
use std::io::Write;
use std::str::Lines;

const FETCH: u8 = 0;
const STORE: u8 = 1;
const PUSH: u8 = 2;
const ADD: u8 = 3;
const SUB: u8 = 4;
const MUL: u8 = 5;
const DIV: u8 = 6;
const MOD: u8 = 7;
const LT: u8 = 8;
const GT: u8 = 9;
const LE: u8 = 10;
const GE: u8 = 11;
const EQ: u8 = 12;
const NE: u8 = 13;
const AND: u8 = 14;
const OR: u8 = 15;
const NEG: u8 = 16;
const NOT: u8 = 17;
const JMP: u8 = 18;
const JZ: u8 = 19;
const PRTC: u8 = 20;
const PRTS: u8 = 21;
const PRTI: u8 = 22;
const HALT: u8 = 23;

const STACK_SIZE: usize = 1000;

#[derive(Debug)]
struct Header {
    data_size: usize,
    string_size: usize,
}

pub struct VirtualMachineInterpreter {
    pc: usize,
    sp: usize,
    byte_code: Vec<u8>,
    string_pool: Vec<String>,
    data: Vec<i32>,
    stack: [i32; STACK_SIZE],
}

impl VirtualMachineInterpreter {
    pub fn interpret(lines: Lines, out: &mut Write) -> Result<()> {
        let mut vm = VirtualMachineInterpreter::assemble(lines)?;
        vm.execute(out)
    }

    fn assemble(mut lines: Lines) -> Result<Self> {
        let header;

        if let Some(line) = lines.next() {
            header = Self::read_header(line)?;
        } else {
            return Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                "empty file",
            ));
        }

        let mut string_pool: Vec<String> = Vec::new();
        for _ in 0..header.string_size {
            if let Some(line) = lines.next() {
                string_pool.push(Self::read_string(line)?);
            } else {
                return Err(CompileError::new(
                    ErrorKind::VirtualMachineError,
                    "unexpected EOF",
                ));
            }
        }

        let mut byte_code: Vec<u8> = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.len() == 0 {
                // empty line
                continue;
            }

            Self::read_instruction(line, &mut byte_code)?;
        }

        Ok(VirtualMachineInterpreter {
            pc: 0,
            sp: 0,
            byte_code,
            string_pool,
            data: vec![0; header.data_size],
            stack: [0; STACK_SIZE],
        })
    }

    fn read_header(line: &str) -> Result<Header> {
        let data_size: usize;
        let string_size: usize;

        let sizes: Vec<&str> = line.trim().split_whitespace().collect();
        if sizes.len() != 4 {
            Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                "invalid datasize format.",
            ))
        } else {
            data_size = sizes[1].parse().or_else(|_| {
                Err(CompileError::new(
                    ErrorKind::VirtualMachineError,
                    "invalid data size",
                ))
            })?;
            string_size = sizes[3].parse().or_else(|_| {
                Err(CompileError::new(
                    ErrorKind::VirtualMachineError,
                    "invalid string data size",
                ))
            })?;
            Ok(Header {
                data_size,
                string_size,
            })
        }
    }

    fn read_string(s: &str) -> Result<String> {
        let mut value = String::new();
        let mut cs = s.chars();
        let mut next_char = cs.next();

        if Some('"') != next_char {
            return Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                "invalid string",
            ));
        }
        next_char = cs.next();
        loop {
            match next_char {
                Some('"') => break,
                Some('\\') => {
                    let escape = cs.next();
                    match escape {
                        Some('n') => value.push('\n'),
                        Some('\\') => value.push('\\'),
                        _ => {
                            return Err(CompileError::new(
                                ErrorKind::VirtualMachineError,
                                "invalid escape",
                            ))
                        }
                    }
                }
                Some(c) => value.push(c),
                None => {
                    return Err(CompileError::new(
                        ErrorKind::VirtualMachineError,
                        "\" not found",
                    ))
                }
            }
            next_char = cs.next();
        }

        Ok(value)
    }

    fn read_integer(s: &str, dst: &mut Vec<u8>) -> Result<()> {
        if let Ok(val) = s.parse::<i32>() {
            dst.extend_from_slice(&val.to_ne_bytes());
            Ok(())
        } else {
            Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                format!("cannot convert to integer: {}", s),
            ))
        }
    }

    fn read_instruction(s: &str, dst: &mut Vec<u8>) -> Result<()> {
        let mnemonic: Vec<&str> = s.split_whitespace().collect();

        if mnemonic.len() == 1 {
            return Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                "invalid code",
            ));
        }

        match mnemonic[1] {
            "fetch" => {
                dst.push(FETCH);
                let v = mnemonic[2];
                Self::read_integer(&v[1..v.len() - 1], dst)?;
            }
            "store" => {
                dst.push(STORE);
                let v = mnemonic[2];
                Self::read_integer(&v[1..v.len() - 1], dst)?;
            }
            "push" => {
                dst.push(PUSH);
                Self::read_integer(mnemonic[2], dst)?;
            }
            "jmp" => {
                dst.push(JMP);
                let v = mnemonic[2];
                &Self::read_integer(&v[1..v.len() - 1], dst)?;
            }
            "jz" => {
                dst.push(JZ);
                let v = mnemonic[2];
                &Self::read_integer(&v[1..v.len() - 1], dst)?;
            }
            "add" => {
                dst.push(ADD);
            }
            "sub" => {
                dst.push(SUB);
            }
            "mul" => {
                dst.push(MUL);
            }
            "div" => {
                dst.push(DIV);
            }
            "mod" => {
                dst.push(MOD);
            }
            "lt" => {
                dst.push(LT);
            }
            "gt" => {
                dst.push(GT);
            }
            "le" => {
                dst.push(LE);
            }
            "ge" => {
                dst.push(GE);
            }
            "eq" => {
                dst.push(EQ);
            }
            "ne" => {
                dst.push(NE);
            }
            "and" => {
                dst.push(AND);
            }
            "or" => {
                dst.push(OR);
            }
            "neg" => {
                dst.push(NEG);
            }
            "not" => {
                dst.push(NOT);
            }
            "prtc" => {
                dst.push(PRTC);
            }
            "prti" => {
                dst.push(PRTI);
            }
            "prts" => {
                dst.push(PRTS);
            }
            "halt" => {
                dst.push(HALT);
            }
            _ => {
                dbg!(mnemonic);
                return Err(CompileError::new(
                    ErrorKind::VirtualMachineError,
                    "illegal instruction",
                ));
            }
        };
        Ok(())
    }

    fn get_integer(&self) -> Result<i32> {
        if let Ok(v) = &self.byte_code[self.pc..(self.pc + 4)].try_into() {
            Ok(i32::from_ne_bytes(*v))
        } else {
            Err(CompileError::new(
                ErrorKind::VirtualMachineError,
                "invalid integer value",
            ))
        }
    }

    fn execute(&mut self, out: &mut Write) -> Result<()> {
        loop {
            let opcode = self.byte_code[self.pc as usize];
            self.pc += 1;

            match opcode {
                FETCH => {
                    let index = self.get_integer()?;
                    self.stack[self.sp] = self.data[index as usize];
                    self.sp += 1;
                    self.pc += 4;
                }
                STORE => {
                    let v = self.stack[self.sp - 1];
                    self.sp -= 1;
                    let index = self.get_integer()?;
                    self.data[index as usize] = v;
                    self.pc += 4;
                }
                PUSH => {
                    let v = self.get_integer()?;
                    self.stack[self.sp] = v;
                    self.sp += 1;
                    self.pc += 4;
                }
                JMP => {
                    let offset = self.get_integer()?;
                    self.pc = self.pc.wrapping_add(offset as usize);
                }
                JZ => {
                    let condition = self.stack[self.sp - 1];
                    self.sp -= 1;

                    if condition == 0 {
                        let offset = self.get_integer()?;
                        self.pc = self.pc.wrapping_add(offset as usize);
                    } else {
                        self.pc += 4;
                    }
                }
                ADD => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = operand0 + operand1;
                    self.sp -= 1;
                }
                SUB => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = operand0 - operand1;
                    self.sp -= 1;
                }
                MUL => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = operand0 * operand1;
                    self.sp -= 1;
                }
                DIV => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = operand0 / operand1;
                    self.sp -= 1;
                }
                MOD => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = operand0 % operand1;
                    self.sp -= 1;
                }
                LT => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 < operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                GT => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 > operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                LE => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 <= operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                GE => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 >= operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                EQ => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 == operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                NE => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 != operand1 { 1 } else { 0 };
                    self.sp -= 1;
                }
                AND => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 != 0 && operand1 != 0 { 1 } else { 0 };
                    self.sp -= 1;
                }
                OR => {
                    let operand0 = self.stack[self.sp - 2];
                    let operand1 = self.stack[self.sp - 1];
                    self.stack[self.sp - 2] = if operand0 != 0 || operand1 != 0 { 1 } else { 0 };
                    self.sp -= 1;
                }
                NEG => {
                    self.stack[self.sp - 1] = -self.stack[self.sp - 1];
                }
                NOT => self.stack[self.sp - 1] = if self.stack[self.sp - 1] == 0 { 1 } else { 0 },
                PRTC => {
                    match std::char::from_u32(self.stack[self.sp - 1] as u32) {
                        Some(c) => match out.write(format!("{}", c).as_bytes()) {
                            Err(e) => {
                                return Err(CompileError::new(
                                    ErrorKind::VirtualMachineError,
                                    format!("output error: {}", e),
                                ));
                            }
                            _ => {}
                        },
                        None => {
                            return Err(CompileError::new(
                                ErrorKind::VirtualMachineError,
                                format!("illegal character value: {}", self.stack[self.sp - 1]),
                            ));
                        }
                    }
                    self.sp -= 1;
                }
                PRTI => {
                    match out.write(format!("{}", self.stack[self.sp - 1]).as_bytes()) {
                        Err(e) => {
                            return Err(CompileError::new(
                                ErrorKind::VirtualMachineError,
                                format!("output error: {}", e),
                            ));
                        }
                        _ => {}
                    }
                    self.sp -= 1;
                }
                PRTS => {
                    match out.write(self.string_pool[self.stack[self.sp - 1] as usize].as_bytes()) {
                        Err(e) => {
                            return Err(CompileError::new(
                                ErrorKind::VirtualMachineError,
                                format!("output error: {}", e),
                            ));
                        }
                        _ => {}
                    }
                    self.sp -= 1;
                }
                HALT => break,
                _ => {
                    return Err(CompileError::new(
                        ErrorKind::VirtualMachineError,
                        format!("illegal instruction: {}", opcode),
                    ))
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_4() {
        let s = r#"Datasize: 1 Strings: 2
"count is: "
"\n"
    0 push  1
    5 store [0]
   10 fetch [0]
   15 push  10
   20 lt
   21 jz     (43) 65
   26 push  0
   31 prts
   32 fetch [0]
   37 prti
   38 push  1
   43 prts
   44 fetch [0]
   49 push  1
   54 add
   55 store [0]
   60 jmp    (-51) 10
   65 halt"#
            .to_string();
        let mut out: Vec<u8> = Vec::new();

        VirtualMachineInterpreter::interpret(s.lines(), &mut out).unwrap();
        println!("{:?}", out);
    }

    #[test]
    fn test_fizzbuzz() {
        let s = r#"Datasize: 1 Strings: 4
"FizzBuzz"
"Fizz"
"Buzz"
"\n"
    0 push  1
    5 store [0]
   10 fetch [0]
   15 push  100
   20 le
   21 jz     (121) 143
   26 fetch [0]
   31 push  15
   36 mod
   37 not
   38 jz     (15) 54
   43 push  0
   48 prts
   49 jmp    (66) 116
   54 fetch [0]
   59 push  3
   64 mod
   65 not
   66 jz     (15) 82
   71 push  1
   76 prts
   77 jmp    (38) 116
   82 fetch [0]
   87 push  5
   92 mod
   93 not
   94 jz     (15) 110
   99 push  2
  104 prts
  105 jmp    (10) 116
  110 fetch [0]
  115 prti
  116 push  3
  121 prts
  122 fetch [0]
  127 push  1
  132 add
  133 store [0]
  138 jmp    (-129) 10
  143 halt"#
            .to_string();
        let mut out: Vec<u8> = Vec::new();

        VirtualMachineInterpreter::interpret(s.lines(), &mut out).unwrap();
        assert_eq!(
            r#"1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
17
Fizz
19
Buzz
Fizz
22
23
Fizz
Buzz
26
Fizz
28
29
FizzBuzz
31
32
Fizz
34
Buzz
Fizz
37
38
Fizz
Buzz
41
Fizz
43
44
FizzBuzz
46
47
Fizz
49
Buzz
Fizz
52
53
Fizz
Buzz
56
Fizz
58
59
FizzBuzz
61
62
Fizz
64
Buzz
Fizz
67
68
Fizz
Buzz
71
Fizz
73
74
FizzBuzz
76
77
Fizz
79
Buzz
Fizz
82
83
Fizz
Buzz
86
Fizz
88
89
FizzBuzz
91
92
Fizz
94
Buzz
Fizz
97
98
Fizz
Buzz
"#
            .as_bytes(),
            &out[..]
        );
    }

    #[test]
    fn test_99_bottles_of_beer() {
        let s = r#"Datasize: 1 Strings: 4
" bottles of beer on the wall\n"
" bottles of beer\n"
"Take one down, pass it around\n"
" bottles of beer on the wall\n\n"
    0 push  99
    5 store [0]
   10 fetch [0]
   15 push  0
   20 gt
   21 jz     (67) 89
   26 fetch [0]
   31 prti
   32 push  0
   37 prts
   38 fetch [0]
   43 prti
   44 push  1
   49 prts
   50 push  2
   55 prts
   56 fetch [0]
   61 push  1
   66 sub
   67 store [0]
   72 fetch [0]
   77 prti
   78 push  3
   83 prts
   84 jmp    (-75) 10
   89 halt"#
            .to_string();
        let mut out: Vec<u8> = Vec::new();

        VirtualMachineInterpreter::interpret(s.lines(), &mut out).unwrap();

        assert_eq!(
            r#"99 bottles of beer on the wall
99 bottles of beer
Take one down, pass it around
98 bottles of beer on the wall

98 bottles of beer on the wall
98 bottles of beer
Take one down, pass it around
97 bottles of beer on the wall
"#
            .as_bytes(),
            &out[..223]
        );

        assert_eq!(
            r#"2 bottles of beer on the wall
2 bottles of beer
Take one down, pass it around
1 bottles of beer on the wall

1 bottles of beer on the wall
1 bottles of beer
Take one down, pass it around
0 bottles of beer on the wall

"#
            .as_bytes(),
            &out[10842..]
        );
    }

    #[test]
    fn test_primes() {
        let s = r#"Datasize: 5 Strings: 3
" is prime\n"
"Total primes found: "
"\n"
   0 push  1
   5 store [0]
  10 push  1
  15 store [1]
  20 push  100
  25 store [2]
  30 fetch [1]
  35 fetch [2]
  40 lt
  41 jz     (160) 202
  46 push  3
  51 store [3]
  56 push  1
  61 store [4]
  66 fetch [1]
  71 push  2
  76 add
  77 store [1]
  82 fetch [3]
  87 fetch [3]
  92 mul
  93 fetch [1]
  98 le
  99 fetch [4]
 104 and
 105 jz     (53) 159
 110 fetch [1]
 115 fetch [3]
 120 div
 121 fetch [3]
 126 mul
 127 fetch [1]
 132 ne
 133 store [4]
 138 fetch [3]
 143 push  2
 148 add
 149 store [3]
 154 jmp    (-73) 82
 159 fetch [4]
 164 jz     (32) 197
 169 fetch [1]
 174 prti
 175 push  0
 180 prts
 181 fetch [0]
 186 push  1
 191 add
 192 store [0]
 197 jmp    (-168) 30
 202 push  1
 207 prts
 208 fetch [0]
 213 prti
 214 push  2
 219 prts
 220 halt"#
            .to_string();
        let mut out: Vec<u8> = Vec::new();

        VirtualMachineInterpreter::interpret(s.lines(), &mut out).unwrap();

        assert_eq!(
            r#"3 is prime
5 is prime
7 is prime
11 is prime
13 is prime
17 is prime
19 is prime
23 is prime
29 is prime
31 is prime
37 is prime
41 is prime
43 is prime
47 is prime
53 is prime
59 is prime
61 is prime
67 is prime
71 is prime
73 is prime
79 is prime
83 is prime
89 is prime
97 is prime
101 is prime
Total primes found: 26
"#
            .as_bytes(),
            &out[..]
        );
    }

    #[test]
    fn test_ascii_mandlebrot() {
        let s = r#"Datasize: 15 Strings: 0
    0 push  420
    5 neg
    6 store [0]
   11 push  300
   16 store [1]
   21 push  300
   26 store [2]
   31 push  300
   36 neg
   37 store [3]
   42 push  7
   47 store [4]
   52 push  15
   57 store [5]
   62 push  200
   67 store [6]
   72 fetch [2]
   77 store [7]
   82 fetch [7]
   87 fetch [3]
   92 gt
   93 jz     (329) 423
   98 fetch [0]
  103 store [8]
  108 fetch [8]
  113 fetch [1]
  118 lt
  119 jz     (276) 396
  124 push  0
  129 store [9]
  134 push  0
  139 store [10]
  144 push  32
  149 store [11]
  154 push  0
  159 store [12]
  164 fetch [12]
  169 fetch [6]
  174 lt
  175 jz     (193) 369
  180 fetch [10]
  185 fetch [10]
  190 mul
  191 push  200
  196 div
  197 store [13]
  202 fetch [9]
  207 fetch [9]
  212 mul
  213 push  200
  218 div
  219 store [14]
  224 fetch [13]
  229 fetch [14]
  234 add
  235 push  800
  240 gt
  241 jz     (56) 298
  246 push  48
  251 fetch [12]
  256 add
  257 store [11]
  262 fetch [12]
  267 push  9
  272 gt
  273 jz     (14) 288
  278 push  64
  283 store [11]
  288 fetch [6]
  293 store [12]
  298 fetch [10]
  303 fetch [9]
  308 mul
  309 push  100
  314 div
  315 fetch [7]
  320 add
  321 store [9]
  326 fetch [13]
  331 fetch [14]
  336 sub
  337 fetch [8]
  342 add
  343 store [10]
  348 fetch [12]
  353 push  1
  358 add
  359 store [12]
  364 jmp    (-201) 164
  369 fetch [11]
  374 prtc
  375 fetch [8]
  380 fetch [4]
  385 add
  386 store [8]
  391 jmp    (-284) 108
  396 push  10
  401 prtc
  402 fetch [7]
  407 fetch [5]
  412 sub
  413 store [7]
  418 jmp    (-337) 82
  423 halt"#
            .to_string();
        let mut out: Vec<u8> = Vec::new();

        VirtualMachineInterpreter::interpret(s.lines(), &mut out).unwrap();

        assert_eq!(r#"1111111111111111111111122222222222222222222222222222222222222222222222222222222222222222222222222211111
1111111111111111111122222222222222222222222222222222222222222222222222222222222222222222222222222222211
1111111111111111112222222222222222222222222222222222222222222222222222222222222222222222222222222222222
1111111111111111222222222222222222233333333333333333333333222222222222222222222222222222222222222222222
1111111111111112222222222222333333333333333333333333333333333333222222222222222222222222222222222222222
1111111111111222222222233333333333333333333333344444456655544443333332222222222222222222222222222222222
1111111111112222222233333333333333333333333444444445567@@6665444444333333222222222222222222222222222222
11111111111222222333333333333333333333334444444445555679@@@@7654444443333333222222222222222222222222222
1111111112222223333333333333333333333444444444455556789@@@@98755544444433333332222222222222222222222222
1111111122223333333333333333333333344444444445556668@@@    @@@76555544444333333322222222222222222222222
1111111222233333333333333333333344444444455566667778@@      @987666555544433333333222222222222222222222
111111122333333333333333333333444444455556@@@@@99@@@@@@    @@@@@@877779@5443333333322222222222222222222
1111112233333333333333333334444455555556679@   @@@               @@@@@@ 8544333333333222222222222222222
1111122333333333333333334445555555556666789@@@                        @86554433333333322222222222222222
1111123333333333333444456666555556666778@@ @                         @@87655443333333332222222222222222
111123333333344444455568@887789@8777788@@@                            @@@@65444333333332222222222222222
111133334444444455555668@@@@@@@@@@@@99@@@                              @@765444333333333222222222222222
111133444444445555556778@@@         @@@@                                @855444333333333222222222222222
11124444444455555668@99@@             @                                 @655444433333333322222222222222
11134555556666677789@@                                                @86655444433333333322222222222222
111                                                                 @@876555444433333333322222222222222
11134555556666677789@@                                                @86655444433333333322222222222222
11124444444455555668@99@@             @                                 @655444433333333322222222222222
111133444444445555556778@@@         @@@@                                @855444333333333222222222222222
111133334444444455555668@@@@@@@@@@@@99@@@                              @@765444333333333222222222222222
111123333333344444455568@887789@8777788@@@                            @@@@65444333333332222222222222222
1111123333333333333444456666555556666778@@ @                         @@87655443333333332222222222222222
1111122333333333333333334445555555556666789@@@                        @86554433333333322222222222222222
1111112233333333333333333334444455555556679@   @@@               @@@@@@ 8544333333333222222222222222222
111111122333333333333333333333444444455556@@@@@99@@@@@@    @@@@@@877779@5443333333322222222222222222222
1111111222233333333333333333333344444444455566667778@@      @987666555544433333333222222222222222222222
1111111122223333333333333333333333344444444445556668@@@    @@@76555544444333333322222222222222222222222
1111111112222223333333333333333333333444444444455556789@@@@98755544444433333332222222222222222222222222
11111111111222222333333333333333333333334444444445555679@@@@7654444443333333222222222222222222222222222
1111111111112222222233333333333333333333333444444445567@@6665444444333333222222222222222222222222222222
1111111111111222222222233333333333333333333333344444456655544443333332222222222222222222222222222222222
1111111111111112222222222222333333333333333333333333333333333333222222222222222222222222222222222222222
1111111111111111222222222222222222233333333333333333333333222222222222222222222222222222222222222222222
1111111111111111112222222222222222222222222222222222222222222222222222222222222222222222222222222222222
1111111111111111111122222222222222222222222222222222222222222222222222222222222222222222222222222222211
"#.as_bytes(), &out[..]);
    }
}
