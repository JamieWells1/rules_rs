// Types

// Aliases

use std::collections::HashMap;

// Tag name -- e.g. "colour"
pub type TagName = String;

// Tag values -- e.g. ["red", "green"]
pub type TagValues = Vec<String>;

// Number of subrule -- e.g. 1
pub type SubRuleNumber = i32;

// Object structure -- e.g. "colour": ["green"]
pub type Object = HashMap<String, Vec<String>>;

// Clause in subrule -- e.g. "colour": "green"
pub type TagKvMap = HashMap<String, String>;

// Structs

pub struct Tag {
    pub name: TagName,
    pub values: TagValues,
}

pub enum ComparisonOp {
    ISEQ,
    NOEQ,
    // To be supported in future:
    // GREQ,
    // LEEQ,
}

pub enum LogicalOp {
    AND,
    OR,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,  // (
    RightParen, // )
    Equals,     // =
    NotEquals,  // !
    And,        // &
    Or,         // |
    Comma,      // ,
    Invalid,    // Initialiser
}

pub struct Node {
    token: Token,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

pub struct Rule {
    pub nodes: Vec<Node>,
}

pub struct SubRule {
    pub expected_count: i32,
    pub actual_count: i32,
    // No. elements (tag_kvs) should be == no. elements (comparison_op - 1)
    pub comparison_ops: Vec<ComparisonOp>,
    pub tag_kvs: TagKvMap,
}

// Impls

impl Default for Node {
    fn default() -> Self {
        Node {
            token: Token::Invalid,
            left: None,
            right: None,
        }
    }
}

impl Default for SubRule {
    fn default() -> Self {
        SubRule {
            expected_count: 2,
            actual_count: 0,
            comparison_ops: Vec::new(),
            tag_kvs: HashMap::new(),
        }
    }
}

impl Token {
    pub fn as_char(&self) -> char {
        match self {
            Token::LeftParen => '(',
            Token::RightParen => ')',
            Token::Equals => '=',
            Token::NotEquals => '!',
            Token::And => '&',
            Token::Or => '|',
            Token::Comma => ',',
            Token::Invalid => panic!("Invalid token has no character representation"),
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '=' => Some(Token::Equals),
            '!' => Some(Token::NotEquals),
            '&' => Some(Token::And),
            '|' => Some(Token::Or),
            ',' => Some(Token::Comma),
            _ => None,
        }
    }
}
