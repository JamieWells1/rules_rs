use crate::err::RulesError;

use std::collections::HashMap;

pub fn write(
    file_name: &str,
    obj_type: String,
    obj: HashMap<String, Vec<String>>,
) -> Result<(), RulesError> {
    // TODO: if object already exists, add non-existent attributes
    Ok(())
}
