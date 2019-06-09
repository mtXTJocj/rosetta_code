use std::fmt;
use std::str::Lines;

#[derive(Debug, Clone)]
pub enum NodeKind {
    Identifier(String),
    String(String),
    Integer(i32),
    Sequence,
    If,
    Prtc,
    Prts,
    Prti,
    While,
    Assign,
    Negate,
    Not,
    Multiply,
    Divide,
    Mod,
    Add,
    Subtract,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    None,
}

#[derive(Debug)]
pub struct ASTNode {
    pub(crate) kind: NodeKind,
    pub(crate) lhs: Option<Box<ASTNode>>,
    pub(crate) rhs: Option<Box<ASTNode>>,
}

impl ASTNode {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn lhs(&self) -> Option<&ASTNode> {
        match self.lhs {
            Some(ref n) => Some(n.as_ref()),
            None => None,
        }
    }

    pub fn rhs(&self) -> Option<&ASTNode> {
        match self.rhs {
            Some(ref n) => Some(n.as_ref()),
            None => None,
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            NodeKind::Identifier(ref i) => write!(f, "Identifier {}\n", i)?,
            NodeKind::String(ref s) => write!(f, "String {:?}\n", s)?,
            NodeKind::Integer(ref i) => write!(f, "Integer {}\n", i)?,
            _ => {
                write!(f, "{:?}\n", self.kind)?;
                match &self.lhs {
                    Some(l) => l.fmt(f)?,
                    None => write!(f, ";\n")?,
                }
                match &self.rhs {
                    Some(r) => r.fmt(f)?,
                    None => write!(f, ";\n")?,
                }
            }
        }
        Ok(())
    }
}

pub struct ASTReader<'a> {
    stream: Lines<'a>,
}

impl<'a> ASTReader<'a> {
    fn make_interior_node(&mut self, kind: NodeKind) -> Option<ASTNode> {
        let lhs = self.make_node();
        let lhs = if let Some(node) = lhs {
            Some(Box::new(node))
        } else {
            None
        };

        let rhs = self.make_node();
        let rhs = if let Some(node) = rhs {
            Some(Box::new(node))
        } else {
            None
        };

        Some(ASTNode { kind, lhs, rhs })
    }

    fn make_identifier(&mut self, identifier: &str) -> Option<ASTNode> {
        Some(ASTNode {
            kind: NodeKind::Identifier(identifier.to_string()),
            lhs: None,
            rhs: None,
        })
    }

    fn make_integer(&mut self, num_str: &str) -> Option<ASTNode> {
        let val = num_str.parse().unwrap();
        Some(ASTNode {
            kind: NodeKind::Integer(val),
            lhs: None,
            rhs: None,
        })
    }

    fn make_string(&mut self, s: &str) -> Option<ASTNode> {
        let mut value = String::new();
        let mut cs = s.chars();
        let mut next_char = cs.next();

        if Some('"') != next_char {
            unreachable!();
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
                        _ => unreachable!(),
                    }
                }
                Some(c) => value.push(c),
                None => unreachable!(),
            }
            next_char = cs.next();
        }
        Some(ASTNode {
            kind: NodeKind::String(value),
            lhs: None,
            rhs: None,
        })
    }

    fn make_node(&mut self) -> Option<ASTNode> {
        match self.stream.next() {
            Some(line) => {
                let elements: Vec<&str> = line.trim().splitn(2, ' ').collect();
                match elements[0] {
                    ";" => None,
                    "Identifier" => self.make_identifier(elements[1].trim()),
                    "Integer" => self.make_integer(elements[1].trim()),
                    "String" => self.make_string(elements[1].trim()),
                    "Sequence" => self.make_interior_node(NodeKind::Sequence),
                    "If" => self.make_interior_node(NodeKind::If),
                    "Prtc" => self.make_interior_node(NodeKind::Prtc),
                    "Prts" => self.make_interior_node(NodeKind::Prts),
                    "Prti" => self.make_interior_node(NodeKind::Prti),
                    "While" => self.make_interior_node(NodeKind::While),
                    "Assign" => self.make_interior_node(NodeKind::Assign),
                    "Negate" => self.make_interior_node(NodeKind::Negate),
                    "Not" => self.make_interior_node(NodeKind::Not),
                    "Multiply" => self.make_interior_node(NodeKind::Multiply),
                    "Divide" => self.make_interior_node(NodeKind::Divide),
                    "Mod" => self.make_interior_node(NodeKind::Mod),
                    "Add" => self.make_interior_node(NodeKind::Add),
                    "Subtract" => self.make_interior_node(NodeKind::Subtract),
                    "Less" => self.make_interior_node(NodeKind::Less),
                    "LessEqual" => self.make_interior_node(NodeKind::LessEqual),
                    "Greater" => self.make_interior_node(NodeKind::Greater),
                    "GreaterEqual" => self.make_interior_node(NodeKind::GreaterEqual),
                    "Equal" => self.make_interior_node(NodeKind::Equal),
                    "NotEqual" => self.make_interior_node(NodeKind::NotEqual),
                    "And" => self.make_interior_node(NodeKind::And),
                    "Or" => self.make_interior_node(NodeKind::Or),
                    _ => unreachable!(),
                }
            }
            None => None,
        }
    }

    pub fn read_ast(stream: Lines) -> ASTNode {
        let mut reader = ASTReader { stream };
        reader.make_node().unwrap()
    }
}
