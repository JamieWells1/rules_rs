use std::collections::HashSet;

// Parser for .tags files
use crate::err::RulesError;
use crate::types::Tag;
use crate::utils::file;
use crate::utils::string::StringIndexing;

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return true;
    }
    return false;
}

pub fn validate_tag(line: &str) -> Result<(), RulesError> {
    if is_blank_or_comment(line) {
        return Ok(());
    }

    let parts: Vec<&str> = line.split(":").collect();
    let mut errors: HashSet<&str> = HashSet::new();

    // Check parts length BEFORE accessing
    if parts.len() < 2 {
        return Err(RulesError::TagParseError(
            "Tag must contain a ':' separator".to_string(),
        ));
    }

    if parts.len() > 2 {
        errors.insert("Tag must only contain one name and one set of values. Ensure there is only one semi-colon");
    }

    // NOW safe to access parts[0] and parts[1]
    let name: &str = parts[0];
    let values: &str = parts[1];

    if let Some(first_char) = name.trim().at(0) {
        if first_char != '-' {
            errors.insert("Tag must begin with '-'");
        }

        let name_no_dash: String = name.trim().chars().skip(1).collect();
        if name_no_dash.split_whitespace().count() > 1 {
            errors.insert("Tag name cannot contain spaces");
        }
    }

    for value in values.split(",") {
        // Contains space and it isn't trailing or leading
        if value.trim().contains(" ") {
            errors.insert("Tag values cannot contain spaces");
        }
    }

    if !errors.is_empty() {
        // Copy references to strings from HashSet into Vec to join as one string
        let error_list: Vec<&str> = errors.iter().copied().collect();

        return Err(RulesError::TagParseError(format!(
            "Errors parsing line: '{}': {}",
            line,
            error_list.join(", ")
        )));
    }

    Ok(())
}

fn get_name_from_tag(parts: &Vec<&str>) -> String {
    // Remove first char ('-') and trim
    parts[0]
        .trim()
        .chars()
        .skip(1)
        .collect::<String>()
        .trim()
        .to_string()
}

fn get_values_from_tag(parts: &Vec<&str>) -> Vec<String> {
    parts[1].split(',').map(|v| v.trim().to_string()).collect()
}

pub fn get_name_and_values_from_tag(line: &str) -> Result<(String, Vec<String>), RulesError> {
    validate_tag(line)?;
    let parts: Vec<&str> = line.trim().split(':').collect();

    let name: String = get_name_from_tag(&parts);
    let values: Vec<String> = get_values_from_tag(&parts);
    Ok((name, values))
}

pub fn parse_tags() -> Result<Vec<Tag>, RulesError> {
    let mut tags: Vec<Tag> = vec![];
    let all_files: Vec<String> = file::read_files_in_dir("config/*.tags")?;

    for file in all_files.iter() {
        for line in file.lines() {
            if is_blank_or_comment(line) {
                continue;
            }

            let raw_tag = get_name_and_values_from_tag(line)?;
            let tag = Tag {
                name: raw_tag.0,
                values: raw_tag.1,
            };
            tags.push(tag);
        }
    }

    Ok(tags)
}
