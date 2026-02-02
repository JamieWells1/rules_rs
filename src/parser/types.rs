// Parser-specific types

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    TagName,      // "colour"
    ComparisonOp, // =
    TagValue,     // "red"
    LogicalOp,    // &
}

// Tokens and their type -- e.g. [("colour", TagName), ("=", ComparisonOp)]
pub type MappedRuleTokens = Vec<(String, TokenType)>;

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

pub struct Rule {
    pub nodes: Vec<Node>,
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
