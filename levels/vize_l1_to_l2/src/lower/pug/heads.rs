//! The head lines of pug's control flow, split as `pug-lexer@5.0.1`
//! splits them: `conditional`, `eachOf` (tried first) and `each`.

use alloc::vec::Vec as StdVec;

/// A conditional head line.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Branch<'s> {
    If { test: &'s str, negate: bool },
    ElseIf(&'s str),
    Else,
}

/// `pug-lexer`'s `conditional`: `^(if|unless|else if|else)\b([^\n]*)`.
pub(super) fn branch(head: &str) -> Result<Branch<'_>, &'static str> {
    let word = |keyword: &str| {
        head.strip_prefix(keyword)
            .filter(|rest| !rest.starts_with(|ch: char| ch.is_ascii_alphanumeric() || ch == '_'))
    };
    fn expression(rest: &str) -> &str {
        rest.trim_matches(super::literal::is_js_space)
    }
    if let Some(rest) = word("if") {
        Ok(Branch::If {
            test: expression(rest),
            negate: false,
        })
    } else if let Some(rest) = word("unless") {
        Ok(Branch::If {
            test: expression(rest),
            negate: true,
        })
    } else if let Some(rest) = word("else if") {
        Ok(Branch::ElseIf(expression(rest)))
    } else if word("else").is_some_and(|rest| expression(rest).is_empty()) {
        Ok(Branch::Else)
    } else {
        Err("`else` cannot have a condition, perhaps you meant `else if`")
    }
}

/// An `each` head: `eachOf` is tried first, as pug's lexer does.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Each<'s> {
    In {
        value: &'s str,
        key: Option<&'s str>,
        subject: &'s str,
    },
    Of {
        names: StdVec<&'s str>,
        pair: bool,
        subject: &'s str,
    },
}

const MALFORMED_EACH: &str = "This `each` has a syntax error. `each` statements should be of the \
                              form: `each VARIABLE_NAME of JS_EXPRESSION`";
const MALFORMED_EACH_OF: &str = "The value variable for each must either be a valid identifier \
                                 (e.g. `item`) or a pair of identifiers in square brackets (e.g. \
                                 `[key, value]`).";

pub(super) fn each(head: &str) -> Result<Each<'_>, &'static str> {
    let rest = head
        .strip_prefix("each")
        .or_else(|| head.strip_prefix("for"))
        .unwrap_or(head);
    // `^(?:each|for) (.*?) of *([^\n]+)`
    if let Some(body) = rest.strip_prefix(' ') {
        let mut from = 0;
        while let Some(at) = body.get(from..).and_then(|tail| tail.find(" of")) {
            let at = from + at;
            let after = body.get(at + 3..).unwrap_or_default();
            if !after.is_empty() {
                let subject = after.trim_start_matches(' ');
                let subject = if subject.is_empty() { after } else { subject };
                let value = body.get(..at).unwrap_or_default().trim();
                return each_of(value, subject);
            }
            from = at + 1;
        }
    }
    // `^(?:each|for) +(ID)(?: *, *(ID))? * in *([^\n]+)`
    let malformed = || MALFORMED_EACH;
    let spaced = rest.trim_start_matches(' ');
    if spaced.len() == rest.len() {
        return Err(malformed());
    }
    let (value, rest) = identifier(spaced).ok_or_else(malformed)?;
    let (key, rest) = match rest.trim_start_matches(' ').strip_prefix(',') {
        Some(after) => {
            let (key, rest) = identifier(after.trim_start_matches(' ')).ok_or_else(malformed)?;
            (Some(key), rest)
        }
        None => (None, rest),
    };
    let spaced = rest.trim_start_matches(' ');
    if spaced.len() == rest.len() {
        return Err(malformed());
    }
    let after = spaced.strip_prefix("in").ok_or_else(malformed)?;
    if after.is_empty() {
        return Err(malformed());
    }
    let subject = after.trim_start_matches(' ');
    Ok(Each::In {
        value,
        key,
        subject: if subject.is_empty() { after } else { subject },
    })
}

fn each_of<'s>(value: &'s str, subject: &'s str) -> Result<Each<'s>, &'static str> {
    if let Some((name, "")) = identifier(value) {
        return Ok(Each::Of {
            names: alloc::vec![name],
            pair: false,
            subject,
        });
    }
    let pair = value
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .and_then(|inner| {
            let (first, rest) = identifier(inner.trim_start_matches(' '))?;
            let rest = rest.trim_start_matches(' ').strip_prefix(',')?;
            let (second, rest) = identifier(rest.trim_start_matches(' '))?;
            rest.trim_start_matches(' ')
                .is_empty()
                .then(|| alloc::vec![first, second])
        });
    match pair {
        Some(names) => Ok(Each::Of {
            names,
            pair: true,
            subject,
        }),
        None => Err(MALFORMED_EACH_OF),
    }
}

/// `[a-zA-Z_$][\w$]*` at the start of `text`, and the rest.
fn identifier(text: &str) -> Option<(&str, &str)> {
    let start = |ch: char| ch.is_ascii_alphabetic() || ch == '_' || ch == '$';
    if !text.starts_with(start) {
        return None;
    }
    let end = text
        .find(|ch: char| !(start(ch) || ch.is_ascii_digit()))
        .unwrap_or(text.len());
    Some((text.get(..end)?, text.get(end..)?))
}

#[cfg(test)]
mod tests {
    use super::{Branch, Each, branch, each};
    use alloc::vec;

    #[test]
    fn heads_split_like_pugs_lexer() {
        assert_eq!(
            branch("unless a && b "),
            Ok(Branch::If {
                test: "a && b",
                negate: true
            })
        );
        assert_eq!(branch("else if x"), Ok(Branch::ElseIf("x")));
        assert!(branch("else  if x").is_err());
        assert_eq!(
            each("each item, i in [1, 2]"),
            Ok(Each::In {
                value: "item",
                key: Some("i"),
                subject: "[1, 2]"
            })
        );
        assert_eq!(
            each("for [k, v] of pairs"),
            Ok(Each::Of {
                names: vec!["k", "v"],
                pair: true,
                subject: "pairs"
            })
        );
        // pug's `eachOf` pattern wins on any ` of`, even inside a word.
        assert!(each("each item in offers").is_err());
        assert!(each("each 1 in x").is_err());
    }
}
