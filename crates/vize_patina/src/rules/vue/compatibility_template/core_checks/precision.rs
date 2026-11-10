//! `no-loss-of-precision`: compare the digits a numeric literal spells with
//! the double it is stored as, the way ESLint's core rule does.

use oxc_ast::ast::NumericLiteral;
use vize_l0::String;

pub(super) fn numeric_literal_may_lose_precision(literal: &NumericLiteral<'_>) -> bool {
    let Some(raw) = literal.raw.as_ref().map(|raw| raw.as_str()) else {
        return false;
    };
    match radix_digits(raw) {
        Some((digits, bits)) => radix_loses_precision(digits, bits),
        None => base_ten_loses_precision(raw, literal.value),
    }
}

/// Digits and bits per digit of a `0x` / `0o` / `0b` or legacy octal literal.
fn radix_digits(raw: &str) -> Option<(&str, u32)> {
    let prefixed = |prefix: [&str; 2], bits| {
        prefix
            .iter()
            .find_map(|prefix| raw.strip_prefix(prefix))
            .map(|digits| (digits, bits))
    };
    prefixed(["0x", "0X"], 4)
        .or_else(|| prefixed(["0o", "0O"], 3))
        .or_else(|| prefixed(["0b", "0B"], 1))
        .or_else(|| {
            raw.strip_prefix('0')
                .filter(|digits| {
                    !digits.is_empty() && digits.bytes().all(|byte| matches!(byte, b'0'..=b'7'))
                })
                .map(|digits| (digits, 3))
        })
}

/// A power-of-two literal survives when its set bits fit one double mantissa.
fn radix_loses_precision(digits: &str, bits: u32) -> bool {
    let mut highest = None;
    let mut lowest = None;
    let mut position = 0_u64;
    for digit in digits.chars().rev().filter(|character| *character != '_') {
        let Some(value) = digit.to_digit(16) else {
            return false;
        };
        if value != 0 {
            lowest.get_or_insert(position + u64::from(value.trailing_zeros()));
            highest = Some(position + u64::from(value.bit_width()) - 1);
        }
        position += u64::from(bits);
    }
    match (highest, lowest) {
        (Some(highest), Some(lowest)) => highest >= 1024 || highest - lowest >= 53,
        _ => false,
    }
}

fn base_ten_loses_precision(raw: &str, value: f64) -> bool {
    let mut raw = raw.replace('_', "").to_ascii_lowercase();
    if let Some(dot) = raw.find(".e") {
        raw.remove(dot);
    } else if raw.ends_with('.') {
        raw.pop();
    }
    let Some(written) = Scientific::parse(&raw, false) else {
        return false;
    };
    if value == 0.0 {
        return !written.coefficient.bytes().all(|byte| byte == b'0');
    }
    let precision = written.coefficient.len();
    if precision > 100 || !value.is_finite() {
        return true;
    }
    let stored = vize_l0::cstr!("{:.*e}", precision.saturating_sub(1), value);
    Scientific::parse(&stored, true).is_none_or(|stored| stored != written)
}

#[derive(PartialEq, Eq)]
struct Scientific {
    coefficient: String,
    magnitude: i64,
}

impl Scientific {
    fn parse(number: &str, as_float: bool) -> Option<Self> {
        let (coefficient, exponent) = match number.split_once('e') {
            Some((coefficient, exponent)) => (coefficient, Some(exponent)),
            None => (number, None),
        };
        let mut normalized = if as_float || number.contains('.') {
            normalize_float(coefficient)
        } else {
            normalize_integer(coefficient)
        };
        if let Some(exponent) = exponent {
            let exponent = exponent.strip_prefix('+').unwrap_or(exponent);
            normalized.magnitude = normalized
                .magnitude
                .checked_add(exponent.parse::<i64>().ok()?)?;
        }
        Some(normalized)
    }
}

fn normalize_integer(integer: &str) -> Scientific {
    let trimmed = trim_leading_zeros(integer);
    Scientific {
        coefficient: String::from(trim_trailing_zeros(trimmed)),
        magnitude: digit_count(trimmed) - 1,
    }
}

fn normalize_float(float: &str) -> Scientific {
    let trimmed = trim_leading_zeros(float);
    match trimmed.find('.') {
        Some(0) => {
            let digits = trim_leading_zeros(trimmed.get(1..).unwrap_or_default());
            Scientific {
                coefficient: String::from(digits),
                magnitude: digit_count(digits) - digit_count(trimmed),
            }
        }
        Some(dot) => Scientific {
            coefficient: String::from(trimmed.replacen('.', "", 1)),
            magnitude: i64::try_from(dot).unwrap_or(i64::MAX) - 1,
        },
        None => Scientific {
            coefficient: String::from(trimmed),
            magnitude: digit_count(trimmed) - 1,
        },
    }
}

/// Strip leading zeros, keeping an all-zero string as written.
fn trim_leading_zeros(number: &str) -> &str {
    match number.trim_start_matches('0') {
        "" => number,
        trimmed => trimmed,
    }
}

/// Strip trailing zeros, keeping an all-zero string as written.
fn trim_trailing_zeros(number: &str) -> &str {
    match number.trim_end_matches('0') {
        "" => number,
        trimmed => trimmed,
    }
}

fn digit_count(digits: &str) -> i64 {
    i64::try_from(digits.len()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{base_ten_loses_precision, radix_digits, radix_loses_precision};

    fn loses(raw: &str) -> bool {
        match radix_digits(raw) {
            Some((digits, bits)) => radix_loses_precision(digits, bits),
            None => {
                let value = raw.replace('_', "").parse::<f64>().unwrap_or(f64::NAN);
                base_ten_loses_precision(raw, value)
            }
        }
    }

    #[test]
    fn matches_the_core_rule_on_written_digits() {
        for raw in [
            "9007199254740993",
            "0x20000000000001",
            "0.12345678901234567890",
            "900719.9254740994",
            "9.007199254740993e15",
            "0b100000000000000000000000000000000000000000000000000001",
            "0o400000000000000001",
            "1e-350",
            "1e400",
        ] {
            assert!(loses(raw), "{raw}");
        }
        for raw in [
            "123",
            "12300000000000000000000000",
            "1_230000000_00000000_00000_000",
            "9007199254740992",
            "0x1fffffffffffff",
            "0x10000000000000000",
            "0.1",
            "1.50",
            "1.",
            "1.e5",
            ".5",
            "0",
            "0.0",
            "00",
            "0777",
            "089",
            "5e-324",
        ] {
            assert!(!loses(raw), "{raw}");
        }
    }
}
