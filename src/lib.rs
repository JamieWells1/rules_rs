// Src files
pub mod err;
pub mod orchestrator;
pub mod types;

// Internal impl directories
// src/lib.rs

mod api;
mod parser;
mod utils;

// Re-export the main Rules struct
pub use rules::Rules;

// Re-export error types for users to handle
pub use err::RulesError;

// Keep the lower-level API available for advanced users
pub mod write {
    pub use crate::api::write::tag::write as write_tag;
    pub use crate::api::write::rule::write as write_rule;
    pub use crate::api::write::object::write as write_object;
}

pub use api::entry::evaluate;

// New module for the Rules struct
mod rules;
