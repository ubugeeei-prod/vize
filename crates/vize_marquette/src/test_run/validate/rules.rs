use crate::ContractDiagnostic;
use crate::test_run::model::TestRunRetainedEvidence;
use crate::validate::rules::{contract_path, validate_identifier};

/// Largest integer every consuming language can represent exactly.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Validates the shared identifier grammar and the schema length bound.
pub(crate) fn check_identifier(id: &str, path: &str, diagnostics: &mut Vec<ContractDiagnostic>) {
    validate_identifier(id, path, "VIZE_MARQUETTE_103", diagnostics);
    if id.is_empty() || id.len() > 128 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_103",
            path,
            "identifier must be between 1 and 128 characters",
        ));
    }
}

/// Validates a lowercase 64-character SHA-256 fingerprint.
pub(crate) fn check_digest(value: &str, path: &str, diagnostics: &mut Vec<ContractDiagnostic>) {
    if !is_lower_hex(value, 64, 64) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_104",
            path,
            "fingerprint must be 64 lowercase hexadecimal characters",
        ));
    }
}

/// Validates an exact source revision digest.
pub(crate) fn check_source_revision(
    value: &str,
    path: &str,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    if !is_lower_hex(value, 40, 128) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_105",
            path,
            "source revision must be 40 to 128 lowercase hexadecimal characters",
        ));
    }
}

/// Validates a millisecond-precision UTC timestamp such as
/// `2026-07-21T00:00:00.000Z`.
///
/// The fixed-width format keeps lexicographic and chronological order
/// identical, which the ordering rules rely on.
pub(crate) fn check_timestamp(value: &str, path: &str, diagnostics: &mut Vec<ContractDiagnostic>) {
    if !is_strict_timestamp(value.as_bytes()) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_107",
            path,
            "timestamp must be a millisecond-precision UTC instant like 2026-01-01T00:00:00.000Z",
        ));
    }
}

/// Rejects integers that lose precision in a consuming language.
pub(crate) fn check_safe_integer(
    value: u64,
    path: &str,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    if value > MAX_SAFE_INTEGER {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_111",
            path,
            "value must not exceed the largest exactly-representable integer",
        ));
    }
}

/// Validates an immutable retained-evidence binding.
///
/// The retrieval reference must be content-addressed and name exactly the
/// fingerprinted bytes; anything else would let retained evidence mutate
/// behind a stable-looking record.
pub(super) fn check_retained_evidence(
    retained: &TestRunRetainedEvidence,
    path: &str,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    check_digest(
        &retained.fingerprint,
        &contract_path(path, "fingerprint"),
        diagnostics,
    );
    match retained.reference.strip_prefix("sha256:") {
        Some(suffix) if is_lower_hex(suffix, 64, 64) => {
            if suffix != retained.fingerprint.as_str() {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_109",
                    contract_path(path, "reference"),
                    "content-addressed reference must name the fingerprinted content",
                ));
            }
        }
        _ => diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_108",
            contract_path(path, "reference"),
            "evidence reference must be sha256: followed by 64 lowercase hexadecimal characters",
        )),
    }
}

fn is_lower_hex(value: &str, minimum: usize, maximum: usize) -> bool {
    value.len() >= minimum
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(super) fn is_strict_timestamp(bytes: &[u8]) -> bool {
    if bytes.len() != 24 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let valid = match index {
            4 | 7 => *byte == b'-',
            10 => *byte == b'T',
            13 | 16 => *byte == b':',
            19 => *byte == b'.',
            23 => *byte == b'Z',
            _ => byte.is_ascii_digit(),
        };
        if !valid {
            return false;
        }
    }
    let digits = |start: usize, end: usize| -> u32 {
        bytes[start..end]
            .iter()
            .fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
    };
    let year = digits(0, 4);
    let month = digits(5, 7);
    let day = digits(8, 10);
    if !(1..=12).contains(&month) {
        return false;
    }
    let max_day = match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    };
    (1..=max_day).contains(&day)
        && digits(11, 13) <= 23
        && digits(14, 16) <= 59
        && digits(17, 19) <= 59
}
