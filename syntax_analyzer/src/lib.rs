pub mod ast_node;

use lexical_analyzer::error::*;
use lexical_analyzer::token::*;
use std::vec::IntoIter;

use ast_node::*;

struct Operator {
    kind: NodeKind,
    right_associative: bool,
    precedence: i32,
}

fn operator(op: &TokenKind) -> &'static Operator {
    // or
    const OR: Operator = Operator {
        kind: NodeKind::Or,
        right_associative: false,
        precedence: 10,
    };
    // and
    const AND: Operator = Operator {
        kind: NodeKind::And,
        right_associative: false,
        precedence: 20,
    };
    // equality
    const EQUAL: Operator = Operator {
        kind: NodeKind::Equal,
        right_associative: false,
        precedence: 30,
    };
    const NOT_EQUAL: Operator = Operator {
        kind: NodeKind::NotEqual,
        right_associative: false,
        precedence: 30,
    };
    // relational
    const LESS: Operator = Operator {
        kind: NodeKind::Less,
        right_associative: false,
        precedence: 40,
    };
    const LESS_EQUAL: Operator = Operator {
        kind: NodeKind::LessEqual,
        right_associative: false,
        precedence: 40,
    };
    const GREATER: Operator = Operator {
        kind: NodeKind::Greater,
        right_associative: false,
        precedence: 40,
    };
    const GREATER_EQUAL: Operator = Operator {
        kind: NodeKind::GreaterEqual,
        right_associative: false,
        precedence: 40,
    };
    // addition
    const ADD: Operator = Operator {
        kind: NodeKind::Add,
        right_associative: false,
        precedence: 50,
    };
    const SUBTRACT: Operator = Operator {
        kind: NodeKind::Subtract,
        right_associative: false,
        precedence: 50,
    };
    // multiplication
    const MULTIPLY: Operator = Operator {
        kind: NodeKind::Multiply,
        right_associative: false,
        precedence: 60,
    };
    const DIVIDE: Operator = Operator {
        kind: NodeKind::Divide,
        right_associative: false,
        precedence: 60,
    };
    const MOD: Operator = Operator {
        kind: NodeKind::Mod,
        right_associative: false,
        precedence: 60,
    };
    // sentinel として使うため、 precedence -1
    const NOT_OPERATOR: Operator = Operator {
        kind: NodeKind::None,
        right_associative: false,
        precedence: -1,
    };

    match op {
        TokenKind::OpOr => &OR,
        TokenKind::OpAnd => &AND,
        TokenKind::OpEqual => &EQUAL,
        TokenKind::OpNotEqual => &NOT_EQUAL,
        TokenKind::OpLess => &LESS,
        TokenKind::OpLessEqual => &LESS_EQUAL,
        TokenKind::OpGreater => &GREATER,
        TokenKind::OpGreaterEqual => &GREATER_EQUAL,
        TokenKind::OpAdd => &ADD,
        TokenKind::OpSubtract => &SUBTRACT,
        TokenKind::OpMultiply => &MULTIPLY,
        TokenKind::OpDivide => &DIVIDE,
        TokenKind::OpMod => &MOD,
        _ => &NOT_OPERATOR,
    }
}

pub struct SyntaxAnalyzer {
    token_iter: IntoIter<Token>,
    next_token: Token,
}

impl SyntaxAnalyzer {
    pub fn parse(mut token_iter: IntoIter<Token>) -> Result<ASTNode> {
        match token_iter.next() {
            Some(next_token) => {
                let mut parser = SyntaxAnalyzer {
                    token_iter,
                    next_token,
                };
                parser.parse_stmt_list()
            }
            None => Ok(ASTNode {
                kind: NodeKind::Sequence,
                lhs: None,
                rhs: None,
            }),
        }
    }

    fn read_token(&mut self) -> Result<Token> {
        let next_token = self.token_iter.next();
        match next_token {
            Some(t) => Ok(std::mem::replace(&mut self.next_token, t)),
            None => Err(CompileError::new(ErrorKind::SyntaxError, "unexpected EOF")),
        }
    }

    fn parse_stmt_list(&mut self) -> Result<ASTNode> {
        let mut node;
        match self.next_token.kind() {
            TokenKind::Semicolon
            | TokenKind::Identifier(_)
            | TokenKind::KeywordWhile
            | TokenKind::KeywordIf
            | TokenKind::KeywordPrint
            | TokenKind::KeywordPutc
            | TokenKind::LeftBrace => {
                node = ASTNode {
                    kind: NodeKind::Sequence,
                    lhs: None,
                    rhs: Some(Box::new(self.parse_stmt()?)),
                }
            }
            _ => {
                return Ok(ASTNode {
                    kind: NodeKind::Sequence,
                    lhs: None,
                    rhs: None,
                });
            }
        };

        loop {
            match self.next_token.kind() {
                TokenKind::Semicolon
                | TokenKind::Identifier(_)
                | TokenKind::KeywordWhile
                | TokenKind::KeywordIf
                | TokenKind::KeywordPrint
                | TokenKind::KeywordPutc
                | TokenKind::LeftBrace => {
                    node = ASTNode {
                        kind: NodeKind::Sequence,
                        lhs: Some(Box::new(node)),
                        rhs: Some(Box::new(self.parse_stmt()?)),
                    }
                }
                _ => {
                    break;
                }
            };
        }

        Ok(node)
    }

    fn parse_stmt(&mut self) -> Result<ASTNode> {
        match self.next_token.kind() {
            TokenKind::Semicolon => {
                self.read_token()?;
                Ok(ASTNode {
                    kind: NodeKind::Sequence,
                    lhs: None,
                    rhs: None,
                })
            }
            TokenKind::Identifier(_) => self.parse_assign_stmt(),
            TokenKind::KeywordWhile => self.parse_while_stmt(),
            TokenKind::KeywordIf => self.parse_if_stmt(),
            TokenKind::KeywordPrint => self.parse_print_stmt(),
            TokenKind::KeywordPutc => self.parse_putc_stmt(),
            TokenKind::LeftBrace => {
                self.read_token()?;

                let node = self.parse_stmt_list()?;

                if *self.next_token.kind() != TokenKind::RightBrace {
                    return Err(CompileError::new(
                        ErrorKind::SyntaxError,
                        "'}' is expected.",
                    ));
                }
                self.read_token()?;
                Ok(node)
            }
            _ => Err(CompileError::new(
                ErrorKind::SyntaxError,
                format!("unexpected token: {:?}", self.next_token),
            )),
        }
    }

    fn parse_assign_stmt(&mut self) -> Result<ASTNode> {
        let Token { kind, .. } = self.read_token()?;

        match kind {
            TokenKind::Identifier(identifier) => {
                let lhs = ASTNode {
                    kind: NodeKind::Identifier(identifier),
                    lhs: None,
                    rhs: None,
                };

                if *self.next_token.kind() != TokenKind::OpAssign {
                    return Err(CompileError::new(
                        ErrorKind::SyntaxError,
                        "'=' is expected.",
                    ));
                }
                self.read_token()?;

                let rhs = self.parse_expr()?;

                if *self.next_token.kind() != TokenKind::Semicolon {
                    return Err(CompileError::new(
                        ErrorKind::SyntaxError,
                        "';' is expected.",
                    ));
                }
                self.read_token()?;

                Ok(ASTNode {
                    kind: NodeKind::Assign,
                    lhs: Some(Box::new(lhs)),
                    rhs: Some(Box::new(rhs)),
                })
            }
            _ => Err(CompileError::new(
                ErrorKind::SyntaxError,
                "Identifier is expected",
            )),
        }
    }

    fn parse_while_stmt(&mut self) -> Result<ASTNode> {
        if *self.next_token.kind() != TokenKind::KeywordWhile {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "\"while\" is expected.",
            ));
        }
        self.read_token()?;

        let lhs = self.parse_paren_expr()?;
        let rhs = self.parse_stmt()?;

        Ok(ASTNode {
            kind: NodeKind::While,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
        })
    }

    fn parse_if_stmt(&mut self) -> Result<ASTNode> {
        if *self.next_token.kind() != TokenKind::KeywordIf {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "\"if\" is expected.",
            ));
        }
        self.read_token()?;

        let condition = self.parse_paren_expr()?;
        let mut node = ASTNode {
            kind: NodeKind::If,
            lhs: Some(Box::new(condition)),
            rhs: None,
        };

        let if_clause = Some(Box::new(self.parse_stmt()?));
        let else_clause = if *self.next_token.kind() == TokenKind::KeywordElse {
            self.read_token()?;
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };

        node.rhs = Some(Box::new(ASTNode {
            kind: NodeKind::If,
            lhs: if_clause,
            rhs: else_clause,
        }));

        Ok(node)
    }

    fn parse_print_stmt(&mut self) -> Result<ASTNode> {
        if *self.next_token.kind() != TokenKind::KeywordPrint {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "\"print\" is expected.",
            ));
        }
        self.read_token()?;

        if *self.next_token.kind() != TokenKind::LeftParen {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "'(' is expected.",
            ));
        }
        self.read_token()?;

        let node = self.parse_prt_list()?;

        if *self.next_token.kind() != TokenKind::RightParen {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "')' is expected.",
            ));
        }
        self.read_token()?;

        if *self.next_token.kind() != TokenKind::Semicolon {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "';' is expected.",
            ));
        }
        self.read_token()?;

        Ok(node)
    }

    fn parse_putc_stmt(&mut self) -> Result<ASTNode> {
        if *self.next_token.kind() != TokenKind::KeywordPutc {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "\"putc\" is expected.",
            ));
        }
        self.read_token()?;

        let lhs = self.parse_paren_expr()?;

        if *self.next_token.kind() != TokenKind::Semicolon {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "';' is expected.",
            ));
        }
        self.read_token()?;

        Ok(ASTNode {
            kind: NodeKind::Prtc,
            lhs: Some(Box::new(lhs)),
            rhs: None,
        })
    }

    fn make_string_node(&mut self) -> Result<ASTNode> {
        let Token { kind, .. } = self.read_token()?;
        if let TokenKind::String(s) = kind {
            Ok(ASTNode {
                kind: NodeKind::String(s),
                lhs: None,
                rhs: None,
            })
        } else {
            unreachable!()
        }
    }

    fn parse_prt_list(&mut self) -> Result<ASTNode> {
        let node = match self.next_token.kind() {
            TokenKind::String(_) => ASTNode {
                kind: NodeKind::Prts,
                lhs: Some(Box::new(self.make_string_node()?)),
                rhs: None,
            },
            _ => ASTNode {
                kind: NodeKind::Prti,
                lhs: Some(Box::new(self.parse_expr()?)),
                rhs: None,
            },
        };

        let mut lhs = ASTNode {
            kind: NodeKind::Sequence,
            lhs: None,
            rhs: Some(Box::new(node)),
        };

        while *self.next_token.kind() == TokenKind::Comma {
            self.read_token()?;

            let node = match self.next_token.kind() {
                TokenKind::String(_) => ASTNode {
                    kind: NodeKind::Prts,
                    lhs: Some(Box::new(self.make_string_node()?)),
                    rhs: None,
                },
                _ => ASTNode {
                    kind: NodeKind::Prti,
                    lhs: Some(Box::new(self.parse_expr()?)),
                    rhs: None,
                },
            };

            lhs = ASTNode {
                kind: NodeKind::Sequence,
                lhs: Some(Box::new(lhs)),
                rhs: Some(Box::new(node)),
            };
        }
        Ok(lhs)
    }

    fn parse_paren_expr(&mut self) -> Result<ASTNode> {
        if *self.next_token.kind() != TokenKind::LeftParen {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "'(' is expected.",
            ));
        }
        self.read_token()?;

        let node = self.parse_expr()?;

        if *self.next_token.kind() != TokenKind::RightParen {
            return Err(CompileError::new(
                ErrorKind::SyntaxError,
                "')' is expected.",
            ));
        }
        self.read_token()?;
        Ok(node)
    }

    ///  演算子優先順位パーザで式を解析する
    fn parse_expr(&mut self) -> Result<ASTNode> {
        let lhs = self.parse_primary()?;
        self.parse_expr_body(lhs, 0)
    }

    fn parse_expr_body(&mut self, node: ASTNode, min_precedence: i32) -> Result<ASTNode> {
        let mut lhs = node;

        let mut next_op = operator(self.next_token.kind());
        while next_op.precedence >= min_precedence {
            let op = next_op;
            self.read_token()?;

            let mut rhs = self.parse_primary()?;
            next_op = operator(self.next_token.kind());

            while next_op.precedence > op.precedence
                || ((next_op.precedence == op.precedence) && next_op.right_associative)
            {
                rhs = self.parse_expr_body(rhs, next_op.precedence)?;
                next_op = operator(self.next_token.kind());
            }

            lhs = ASTNode {
                kind: op.kind.clone(),
                lhs: Some(Box::new(lhs)),
                rhs: Some(Box::new(rhs)),
            };
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<ASTNode> {
        let Token { kind, .. } = self.read_token()?;
        match kind {
            TokenKind::Identifier(identifier) => Ok(ASTNode {
                kind: NodeKind::Identifier(identifier),
                lhs: None,
                rhs: None,
            }),
            TokenKind::Integer(kind) => Ok(ASTNode {
                kind: NodeKind::Integer(kind),
                lhs: None,
                rhs: None,
            }),
            TokenKind::LeftParen => {
                let node = self.parse_expr()?;

                if *self.next_token.kind() != TokenKind::RightParen {
                    return Err(CompileError::new(
                        ErrorKind::SyntaxError,
                        "')' is expected.",
                    ));
                }
                self.read_token()?;

                Ok(node)
            }

            TokenKind::OpAdd => self.parse_primary(),
            TokenKind::OpSubtract => Ok(ASTNode {
                kind: NodeKind::Negate,
                lhs: Some(Box::new(self.parse_primary()?)),
                rhs: None,
            }),
            TokenKind::OpNot => Ok(ASTNode {
                kind: NodeKind::Not,
                lhs: Some(Box::new(self.parse_primary()?)),
                rhs: None,
            }),
            _ => Err(CompileError::new(ErrorKind::SyntaxError, "invalid primary")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexical_analyzer::*;

    fn create_tokens(s: String) -> Vec<Token> {
        let mut lexer = LexicalAnalyzer::new(s.chars());
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            let token = lexer.next_token().unwrap();
            tokens.push(token);
            if *tokens.last().unwrap().kind() == TokenKind::EndOfInput {
                return tokens;
            }
        }
    }

    fn create_parser(mut token_iter: IntoIter<Token>) -> SyntaxAnalyzer {
        match token_iter.next() {
            Some(next_token) => SyntaxAnalyzer {
                token_iter,
                next_token,
            },
            None => unreachable!(),
        }
    }

    #[test]
    fn test_identifier() {
        let tokens = create_tokens(r#"count"#.to_string());
        assert_eq!(
            r#"Identifier count
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"+count"#.to_string());
        assert_eq!(
            r#"Identifier count
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"-count"#.to_string());
        assert_eq!(
            r#"Negate
Identifier count
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"!count"#.to_string());
        assert_eq!(
            r#"Not
Identifier count
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );
    }

    #[test]
    fn test_integer() {
        let tokens = create_tokens(r#"1234"#.to_string());
        assert_eq!(
            r#"Integer 1234
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"+1234"#.to_string());
        assert_eq!(
            r#"Integer 1234
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"-1234"#.to_string());
        assert_eq!(
            r#"Negate
Integer 1234
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );

        let tokens = create_tokens(r#"!1234"#.to_string());
        assert_eq!(
            r#"Not
Integer 1234
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_primary().unwrap()
            ),
        );
    }

    #[test]
    fn test_expr() {
        let tokens = create_tokens(r#"count + 1"#.to_string());
        assert_eq!(
            r#"Add
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"+count * 1"#.to_string());
        assert_eq!(
            r#"Multiply
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"count < 1"#.to_string());
        assert_eq!(
            r#"Less
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"count == 1"#.to_string());
        assert_eq!(
            r#"Equal
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"count && 1"#.to_string());
        assert_eq!(
            r#"And
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"count || 1"#.to_string());
        assert_eq!(
            r#"Or
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(
            r#"a || b && c != d > e - f / g % h + i >= j == k && l || m"#.to_string(),
        );
        assert_eq!(
            r#"Or
Or
Identifier a
And
And
Identifier b
Equal
NotEqual
Identifier c
GreaterEqual
Greater
Identifier d
Add
Subtract
Identifier e
Mod
Divide
Identifier f
Identifier g
Identifier h
Identifier i
Identifier j
Identifier k
Identifier l
Identifier m
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"(a * b) + c"#.to_string());
        assert_eq!(
            r#"Add
Multiply
Identifier a
Identifier b
Identifier c
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );

        let tokens = create_tokens(r#"a * (b + c)"#.to_string());
        assert_eq!(
            r#"Multiply
Identifier a
Add
Identifier b
Identifier c
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_expr().unwrap()
            ),
        );
    }

    #[test]
    fn test_paren_expr() {
        let tokens = create_tokens(r#"(b + c)"#.to_string());
        assert_eq!(
            r#"Add
Identifier b
Identifier c
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter())
                    .parse_paren_expr()
                    .unwrap()
            ),
        );
    }

    #[test]
    fn test_prt_list() {
        let tokens = create_tokens(r#"a + b"#.to_string());
        assert_eq!(
            r#"Sequence
;
Prti
Add
Identifier a
Identifier b
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_prt_list().unwrap()
            ),
        );

        let tokens = create_tokens(r#""hoge""#.to_string());
        assert_eq!(
            r#"Sequence
;
Prts
String "hoge"
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_prt_list().unwrap()
            ),
        );

        let tokens = create_tokens(r#"a + b, "hoge", 1"#.to_string());
        assert_eq!(
            r#"Sequence
Sequence
Sequence
;
Prti
Add
Identifier a
Identifier b
;
Prts
String "hoge"
;
Prti
Integer 1
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_prt_list().unwrap()
            ),
        );
    }

    #[test]
    fn test_putc_stmt() {
        let tokens = create_tokens(r#"putc(a + b);"#.to_string());
        assert_eq!(
            r#"Prtc
Add
Identifier a
Identifier b
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_putc_stmt().unwrap()
            ),
        );
    }

    #[test]
    fn test_print_stmt() {
        let tokens = create_tokens(r#"print("count is: ", count, "\n");"#.to_string());
        assert_eq!(
            r#"Sequence
Sequence
Sequence
;
Prts
String "count is: "
;
Prti
Identifier count
;
Prts
String "\n"
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter())
                    .parse_print_stmt()
                    .unwrap()
            ),
        );
    }

    #[test]
    fn test_if_stmt() {
        let tokens = create_tokens(
            r#"if (!(i % 15))
    print("FizzBuzz");
"#
            .to_string(),
        );
        assert_eq!(
            r#"If
Not
Mod
Identifier i
Integer 15
;
If
Sequence
;
Prts
String "FizzBuzz"
;
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_if_stmt().unwrap()
            ),
        );

        let tokens = create_tokens(
            r#"if (!(i % 15))
    print("FizzBuzz");
else if (!(i % 3))
    print("Fizz");
else if (!(i % 5))
    print("Buzz");
else
    print(i);
"#
            .to_string(),
        );
        assert_eq!(
            r#"If
Not
Mod
Identifier i
Integer 15
;
If
Sequence
;
Prts
String "FizzBuzz"
;
If
Not
Mod
Identifier i
Integer 3
;
If
Sequence
;
Prts
String "Fizz"
;
If
Not
Mod
Identifier i
Integer 5
;
If
Sequence
;
Prts
String "Buzz"
;
Sequence
;
Prti
Identifier i
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_if_stmt().unwrap()
            ),
        );
    }

    #[test]
    fn test_while_stmt() {
        let tokens = create_tokens(
            r#"while (count < 10) {
     print("count is: ", count, "\n");
     count = count + 1;
}"#
            .to_string(),
        );
        assert_eq!(
            r#"While
Less
Identifier count
Integer 10
Sequence
Sequence
;
Sequence
Sequence
Sequence
;
Prts
String "count is: "
;
Prti
Identifier count
;
Prts
String "\n"
;
Assign
Identifier count
Add
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter())
                    .parse_while_stmt()
                    .unwrap()
            ),
        );
    }

    #[test]
    fn test_assign_stmt() {
        let tokens = create_tokens(r#"count = count + 1;"#.to_string());
        assert_eq!(
            r#"Assign
Identifier count
Add
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter())
                    .parse_assign_stmt()
                    .unwrap()
            ),
        );
    }

    #[test]
    fn test_stmt_list() {
        let tokens = create_tokens(r#""#.to_string());
        assert_eq!(
            r#"Sequence
;
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_stmt_list().unwrap()
            ),
        );

        let tokens = create_tokens(r#";"#.to_string());
        assert_eq!(
            r#"Sequence
;
Sequence
;
;
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_stmt_list().unwrap()
            ),
        );

        let tokens = create_tokens(r#"count = 1;"#.to_string());
        assert_eq!(
            r#"Sequence
;
Assign
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_stmt_list().unwrap()
            ),
        );

        let tokens = create_tokens(
            r#"    print("count is: ", count, "\n");
        count = count + 1;
"#
            .to_string(),
        );
        assert_eq!(
            r#"Sequence
Sequence
;
Sequence
Sequence
Sequence
;
Prts
String "count is: "
;
Prti
Identifier count
;
Prts
String "\n"
;
Assign
Identifier count
Add
Identifier count
Integer 1
"#,
            format!(
                "{}",
                create_parser(tokens.into_iter()).parse_stmt_list().unwrap()
            ),
        );
    }
}
