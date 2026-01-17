//! Validation utilities for Rust Teams application

use validator::Validate;
use crate::error::AppError;

/// Validate a request and return an AppError if validation fails
pub fn validate_request<T: Validate>(request: &T) -> Result<(), AppError> {
    request.validate().map_err(AppError::from)
}

/// Common validation patterns
pub mod patterns {
    use regex::Regex;
    use std::sync::LazyLock;

    pub static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{3,30}$").unwrap()
    });

    pub static CHANNEL_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{1,100}$").unwrap()
    });
}

/// Validate username format
pub fn validate_username(username: &str) -> bool {
    patterns::USERNAME_REGEX.is_match(username)
}

/// Validate channel name format
pub fn validate_channel_name(name: &str) -> bool {
    patterns::CHANNEL_NAME_REGEX.is_match(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("john_doe"));
        assert!(validate_username("user123"));
        assert!(validate_username("test-user"));
        assert!(!validate_username("ab")); // too short
        assert!(!validate_username("user@name")); // invalid character
    }

    #[test]
    fn test_validate_channel_name() {
        assert!(validate_channel_name("general"));
        assert!(validate_channel_name("dev-team"));
        assert!(validate_channel_name("project_alpha"));
        assert!(!validate_channel_name("")); // empty
        assert!(!validate_channel_name("channel name")); // space not allowed
    }
}
