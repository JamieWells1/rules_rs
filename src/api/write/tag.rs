use crate::err::RulesError;
use crate::parser::tags;
use crate::utils::file;
use crate::utils::string::StringIndexing;

pub fn write(file_name: &str, tag_name: String, tag_values: Vec<String>) -> Result<(), RulesError> {
    let file: String = file::read_file(&format!("config/{}", file_name))?;
    let mut lines: Vec<String> = file.lines().map(|l| l.to_string()).collect();
    let mut tag_exists: bool = false;

    for line in &mut lines {
        // Skip over comments and blank lines
        if line.trim().at(0) == Some('#') || line.len() == 0 {
            continue;
        }

        let (extracted_name, _extracted_values) = tags::get_name_and_values_from_tag(line)?;

        if extracted_name.trim() == tag_name.trim() {
            line.push_str(&format!(", {}", tag_values.join(", ")));
            tag_exists = true;
        }
    }

    if !tag_exists {
        lines.push(format!(
            "\n- {}: {}",
            tag_name.trim(),
            tag_values.join(", ")
        ));
    }

    std::fs::write(&format!("config/{}", file_name), lines.join("\n"))?;
    Ok(())
}
