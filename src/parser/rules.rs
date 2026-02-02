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
        let last_token = parsed_tokens
            .last()
            .ok_or_else(|| RulesError::RuleParseError("Empty token vector".to_string()))?;

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
        let mut token_map: HashMap<String, TokenType> = HashMap::new();
        let mut parsed_tokens: Vec<String> = Vec::new();
        let mut current_word = String::new();
        let mut parenthesis_depth = 0;

        for c in rule.trim().chars() {
            if ALL_OP_CHARS.contains(&c) {
                if !current_word.is_empty() {
                    let expected_token_type =
                        Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
                    parsed_tokens.push(current_word.trim().to_string());
                    token_map.insert(current_word.trim().to_string(), expected_token_type);
                    current_word.clear();
                }

                let expected_token_type =
                    Self::get_expected_token_type(&parsed_tokens, parenthesis_depth)?;
                token_map.insert(c.to_string(), expected_token_type);
                parsed_tokens.push(c.to_string());

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
                // ... rest of the logic
            }
        }

        if parenthesis_depth != 0 {
            return Err(RulesError::RuleParseError(
                "Unmatched opening parenthesis".to_string(),
            ));
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
