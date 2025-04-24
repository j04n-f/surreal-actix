use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

static STRONG_PASSWORD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:[^A-Z]*[A-Z])[^a-z]*[a-z][^0-9]*[0-9][^#?!@$%^&*-]*[#?!@$%^&*-].*$").unwrap()
});

static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

pub fn is_email(email: &str) -> Result<(), ValidationError> {
    if email.len() < 3 || email.len() > 255 {
        return Err(ValidationError::new("0")
            .with_message(Cow::from("Email must contain between 3 and 255 characters")));
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::new("0").with_message(Cow::from("Invalid email format")));
    }

    Ok(())
}

pub fn is_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 || password.len() > 72 {
        return Err(ValidationError::new("0").with_message(Cow::from(
            "Password must contain between 8 and 72 characters",
        )));
    }

    if !STRONG_PASSWORD.is_match(password) {
        return Err(ValidationError::new("0")
            .with_message(Cow::from(
                "Password must contain at least one uppercase letter, one lowercase letter, one digit and one special character",
            )));
    }

    Ok(())
}

pub fn is_name(name: &str) -> Result<(), ValidationError> {
    if name.len() < 3 {
        return Err(ValidationError::new("0")
            .with_message(Cow::from("Name must have at least 3 characters")));
    }

    Ok(())
}
