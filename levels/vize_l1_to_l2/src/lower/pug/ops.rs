//! JavaScript's operators and property reads over [`Value`]s, refusing
//! what would depend on object identity or inherited members.

use oxc_ast::ast::BinaryOperator;
use vize_l0::String;

use super::value::Value;

/// The arithmetic, equality and relational operators, except where
/// the result would depend on object identity.
pub(super) fn binary_value(operator: BinaryOperator, left: &Value, right: &Value) -> Option<Value> {
    use BinaryOperator as Op;
    let (x, y) = (left.to_number(), right.to_number());
    Some(match operator {
        Op::Addition => match (left.to_primitive(), right.to_primitive()) {
            (left @ Value::Str(_), right) | (left, right @ Value::Str(_)) => {
                let mut out = left.to_js_string();
                out.push_str(&right.to_js_string());
                Value::Str(out)
            }
            _ => Value::Number(x + y),
        },
        Op::Subtraction => Value::Number(x - y),
        Op::Multiplication => Value::Number(x * y),
        Op::Division => Value::Number(x / y),
        Op::Remainder => Value::Number(x % y),
        Op::Exponential => Value::Number(power(x, y)),
        Op::StrictEquality => Value::Bool(left.strict_equals(right)?),
        Op::StrictInequality => Value::Bool(!left.strict_equals(right)?),
        Op::Equality => Value::Bool(loosely_equal(left, right)?),
        Op::Inequality => Value::Bool(!loosely_equal(left, right)?),
        Op::LessThan | Op::GreaterThan | Op::LessEqualThan | Op::GreaterEqualThan => {
            let ordering = match (left.to_primitive(), right.to_primitive()) {
                (Value::Str(left), Value::Str(right)) => {
                    Some(left.encode_utf16().cmp(right.encode_utf16()))
                }
                (left, right) => left.to_number().partial_cmp(&right.to_number()),
            };
            Value::Bool(ordering.is_some_and(|ordering| match operator {
                Op::LessThan => ordering.is_lt(),
                Op::GreaterThan => ordering.is_gt(),
                Op::LessEqualThan => ordering.is_le(),
                _ => ordering.is_ge(),
            }))
        }
        _ => return None,
    })
}

/// `Number::exponentiate`, where it differs from `powf`.
fn power(base: f64, exponent: f64) -> f64 {
    if exponent.is_nan() || (base.abs() == 1.0 && exponent.is_infinite()) {
        f64::NAN
    } else {
        base.powf(exponent)
    }
}

/// `IsLooselyEqual`; `None` when object identity decides.
fn loosely_equal(left: &Value, right: &Value) -> Option<bool> {
    let object = |value: &Value| matches!(value, Value::Array(_) | Value::Object(_));
    match (left, right) {
        _ if left.is_nullish() || right.is_nullish() => {
            Some(left.is_nullish() && right.is_nullish())
        }
        _ if object(left) && object(right) => None,
        (Value::Bool(_), _) => loosely_equal(&Value::Number(left.to_number()), right),
        (_, Value::Bool(_)) => loosely_equal(left, &Value::Number(right.to_number())),
        _ if object(left) || object(right) => {
            loosely_equal(&left.to_primitive(), &right.to_primitive())
        }
        (Value::Str(_), Value::Number(_)) | (Value::Number(_), Value::Str(_)) => {
            Some(left.to_number() == right.to_number())
        }
        _ => left.strict_equals(right),
    }
}

/// `object[key]` for own data only: `length` and indices of strings and
/// arrays, own keys of objects (absent keys read `undefined`). Inherited
/// members such as methods are refused.
pub(super) fn property(object: &Value, key: &str) -> Option<Value> {
    match object {
        Value::Str(text) => {
            if key == "length" {
                return Some(Value::Number(text.encode_utf16().count() as f64));
            }
            let index = array_index(key)?;
            match text.encode_utf16().nth(index) {
                Some(unit) => char::from_u32(u32::from(unit))
                    .map(|ch| Value::Str(String::from(ch.encode_utf8(&mut [0; 4]) as &str))),
                None => Some(Value::Undefined),
            }
        }
        Value::Array(items) => {
            if key == "length" {
                return Some(Value::Number(items.len() as f64));
            }
            let index = array_index(key)?;
            Some(items.get(index).cloned().unwrap_or(Value::Undefined))
        }
        Value::Object(entries) => match entries.iter().find(|(bound, _)| bound == key) {
            Some((_, value)) => Some(value.clone()),
            None if OBJECT_PROTOTYPE.contains(&key) => None,
            None => Some(Value::Undefined),
        },
        _ => None,
    }
}

const OBJECT_PROTOTYPE: [&str; 12] = [
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
    "__proto__",
    "constructor",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "valueOf",
];

/// A canonical array index (`0`, `1`, … without leading zeros).
fn array_index(key: &str) -> Option<usize> {
    is_array_index(key).then(|| key.parse().ok()).flatten()
}

pub(super) fn is_array_index(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 10
        && key.bytes().all(|byte| byte.is_ascii_digit())
        && (key == "0" || !key.starts_with('0'))
        && key
            .parse::<u64>()
            .is_ok_and(|index| index < u64::from(u32::MAX))
}
