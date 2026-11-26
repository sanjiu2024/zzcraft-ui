//! Validation for forms.
//!
//! GPUI Kit's form module handles layout only: its documentation says the
//! application owns the values, the validation and the submission. Every screen
//! would otherwise repeat the same three rules — required, length, pattern — plus
//! the same aggregation of field errors, so they live here.
//!
//! The design keeps validation out of the widgets: a form collects values into a
//! [`FormState`], runs the rules for the fields that changed, and renders
//! whatever errors the state holds. That makes the rules testable without a
//! window, which matters because "does the form block a bad value" is the part
//! that actually breaks.

use std::collections::BTreeMap;

/// One validation rule.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    /// The value must not be empty after trimming.
    Required,
    /// The trimmed value must be at least this many characters.
    MinLen(usize),
    /// The trimmed value must be at most this many characters.
    MaxLen(usize),
    /// The value must look like an email address.
    Email,
    /// The value must be entirely decimal digits.
    Digits,
    /// The value must parse as a number within the inclusive range.
    Range { min: f64, max: f64 },
}

impl Rule {
    /// Check one value, returning the first failure.
    pub fn check(&self, value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        match self {
            Rule::Required => {
                if trimmed.is_empty() {
                    return Err("此项为必填".to_string());
                }
            }
            Rule::MinLen(len) => {
                if trimmed.chars().count() < *len {
                    return Err(format!("至少 {len} 个字符"));
                }
            }
            Rule::MaxLen(len) => {
                if trimmed.chars().count() > *len {
                    return Err(format!("最多 {len} 个字符"));
                }
            }
            Rule::Email => {
                // Deliberately loose: a form rejects what is obviously not an
                // address and leaves the rest to the server, because a stricter
                // pattern rejects valid addresses.
                let parts: Vec<&str> = trimmed.split('@').collect();
                let looks_like_email =
                    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.');
                if !looks_like_email {
                    return Err("邮箱格式不正确".to_string());
                }
            }
            Rule::Digits => {
                if trimmed.is_empty() || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
                    return Err("只能包含数字".to_string());
                }
            }
            Rule::Range { min, max } => match trimmed.parse::<f64>() {
                Ok(number) if number >= *min && number <= *max => {}
                Ok(_) => return Err(format!("需要在 {min} 到 {max} 之间")),
                Err(_) => return Err("请输入数字".to_string()),
            },
        }
        Ok(())
    }
}

/// Errors keyed by field name.
pub type FieldErrors = BTreeMap<String, String>;

/// State shared by the form and its widgets.
#[derive(Debug, Default, Clone)]
pub struct FormState {
    errors: FieldErrors,
    submitted: bool,
}

impl FormState {
    /// Whether the form has been submitted at least once.
    ///
    /// Rules run only after the first submit attempt; showing "required" on an
    /// empty form the moment it opens is how a form feels hostile.
    pub fn submitted(&self) -> bool {
        self.submitted
    }

    /// Validate one field against its rules and store or clear its error.
    ///
    /// Returning the error keeps the call sites readable:
    /// `if let Some(err) = form.validate_field("email", &value, &rules) { … }`
    pub fn validate_field(&mut self, field: &str, value: &str, rules: &[Rule]) -> Option<String> {
        let mut first_error = None;
        for rule in rules {
            if let Err(message) = rule.check(value) {
                first_error = Some(message);
                break;
            }
        }
        match &first_error {
            Some(message) => {
                self.errors.insert(field.to_string(), message.clone());
            }
            None => {
                self.errors.remove(field);
            }
        }
        first_error
    }

    /// Validate every field at once. Used on submit.
    ///
    /// Values that are not listed keep their existing state, so a form can
    /// validate the visible fields on submit without knowing about fields in
    /// other tabs.
    pub fn validate_all(&mut self, values: &BTreeMap<String, (String, Vec<Rule>)>) -> bool {
        self.submitted = true;
        let mut valid = true;
        for (field, (value, rules)) in values {
            if self.validate_field(field, value, rules).is_some() {
                valid = false;
            }
        }
        valid
    }

    /// Current error for a field, if any.
    pub fn error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(String::as_str)
    }

    /// All current errors.
    pub fn errors(&self) -> &FieldErrors {
        &self.errors
    }

    /// Clear one field's error, for example when the user edits it.
    pub fn clear(&mut self, field: &str) {
        self.errors.remove(field);
    }

    /// Clear every error, for example when the dialog closes.
    pub fn reset(&mut self) {
        self.errors.clear();
        self.submitted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_rejects_blank_and_whitespace() {
        let rule = Rule::Required;
        assert!(rule.check("   ").is_err());
        assert!(rule.check("x").is_ok());
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        // "密码" is two characters but six bytes; counting bytes would reject a
        // value the user sees as long enough.
        let rule = Rule::MinLen(2);
        assert!(rule.check("密码").is_ok());
    }

    #[test]
    fn email_accepts_ordinary_addresses_and_rejects_obvious_noise() {
        assert!(Rule::Email.check("a@b.co").is_ok());
        assert!(Rule::Email.check("a@b").is_err());
        assert!(Rule::Email.check("@b.co").is_err());
    }

    #[test]
    fn range_reports_both_bounds_and_non_numbers() {
        let rule = Rule::Range {
            min: 1.0,
            max: 10.0,
        };
        assert!(rule.check("5").is_ok());
        assert_eq!(rule.check("0").unwrap_err(), "需要在 1 到 10 之间");
        assert_eq!(rule.check("abc").unwrap_err(), "请输入数字");
    }

    #[test]
    fn field_error_is_replaced_and_cleared() {
        let mut form = FormState::default();
        assert!(
            form.validate_field("email", "", &[Rule::Required])
                .is_some()
        );
        assert_eq!(form.error("email"), Some("此项为必填"));

        assert!(
            form.validate_field("email", "a@b.co", &[Rule::Required])
                .is_none()
        );
        assert_eq!(form.error("email"), None);
    }

    #[test]
    fn validate_all_reports_failure_and_marks_the_form_submitted() {
        let mut form = FormState::default();
        assert!(!form.submitted());

        let mut values = BTreeMap::new();
        values.insert("name".to_string(), ("".to_string(), vec![Rule::Required]));
        values.insert("count".to_string(), ("3".to_string(), vec![Rule::Digits]));

        assert!(!form.validate_all(&values));
        assert!(form.submitted());
        assert_eq!(form.errors().len(), 1);

        values.insert("name".to_string(), ("ok".to_string(), vec![Rule::Required]));
        assert!(form.validate_all(&values));
        assert!(form.errors().is_empty());
    }
}
