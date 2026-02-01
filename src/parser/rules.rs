// Parser for .rules files
use crate::types::ComparisonOp as op;
use crate::utils::file;
use crate::utils::string;
use crate::{err::RulesError, types::Rule, types::SubRule};

const OPERATION_CHARS: &[char] = &['(', ')', '=', '!', '&', '|', ','];

// Split rule into operators, tag names and tag values
// E.g. ["colour", "=", "red"]
fn split_rule(rule: &str) -> Vec<String> {
    let mut split_rule: Vec<String> = Vec::new();
    let mut current_word = String::new();

    for c in rule.chars() {
        if OPERATION_CHARS.contains(&c) {
            if !current_word.is_empty() {
                split_rule.push(current_word.trim().to_string());
                current_word.clear();
            }
            split_rule.push(c.to_string());
        } else if c == ' ' {
            if !current_word.is_empty() {
                split_rule.push(current_word.trim().to_string());
                current_word.clear();
            }
        } else {
            current_word.push(c);
        }
    }

    // Push last word
    if !current_word.is_empty() {
        split_rule.push(current_word.trim().to_string());
    }

    split_rule
}

fn validate_rule(line: &str) -> Result<(), RulesError> {
    if file::line_blank_or_comment(line) {
        return Ok(());
    }

    let mut line = string::normalise(line);

    Ok(())
}

fn string_to_rule(rule_str: &str) -> Result<Rule, RulesError> {
    // TODO
}

fn rule_to_dnf_subrule(rule: Rule) -> Result<SubRule, RulesError> {
    // TODO
}

pub fn parse_rules() -> Result<Vec<SubRule>, RulesError> {
    let mut dnf_subrules: Vec<SubRule> = Vec::new();
    let all_files: Vec<String> = file::read_files_in_dir("config/*.rules")?;

    for file in all_files.iter() {
        for line in file.lines() {
            if file::line_blank_or_comment(line) {
                continue;
            }

            let rule = string_to_rule(line)?;
            let subrule = rule_to_dnf_subrule(rule)?;

            // TODO
            dnf_subrules.push(subrule);
        }
    }

    // TODO
    Ok(Vec::new())
}
