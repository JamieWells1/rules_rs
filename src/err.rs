// Errors used across the codebase

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RulesError {
    // Error conversions
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error finding matching files: {0}")]
    GlobPatternError(#[from] glob::PatternError),

    #[error("Error finding file: {0}")]
    GlobError(#[from] glob::GlobError),

    #[error("Error parsing Tag: {0}")]
    TagParseError(String),
}
