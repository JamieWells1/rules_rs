// File utils
use crate::err::RulesError;

use std::fs;

use glob::glob;

pub fn read_file(path: &str) -> Result<String, RulesError> {
    let file_content = fs::read_to_string(path)?;
    Ok(file_content)
}

pub fn read_files_in_dir(pattern: &str) -> Result<Vec<String>, RulesError> {
    let mut contents = Vec::new();

    for entry in glob(pattern)? {
        let path = entry?;
        let file_content = fs::read_to_string(path)?;
        contents.push(file_content);
    }

    Ok(contents)
}

pub fn line_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return true;
    }
    return false;
}
