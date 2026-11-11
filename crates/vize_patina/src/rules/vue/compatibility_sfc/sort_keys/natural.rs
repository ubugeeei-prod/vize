//! Port of `natural-compare` 1.4.0, the comparator eslint-plugin-vue sorts with.

use std::cmp::Ordering;

/// Character class of the code unit at `position`, or `0` past the end.
fn code(text: &[u16], position: usize) -> f64 {
    let code = text.get(position).copied().map_or(0, i32::from);
    f64::from(match code {
        code if !(45..=127).contains(&code) => code,
        45 => 65,
        46..=47 => code - 1,
        48..=57 => code + 18,
        58..=64 => code - 11,
        65..=90 => code + 11,
        91..=96 => code - 37,
        97..=122 => code + 5,
        _ => code - 63,
    })
}

fn is_digit(code: f64) -> bool {
    code > 65.0 && code < 76.0
}

/// The decimal run that starts one unit before `start`, and where it ends.
fn number(text: &[u16], start: usize) -> (f64, usize) {
    let mut end = start;
    while is_digit(code(text, end)) {
        end += 1;
    }
    let digits: vize_carton::String = text
        .get(start.saturating_sub(1)..end)
        .unwrap_or_default()
        .iter()
        .map(|unit| char::from_u32(u32::from(*unit)).unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect();
    (digits.parse::<f64>().unwrap_or(f64::NAN), end)
}

/// `naturalCompare(left, right)`.
pub(super) fn compare(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    let left: Vec<u16> = left.encode_utf16().collect();
    let right: Vec<u16> = right.encode_utf16().collect();
    let (mut left_at, mut right_at) = (0, 0);
    loop {
        let mut left_code = code(&left, left_at);
        let mut right_code = code(&right, right_at);
        left_at += 1;
        right_at += 1;
        // Both sides start a number at a nonzero digit: compare the values.
        if left_code < 76.0 && right_code < 76.0 && left_code > 66.0 && right_code > 66.0 {
            (left_code, left_at) = number(&left, left_at);
            (right_code, right_at) = number(&right, right_at);
        }
        if left_code != right_code {
            return if left_code < right_code {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        if right_code == 0.0 {
            return Ordering::Equal;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compare;
    use std::cmp::Ordering;

    #[test]
    fn orders_numbers_by_value() {
        assert_eq!(compare("b2", "b10"), Ordering::Less);
        assert_eq!(compare("b10", "b2"), Ordering::Greater);
        assert_eq!(compare("a", "B"), Ordering::Greater);
        assert_eq!(compare("a", "a1"), Ordering::Less);
        assert_eq!(compare("x01", "x1"), Ordering::Less);
    }
}
