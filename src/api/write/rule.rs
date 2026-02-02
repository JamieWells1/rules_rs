use crate::err::RulesError;
use crate::parser::rules::RuleParser;
use crate::types::{TagName, TagValues};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Normalizes a filename to ensure it has the .rules extension
fn normalize_filename(file_name: &str) -> String {
    if file_name.ends_with(".rules") {
        file_name.to_string()
    } else {
        format!("{}.rules", file_name)
    }
}

/// Ensures the config directory exists
fn ensure_config_dir(base_dir: &str) -> Result<(), RulesError> {
    let config_dir = Path::new(base_dir);
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }
    Ok(())
}

/// Writes a rule to a .rules file
///
/// # Arguments
/// * `file_name` - Name of the file (with or without .rules extension)
/// * `rule` - The rule string to write (should start with '-')
/// * `tags` - HashMap of valid tags for validation
///
/// # Examples
/// ```ignore
/// write("my_rules", "-colour = red & size = large", tags)?;
/// write("my_rules.rules", "-colour = blue", tags)?;
/// ```
pub fn write(
    file_name: &str,
    rule: &str,
    tags: HashMap<TagName, TagValues>,
) -> Result<(), RulesError> {
    write_with_base_dir(file_name, rule, tags, "config")
}

/// Internal function for writing rules with a custom base directory.
/// Used primarily for testing to avoid touching production config files.
#[cfg(test)]
pub(crate) fn write_with_base_dir(
    file_name: &str,
    rule: &str,
    tags: HashMap<TagName, TagValues>,
    base_dir: &str,
) -> Result<(), RulesError> {
    let base = base_dir;

    write_internal(file_name, rule, tags, base)
}

/// Non-test version that's always available for internal use
#[cfg(not(test))]
pub(crate) fn write_with_base_dir(
    file_name: &str,
    rule: &str,
    tags: HashMap<TagName, TagValues>,
    base_dir: &str,
) -> Result<(), RulesError> {
    write_internal(file_name, rule, tags, base_dir)
}

/// Actual implementation shared by both public and internal functions
fn write_internal(
    file_name: &str,
    rule: &str,
    tags: HashMap<TagName, TagValues>,
    base_dir: &str,
) -> Result<(), RulesError> {
    // Normalize filename
    let normalised_name = normalize_filename(file_name);
    let full_path = format!("{}/{}", base_dir, normalised_name);

    // Ensure config directory exists
    ensure_config_dir(base_dir)?;

    // Validate the rule using RuleParser
    let parser = RuleParser::new(tags);
    parser.validate_rule(rule)?;

    // Read existing file or create new content
    let mut lines: Vec<String> = if Path::new(&full_path).exists() {
        fs::read_to_string(&full_path)?
            .lines()
            .map(|l: &str| l.to_string())
            .collect()
    } else {
        Vec::new()
    };

    // Check if the exact rule already exists
    let rule_trimmed = rule.trim();
    if lines
        .iter()
        .any(|line: &String| line.trim() == rule_trimmed)
    {
        return Err(RulesError::RuleParseError(
            "Rule already exists in file".to_string(),
        ));
    }

    // Add the new rule
    lines.push(rule_trimmed.to_string());

    // Write back to file
    fs::write(&full_path, lines.join("\n"))?;

    Ok(())
}
