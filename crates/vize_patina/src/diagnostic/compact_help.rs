//! Compact remediation text for terminal and CI output.

use vize_s0::{String, ToCompactString};

/// Return one actionable help sentence without markdown or inline examples.
pub(super) fn compact_help_text(text: &str) -> String {
    let mut in_code_block = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || trimmed.is_empty() {
            continue;
        }

        let stripped = trimmed.replace("**", "").replace("__", "").replace('`', "");
        let stripped = stripped.trim_start_matches('#').trim();
        if !stripped.is_empty() {
            return compact_plain_help_line(stripped);
        }
    }
    text.lines().next().unwrap_or(text).to_compact_string()
}

fn compact_plain_help_line(line: &str) -> String {
    let without_examples = remove_example_asides(line);
    let without_reason = truncate_before_reason(&without_examples);
    let sentence = first_sentence(without_reason.trim());
    let mut compact = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
    for (spaced, punctuation) in [
        (" .", "."),
        (" ,", ","),
        (" ;", ";"),
        (" !", "!"),
        (" ?", "?"),
        (" 。", "。"),
        (" ，", "，"),
        (" ！", "！"),
        (" ？", "？"),
    ] {
        compact = compact.replace(spaced, punctuation);
    }
    compact.into()
}

fn remove_example_asides(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut output = String::with_capacity(line.len());
    let mut index = 0;

    while index < chars.len() {
        let Some(close) = matching_parenthesis(chars[index]) else {
            output.push(chars[index]);
            index += 1;
            continue;
        };
        let Some(end) = find_matching_parenthesis(&chars, index, chars[index], close) else {
            output.push(chars[index]);
            index += 1;
            continue;
        };
        let content: String = chars[index + 1..end].iter().collect();
        if is_example_aside(content.trim()) {
            index = end + 1;
            continue;
        }

        output.extend(chars[index..=end].iter());
        index = end + 1;
    }

    output
}

fn matching_parenthesis(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '（' => Some('）'),
        _ => None,
    }
}

fn find_matching_parenthesis(
    chars: &[char],
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0;
    for (index, &character) in chars.iter().enumerate().skip(start) {
        if character == open {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn is_example_aside(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.starts_with("e.g.")
        || lower.starts_with("eg.")
        || lower.starts_with("for example")
        || content.starts_with("例:")
        || content.starts_with("例：")
        || content.starts_with("例えば")
        || content.starts_with("例如")
}

fn truncate_before_reason(line: &str) -> &str {
    [" Reason:", " 理由:", " 理由：", " 原因:", " 原因："]
        .iter()
        .filter_map(|marker| line.find(marker))
        .min()
        .map_or(line, |index| &line[..index])
}

fn first_sentence(line: &str) -> &str {
    let ascii_end = line.find(". ").map(|index| index + 1);
    let localized_end = ['。', '！', '？']
        .iter()
        .filter_map(|punctuation| {
            line.find(*punctuation)
                .map(|index| index + punctuation.len_utf8())
        })
        .min();
    ascii_end
        .into_iter()
        .chain(localized_end)
        .min()
        .map_or(line, |end| &line[..end])
}
