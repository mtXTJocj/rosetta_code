use std::fmt;

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

impl Token {
    pub(crate) fn new(kind: TokenKind, line_number: usize, column_number: usize) -> Token {
        Token {
            kind,
            line_number,
            column_number,
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
