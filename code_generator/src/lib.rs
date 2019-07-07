use std::collections::HashMap;
use std::string::ToString;

use instruction::*;
use lexical_analyzer::error::*;
use syntax_analyzer::ast_node::*;

mod instruction;

pub struct CodeGenerator<'a> {
    data_addr: HashMap<&'a str, u32>,
    string_pool: Vec<&'a str>,
    pc: u32,
    instructions: Vec<Instruction>,
}

impl<'a> CodeGenerator<'a> {
    pub fn generate(ast: &'a ASTNode) -> Result<String> {
        let mut generator = CodeGenerator {
            data_addr: HashMap::new(),
            string_pool: Vec::new(),
            pc: 0,
            instructions: Vec::new(),
        };

        generator.generate_body(ast)?;

        generator
            .instructions
            .push(Instruction::new(InstructionKind::Halt, generator.pc));

        let mut code = format!(
            "Datasize: {} Strings: {}\n",
            generator.data_addr.len(),
            generator.string_pool.len()
        );
        if generator.string_pool.len() > 0 {
            code += &generator
                .string_pool
                .iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<String>>()
                .join("\n");
            code += "\n";
        }
        code += &generator
            .instructions
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(code)
    }

    fn generate_body(&mut self, ast: &'a ASTNode) -> Result<()> {
        match ast.kind() {
            NodeKind::Identifier(identifier) => self.generate_fetch(identifier),
            NodeKind::Integer(value) => self.generate_integer(*value),
            NodeKind::Sequence => self.generate_sequence(ast),
            NodeKind::If => self.generate_if(ast),
            NodeKind::Prtc => self.generate_prtc(ast),
            NodeKind::Prts => self.generate_prts(ast),
            NodeKind::Prti => self.generate_prti(ast),
            NodeKind::While => self.generate_while(ast),
            NodeKind::Assign => self.generate_assign(ast),
            NodeKind::Negate | NodeKind::Not => self.generate_unary_op(ast),
            NodeKind::Multiply
            | NodeKind::Divide
            | NodeKind::Mod
            | NodeKind::Add
            | NodeKind::Subtract
            | NodeKind::Less
            | NodeKind::LessEqual
            | NodeKind::Greater
            | NodeKind::GreaterEqual
            | NodeKind::Equal
            | NodeKind::NotEqual
            | NodeKind::And
            | NodeKind::Or => self.generate_binary_op(ast),
            _ => Err(CompileError::new(
                ErrorKind::CodeGenerationError,
                "unknown instruction",
            )),
        }
    }

    fn generate_fetch(&mut self, identifier: &'a str) -> Result<()> {
        match self.data_addr.get(identifier) {
            Some(addr) => {
                self.instructions
                    .push(Instruction::new(InstructionKind::Fetch(*addr), self.pc));
                self.pc += 1 + 4;
                Ok(())
            }
            None => Err(CompileError::new(
                ErrorKind::CodeGenerationError,
                format!("unknown identifier: {}", identifier),
            )),
        }
    }

    fn generate_integer(&mut self, value: i32) -> Result<()> {
        self.instructions
            .push(Instruction::new(InstructionKind::Push(value), self.pc));
        self.pc += 1 + 4;
        Ok(())
    }

    fn backpatch(&mut self, instruction_index: usize) {
        let instruction = &mut self.instructions[instruction_index];
        if let Instruction {
            kind: InstructionKind::Jump(ref mut rel),
            address,
        }
        | Instruction {
            kind: InstructionKind::Jz(ref mut rel),
            address,
        } = instruction
        {
            *rel = (self.pc - (*address + 1)) as i32;
        } else {
            unreachable!();
        }
    }

    fn generate_if(&mut self, ast: &'a ASTNode) -> Result<()> {
        // condition
        self.generate_body(ast.lhs().unwrap())?;
        self.instructions
            .push(Instruction::new(InstructionKind::Jz(0), self.pc));
        self.pc += 1 + 4;
        let jump_if_clause_idx = &self.instructions.len() - 1;

        // if-clause
        let body = ast.rhs().unwrap();
        self.generate_body(body.lhs().unwrap())?;

        // else-clause
        match body.rhs() {
            Some(else_clause) => {
                self.instructions
                    .push(Instruction::new(InstructionKind::Jump(0), self.pc));
                let jump_instruction_idx = self.instructions.len() - 1;
                self.pc += 1 + 4;
                self.backpatch(jump_if_clause_idx);

                self.generate_body(else_clause)?;
                self.backpatch(jump_instruction_idx);
            }
            None => {
                self.backpatch(jump_if_clause_idx);
            }
        }
        Ok(())
    }

    fn generate_while(&mut self, ast: &'a ASTNode) -> Result<()> {
        // condition
        let entry_address = self.pc;
        self.generate_body(ast.lhs().unwrap())?;
        self.instructions
            .push(Instruction::new(InstructionKind::Jz(0), self.pc));
        let entry_index = self.instructions.len() - 1;
        self.pc += 1 + 4;

        // body
        self.generate_body(ast.rhs().unwrap())?;
        self.instructions.push(Instruction::new(
            InstructionKind::Jump(entry_address.wrapping_sub(self.pc + 1) as i32),
            self.pc,
        ));

        self.pc += 1 + 4;
        self.backpatch(entry_index);
        Ok(())
    }

    fn intern_string(&mut self, s: &'a str) -> u32 {
        for (i, &st) in self.string_pool.iter().enumerate() {
            if s == st {
                return i as u32;
            }
        }
        self.string_pool.push(s);
        (self.string_pool.len() - 1) as u32
    }

    fn generate_prts(&mut self, ast: &'a ASTNode) -> Result<()> {
        let string_node = ast.lhs().unwrap();
        if let NodeKind::String(s) = string_node.kind() {
            let addr = self.intern_string(s) as i32;
            self.instructions
                .push(Instruction::new(InstructionKind::Push(addr), self.pc));
            self.pc += 1 + 4;
        } else {
            return Err(CompileError::new(
                ErrorKind::CodeGenerationError,
                "string expected",
            ));
        }
        self.instructions
            .push(Instruction::new(InstructionKind::Prts, self.pc));
        self.pc += 1;
        Ok(())
    }

    fn generate_prtc(&mut self, ast: &'a ASTNode) -> Result<()> {
        self.generate_body(ast.lhs().unwrap())?;
        self.instructions
            .push(Instruction::new(InstructionKind::Prtc, self.pc));
        self.pc += 1;
        Ok(())
    }

    fn generate_prti(&mut self, ast: &'a ASTNode) -> Result<()> {
        self.generate_body(ast.lhs().unwrap())?;
        self.instructions
            .push(Instruction::new(InstructionKind::Prti, self.pc));
        self.pc += 1;
        Ok(())
    }

    fn generate_sequence(&mut self, ast: &'a ASTNode) -> Result<()> {
        if let Some(lhs) = ast.lhs() {
            self.generate_body(lhs)?;
        }
        if let Some(rhs) = ast.rhs() {
            self.generate_body(rhs)?;
        }
        Ok(())
    }

    fn intern(&mut self, name: &'a str) -> u32 {
        match self.data_addr.get(name) {
            Some(addr) => *addr,
            None => {
                let addr = self.data_addr.len() as u32;
                self.data_addr.insert(name, addr);
                addr
            }
        }
    }

    fn generate_assign(&mut self, ast: &'a ASTNode) -> Result<()> {
        let identifier_node = ast.lhs().unwrap();
        self.generate_body(ast.rhs().unwrap())?;

        if let NodeKind::Identifier(ref identifier) = identifier_node.kind() {
            let addr = self.intern(identifier);
            self.instructions
                .push(Instruction::new(InstructionKind::Store(addr), self.pc));
            self.pc += 1 + 4;
        } else {
            return Err(CompileError::new(
                ErrorKind::CodeGenerationError,
                "identifier is expected",
            ));
        }
        Ok(())
    }

    fn generate_unary_op(&mut self, ast: &'a ASTNode) -> Result<()> {
        self.generate_body(ast.lhs().unwrap())?;

        let instruction_kind = match ast.kind() {
            NodeKind::Negate => InstructionKind::Neg,
            NodeKind::Not => InstructionKind::Not,
            _ => {
                return Err(CompileError::new(
                    ErrorKind::CodeGenerationError,
                    "invalid unary operator",
                ))
            }
        };
        self.instructions
            .push(Instruction::new(instruction_kind, self.pc));
        self.pc += 1;
        Ok(())
    }

    fn generate_binary_op(&mut self, ast: &'a ASTNode) -> Result<()> {
        self.generate_body(ast.lhs().unwrap())?;
        self.generate_body(ast.rhs().unwrap())?;

        let instruction_kind = match ast.kind() {
            NodeKind::Multiply => InstructionKind::Mul,
            NodeKind::Divide => InstructionKind::Div,
            NodeKind::Mod => InstructionKind::Mod,
            NodeKind::Add => InstructionKind::Add,
            NodeKind::Subtract => InstructionKind::Sub,
            NodeKind::Less => InstructionKind::Lt,
            NodeKind::LessEqual => InstructionKind::Le,
            NodeKind::Greater => InstructionKind::Gt,
            NodeKind::GreaterEqual => InstructionKind::Ge,
            NodeKind::Equal => InstructionKind::Eq,
            NodeKind::NotEqual => InstructionKind::Ne,
            NodeKind::And => InstructionKind::And,
            NodeKind::Or => InstructionKind::Or,
            _ => {
                return Err(CompileError::new(
                    ErrorKind::CodeGenerationError,
                    "invalid binary operator",
                ))
            }
        };
        self.instructions
            .push(Instruction::new(instruction_kind, self.pc));
        self.pc += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        let s = r#"Integer 1"#.to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_add() {
        // 1 + 2
        let s = r#"Add
Integer 1
Integer 2
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_subtract() {
        // 1 + 2
        let s = r#"Subtract
Integer 1
Integer 2
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_negate() {
        // -1
        let s = r#"Negate
Integer 1
;
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_assign() {
        // count = 1
        let s = r#"Assign
Identifier count
Integer 1
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_identifier() {
        // count = 1;
        // count = count + 1;
        let s = r#"Sequence
Sequence
;
Assign
Identifier count
Integer 1
Assign
Identifier count
Add
Identifier count
Integer 1
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_prti() {
        // count = 1;
        // print(count);
        let s = r#"Sequence
Sequence
;
Assign
Identifier count
Integer 1
Sequence
;
Prti
Identifier count
;
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_prtc() {
        // count = 1;
        // putc(count);
        let s = r#"Sequence
Sequence
;
Assign
Identifier count
Integer 1
Sequence
;
Prtc
Identifier count
;
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_prts() {
        // print("abc", "cba", "abc");
        let s = r#"Sequence
;
Sequence
Sequence
Sequence
;
Prts
String "abc"
;
Prts
String "cba"
;
Prts
String "abc"
;
"#
        .to_string();
        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_if() {
        // if (1) {
        //     print(2);
        // } else {
        //     print(3);
        // }
        let s = r#"Sequence
;
If
Integer 1
If
Sequence
;
Sequence
;
Prti
Integer 2
;
Sequence
;
Sequence
;
Prti
Integer 3
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());

        println!("{:?}", CodeGenerator::generate(&ast));
    }

    #[test]
    fn test_hello_world() {
        let s = r#"Sequence
;
Sequence
;
Prts
String        "Hello, World!\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 0 Strings: 1
"Hello, World!\n"
0 push 0
5 prts
6 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_phoenix_number() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier    phoenix_number
Integer       142857
Sequence
Sequence
;
Prti
Identifier    phoenix_number
;
Prts
String        "\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 1
"\n"
0 push 142857
5 store [0]
10 fetch [0]
15 prti
16 push 0
21 prts
22 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_case_4() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier    count
Integer       1
While
Less
Identifier    count
Integer       10
Sequence
Sequence
;
Sequence
Sequence
Sequence
;
Prts
String        "count is: "
;
Prti
Identifier    count
;
Prts
String        "\n"
;
Assign
Identifier    count
Add
Identifier    count
Integer       1
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 2
"count is: "
"\n"
0 push 1
5 store [0]
10 fetch [0]
15 push 10
20 lt
21 jz (43) 65
26 push 0
31 prts
32 fetch [0]
37 prti
38 push 1
43 prts
44 fetch [0]
49 push 1
54 add
55 store [0]
60 jmp (-51) 10
65 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_count() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier    count
Integer       1
While
Less
Identifier    count
Integer       10
Sequence
Sequence
;
Sequence
Sequence
Sequence
;
Prts
String        "count is: "
;
Prti
Identifier    count
;
Prts
String        "\n"
;
Assign
Identifier    count
Add
Identifier    count
Integer       1"#
            .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 2
"count is: "
"\n"
0 push 1
5 store [0]
10 fetch [0]
15 push 10
20 lt
21 jz (43) 65
26 push 0
31 prts
32 fetch [0]
37 prti
38 push 1
43 prts
44 fetch [0]
49 push 1
54 add
55 store [0]
60 jmp (-51) 10
65 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_100_doors() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier     i
Integer        1
While
LessEqual
Multiply
Identifier     i
Identifier     i
Integer        100
Sequence
Sequence
;
Sequence
Sequence
Sequence
;
Prts
String         "door "
;
Prti
Multiply
Identifier     i
Identifier     i
;
Prts
String         " is open\n"
;
Assign
Identifier     i
Add
Identifier     i
Integer        1"#
            .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 2
"door "
" is open\n"
0 push 1
5 store [0]
10 fetch [0]
15 fetch [0]
20 mul
21 push 100
26 le
27 jz (49) 77
32 push 0
37 prts
38 fetch [0]
43 fetch [0]
48 mul
49 prti
50 push 1
55 prts
56 fetch [0]
61 push 1
66 add
67 store [0]
72 jmp (-63) 10
77 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_negative_tests() {
        let s = r#"Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier    a
Multiply
Negate
Integer       1
;
Divide
Multiply
Negate
Integer       1
;
Multiply
Integer       5
Integer       15
Integer       10
Sequence
Sequence
;
Prti
Identifier    a
;
Prts
String        "\n"
;
Assign
Identifier    b
Negate
Identifier    a
;
Sequence
Sequence
;
Prti
Identifier    b
;
Prts
String        "\n"
;
Sequence
Sequence
;
Prti
Negate
Identifier    b
;
;
Prts
String        "\n"
;
Sequence
Sequence
;
Prti
Negate
Integer       1
;
;
Prts
String        "\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 2 Strings: 1
"\n"
0 push 1
5 neg
6 push 1
11 neg
12 push 5
17 push 15
22 mul
23 mul
24 push 10
29 div
30 mul
31 store [0]
36 fetch [0]
41 prti
42 push 0
47 prts
48 fetch [0]
53 neg
54 store [1]
59 fetch [1]
64 prti
65 push 0
70 prts
71 fetch [1]
76 neg
77 prti
78 push 0
83 prts
84 push 1
89 neg
90 prti
91 push 0
96 prts
97 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_deep() {
        let s = r#"Sequence
Sequence
Sequence
;
Sequence
Sequence
;
Prti
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Negate
Integer       5
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
;
Prts
String        "\n"
;
Sequence
Sequence
;
Prti
Multiply
Add
Integer       3
Integer       2
Integer       2
;
Prts
String        "\n"
;
If
Integer       1
If
Sequence
;
If
Integer       1
If
Sequence
;
If
Integer       1
If
Sequence
;
If
Integer       1
If
Sequence
;
If
Integer       1
If
Sequence
;
Sequence
Sequence
;
Prti
Integer       15
;
Prts
String        "\n"
;
;
;
;
;
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 0 Strings: 1
"\n"
0 push 5
5 neg
6 neg
7 neg
8 neg
9 neg
10 neg
11 neg
12 neg
13 neg
14 neg
15 neg
16 neg
17 neg
18 neg
19 neg
20 neg
21 neg
22 neg
23 neg
24 neg
25 neg
26 neg
27 neg
28 neg
29 neg
30 neg
31 neg
32 neg
33 neg
34 neg
35 neg
36 neg
37 neg
38 prti
39 push 0
44 prts
45 push 3
50 push 2
55 add
56 push 2
61 mul
62 prti
63 push 0
68 prts
69 push 1
74 jz (56) 131
79 push 1
84 jz (46) 131
89 push 1
94 jz (36) 131
99 push 1
104 jz (26) 131
109 push 1
114 jz (16) 131
119 push 15
124 prti
125 push 0
130 prts
131 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_greatest_common_divisor() {
        let s = r#"Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     a
Integer        1071
Assign
Identifier     b
Integer        1029
While
NotEqual
Identifier     b
Integer        0
Sequence
Sequence
Sequence
;
Assign
Identifier     new_a
Identifier     b
Assign
Identifier     b
Mod
Identifier     a
Identifier     b
Assign
Identifier     a
Identifier     new_a
Sequence
;
Prti
Identifier     a
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 3 Strings: 0
0 push 1071
5 store [0]
10 push 1029
15 store [1]
20 fetch [1]
25 push 0
30 ne
31 jz (45) 77
36 fetch [1]
41 store [2]
46 fetch [0]
51 fetch [1]
56 mod
57 store [1]
62 fetch [2]
67 store [0]
72 jmp (-53) 20
77 fetch [0]
82 prti
83 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_factorial() {
        let s = r#"Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     n
Integer        12
Assign
Identifier     result
Integer        1
Assign
Identifier     i
Integer        1
While
LessEqual
Identifier     i
Identifier     n
Sequence
Sequence
;
Assign
Identifier     result
Multiply
Identifier     result
Identifier     i
Assign
Identifier     i
Add
Identifier     i
Integer        1
Sequence
;
Prti
Identifier     result
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 3 Strings: 0
0 push 12
5 store [0]
10 push 1
15 store [1]
20 push 1
25 store [2]
30 fetch [2]
35 fetch [0]
40 le
41 jz (41) 83
46 fetch [1]
51 fetch [2]
56 mul
57 store [1]
62 fetch [2]
67 push 1
72 add
73 store [2]
78 jmp (-49) 30
83 fetch [1]
88 prti
89 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_fibonacci_sequence() {
        let s = r#"Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     n
Integer        44
Assign
Identifier     i
Integer        1
Assign
Identifier     a
Integer        0
Assign
Identifier     b
Integer        1
While
Less
Identifier     i
Identifier     n
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     w
Add
Identifier     a
Identifier     b
Assign
Identifier     a
Identifier     b
Assign
Identifier     b
Identifier     w
Assign
Identifier     i
Add
Identifier     i
Integer        1
Sequence
Sequence
;
Prti
Identifier     w
;
Prts
String         "\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 5 Strings: 1
"\n"
0 push 44
5 store [0]
10 push 1
15 store [1]
20 push 0
25 store [2]
30 push 1
35 store [3]
40 fetch [1]
45 fetch [0]
50 lt
51 jz (61) 113
56 fetch [2]
61 fetch [3]
66 add
67 store [4]
72 fetch [3]
77 store [2]
82 fetch [4]
87 store [3]
92 fetch [1]
97 push 1
102 add
103 store [1]
108 jmp (-69) 40
113 fetch [4]
118 prti
119 push 0
124 prts
125 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_fizzbuzz() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier     i
Integer        1
While
LessEqual
Identifier     i
Integer        100
Sequence
Sequence
Sequence
;
If
Not
Mod
Identifier     i
Integer        15
;
If
Sequence
;
Prts
String         "FizzBuzz"
;
If
Not
Mod
Identifier     i
Integer        3
;
If
Sequence
;
Prts
String         "Fizz"
;
If
Not
Mod
Identifier     i
Integer        5
;
If
Sequence
;
Prts
String         "Buzz"
;
Sequence
;
Prti
Identifier     i
;
Sequence
;
Prts
String         "\n"
;
Assign
Identifier     i
Add
Identifier     i
Integer        1"#
            .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 4
"FizzBuzz"
"Fizz"
"Buzz"
"\n"
0 push 1
5 store [0]
10 fetch [0]
15 push 100
20 le
21 jz (121) 143
26 fetch [0]
31 push 15
36 mod
37 not
38 jz (15) 54
43 push 0
48 prts
49 jmp (66) 116
54 fetch [0]
59 push 3
64 mod
65 not
66 jz (15) 82
71 push 1
76 prts
77 jmp (38) 116
82 fetch [0]
87 push 5
92 mod
93 not
94 jz (15) 110
99 push 2
104 prts
105 jmp (10) 116
110 fetch [0]
115 prti
116 push 3
121 prts
122 fetch [0]
127 push 1
132 add
133 store [0]
138 jmp (-129) 10
143 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_99_bottles_of_beer() {
        let s = r#"Sequence
Sequence
;
Assign
Identifier     bottles
Integer        99
While
Greater
Identifier     bottles
Integer        0
Sequence
Sequence
Sequence
Sequence
Sequence
;
Sequence
Sequence
;
Prti
Identifier     bottles
;
Prts
String         " bottles of beer on the wall\n"
;
Sequence
Sequence
;
Prti
Identifier     bottles
;
Prts
String         " bottles of beer\n"
;
Sequence
;
Prts
String         "Take one down, pass it around\n"
;
Assign
Identifier     bottles
Subtract
Identifier     bottles
Integer        1
Sequence
Sequence
;
Prti
Identifier     bottles
;
Prts
String         " bottles of beer on the wall\n\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 1 Strings: 4
" bottles of beer on the wall\n"
" bottles of beer\n"
"Take one down, pass it around\n"
" bottles of beer on the wall\n\n"
0 push 99
5 store [0]
10 fetch [0]
15 push 0
20 gt
21 jz (67) 89
26 fetch [0]
31 prti
32 push 0
37 prts
38 fetch [0]
43 prti
44 push 1
49 prts
50 push 2
55 prts
56 fetch [0]
61 push 1
66 sub
67 store [0]
72 fetch [0]
77 prti
78 push 3
83 prts
84 jmp (-75) 10
89 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_primes() {
        let s = r#"Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier    count
Integer       1
Assign
Identifier    n
Integer       1
Assign
Identifier    limit
Integer       100
While
Less
Identifier    n
Identifier    limit
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier    k
Integer       3
Assign
Identifier    p
Integer       1
Assign
Identifier    n
Add
Identifier    n
Integer       2
While
And
LessEqual
Multiply
Identifier    k
Identifier    k
Identifier    n
Identifier    p
Sequence
Sequence
;
Assign
Identifier    p
NotEqual
Multiply
Divide
Identifier    n
Identifier    k
Identifier    k
Identifier    n
Assign
Identifier    k
Add
Identifier    k
Integer       2
If
Identifier    p
If
Sequence
Sequence
;
Sequence
Sequence
;
Prti
Identifier    n
;
Prts
String        " is prime\n"
;
Assign
Identifier    count
Add
Identifier    count
Integer       1
;
Sequence
Sequence
Sequence
;
Prts
String        "Total primes found: "
;
Prti
Identifier    count
;
Prts
String        "\n"
;"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 5 Strings: 3
" is prime\n"
"Total primes found: "
"\n"
0 push 1
5 store [0]
10 push 1
15 store [1]
20 push 100
25 store [2]
30 fetch [1]
35 fetch [2]
40 lt
41 jz (160) 202
46 push 3
51 store [3]
56 push 1
61 store [4]
66 fetch [1]
71 push 2
76 add
77 store [1]
82 fetch [3]
87 fetch [3]
92 mul
93 fetch [1]
98 le
99 fetch [4]
104 and
105 jz (53) 159
110 fetch [1]
115 fetch [3]
120 div
121 fetch [3]
126 mul
127 fetch [1]
132 ne
133 store [4]
138 fetch [3]
143 push 2
148 add
149 store [3]
154 jmp (-73) 82
159 fetch [4]
164 jz (32) 197
169 fetch [1]
174 prti
175 push 0
180 prts
181 fetch [0]
186 push 1
191 add
192 store [0]
197 jmp (-168) 30
202 push 1
207 prts
208 fetch [0]
213 prti
214 push 2
219 prts
220 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }

    #[test]
    fn test_ascii_mandlebrot() {
        let s = r#"Sequence
;
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     left_edge
Negate
Integer        420
;
Assign
Identifier     right_edge
Integer        300
Assign
Identifier     top_edge
Integer        300
Assign
Identifier     bottom_edge
Negate
Integer        300
;
Assign
Identifier     x_step
Integer        7
Assign
Identifier     y_step
Integer        15
Assign
Identifier     max_iter
Integer        200
Assign
Identifier     y0
Identifier     top_edge
While
Greater
Identifier     y0
Identifier     bottom_edge
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     x0
Identifier     left_edge
While
Less
Identifier     x0
Identifier     right_edge
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     y
Integer        0
Assign
Identifier     x
Integer        0
Assign
Identifier     the_char
Integer        32
Assign
Identifier     i
Integer        0
While
Less
Identifier     i
Identifier     max_iter
Sequence
Sequence
Sequence
Sequence
Sequence
Sequence
;
Assign
Identifier     x_x
Divide
Multiply
Identifier     x
Identifier     x
Integer        200
Assign
Identifier     y_y
Divide
Multiply
Identifier     y
Identifier     y
Integer        200
If
Greater
Add
Identifier     x_x
Identifier     y_y
Integer        800
If
Sequence
Sequence
Sequence
;
Assign
Identifier     the_char
Add
Integer        48
Identifier     i
If
Greater
Identifier     i
Integer        9
If
Sequence
;
Assign
Identifier     the_char
Integer        64
;
Assign
Identifier     i
Identifier     max_iter
;
Assign
Identifier     y
Add
Divide
Multiply
Identifier     x
Identifier     y
Integer        100
Identifier     y0
Assign
Identifier     x
Add
Subtract
Identifier     x_x
Identifier     y_y
Identifier     x0
Assign
Identifier     i
Add
Identifier     i
Integer        1
Prtc
Identifier     the_char
;
Assign
Identifier     x0
Add
Identifier     x0
Identifier     x_step
Prtc
Integer        10
;
Assign
Identifier     y0
Subtract
Identifier     y0
Identifier     y_step"#
            .to_string();

        let ast = ASTReader::read_ast(s.lines());
        assert_eq!(
            r#"Datasize: 15 Strings: 0
0 push 420
5 neg
6 store [0]
11 push 300
16 store [1]
21 push 300
26 store [2]
31 push 300
36 neg
37 store [3]
42 push 7
47 store [4]
52 push 15
57 store [5]
62 push 200
67 store [6]
72 fetch [2]
77 store [7]
82 fetch [7]
87 fetch [3]
92 gt
93 jz (329) 423
98 fetch [0]
103 store [8]
108 fetch [8]
113 fetch [1]
118 lt
119 jz (276) 396
124 push 0
129 store [9]
134 push 0
139 store [10]
144 push 32
149 store [11]
154 push 0
159 store [12]
164 fetch [12]
169 fetch [6]
174 lt
175 jz (193) 369
180 fetch [10]
185 fetch [10]
190 mul
191 push 200
196 div
197 store [13]
202 fetch [9]
207 fetch [9]
212 mul
213 push 200
218 div
219 store [14]
224 fetch [13]
229 fetch [14]
234 add
235 push 800
240 gt
241 jz (56) 298
246 push 48
251 fetch [12]
256 add
257 store [11]
262 fetch [12]
267 push 9
272 gt
273 jz (14) 288
278 push 64
283 store [11]
288 fetch [6]
293 store [12]
298 fetch [10]
303 fetch [9]
308 mul
309 push 100
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
353 push 1
358 add
359 store [12]
364 jmp (-201) 164
369 fetch [11]
374 prtc
375 fetch [8]
380 fetch [4]
385 add
386 store [8]
391 jmp (-284) 108
396 push 10
401 prtc
402 fetch [7]
407 fetch [5]
412 sub
413 store [7]
418 jmp (-337) 82
423 halt"#,
            CodeGenerator::generate(&ast).unwrap()
        );
    }
}
