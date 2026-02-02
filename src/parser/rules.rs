// Parser for .rules files
use crate::parser::types::{MappedRuleTokens, Rule, TokenType};
use crate::types::{self, SubRule};
use crate::utils::file;
use crate::utils::string;
use crate::err::RulesError;

use std::collections::HashMap;

// All valid operator characters in rule syntax
const ALL_OP_CHARS: &[char] = &['(', ')', '=', '!', '&', '|', ','];
// Operators that expect a TagValue on the right-hand side
const RHS_CHARS: &[char] = &['=', '!', ',', ')'];
// Operators that expect a TagName on the left-hand side
const LHS_CHARS: &[char] = &['&', '|', '('];

pub struct RuleParser {
    m_tags: HashMap<types::TagName, types::TagValues>,
}

impl RuleParser {
    /// Infers the expected type of the next token based on parsing context.
    /// Uses the last token (and sometimes second-to-last) to determine what should come next.
    /// Example: after '(' we expect TagName, after '=' we expect TagValue
    fn get_expected_token_type(
        parsed_tokens: &Vec<String>,
        parenthesis_depth: i32,
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
                    if parenthesis_depth > 0 {
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

    fn map_rule_tokens(rule: &str) -> Result<MappedRuleTokens, RulesError> {
        let mut mapped_token_list: Vec<(String, TokenType)> = Vec::new();
        let mut parsed_tokens: Vec<String> = Vec::new();
        let mut current_word = String::new();
        let mut parenthesis_depth = 0;

        for c in rule.trim().chars() {
            if ALL_OP_CHARS.contains(&c) {
                if !current_word.is_empty() {
                    let expected_token_type =
                        Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
                    let token = current_word.trim().to_string();
                    parsed_tokens.push(token.clone());
                    mapped_token_list.push((token, expected_token_type));
                    current_word.clear();
                }

                let expected_token_type =
                    Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
                let token = c.to_string();
                mapped_token_list.push((token.clone(), expected_token_type));
                parsed_tokens.push(token);

                // Update parenthesis depth after processing token
                if c == '(' {
                    parenthesis_depth += 1;
                } else if c == ')' {
                    parenthesis_depth -= 1;
                    if parenthesis_depth < 0 {
                        return Err(RulesError::RuleParseError(
                            "Unmatched closing parenthesis".to_string(),
                        ));
                    }
                }
            } else if c == ' ' {
                // Space acts as word boundary
                if !current_word.is_empty() {
                    let expected_token_type =
                        Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
                    let token = current_word.trim().to_string();
                    parsed_tokens.push(token.clone());
                    mapped_token_list.push((token, expected_token_type));
                    current_word.clear();
                }
            } else {
                // Accumulate characters into current word
                current_word.push(c);
            }
        }

        // Flush final word if present
        if !current_word.is_empty() {
            let expected_token_type =
                Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
            let token = current_word.trim().to_string();
            parsed_tokens.push(token.clone());
            mapped_token_list.push((token, expected_token_type));
        }

        if parenthesis_depth != 0 {
            return Err(RulesError::RuleParseError(
                "Unmatched opening parenthesis".to_string(),
            ));
        }

        Ok(mapped_token_list)
    }

    fn validate_rule(line: &str) -> Result<(), RulesError> {
        if file::line_blank_or_comment(line) {
            return Ok(());
        }

        let line = string::normalise(line);
        let tokens: MappedRuleTokens = Self::map_rule_tokens(&line)?;

        // NEXT TASK: Ensure that parts conform to correct syntactical structure

        Ok(())
    }

    fn string_to_rule(rule_str: &str) -> Result<Rule, RulesError> {
        // TODO: Parse string into AST representation
        Self::validate_rule(rule_str)?;
        unimplemented!()
    }

    fn rule_to_dnf_subrule(rule: Rule) -> Result<SubRule, RulesError> {
        // TODO: Convert AST to Disjunctive Normal Form
        unimplemented!()
    }

    /// Main entry point for parsing rule files.
    /// Converts all .rules files into Disjunctive Normal Form (DNF) subrules.
    pub fn parse_rules(
        tags: HashMap<types::TagName, types::TagValues>,
    ) -> Result<Vec<SubRule>, RulesError> {
        let RuleParser { m_tags: tags };

        let mut dnf_subrules: Vec<SubRule> = Vec::new();
        let all_files: Vec<String> = file::read_files_in_dir("config/*.rules")?;

        for file in all_files.iter() {
            for line in file.lines() {
                if file::line_blank_or_comment(line) {
                    continue;
                }

                // Parse string to AST, then convert to DNF representation
                let rule = Self::string_to_rule(line)?;
                let subrule = Self::rule_to_dnf_subrule(rule)?;

                dnf_subrules.push(subrule);
            }
        }

        // TODO
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Find a token in the token list
    fn find_token<'a>(
        tokens: &'a [(String, TokenType)],
        token_str: &str,
    ) -> Option<&'a TokenType> {
        tokens
            .iter()
            .find(|(s, _)| s == token_str)
            .map(|(_, t)| t)
    }

    // Count occurrences of a token
    fn count_token(tokens: &[(String, TokenType)], token_str: &str) -> usize {
        tokens.iter().filter(|(s, _)| s == token_str).count()
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

    // Tests for map_rule_tokens
    #[test]
    fn test_map_rule_tokens_simple_rule() {
        let rule = "colour = red";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
    }

    #[test]
    fn test_map_rule_tokens_with_parentheses() {
        let rule = "(colour = red)";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(find_token(&tokens, "("), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
        assert_eq!(find_token(&tokens, ")"), Some(&TokenType::LogicalOp));
    }

    #[test]
    fn test_map_rule_tokens_nested_parentheses() {
        let rule = "((colour = red))";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        // Should have 2 open parens, 2 close parens, colour, =, red
        assert_eq!(count_token(&tokens, "("), 2);
        assert_eq!(count_token(&tokens, ")"), 2);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
    }

    #[test]
    fn test_map_rule_tokens_with_and_operator() {
        let rule = "colour = red & size = large";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
        assert_eq!(find_token(&tokens, "&"), Some(&TokenType::LogicalOp));
        assert_eq!(find_token(&tokens, "size"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "large"), Some(&TokenType::TagValue));
        // Check that both = operators are present
        assert_eq!(count_token(&tokens, "="), 2);
    }

    #[test]
    fn test_map_rule_tokens_with_or_operator() {
        let rule = "colour = red | colour = blue";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        // "colour" appears twice
        assert_eq!(count_token(&tokens, "colour"), 2);
        assert_eq!(find_token(&tokens, "|"), Some(&TokenType::LogicalOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
        assert_eq!(find_token(&tokens, "blue"), Some(&TokenType::TagValue));
        // Check that both = operators are present
        assert_eq!(count_token(&tokens, "="), 2);
    }

    #[test]
    fn test_map_rule_tokens_with_not_equals() {
        let rule = "colour ! red";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "!"), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
    }

    #[test]
    fn test_map_rule_tokens_complex_nested() {
        let rule = "((colour = red) & (size = large))";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
        assert_eq!(find_token(&tokens, "&"), Some(&TokenType::LogicalOp));
        assert_eq!(find_token(&tokens, "size"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "large"), Some(&TokenType::TagValue));
        // Check that both = operators are present
        assert_eq!(count_token(&tokens, "="), 2);
        // Check parentheses counts
        assert_eq!(count_token(&tokens, "("), 3);
        assert_eq!(count_token(&tokens, ")"), 3);
    }

    #[test]
    fn test_map_rule_tokens_unmatched_opening_paren() {
        let rule = "(colour = red";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Unmatched opening parenthesis"));
        } else {
            panic!("Expected RuleParseError about unmatched opening parenthesis");
        }
    }

    #[test]
    fn test_map_rule_tokens_unmatched_closing_paren() {
        let rule = "colour = red)";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_err());
        if let Err(RulesError::RuleParseError(msg)) = result {
            assert!(msg.contains("Unmatched closing parenthesis"));
        } else {
            panic!("Expected RuleParseError about unmatched closing parenthesis");
        }
    }

    #[test]
    fn test_map_rule_tokens_extra_whitespace() {
        let rule = "  colour   =   red  ";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
    }

    #[test]
    fn test_map_rule_tokens_no_spaces() {
        let rule = "colour=red";
        let result = RuleParser::map_rule_tokens(rule);

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(find_token(&tokens, "colour"), Some(&TokenType::TagName));
        assert_eq!(find_token(&tokens, "="), Some(&TokenType::ComparisonOp));
        assert_eq!(find_token(&tokens, "red"), Some(&TokenType::TagValue));
    }
}
