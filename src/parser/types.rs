// Parser-specific types

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    TagName,      // "colour"
    ComparisonOp, // =
    TagValue,     // "red"
    LogicalOp,    // &
}

pub type TokenDepth = i32;

// Tokens, their type and their parenthesis depth -- e.g. [("colour", TagName, 2), ("=", ComparisonOp, 0)]
pub type MappedRuleTokens = Vec<(String, TokenType, TokenDepth)>;

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
    pub token: Token,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

// Node represented as strings -- e.g. ("|", ["colour", "=", "blue"], ["colour", "!", "red"])
pub type NodeStr = (String, Vec<String>, Vec<String>);

pub struct Rule {
    pub root_node: Node,
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
