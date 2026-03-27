use thiserror::Error;

pub type ValidationResult<T> = Result<T, ValidationError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("{field} must not be empty")]
    Empty { field: &'static str },

    #[error("{field} must be <= {max} characters")]
    TooLong { field: &'static str, max: usize },

    #[error("{field} must match [a-z][a-z0-9_]*")]
    InvalidSlug { field: &'static str },

    #[error("document type 'constitution' is reserved for the system")]
    ReservedConstitutionType,

    #[error("content must not be null")]
    ContentNull,

    #[error("extensions keys must not be empty")]
    EmptyExtensionKey,

    #[error("timestamp fields inconsistent with status")]
    StatusTimestampMismatch,

    #[error("modified_at must be >= created_at")]
    ModifiedBeforeCreated,
}

pub(crate) fn validate_slug(field: &'static str, value: &str) -> ValidationResult<()> {
    if value.is_empty() {
        return Err(ValidationError::Empty { field });
    }

    // Keep these identifiers compact: they become query/filter keys.
    if value.len() > 64 {
        return Err(ValidationError::TooLong { field, max: 64 });
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(ValidationError::Empty { field });
    };
    if !matches!(first, 'a'..='z') {
        return Err(ValidationError::InvalidSlug { field });
    }
    for c in chars {
        if !matches!(c, 'a'..='z' | '0'..='9' | '_') {
            return Err(ValidationError::InvalidSlug { field });
        }
    }

    Ok(())
}
