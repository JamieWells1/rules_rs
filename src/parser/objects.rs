// Parser for objects in .yaml files in config dir
use crate::{err::RulesError, types::Object};

pub fn validate_object(obj: Object) -> Result<(), RulesError> {
    // TODO: Check its tags are valid
    Ok(())
}
