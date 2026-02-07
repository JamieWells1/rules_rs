// Parser for .rules files
use crate::err::RulesError;
use crate::parser::types::{MappedRuleTokens, Node, NodeStr, Rule, TokenDepth, TokenType};
use crate::types::{self, SubRule};
use crate::utils::file;
use crate::utils::string;

use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

static TOKEN_PRECEDENCE: LazyLock<HashMap<&String, i32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("&", 0);
    m.insert("|", 1);
    m.insert("=", 2);
    m.insert("!", 2);
    m
});

// All valid operator characters in rule syntax
const ALL_OP_CHARS: &[char] = &['(', ')', '=', '!', '&', '|', ','];
// Operators that expect a TagValue on the right-hand side
const RHS_CHARS: &[char] = &['=', '!', ',', ')'];
// Operators that expect a TagName on the left-hand side
const LHS_CHARS: &[char] = &['&', '|', '('];

pub struct RuleParser {
    m_mapped_tags: HashMap<types::TagName, types::TagValues>,
}

impl RuleParser {
    pub fn new(tags: HashMap<types::TagName, types::TagValues>) -> Self {
        RuleParser {
            m_mapped_tags: tags,
        }
    }

    pub fn validate_rule(&self, rule: &str) -> Result<(), RulesError> {
        self.validate_rule_internal(rule)
    }

    fn validate_rule_internal(&self, line: &str) -> Result<(), RulesError> {
        if file::line_blank_or_comment(line) {
            return Ok(());
        }

        let original_line = line.to_string();

        let line =
            string::normalise(line).map_err(|e| Self::add_error_context(e, &original_line))?;

        let tokens: MappedRuleTokens =
            Self::map_rule_tokens(&Self::tokenise_rule(&line.to_string())?);

        Self::check_rule_syntax(&tokens).map_err(|e| Self::add_error_context(e, &original_line))?;

        self.check_valid_tags(&tokens)
            .map_err(|e| Self::add_error_context(e, &original_line))?;

        Ok(())
    }

    fn add_error_context(error: RulesError, rule: &str) -> RulesError {
        match error {
            RulesError::RuleParseError(msg) => {
                RulesError::RuleParseError(format!("'{}': {}", rule, msg))
            }
            other => other,
        }
    }

    fn get_expected_token_type(
        parsed_tokens: &Vec<String>,
        paren_depth: i32,
    ) -> Result<TokenType, RulesError> {
        // If no tokens yet, first token should be TagName or opening paren
        if parsed_tokens.is_empty() {
            return Ok(TokenType::TagName);
        }

        let last_token = parsed_tokens.last().unwrap();

        // Last token is an operator
        let c = if last_token.len() == 1 {
            last_token.chars().next()
        } else {
            None
        };

        if let Some(ch) = c {
            // Last token is an operator
            if ALL_OP_CHARS.contains(&ch) {
                if ch == '(' {
                    // After '(', could be TagName or another '(' for nesting
                    Ok(TokenType::TagName) // Both '(' and TagName are valid here
                } else if ch == ')' {
                    // After ')', could be LogicalOp, another ')', or end of expression
                    if paren_depth > 0 {
                        // Still inside parens, could be ')' or LogicalOp
                        Ok(TokenType::LogicalOp) // Accept both
                    } else {
                        // All parens closed, must be LogicalOp or end
                        Ok(TokenType::LogicalOp)
                    }
                } else if RHS_CHARS.contains(&ch) {
                    Ok(TokenType::TagValue)
                } else if LHS_CHARS.contains(&ch) {
                    Ok(TokenType::TagName)
                } else {
                    Err(RulesError::RuleParseError(
                        format!("Invalid token encountered: {}", ch).to_string(),
                    ))
                }
            } else {
                Err(RulesError::RuleParseError(
                    format!("Invalid token encountered: {}", ch).to_string(),
                ))
            }
        } else {
            // Last token is a word, check operator before it
            if parsed_tokens.len() < 2 {
                // Only one token (the word itself), next should be a comparison operator
                return Ok(TokenType::ComparisonOp);
            }

            let second_to_last_token = &parsed_tokens[parsed_tokens.len() - 2];
            if second_to_last_token.len() > 1 {
                return Err(RulesError::RuleParseError(
                    format!(
                        "Expected operator but got string instead: {}",
                        second_to_last_token
                    )
                    .to_string(),
                ));
            }

            let c: char = second_to_last_token.chars().next().unwrap();

            if c == '(' {
                Ok(TokenType::ComparisonOp)
            } else if c == ')' {
                Ok(TokenType::TagName)
            } else if RHS_CHARS.contains(&c) {
                Ok(TokenType::LogicalOp)
            } else if LHS_CHARS.contains(&c) {
                Ok(TokenType::ComparisonOp)
            } else {
                Err(RulesError::RuleParseError(
                    format!("Invalid token encountered: {}", c).to_string(),
                ))
            }
        }
    }

    fn tokenise_rule(rule: &String) -> Result<Vec<String>, RulesError> {
        let mut parsed_tokens: Vec<String> = Vec::new();
        let mut current_word = String::new();
        let mut paren_depth = 0;

        // For comma expansion
        let mut last_tag_name: Option<String> = None;
        let mut last_comparison_op: Option<String> = None;

        for c in rule.trim().chars() {
            if ALL_OP_CHARS.contains(&c) {
                if !current_word.is_empty() {
                    let expected_token_type =
                        Self::get_expected_token_type(&parsed_tokens, paren_depth)?;
                    let token = current_word.trim().to_string();
                    parsed_tokens.push(token.clone());

                    if expected_token_type == TokenType::TagName {
                        last_tag_name = Some(token);
                    }

                    current_word.clear();
                }

                // Expand comma to regular OR expression
                if c == ',' {
                    let tag_name = last_tag_name.as_ref().ok_or_else(|| {
                        RulesError::RuleParseError(
                            "Comma must follow a complete tag comparison".to_string(),
                        )
                    })?;
                    let comparison_op = last_comparison_op.as_ref().ok_or_else(|| {
                        RulesError::RuleParseError(
                            "Comma must follow a complete tag comparison".to_string(),
                        )
                    })?;

                    parsed_tokens.push("|".to_string());
                    parsed_tokens.push(tag_name.to_string());
                    parsed_tokens.push(comparison_op.to_string());

                    continue;
                }

                let expected_token_type =
                    Self::get_expected_token_type(&parsed_tokens, paren_depth)?;
                let token = c.to_string();
                parsed_tokens.push(token.clone());

                if expected_token_type == TokenType::ComparisonOp {
                    last_comparison_op = Some(token);
                }

                if c == '(' {
                    paren_depth += 1;
                } else if c == ')' {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        return Err(RulesError::RuleParseError(
                            "Unmatched closing parenthesis".to_string(),
                        ));
                    }
                }
            } else if c == ' ' {
                if !current_word.is_empty() {
                    let expected_token_type =
                        Self::get_expected_token_type(&parsed_tokens, paren_depth)?;
                    let token = current_word.trim().to_string();
                    parsed_tokens.push(token.clone());

                    if expected_token_type == TokenType::TagName {
                        last_tag_name = Some(token);
                    }

                    current_word.clear();
                }
            } else {
                current_word.push(c);
            }
        }

        if !current_word.is_empty() {
            let token = current_word.trim().to_string();
            parsed_tokens.push(token);
        }

        if paren_depth != 0 {
            return Err(RulesError::RuleParseError(
                "Unmatched opening parenthesis".to_string(),
            ));
        }

        Ok(parsed_tokens)
    }

    fn map_rule_tokens(tokens: &Vec<String>) -> MappedRuleTokens {
        let mut mapped_token_list: Vec<(String, TokenType, TokenDepth)> = Vec::new();
        let mut parsed_tokens: Vec<String> = Vec::new();
        let mut paren_depth = 0;

        for token in tokens {
            let expected_token_type = Self::get_expected_token_type(&parsed_tokens, paren_depth);
            parsed_tokens.push(token.clone());
            mapped_token_list.push((token.clone(), expected_token_type.unwrap(), paren_depth));

            if token == "(" {
                paren_depth += 1;
            } else if token == ")" {
                paren_depth -= 1;
            }
        }

        mapped_token_list
    }

    fn check_rule_syntax(tokens: &MappedRuleTokens) -> Result<(), RulesError> {
        let mut prev_token: Option<&TokenType> = None;

        for (key, token_type, _paren_depth) in tokens.iter() {
            if key == "(" || key == ")" {
                continue;
            }

            match (prev_token, token_type) {
                // Valid transitions
                (None, TokenType::TagName) => {}
                (Some(TokenType::TagName), TokenType::ComparisonOp) => {}
                (Some(TokenType::ComparisonOp), TokenType::TagValue) => {}
                (Some(TokenType::TagValue), TokenType::LogicalOp) => {}
                (Some(TokenType::LogicalOp), TokenType::TagName) => {}

                // Invalid transitions
                (None, _) => {
                    return Err(RulesError::RuleParseError(format!(
                        "Rule must start with a tag name, found {:?}",
                        token_type
                    )));
                }
                (Some(prev), current) => {
                    return Err(RulesError::RuleParseError(format!(
                        "Invalid token sequence: {:?} followed by {:?}",
                        prev, current
                    )));
                }
            }

            prev_token = Some(token_type);
        }

        match prev_token {
            Some(TokenType::TagValue) => Ok(()),
            Some(other) => Err(RulesError::RuleParseError(format!(
                "Rule must end with a tag value, ended with {:?}",
                other
            ))),
            None => Err(RulesError::RuleParseError("Empty rule".to_string())),
        }
    }

    fn check_valid_tags(&self, tokens: &MappedRuleTokens) -> Result<(), RulesError> {
        let mut last_tag_name: Option<String> = None;

        for (key, token_type, _paren_depth) in tokens.iter() {
            let key = String::from(key).to_lowercase();
            if *token_type == TokenType::TagName {
                if key == "(" || key == ")" {
                    continue;
                }

                if !self.m_mapped_tags.contains_key(&key) {
                    return Err(RulesError::RuleParseError(format!(
                        "Rule contains invalid TagName: {}",
                        key
                    )));
                }
                last_tag_name = Some(key.clone());
            } else if *token_type == TokenType::TagValue {
                let tag_name = last_tag_name.as_ref().ok_or_else(|| {
                    RulesError::RuleParseError(format!(
                        "TagValue '{}' has no associated TagName",
                        key
                    ))
                })?;

                let valid_values = self.m_mapped_tags.get(tag_name).ok_or_else(|| {
                    RulesError::RuleParseError(format!(
                        "No TagName '{}' found for TagValue '{}'",
                        tag_name, key
                    ))
                })?;

                if !valid_values.contains(&key) {
                    return Err(RulesError::RuleParseError(format!(
                        "Rule contains invalid TagValue: '{}' is not a valid value for TagName '{}'",
                        key, tag_name
                    )));
                }
            }
        }

        Ok(())
    }

    fn find_lowest_prec_op_index(tokens: &Vec<String>) -> usize {
        let mut lowest_prec_token: Option<(usize, TokenDepth, i32)> = None; // Index, parenthesis depth and precedence
        let mapped_tokens: MappedRuleTokens = Self::map_rule_tokens(&tokens);

        for (i, token) in mapped_tokens.iter().enumerate() {
            let paren_depth: i32 = token.2;
            let token: &String = &token.0;

            if let Some(&token_prec) = TOKEN_PRECEDENCE.get(token) {
                let mut reassign: bool = false;

                if let Some(lowest) = lowest_prec_token {
                    let lowest_prec_token_depth: i32 = lowest.1;
                    if paren_depth < lowest_prec_token_depth {
                        reassign = true;
                    } else if lowest_prec_token_depth == paren_depth && token_prec < lowest.2 {
                        reassign = true;
                    }
                } else {
                    reassign = true;
                }

                if reassign {
                    lowest_prec_token = Some((i, paren_depth, token_prec));
                }
            }
        }

        lowest_prec_token.map(|(idx, _, _)| idx).unwrap_or(0)
    }

    fn contains_logical_op(tokens: &[String]) -> bool {
        tokens.iter().any(|t| t == "&" || t == "|")
    }

    fn create_leaf_node(tokens: Vec<String>) -> Result<Node, RulesError> {
        if tokens.len() != 3 {
            return Err(RulesError::RuleParseError(format!(
                "Invalid leaf node: expected 3 tokens, got {}",
                tokens.len()
            )));
        }

        // Middle token is comparator
        let operator = &tokens[1];
        let token = if operator == "=" {
            crate::parser::types::Token::Equals
        } else if operator == "!" {
            crate::parser::types::Token::NotEquals
        } else {
            return Err(RulesError::RuleParseError(format!(
                "Invalid comparison operator: {}",
                operator
            )));
        };

        Ok(Node {
            token,
            left: None,
            right: None,
        })
    }

    // Recursive function to build AST from tokens
    fn build_ast(tokens: Vec<String>) -> Result<Node, RulesError> {
        let tokens: Vec<String> = if tokens.len() > 2
            && tokens.first() == Some(&"(".to_string())
            && tokens.last() == Some(&")".to_string())
        {
            tokens[1..tokens.len() - 1].to_vec()
        } else {
            tokens
        };

        // If no logical operators, this is a leaf node
        // E.g., ["colour", "=", "red"] is a leaf
        if !Self::contains_logical_op(&tokens) {
            return Self::create_leaf_node(tokens);
        }

        let op_index = Self::find_lowest_prec_op_index(&tokens);

        let operator_str = &tokens[op_index];
        let operator_token = if operator_str == "&" {
            crate::parser::types::Token::And
        } else if operator_str == "|" {
            crate::parser::types::Token::Or
        } else {
            return Err(RulesError::RuleParseError(format!(
                "Expected logical operator, found: {}",
                operator_str
            )));
        };

        // Split tokens into left and right subtrees
        // e.g. left: ["colour", "=", "red"]
        //      right: ["size", "=", "large"]
        let left_tokens = tokens[0..op_index].to_vec();
        let right_tokens = tokens[op_index + 1..].to_vec();

        // Build subtrees
        let left_child = Self::build_ast(left_tokens)?;
        let right_child = Self::build_ast(right_tokens)?;

        Ok(Node {
            token: operator_token,
            left: Some(Box::new(left_child)),
            right: Some(Box::new(right_child)),
        })
    }

    fn string_to_rule(&self, rule_str: &str) -> Result<Rule, RulesError> {
        // Validate the rule syntax first
        Self::validate_rule(&self, rule_str)?;

        // Tokenize the rule string into a vector of tokens
        // E.g., "colour = red & size = large" becomes:
        // ["colour", "=", "red", "&", "size", "=", "large"]
        let tokens = Self::tokenise_rule(&rule_str.to_string())?;
        let root: Node = Self::build_ast(tokens)?;

        Ok(Rule { root_node: root })
    }

    fn rule_to_dnf_subrule(&self, rule: Rule) -> Result<SubRule, RulesError> {
        // TODO: Convert AST to Disjunctive Normal Form
        unimplemented!()
    }

    // Main entry point for parsing rule files.
    // Converts all .rules files into Disjunctive Normal Form (DNF) subrules.
    pub fn parse_rules(
        mapped_tags: HashMap<types::TagName, types::TagValues>,
    ) -> Result<Vec<SubRule>, RulesError> {
        let parser = RuleParser {
            m_mapped_tags: mapped_tags,
        };

        let mut dnf_subrules: Vec<SubRule> = Vec::new();
        let all_files: Vec<String> = file::read_files_in_dir("config/*.rules")?;

        for file in all_files.iter() {
            for line in file.lines() {
                if file::line_blank_or_comment(line) {
                    continue;
                }

                // Parse string to AST, then convert to DNF representation
                let rule = parser.string_to_rule(line)?;
                let subrule = parser.rule_to_dnf_subrule(rule)?;

                dnf_subrules.push(subrule);
            }
        }

        Ok(dnf_subrules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Find a token in the token list
    fn find_token<'a>(
        tokens: &'a [(String, TokenType, TokenDepth)],
        token_str: &str,
    ) -> Option<&'a TokenType> {
        tokens
            .iter()
            .find(|(s, _, _)| s == token_str)
            .map(|(_, t, _)| t)
    }

    // Count occurrences of a token
    fn count_token(tokens: &[(String, TokenType, TokenDepth)], token_str: &str) -> usize {
        tokens.iter().filter(|(s, _, _)| s == token_str).count()
    }

    // Tests for get_expected_token_type
    #[test]
    fn test_get_expected_token_type_after_open_paren() {
        let tokens = vec!["(".to_string()];
        let result = RuleParser::get_expected_token_type(&tokens, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagName);
    }

    #[test]
    fn test_get_expected_token_type_after_close_paren() {
        let tokens = vec![
            "(".to_string(),
            "colour".to_string(),
            "=".to_string(),
            "red".to_string(),
            ")".to_string(),
        ];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::LogicalOp);
    }

    #[test]
    fn test_get_expected_token_type_after_close_paren_nested() {
        let tokens = vec![
            "(".to_string(),
            "(".to_string(),
            "colour".to_string(),
            "=".to_string(),
            "red".to_string(),
            ")".to_string(),
        ];
        let result = RuleParser::get_expected_token_type(&tokens, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::LogicalOp);
    }

    #[test]
    fn test_get_expected_token_type_after_equals() {
        let tokens = vec!["colour".to_string(), "=".to_string()];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagValue);
    }

    #[test]
    fn test_get_expected_token_type_after_not_equals() {
        let tokens = vec!["colour".to_string(), "!".to_string()];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagValue);
    }

    #[test]
    fn test_get_expected_token_type_after_and() {
        let tokens = vec![
            "colour".to_string(),
            "=".to_string(),
            "red".to_string(),
            "&".to_string(),
        ];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagName);
    }

    #[test]
    fn test_get_expected_token_type_after_or() {
        let tokens = vec![
            "colour".to_string(),
            "=".to_string(),
            "red".to_string(),
            "|".to_string(),
        ];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagName);
    }

    #[test]
    fn test_get_expected_token_type_after_tag_name_following_open_paren() {
        let tokens = vec!["(".to_string(), "colour".to_string()];
        let result = RuleParser::get_expected_token_type(&tokens, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::ComparisonOp);
    }

    #[test]
    fn test_get_expected_token_type_after_tag_value() {
        let tokens = vec!["colour".to_string(), "=".to_string(), "red".to_string()];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::LogicalOp);
    }

    #[test]
    fn test_get_expected_token_type_empty_vector() {
        let tokens = vec![];
        let result = RuleParser::get_expected_token_type(&tokens, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TokenType::TagName);
    }

    // Tests for tokenise_rule (which now does the parsing)
    #[test]
    fn test_tokenise_rule_simple_rule() {
        let rule = "colour = red";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens, vec!["colour", "=", "red"]);
    }

    #[test]
    fn test_tokenise_rule_with_parentheses() {
        let rule = "(colour = red)";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens, vec!["(", "colour", "=", "red", ")"]);
    }

    #[test]
    fn test_tokenise_rule_nested_parentheses() {
        let rule = "((colour = red))";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens, vec!["(", "(", "colour", "=", "red", ")", ")"]);
    }

    #[test]
    fn test_tokenise_rule_with_and_operator() {
        let rule = "colour = red & size = large";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(
            tokens,
            vec!["colour", "=", "red", "&", "size", "=", "large"]
        );
    }

    #[test]
    fn test_tokenise_rule_with_or_operator() {
        let rule = "colour = red | colour = blue";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(
            tokens,
            vec!["colour", "=", "red", "|", "colour", "=", "blue"]
        );
    }

    #[test]
    fn test_tokenise_rule_with_not_equals() {
        let rule = "colour ! red";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens, vec!["colour", "!", "red"]);
    }

    #[test]
    fn test_tokenise_rule_complex_nested() {
        let rule = "((colour = red) & (size = large))";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 13);
        assert_eq!(
            tokens,
            vec![
                "(", "(", "colour", "=", "red", ")", "&", "(", "size", "=", "large", ")", ")"
            ]
        );
    }

    #[test]
    fn test_tokenise_rule_unmatched_opening_paren() {
        let rule = "(colour = red";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Unmatched opening parenthesis"));
        } else {
            panic!("Expected RuleParseError about unmatched opening parenthesis");
        }
    }

    #[test]
    fn test_tokenise_rule_unmatched_closing_paren() {
        let rule = "colour = red)";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Unmatched closing parenthesis"));
        } else {
            panic!("Expected RuleParseError about unmatched closing parenthesis");
        }
    }

    #[test]
    fn test_tokenise_rule_extra_whitespace() {
        let rule = "  colour   =   red  ";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens, vec!["colour", "=", "red"]);
    }

    #[test]
    fn test_tokenise_rule_no_spaces() {
        let rule = "colour=red";
        let result = RuleParser::tokenise_rule(&rule.to_string());

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens, vec!["colour", "=", "red"]);
    }

    // Tests for map_rule_tokens (now takes Vec<String> and returns MappedRuleTokens)
    #[test]
    fn test_map_rule_tokens_simple() {
        let tokens = vec!["colour".to_string(), "=".to_string(), "red".to_string()];
        let mapped = RuleParser::map_rule_tokens(&tokens);

        assert_eq!(mapped.len(), 3);
        assert_eq!(find_token(&mapped, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&mapped, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&mapped, "red"), Some(&TokenType::TagValue));
    }

    #[test]
    fn test_map_rule_tokens_with_logical_op() {
        let tokens = vec![
            "colour".to_string(),
            "=".to_string(),
            "red".to_string(),
            "&".to_string(),
            "size".to_string(),
            "=".to_string(),
            "large".to_string(),
        ];
        let mapped = RuleParser::map_rule_tokens(&tokens);

        assert_eq!(mapped.len(), 7);
        assert_eq!(find_token(&mapped, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&mapped, "red"), Some(&TokenType::TagValue));
        assert_eq!(find_token(&mapped, "&"), Some(&TokenType::LogicalOp));
        assert_eq!(find_token(&mapped, "size"), Some(&TokenType::TagName));
        assert_eq!(find_token(&mapped, "large"), Some(&TokenType::TagValue));
        assert_eq!(count_token(&mapped, "="), 2);
    }

    // Tests for check_rule_syntax
    #[test]
    fn test_check_rule_syntax_valid_simple() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rule_syntax_valid_with_parentheses() {
        let tokens = vec![
            ("(".to_string(), TokenType::TagName, 1),
            ("colour".to_string(), TokenType::TagName, 1),
            ("=".to_string(), TokenType::ComparisonOp, 1),
            ("red".to_string(), TokenType::TagValue, 1),
            (")".to_string(), TokenType::LogicalOp, 1),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rule_syntax_valid_with_logical_op() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 1),
            ("=".to_string(), TokenType::ComparisonOp, 1),
            ("red".to_string(), TokenType::TagValue, 1),
            ("&".to_string(), TokenType::LogicalOp, 1),
            ("size".to_string(), TokenType::TagName, 1),
            ("=".to_string(), TokenType::ComparisonOp, 1),
            ("large".to_string(), TokenType::TagValue, 1),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rule_syntax_valid_complex() {
        let tokens = vec![
            ("(".to_string(), TokenType::TagName, 1),
            ("colour".to_string(), TokenType::TagName, 1),
            ("=".to_string(), TokenType::ComparisonOp, 1),
            ("red".to_string(), TokenType::TagValue, 1),
            (")".to_string(), TokenType::LogicalOp, 1),
            ("&".to_string(), TokenType::LogicalOp, 0),
            ("(".to_string(), TokenType::TagName, 1),
            ("size".to_string(), TokenType::TagName, 1),
            ("!".to_string(), TokenType::ComparisonOp, 1),
            ("small".to_string(), TokenType::TagValue, 1),
            (")".to_string(), TokenType::LogicalOp, 1),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rule_syntax_starts_with_comparison_op() {
        let tokens = vec![
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("must start with a tag name"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_starts_with_tag_value() {
        let tokens = vec![
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("must start with a tag name"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_two_tag_names_in_a_row() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("size".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Invalid token sequence"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_two_comparison_ops_in_a_row() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("!".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Invalid token sequence"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_two_tag_values_in_a_row() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("blue".to_string(), TokenType::TagValue, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Invalid token sequence"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_two_logical_ops_in_a_row() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
            ("|".to_string(), TokenType::LogicalOp, 0),
            ("size".to_string(), TokenType::TagName, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Invalid token sequence"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_ends_with_tag_name() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
            ("size".to_string(), TokenType::TagName, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("must end with a tag value"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_ends_with_comparison_op() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("must end with a tag value"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_ends_with_logical_op() {
        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("must end with a tag value"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_empty_rule() {
        let tokens = vec![];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Empty rule"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    #[test]
    fn test_check_rule_syntax_only_parentheses() {
        let tokens = vec![
            ("(".to_string(), TokenType::TagName, 1),
            (")".to_string(), TokenType::LogicalOp, 1),
        ];
        let result = RuleParser::check_rule_syntax(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Empty rule"));
        } else {
            panic!("Expected RuleParseError");
        }
    }

    // Helper function to create test tags
    fn create_test_tags() -> HashMap<String, Vec<String>> {
        let mut tags = HashMap::new();
        tags.insert(
            "colour".to_string(),
            vec!["red".to_string(), "blue".to_string(), "green".to_string()],
        );
        tags.insert(
            "size".to_string(),
            vec![
                "small".to_string(),
                "medium".to_string(),
                "large".to_string(),
            ],
        );
        tags.insert(
            "shape".to_string(),
            vec!["circle".to_string(), "square".to_string()],
        );
        tags
    }

    // Tests for check_valid_tags
    #[test]
    fn test_check_valid_tags_all_valid() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
            ("size".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("large".to_string(), TokenType::TagValue, 0),
        ];

        let result = parser.check_valid_tags(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_valid_tags_invalid_tag_name() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let tokens = vec![
            ("invalid_tag".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
        ];

        let result = parser.check_valid_tags(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("invalid TagName"));
            assert!(msg.contains("invalid_tag"));
        } else {
            panic!("Expected RuleParseError about invalid tag name");
        }
    }

    #[test]
    fn test_check_valid_tags_invalid_tag_value() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("purple".to_string(), TokenType::TagValue, 0),
        ];

        let result = parser.check_valid_tags(&tokens);
        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("invalid TagValue"));
            assert!(msg.contains("purple"));
        } else {
            panic!("Expected RuleParseError about invalid tag value");
        }
    }

    #[test]
    fn test_check_valid_tags_with_parentheses() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let tokens = vec![
            ("(".to_string(), TokenType::TagName, 1),
            ("colour".to_string(), TokenType::TagName, 1),
            ("=".to_string(), TokenType::ComparisonOp, 1),
            ("blue".to_string(), TokenType::TagValue, 1),
            (")".to_string(), TokenType::LogicalOp, 1),
        ];

        let result = parser.check_valid_tags(&tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_valid_tags_multiple_conditions() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let tokens = vec![
            ("colour".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("red".to_string(), TokenType::TagValue, 0),
            ("&".to_string(), TokenType::LogicalOp, 0),
            ("size".to_string(), TokenType::TagName, 0),
            ("!".to_string(), TokenType::ComparisonOp, 0),
            ("small".to_string(), TokenType::TagValue, 0),
            ("|".to_string(), TokenType::LogicalOp, 0),
            ("shape".to_string(), TokenType::TagName, 0),
            ("=".to_string(), TokenType::ComparisonOp, 0),
            ("circle".to_string(), TokenType::TagValue, 0),
        ];

        let result = parser.check_valid_tags(&tokens);
        assert!(result.is_ok());
    }

    // Tests for validate_rule
    // Note: validate_rule expects rules to start with '-' (as they appear in config files)
    // and normalises them by removing the first character
    #[test]
    fn test_validate_rule_valid() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let valid_rules = vec![
            "-colour = red",
            "-colour = red & size = large",
            "-colour = red | colour = blue",
            "-(colour = red)",
            "-(colour = red) & (size = large)",
            "-((colour = red) | (colour = blue)) & (size = large)",
            "-colour ! red",
            "-colour = red & size ! small",
        ];

        for rule in valid_rules {
            let result = parser.validate_rule(rule);
            assert!(result.is_ok(), "Expected rule to be valid: {}", rule);
        }
    }

    #[test]
    fn test_validate_rule_invalid() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let invalid_rules = vec![
            // Missing dash at start
            "colour = red",
            "colour = red & size = large",
            "(colour = red)",
            // Multiple dashes
            "-colour = -red",
            "-colour - red",
            "--colour = red",
            "-colour = red & -size = large",
            // Missing operands (remember first char is skipped)
            "-colour =",
            "-= red",
            "-colour",
            // Double operators
            "-colour = = red",
            "-colour red",
            // Ending with operator
            "-colour = red &",
            "-colour = red |",
            // Starting with operator (after '-' is removed)
            "-& colour = red",
            "-| colour = red",
            // Mismatched parentheses
            "-(colour = red",
            "-colour = red)",
            "-((colour = red)",
            // Empty or only operators (first char removed)
            "--",
            "-=",
            "-&",
            "-()",
            // Invalid sequences
            "-colour & size = large",
            "-colour = red blue",
        ];

        for rule in invalid_rules {
            let result = parser.validate_rule(rule);
            assert!(result.is_err(), "Expected rule to be invalid: {}", rule);
        }
    }

    #[test]
    fn test_validate_rule_invalid_tag_names() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let invalid_rules = vec!["-invalid_tag = red", "-colour = red & unknown = value"];

        for rule in invalid_rules {
            let result = parser.validate_rule(rule);
            assert!(
                result.is_err(),
                "Expected rule to be invalid due to unknown tag: {}",
                rule
            );
            if let Err(RulesError::RuleParseError(msg)) = result {
                assert!(msg.contains("invalid TagName") || msg.contains("invalid TagValue"));
            }
        }
    }

    #[test]
    fn test_validate_rule_invalid_tag_values() {
        let parser = RuleParser {
            m_mapped_tags: create_test_tags(),
        };

        let invalid_rules = vec!["-colour = purple", "-colour = red & size = huge"];

        for rule in invalid_rules {
            let result = parser.validate_rule(rule);
            assert!(
                result.is_err(),
                "Expected rule to be invalid due to unknown value: {}",
                rule
            );
            if let Err(RulesError::RuleParseError(msg)) = result {
                assert!(msg.contains("invalid TagValue"));
            }
        }
    }
}
