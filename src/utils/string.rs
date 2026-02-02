use crate::err::RulesError;

pub trait StringUtils {
    fn at(&self, index: usize) -> Option<char>;
}

impl StringUtils for str {
    fn at(&self, index: usize) -> Option<char> {
        self.chars().nth(index)
    }
}

// Remove first char ('-') and trim
pub fn normalise(string: &str) -> Result<String, RulesError> {
    if string.at(0) != Some('-') {
        return Err(RulesError::RuleParseError(
            "Rule is missing initial dash.".to_string(),
        ));
    }

    let dash_count = string.matches('-').count();
    if dash_count > 1 {
        return Err(RulesError::RuleParseError(format!(
            "Rule should only contain 1 dash, found {}",
            dash_count
        )));
    }

    Ok(string
        .trim()
        .chars()
        .skip(1)
        .collect::<String>()
        .trim()
        .to_string())
}
