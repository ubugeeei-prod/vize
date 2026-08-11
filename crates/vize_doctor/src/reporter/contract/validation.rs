//! Fail-closed validation for reporter descriptors.

use std::{error::Error, fmt};

use super::{DOCTOR_REPORTER_CONTRACT_VERSION, ReporterDescriptor};

impl ReporterDescriptor {
    /// Validates all descriptor invariants without consulting global state.
    pub fn validate(&self) -> Result<(), ReporterContractError> {
        if self.contract_version != DOCTOR_REPORTER_CONTRACT_VERSION {
            return Err(ReporterContractError::new(
                "contractVersion",
                "unsupported reporter contract version",
            ));
        }
        if !is_stable_id(&self.id) {
            return Err(ReporterContractError::new(
                "id",
                "must start with an ASCII lowercase letter and contain only lowercase letters, digits, dots, and hyphens",
            ));
        }
        if self.display_name.trim().is_empty() || self.display_name.chars().any(char::is_control) {
            return Err(ReporterContractError::new(
                "displayName",
                "must contain visible text without control characters",
            ));
        }
        if self.format_version == 0 {
            return Err(ReporterContractError::new(
                "formatVersion",
                "must be greater than zero",
            ));
        }
        if !is_media_type(&self.media_type) {
            return Err(ReporterContractError::new(
                "mediaType",
                "must be a lowercase type/subtype without parameters",
            ));
        }
        if self
            .file_extension
            .as_deref()
            .is_some_and(|extension| !is_file_extension(extension))
        {
            return Err(ReporterContractError::new(
                "fileExtension",
                "must contain only lowercase ASCII letters and digits without a leading dot",
            ));
        }
        validate_set(
            "audiences",
            &self.audiences,
            "must declare at least one intended consumer",
        )?;
        validate_set(
            "capabilities",
            &self.capabilities,
            "must declare at least one preserved Doctor semantic",
        )?;
        Ok(())
    }
}

fn validate_set<T: Ord>(
    field: &'static str,
    values: &[T],
    empty_reason: &'static str,
) -> Result<(), ReporterContractError> {
    if values.is_empty() {
        return Err(ReporterContractError::new(field, empty_reason));
    }
    if !values.windows(2).all(|window| window[0] < window[1]) {
        return Err(ReporterContractError::new(
            field,
            "must be sorted and contain no duplicates",
        ));
    }
    Ok(())
}

/// A descriptor field that violates the public reporter contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReporterContractError {
    field: &'static str,
    reason: &'static str,
}

impl ReporterContractError {
    const fn new(field: &'static str, reason: &'static str) -> Self {
        Self { field, reason }
    }

    /// Returns the language-neutral serialized field name.
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the stable validation reason.
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for ReporterContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid reporter {}: {}",
            self.field, self.reason
        )
    }
}

impl Error for ReporterContractError {}

fn is_stable_id(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        && !value.ends_with(['.', '-'])
        && !value.contains("..")
        && !value.contains("--")
}

fn is_media_type(value: &str) -> bool {
    let Some((top, subtype)) = value.split_once('/') else {
        return false;
    };
    !top.is_empty()
        && !subtype.is_empty()
        && !subtype.contains('/')
        && top.bytes().all(is_media_token)
        && subtype.bytes().all(is_media_token)
}

fn is_media_token(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || matches!(
            byte,
            b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
        )
}

fn is_file_extension(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}
