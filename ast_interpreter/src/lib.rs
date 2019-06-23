use lexical_analyzer::error::*;
use syntax_analyzer::ast_node::*;

use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'a> {
    Integer(i32),
    String(&'a str),
}

pub struct ASTInterpreter<'a> {
    global: HashMap<&'a str, Value<'a>>,
}

impl<'a> ASTInterpreter<'a> {
    pub fn interpret(node: &'a ASTNode, writer: &mut Write) -> Result<Option<Value<'a>>> {
        let mut interpreter = ASTInterpreter {
            global: HashMap::new(),
        };
        interpreter.interpret_body(node, writer)
    }

    fn interpret_body(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        match node.kind() {
            NodeKind::Sequence => {
                if let Some(ref lhs) = node.lhs() {
                    self.interpret_body(lhs, writer)?;
                }
                if let Some(ref rsh) = node.rhs() {
                    self.interpret_body(rsh, writer)?;
                }
                Ok(None)
            }
            NodeKind::Assign => self.interpret_assign(node, writer),
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
            | NodeKind::Or => self.interpret_binary_op(node, writer),
            NodeKind::Negate | NodeKind::Not => self.interpret_unary_op(node, writer),
            NodeKind::If => self.interpret_if(node, writer),
            NodeKind::While => self.interpret_while(node, writer),
            NodeKind::Identifier(value) => self.interpret_identifier(value),
            NodeKind::Prtc => self.interpret_prtc(node, writer),
            NodeKind::Prti => self.interpret_prti(node, writer),
            NodeKind::Prts => self.interpret_prts(node, writer),
            NodeKind::String(value) => Ok(Some(Value::String(value))),
            NodeKind::Integer(value) => Ok(Some(Value::Integer(*value))),
            _ => Err(CompileError::new(
                ErrorKind::InterpretationError,
                "unknown node.",
            )),
        }
    }

    fn interpret_identifier(&mut self, identifier: &'a str) -> Result<Option<Value<'a>>> {
        Ok(Some(self.global[identifier]))
    }

    fn interpret_assign(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let variable = node.lhs().unwrap();
        let value = self.interpret_body(node.rhs().unwrap(), writer)?.unwrap();

        match variable.kind() {
            NodeKind::Identifier(ref identifier) => {
                self.global.insert(identifier, value);
                Ok(None)
            }
            _ => Err(CompileError::new(
                ErrorKind::InterpretationError,
                "Identifier is expected.",
            )),
        }
    }

    fn interpret_binary_op(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let loperand = self.interpret_body(&node.lhs().unwrap(), writer)?.unwrap();
        let roperand = self.interpret_body(&node.rhs().unwrap(), writer)?.unwrap();

        match loperand {
            Value::Integer(lop) => match roperand {
                Value::Integer(rop) => match node.kind() {
                    NodeKind::Multiply => Ok(Some(Value::Integer(lop * rop))),
                    NodeKind::Divide => Ok(Some(Value::Integer(lop / rop))),
                    NodeKind::Mod => Ok(Some(Value::Integer(lop % rop))),
                    NodeKind::Add => Ok(Some(Value::Integer(lop + rop))),
                    NodeKind::Subtract => Ok(Some(Value::Integer(lop - rop))),
                    NodeKind::Less => {
                        if lop < rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::LessEqual => {
                        if lop <= rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::Greater => {
                        if lop > rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::GreaterEqual => {
                        if lop >= rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::Equal => {
                        if lop == rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::NotEqual => {
                        if lop != rop {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::And => {
                        if lop != 0 && rop != 0 {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    NodeKind::Or => {
                        if lop != 0 || rop != 0 {
                            Ok(Some(Value::Integer(1)))
                        } else {
                            Ok(Some(Value::Integer(0)))
                        }
                    }
                    _ => {
                        return Err(CompileError::new(
                            ErrorKind::InterpretationError,
                            "Unknown Node.",
                        ));
                    }
                },
                _ => {
                    return Err(CompileError::new(
                        ErrorKind::InterpretationError,
                        "Integer value is expected",
                    ));
                }
            },
            _ => {
                return Err(CompileError::new(
                    ErrorKind::InterpretationError,
                    "Integer value is expected",
                ));
            }
        }
    }

    fn interpret_unary_op(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let operand = self.interpret_body(node.lhs().unwrap(), writer)?.unwrap();

        match operand {
            Value::Integer(val) => match node.kind() {
                NodeKind::Negate => Ok(Some(Value::Integer(-val))),
                NodeKind::Not => {
                    if val == 0 {
                        Ok(Some(Value::Integer(1)))
                    } else {
                        Ok(Some(Value::Integer(0)))
                    }
                }
                _ => {
                    return Err(CompileError::new(
                        ErrorKind::InterpretationError,
                        "Integer value is expected",
                    ));
                }
            },
            _ => {
                return Err(CompileError::new(
                    ErrorKind::InterpretationError,
                    "Integer value is expected",
                ));
            }
        }
    }

    fn interpret_if(&mut self, node: &'a ASTNode, writer: &mut Write) -> Result<Option<Value<'a>>> {
        let condition = self.interpret_body(node.lhs().unwrap(), writer)?.unwrap();
        let statement_node = node.rhs().unwrap();

        if condition != Value::Integer(0) {
            self.interpret_body(statement_node.lhs().unwrap(), writer)?;
        } else {
            if let Some(else_clause) = statement_node.rhs() {
                self.interpret_body(else_clause, writer)?;
            }
        }

        Ok(None)
    }

    fn interpret_while(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let condition = node.lhs().unwrap();
        let statement = node.rhs().unwrap();

        while self.interpret_body(condition, writer)? != Some(Value::Integer(0)) {
            self.interpret_body(statement, writer)?;
        }
        Ok(None)
    }

    fn interpret_prtc(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let value = self.interpret_body(node.lhs().unwrap(), writer)?.unwrap();

        if let Value::Integer(i) = value {
            match std::char::from_u32(i as u32) {
                Some(c) => match writer.write(format!("{}", c).as_bytes()) {
                    Ok(_) => Ok(None),
                    Err(e) => Err(CompileError::new(
                        ErrorKind::InterpretationError,
                        e.to_string(),
                    )),
                },
                None => Err(CompileError::new(
                    ErrorKind::InterpretationError,
                    "non-integer value appeared.",
                )),
            }
        } else {
            Err(CompileError::new(
                ErrorKind::InterpretationError,
                "integer is expected.",
            ))
        }
    }

    fn interpret_prti(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let value = self.interpret_body(node.lhs().unwrap(), writer)?.unwrap();

        if let Value::Integer(i) = value {
            match writer.write(format!("{}", i).as_bytes()) {
                Ok(_) => Ok(None),
                Err(e) => Err(CompileError::new(
                    ErrorKind::InterpretationError,
                    e.to_string(),
                )),
            }
        } else {
            Err(CompileError::new(
                ErrorKind::InterpretationError,
                "integet is expected.",
            ))
        }
    }

    fn interpret_prts(
        &mut self,
        node: &'a ASTNode,
        writer: &mut Write,
    ) -> Result<Option<Value<'a>>> {
        let value = self.interpret_body(node.lhs().unwrap(), writer)?.unwrap();

        if let Value::String(s) = value {
            match writer.write(s.as_bytes()) {
                Ok(_) => Ok(None),
                Err(e) => Err(CompileError::new(
                    ErrorKind::InterpretationError,
                    e.to_string(),
                )),
            }
        } else {
            Err(CompileError::new(
                ErrorKind::InterpretationError,
                "string is expected.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
    }

    #[test]
    fn test_test_case_4() {
        let s = r#"Sequence
Sequence
Sequence
;
Sequence
;
Prti
Integer       42
;
Sequence
;
Prts
String        "\nHello World\nGood Bye\nok\n"
;
Sequence
;
Prts
String        "Print a slash n - \\n.\n"
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
Integer       1
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
Integer        1
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();
    }

    #[test]
    fn test_fizz_buzz() {
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
Integer        1
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();

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
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();

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
;
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();

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
Identifier     y_step
"#
        .to_string();

        let ast = ASTReader::read_ast(s.lines());
        let mut out = Vec::new();
        ASTInterpreter::interpret(&ast, &mut out).unwrap();

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
