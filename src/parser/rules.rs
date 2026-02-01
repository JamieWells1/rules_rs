// Parser for .rules files
use crate::types;
use crate::utils::file;
use crate::utils::string;
use crate::{
    err::RulesError, types::MappedRuleTokens, types::Rule, types::SubRule, types::TokenType,
};

use std::collections::HashMap;

// All valid operator characters in rule syntax
const ALL_OP_CHARS: &[char] = &['(', ')', '=', '!', '&', '|', ','];
// Operators that expect a TagValue on the right-hand side
const RHS_CHARS: &[char] = &['=', '!', ',', ')'];
// Operators that expect a TagName on the left-hand side
const LHS_CHARS: &[char] = &['&', '|', '('];

pub struct Parser {
    m_tags: HashMap<types::TagName, types::TagValues>,
}

impl Parser {
    /// Infers the expected type of the next token based on parsing context.
    /// Uses the last token (and sometimes second-to-last) to determine what should come next.
    /// Example: after '(' we expect TagName, after '=' we expect TagValue
    fn get_expected_token_type(token_vec: &Vec<String>) -> Result<TokenType, RulesError> {
        let last_token = token_vec
            .last()
            .ok_or_else(|| RulesError::RuleParseError("Empty token vector".to_string()))?;

        // Check if last token is a single-character operator
        let c = if last_token.len() == 1 {
            last_token.chars().next()
        } else {
            None
        };

        // Last token is an operator
        if let Some(ch) = c {
            if ALL_OP_CHARS.contains(&ch) {
                if ch == '(' {
                    Ok(TokenType::TagName)
                } else if ch == ')' {
                    Ok(TokenType::LogicalOp)
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
            // Last token is a word (TagName/TagValue), look at the operator before it
            let second_to_last_token = &token_vec[token_vec.len() - 2];
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

    /// Tokenizes a rule string into a map of tokens with their types.
    /// Splits on operator characters while preserving them as separate tokens.
    /// Example: "colour = red" -> {"colour": TagName, "=": ComparisonOp, "red": TagValue}
    fn map_rule_tokens(rule: &str) -> Result<MappedRuleTokens, RulesError> {
        let mut token_map: HashMap<String, TokenType> = HashMap::new();
        let mut token_vec: Vec<String> = Vec::new();
        let mut current_word = String::new();

        let expected_token_type = Self::get_expected_token_type(&token_vec)
            .map_err(|rule| RulesError::RuleParseError(format!("Error parsing rule: {}", rule)))?;

        for c in rule.trim().chars() {
            if ALL_OP_CHARS.contains(&c) {
                // Flush accumulated word before adding operator
                if !current_word.is_empty() {
                    token_vec.push(current_word.trim().to_string());
                    token_map.insert(current_word.trim().to_string(), expected_token_type);
                    current_word.clear();
                }

                token_map.insert(c.to_string(), expected_token_type);
            } else if c == ' ' {
                // Space acts as word boundary
                if !current_word.is_empty() {
                    token_map.insert(current_word.trim().to_string(), expected_token_type);
                    current_word.clear();
                }
            } else {
                // Accumulate characters into current word
                current_word.push(c);
            }
        }

        // Flush final word if present
        if !current_word.is_empty() {
            token_map.insert(current_word.trim().to_string(), expected_token_type);
        }

        Ok(token_map)
    }

    fn validate_rule(line: &str) -> Result<(), RulesError> {
        if file::line_blank_or_comment(line) {
            return Ok(());
        }

        let line = string::normalise(line);
        let tokens: MappedRuleTokens = Self::map_rule_tokens(&line)?;

        // NEXT TASK: Ensure that parts conform to correct syntactical structure
        // !!! getting expected token type based on previous tokens is invalid for more complex rules e.g. ones that have nested parenthesis

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
        let Parser { m_tags: tags };

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
