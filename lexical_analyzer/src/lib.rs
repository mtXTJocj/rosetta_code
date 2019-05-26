pub mod error;
pub mod token;

use error::*;
use token::{Token, TokenKind};

use std::str::Chars;

pub struct LexicalAnalyzer<'a> {
    /// 先読みした一文字。EOF の場合に None。
    next_char: Option<char>,
    /// ソースの読み込み元
    stream: Chars<'a>,
    /// 現在の行数
    line_number: usize,
    /// 現在の列数
    column_number: usize,
}

/// c が '0' - '9' なら true
fn is_number(c: char) -> bool {
    c.is_digit(10)
}

/// c が [_,a-z,A-Z] なら true
fn is_alpha(c: char) -> bool {
    c == '_' || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

/// c が [_,a-z,A-Z,0-9] なら true
fn is_alnum(c: char) -> bool {
    is_alpha(c) || is_number(c)
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(mut stream: Chars<'a>) -> Self {
        let next_char = stream.next();
        LexicalAnalyzer {
            next_char,
            stream,
            line_number: 1,
            column_number: 1,
        }
    }

    /// stream を一文字進め、next_char に格納する。
    fn read_char(&mut self) {
        if self.next_char == Some('\n') {
            self.line_number += 1;
            self.column_number = 0;
        }
        self.next_char = self.stream.next();
        self.column_number += 1;
    }

    fn read_escaped_sequence(&mut self) -> Result<char> {
        assert!(self.next_char == Some('\\'));
        self.read_char();
        match self.next_char {
            Some(c) if c == '\\' => Ok('\\'),
            Some(c) if c == 'n' => Ok('\n'),
            Some(_) => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "Unknown escape sequence",
            )),
            None => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "unexpected EOI",
            )),
        }
    }

    /// 空白文字を読みとばす。
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

    /// コメントを読み飛ばす。
    fn discard_comment(&mut self) -> Result<()> {
        loop {
            match self.next_char {
                Some('*') => {
                    self.read_char();
                    match self.next_char {
                        Some('/') => {
                            // コメント終了
                            self.read_char();
                            return Ok(());
                        }
                        Some(_) => {}
                        None => {
                            // コメントの途中で EOF
                            return Err(CompileError::new(
                                ErrorKind::LexicalAnalyzerError,
                                "End-of-file in comment. Closing comment characters not found",
                            ));
                        }
                    }
                }
                Some(_) => {}
                None => {
                    // コメントの途中で EOF
                    return Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "End-of-file in comment. Closing comment characters not found",
                    ));
                }
            }
            self.read_char();
        }
    }

    /// 一文字目が '<' のトークン
    fn read_less(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('=') {
            self.read_char();
            Ok(Token::new(
                TokenKind::OpLessEqual,
                line_number,
                column_number,
            ))
        } else {
            Ok(Token::new(TokenKind::OpLess, line_number, column_number))
        }
    }

    /// 一文字目が '>' のトークン
    fn read_greater(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('=') {
            self.read_char();
            Ok(Token::new(
                TokenKind::OpGreaterEqual,
                line_number,
                column_number,
            ))
        } else {
            Ok(Token::new(TokenKind::OpGreater, line_number, column_number))
        }
    }

    /// 一文字目が '=' のトークン
    fn read_equal(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('=') {
            self.read_char();
            Ok(Token::new(TokenKind::OpEqual, line_number, column_number))
        } else {
            Ok(Token::new(TokenKind::OpAssign, line_number, column_number))
        }
    }

    /// 一文字目が '!' のトークン
    fn read_not(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('=') {
            self.read_char();
            Ok(Token::new(
                TokenKind::OpNotEqual,
                line_number,
                column_number,
            ))
        } else {
            Ok(Token::new(TokenKind::OpNot, line_number, column_number))
        }
    }

    /// 一文字目が '&' のトークン
    fn read_and(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('&') {
            self.read_char();
            Ok(Token::new(TokenKind::OpAnd, line_number, column_number))
        } else {
            Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "invalid character: '&' is expected",
            ))
        }
    }

    /// 一文字目が '|' のトークン
    fn read_or(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('|') {
            self.read_char();
            Ok(Token::new(TokenKind::OpOr, line_number, column_number))
        } else {
            Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "invalid character: '|' is expected",
            ))
        }
    }

    /// '/' の次の文字が '*' 以外: OpDivide
    /// '/' の次の文字が '*'     : コメントを読み飛ばし、次のトークンを返す
    fn read_div(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        if self.next_char == Some('*') {
            // comment
            self.read_char();
            self.discard_comment()?;
            self.next_token()
        } else {
            Ok(Token::new(TokenKind::OpDivide, line_number, column_number))
        }
    }

    /// identifier、もしくはキーワードを読み込む。
    /// 一旦中て identifier として読み込んだ後で、キーワードであるかをチェック
    fn read_identifier(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        let mut identifier = String::new();
        identifier.push(self.next_char.unwrap());
        self.read_char();

        loop {
            if let Some(c) = self.next_char {
                if is_alnum(c) {
                    identifier.push(c);
                    self.read_char();
                } else {
                    break;
                }
            } else {
                // End of Input
                break;
            }
        }

        Ok(Token::new(
            match &identifier[..] {
                "if" => TokenKind::KeywordIf,
                "else" => TokenKind::KeywordElse,
                "while" => TokenKind::KeywordWhile,
                "print" => TokenKind::KeywordPrint,
                "putc" => TokenKind::KeywordPutc,
                _ => TokenKind::Identifier(identifier),
            },
            line_number,
            column_number,
        ))
    }

    /// 数値リテラルを読み込む
    fn read_integer_literal(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        let mut number_string = String::new();
        number_string.push(self.next_char.unwrap());
        self.read_char();

        loop {
            match self.next_char {
                Some(c) if is_number(c) => {
                    number_string.push(c);
                    self.read_char();
                }
                Some(c) if c.is_alphabetic() => {
                    return Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "Invalid number. Starts like a number, but ends in non-numeric-characters",
                    ));
                }
                _ => {
                    break;
                }
            }
        }

        match number_string.parse() {
            Ok(num) => Ok(Token::new(
                TokenKind::Integer(num),
                line_number,
                column_number,
            )),
            Err(_) => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "invalid number.",
            )),
        }
    }

    fn read_char_literal(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        self.read_char();

        match self.next_char {
            Some('\'') => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "Empty character constant",
            )),
            Some('\n') => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "invalid char literal",
            )),
            Some(c) => {
                let n = if c == '\\' {
                    self.read_escaped_sequence()?
                } else {
                    c
                } as i32;
                self.read_char();
                match self.next_char {
                    Some(c) if c == '\'' => {
                        self.read_char();
                        Ok(Token::new(
                            TokenKind::Integer(n),
                            line_number,
                            column_number,
                        ))
                    }
                    Some(_) => Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "Multi-character constant.",
                    )),
                    None => Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "unexpected EOI",
                    )),
                }
            }
            None => Err(CompileError::new(
                ErrorKind::LexicalAnalyzerError,
                "unexpected EOI",
            )),
        }
    }

    /// 文字列リテラルを読み込む。
    fn read_string_literal(&mut self, line_number: usize, column_number: usize) -> Result<Token> {
        let mut s = String::new();

        self.read_char();
        loop {
            match self.next_char {
                Some('"') => {
                    self.read_char();
                    return Ok(Token::new(TokenKind::String(s), line_number, column_number));
                }
                Some('\n') => {
                    return Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "End-of-line while scanning string literal",
                    ));
                }
                Some(c) => {
                    s.push(if c == '\\' {
                        self.read_escaped_sequence()?
                    } else {
                        c
                    });
                }
                None => {
                    return Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        "unexpected EOF",
                    ));
                }
            }
            self.read_char();
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.discard_whitespace();

        let start_line = self.line_number;
        let start_column = self.column_number;
        match self.next_char {
            Some('*') => {
                self.read_char();
                Ok(Token::new(TokenKind::OpMultiply, start_line, start_column))
            }
            Some('%') => {
                self.read_char();
                Ok(Token::new(TokenKind::OpMod, start_line, start_column))
            }
            Some('+') => {
                self.read_char();
                Ok(Token::new(TokenKind::OpAdd, start_line, start_column))
            }
            Some('-') => {
                self.read_char();
                Ok(Token::new(TokenKind::OpSubtract, start_line, start_column))
            }
            Some('(') => {
                self.read_char();
                Ok(Token::new(TokenKind::LeftParen, start_line, start_column))
            }
            Some(')') => {
                self.read_char();
                Ok(Token::new(TokenKind::RightParen, start_line, start_column))
            }
            Some('{') => {
                self.read_char();
                Ok(Token::new(TokenKind::LeftBrace, start_line, start_column))
            }
            Some('}') => {
                self.read_char();
                Ok(Token::new(TokenKind::RightBrace, start_line, start_column))
            }
            Some(';') => {
                self.read_char();
                Ok(Token::new(TokenKind::Semicolon, start_line, start_column))
            }
            Some(',') => {
                self.read_char();
                Ok(Token::new(TokenKind::Comma, start_line, start_column))
            }
            Some('<') => self.read_less(start_line, start_column),
            Some('>') => self.read_greater(start_line, start_column),
            Some('=') => self.read_equal(start_line, start_column),
            Some('!') => self.read_not(start_line, start_column),
            Some('&') => self.read_and(start_line, start_column),
            Some('|') => self.read_or(start_line, start_column),

            Some('/') => self.read_div(start_line, start_column),

            Some(c) => {
                if is_alpha(c) {
                    self.read_identifier(start_line, start_column)
                } else if is_number(c) {
                    self.read_integer_literal(start_line, start_column)
                } else if c == '\'' {
                    self.read_char_literal(start_line, start_column)
                } else if c == '"' {
                    self.read_string_literal(start_line, start_column)
                } else {
                    Err(CompileError::new(
                        ErrorKind::LexicalAnalyzerError,
                        format!("Unrecognized character: {}", c),
                    ))
                }
            }

            None => Ok(Token::new(TokenKind::EndOfInput, start_line, start_column)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let s = "".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());

        // EOI は何度呼んでも EOI のまま
        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_whitespace() {
        let s = "\n\t ".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(3, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    /// 一文字だけで確定できるトークン
    fn test_one_char() {
        let s = "*%+-(){};,".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::OpMultiply, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(2, token.column_number());
        assert_eq!(TokenKind::OpMod, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(3, token.column_number());
        assert_eq!(TokenKind::OpAdd, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(4, token.column_number());
        assert_eq!(TokenKind::OpSubtract, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(5, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(TokenKind::LeftBrace, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(8, token.column_number());
        assert_eq!(TokenKind::RightBrace, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(9, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(10, token.column_number());
        assert_eq!(TokenKind::Comma, *token.kind());
    }

    #[test]
    /// 2 文字目で確定できるトークン
    fn test_two_chars() {
        let s = "<<=>>====!!=&&||".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::OpLess, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(2, token.column_number());
        assert_eq!(TokenKind::OpLessEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(4, token.column_number());
        assert_eq!(TokenKind::OpGreater, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(5, token.column_number());
        assert_eq!(TokenKind::OpGreaterEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(TokenKind::OpEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(9, token.column_number());
        assert_eq!(TokenKind::OpAssign, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(10, token.column_number());
        assert_eq!(TokenKind::OpNot, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(11, token.column_number());
        assert_eq!(TokenKind::OpNotEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(13, token.column_number());
        assert_eq!(TokenKind::OpAnd, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(15, token.column_number());
        assert_eq!(TokenKind::OpOr, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(17, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_comment() {
        let s = "//**a*b**// /* a / b *//".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::OpDivide, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(24, token.column_number());
        assert_eq!(TokenKind::OpDivide, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(25, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_identifier() {
        let s = "ifprint fred42".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::Identifier("ifprint".to_string()), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(9, token.column_number());
        assert_eq!(TokenKind::Identifier("fred42".to_string()), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(15, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_keyword() {
        let s = "if else while print putc".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordIf, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(4, token.column_number());
        assert_eq!(TokenKind::KeywordElse, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(9, token.column_number());
        assert_eq!(TokenKind::KeywordWhile, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(15, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(21, token.column_number());
        assert_eq!(TokenKind::KeywordPutc, *token.kind());
    }

    #[test]
    fn test_integer_literal() {
        let s = "0 42<43".to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::Integer(0), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(3, token.column_number());
        assert_eq!(TokenKind::Integer(42), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(5, token.column_number());
        assert_eq!(TokenKind::OpLess, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::Integer(43), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(8, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_char_literal() {
        let s = r#"'a''\\''\n'"#.to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::Integer(97), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(4, token.column_number());
        assert_eq!(TokenKind::Integer(92), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(1, token.line_number());
        assert_eq!(8, token.column_number());
        assert_eq!(TokenKind::Integer(10), *token.kind());
    }

    #[test]
    fn test_case_1() {
        let s = r#"/*
  Hello world
 */
print("Hello, World!\n");
"#
        .to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(
            TokenKind::String("Hello, World!\n".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(24, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(25, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_case_2() {
        let s = r#"/*
  Show Ident and Integers
 */
phoenix_number = 142857;
print(phoenix_number, "\n");
"#
        .to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(
            TokenKind::Identifier("phoenix_number".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpAssign, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(18, token.column_number());
        assert_eq!(TokenKind::Integer(142857), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(24, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(
            TokenKind::Identifier("phoenix_number".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(21, token.column_number());
        assert_eq!(TokenKind::Comma, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(23, token.column_number());
        assert_eq!(TokenKind::String("\n".to_string()), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(27, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(28, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(6, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_case_3() {
        let s = r#"/*
  All lexical tokens - not syntactically correct, but that will
  have to wait until syntax analysis
 */
/* Print   */  print    /* Sub     */  -
/* Putc    */  putc     /* Lss     */  <
/* If      */  if       /* Gtr     */  >
/* Else    */  else     /* Leq     */  <=
/* While   */  while    /* Geq     */  >=
/* Lbrace  */  {        /* Eq      */  ==
/* Rbrace  */  }        /* Neq     */  !=
/* Lparen  */  (        /* And     */  &&
/* Rparen  */  )        /* Or      */  ||
/* Uminus  */  -        /* Semi    */  ;
/* Not     */  !        /* Comma   */  ,
/* Mul     */  *        /* Assign  */  =
/* Div     */  /        /* Integer */  42
/* Mod     */  %        /* String  */  "String literal"
/* Add     */  +        /* Ident   */  variable_name
/* character literal */  '\n'
/* character literal */  '\\'
/* character literal */  ' '
"#
        .to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpSubtract, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(6, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::KeywordPutc, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(6, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpLess, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(7, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::KeywordIf, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(7, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpGreater, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(8, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::KeywordElse, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(8, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpLessEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(9, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::KeywordWhile, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(9, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpGreaterEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(10, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::LeftBrace, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(10, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(11, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::RightBrace, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(11, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpNotEqual, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(12, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(12, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpAnd, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(13, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(13, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpOr, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(14, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpSubtract, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(14, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(15, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpNot, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(15, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::Comma, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(16, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpMultiply, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(16, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::OpAssign, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(17, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpDivide, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(17, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(TokenKind::Integer(42), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(18, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpMod, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(18, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(
            TokenKind::String("String literal".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(19, token.line_number());
        assert_eq!(16, token.column_number());
        assert_eq!(TokenKind::OpAdd, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(19, token.line_number());
        assert_eq!(40, token.column_number());
        assert_eq!(
            TokenKind::Identifier("variable_name".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(20, token.line_number());
        assert_eq!(26, token.column_number());
        assert_eq!(TokenKind::Integer(10), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(21, token.line_number());
        assert_eq!(26, token.column_number());
        assert_eq!(TokenKind::Integer(92), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(22, token.line_number());
        assert_eq!(26, token.column_number());
        assert_eq!(TokenKind::Integer(32), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(23, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }

    #[test]
    fn test_case_4() {
        let s = r#"/*** test printing, embedded \n and comments with lots of '*' ***/
print(42);
print("\nHello World\nGood Bye\nok\n");
print("Print a slash n - \\n.\n");
"#
        .to_string();

        let mut lexer = LexicalAnalyzer::new(s.chars());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(TokenKind::Integer(42), *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(9, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(2, token.line_number());
        assert_eq!(10, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(3, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(3, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(3, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(
            TokenKind::String("\nHello World\nGood Bye\nok\n".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(3, token.line_number());
        assert_eq!(38, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(3, token.line_number());
        assert_eq!(39, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::KeywordPrint, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(6, token.column_number());
        assert_eq!(TokenKind::LeftParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(7, token.column_number());
        assert_eq!(
            TokenKind::String("Print a slash n - \\n.\n".to_string()),
            *token.kind()
        );

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(33, token.column_number());
        assert_eq!(TokenKind::RightParen, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(4, token.line_number());
        assert_eq!(34, token.column_number());
        assert_eq!(TokenKind::Semicolon, *token.kind());

        let token = lexer.next_token().unwrap();
        assert_eq!(5, token.line_number());
        assert_eq!(1, token.column_number());
        assert_eq!(TokenKind::EndOfInput, *token.kind());
    }
}
