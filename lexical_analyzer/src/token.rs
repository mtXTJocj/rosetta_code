use super::error::*;
use std::fmt;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    OpMultiply,
    OpDivide,
    OpMod,
    OpAdd,
    OpSubtract,
    //    OpNegate, 使わない
    OpLess,
    OpLessEqual,
    OpGreater,
    OpGreaterEqual,
    OpEqual,
    OpNotEqual,
    OpNot,
    OpAssign,
    OpAnd,
    OpOr,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    KeywordIf,
    KeywordElse,
    KeywordWhile,
    KeywordPrint,
    KeywordPutc,
    Identifier(String),
    Integer(i32),
    String(String),
    EndOfInput,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    line_number: usize,
    column_number: usize,
}

struct TokenReader<'a> {
    stream: Chars<'a>,
    next_char: Option<char>,
}

impl<'a> TokenReader<'a> {
    fn new(mut stream: Chars<'a>) -> Self {
        let next_char = stream.next();

        TokenReader { stream, next_char }
    }

    fn read_char(&mut self) {
        self.next_char = self.stream.next();
    }

    fn discard_whitespace(&mut self) {
        loop {
            match self.next_char {
                Some(c) => {
                    if !c.is_whitespace() {
                        break;
                    }
                }
                None => break,
            }
            self.read_char();
        }
    }

    fn next_element(&mut self) -> Result<String> {
        let mut element = String::new();

        self.discard_whitespace();
        loop {
            match self.next_char {
                Some(c) if !c.is_whitespace() => element.push(c),
                _ => {
                    break;
                }
            }
            self.read_char();
        }

        Ok(element)
    }

    fn read_string(&mut self) -> Result<String> {
        self.discard_whitespace();

        if self.next_char != Some('"') {
            return Err(CompileError::new(ErrorKind::ReadError, "'\"' is expected"));
        }
        self.read_char();

        let mut s = String::new();
        loop {
            match self.next_char {
                Some('"') => {
                    self.read_char();
                    break;
                }
                Some('\\') => {
                    self.read_char();
                    match self.next_char {
                        Some('n') => s.push('\n'),
                        Some('\\') => s.push('\\'),
                        _ => {
                            return Err(CompileError::new(
                                ErrorKind::ReadError,
                                "unknown escape sequence",
                            ))
                        }
                    }
                    self.read_char();
                }
                Some(c) => {
                    s.push(c);
                    self.read_char();
                }
                None => return Err(CompileError::new(ErrorKind::ReadError, "unexpected EOF.")),
            }
        }

        Ok(s)
    }
}

impl Token {
    pub(crate) fn new(kind: TokenKind, line_number: usize, column_number: usize) -> Token {
        Token {
            kind,
            line_number,
            column_number,
        }
    }

    pub fn from_line(line: &str) -> Result<Token> {
        let cs = line.trim().chars();

        let mut reader = TokenReader::new(cs);
        let buf = reader.next_element()?;
        let line_number: usize = buf.parse().unwrap();

        let buf = reader.next_element()?;
        let column_number: usize = buf.parse().unwrap();

        let buf = reader.next_element()?;
        match &buf[..] {
            "Op_multiply" => Ok(Token::new(
                TokenKind::OpMultiply,
                line_number,
                column_number,
            )),
            "Op_divide" => Ok(Token::new(TokenKind::OpDivide, line_number, column_number)),
            "Op_mod" => Ok(Token::new(TokenKind::OpMod, line_number, column_number)),
            "Op_add" => Ok(Token::new(TokenKind::OpAdd, line_number, column_number)),
            "Op_subtract" => Ok(Token::new(
                TokenKind::OpSubtract,
                line_number,
                column_number,
            )),
            "Op_less" => Ok(Token::new(TokenKind::OpLess, line_number, column_number)),
            "Op_lessequal" => Ok(Token::new(
                TokenKind::OpLessEqual,
                line_number,
                column_number,
            )),
            "Op_greater" => Ok(Token::new(TokenKind::OpGreater, line_number, column_number)),
            "Op_greaterequal" => Ok(Token::new(
                TokenKind::OpGreaterEqual,
                line_number,
                column_number,
            )),
            "Op_equal" => Ok(Token::new(TokenKind::OpEqual, line_number, column_number)),
            "Op_notequal" => Ok(Token::new(
                TokenKind::OpNotEqual,
                line_number,
                column_number,
            )),
            "Op_not" => Ok(Token::new(TokenKind::OpNot, line_number, column_number)),
            "Op_assign" => Ok(Token::new(TokenKind::OpAssign, line_number, column_number)),
            "Op_and" => Ok(Token::new(TokenKind::OpAnd, line_number, column_number)),
            "Op_or" => Ok(Token::new(TokenKind::OpOr, line_number, column_number)),
            "LeftParen" => Ok(Token::new(TokenKind::LeftParen, line_number, column_number)),
            "RightParen" => Ok(Token::new(
                TokenKind::RightParen,
                line_number,
                column_number,
            )),
            "LeftBrace" => Ok(Token::new(TokenKind::LeftBrace, line_number, column_number)),
            "RightBrace" => Ok(Token::new(
                TokenKind::RightBrace,
                line_number,
                column_number,
            )),
            "Semicolon" => Ok(Token::new(TokenKind::Semicolon, line_number, column_number)),
            "Comma" => Ok(Token::new(TokenKind::Comma, line_number, column_number)),
            "Keyword_if" => Ok(Token::new(TokenKind::KeywordIf, line_number, column_number)),
            "Keyword_else" => Ok(Token::new(
                TokenKind::KeywordElse,
                line_number,
                column_number,
            )),
            "Keyword_while" => Ok(Token::new(
                TokenKind::KeywordWhile,
                line_number,
                column_number,
            )),
            "Keyword_print" => Ok(Token::new(
                TokenKind::KeywordPrint,
                line_number,
                column_number,
            )),
            "Keyword_putc" => Ok(Token::new(
                TokenKind::KeywordPutc,
                line_number,
                column_number,
            )),
            "Integer" => {
                let i = reader.next_element()?.parse().unwrap();
                Ok(Token::new(
                    TokenKind::Integer(i),
                    line_number,
                    column_number,
                ))
            }
            "Identifier" => {
                let identifier = reader.next_element()?;
                Ok(Token::new(
                    TokenKind::Identifier(identifier),
                    line_number,
                    column_number,
                ))
            }
            "String" => {
                let s = reader.read_string()?;
                Ok(Token::new(TokenKind::String(s), line_number, column_number))
            }
            "End_of_input" => Ok(Token::new(
                TokenKind::EndOfInput,
                line_number,
                column_number,
            )),
            _ => Err(CompileError::new(
                ErrorKind::ReadError,
                format!("unknown token kind: {}", buf),
            )),
        }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn column_number(&self) -> usize {
        self.column_number
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            TokenKind::OpMultiply => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_multiply"
            ),
            TokenKind::OpDivide => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_divide"
            ),
            TokenKind::OpMod => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_mod"
            ),
            TokenKind::OpAdd => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_add"
            ),
            TokenKind::OpSubtract => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_subtract"
            ),
            TokenKind::OpLess => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_less"
            ),
            TokenKind::OpLessEqual => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_lessequal"
            ),
            TokenKind::OpGreater => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, "Op_greater"
            ),
            TokenKind::OpGreaterEqual => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_greaterequal"
            ),
            TokenKind::OpEqual => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_equal"
            ),
            TokenKind::OpNotEqual => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_notequal"
            ),
            TokenKind::OpNot => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_not"
            ),
            TokenKind::OpAssign => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_assign"
            ),
            TokenKind::OpAnd => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Op_and"
            ),
            TokenKind::OpOr => write!(f, "{} {} {}", self.line_number, self.column_number, "Op_or"),
            TokenKind::KeywordIf => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Keyword_if"
            ),
            TokenKind::KeywordElse => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Keyword_else"
            ),
            TokenKind::KeywordWhile => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Keyword_while"
            ),
            TokenKind::KeywordPrint => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Keyword_print"
            ),
            TokenKind::KeywordPutc => write!(
                f,
                "{} {} {}",
                self.line_number, self.column_number, "Keyword_putc"
            ),
            TokenKind::Identifier(ref identifier) => write!(
                f,
                "{} {} Identifier {}",
                self.line_number, self.column_number, identifier
            ),
            TokenKind::Integer(val) => write!(
                f,
                "{} {} Integer {}",
                self.line_number, self.column_number, val
            ),
            TokenKind::String(ref s) => write!(
                f,
                "{} {} String {:?}",
                self.line_number, self.column_number, s
            ),
            TokenKind::EndOfInput => write!(
                f,
                "{} {} String End_of_input",
                self.line_number, self.column_number
            ),
            _ => write!(
                f,
                "{} {} {:?}",
                self.line_number, self.column_number, self.kind
            ),
        }
    }
}
