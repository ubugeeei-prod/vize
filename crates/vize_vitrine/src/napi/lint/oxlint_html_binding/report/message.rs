//! The existing plugin's message/help contract, applied only at final conversion.
use std::sync::LazyLock;
use vize_patina::LintDiagnostic;

pub(super) fn format(finding: &LintDiagnostic, help_level: &str) -> Result<String, String> {
    let message = finding.message.as_str();
    let summary = message
        .find(". ")
        .and_then(|end| message.get(..end + 1))
        .unwrap_or(message);
    let mut output = String::from(summary);
    if let Some(details) = message
        .strip_prefix(summary)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        section(&mut output, "Details", details);
    }
    let Some(help) = finding.help.as_ref() else {
        return Ok(output);
    };
    if help_level == "none" {
        return Ok(output);
    }
    let help = plain(help.as_str())?;
    let selected = if help_level == "short" {
        help.lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("")
    } else {
        help.trim()
    };
    if !selected.is_empty() {
        section(&mut output, "Help", selected);
    }
    Ok(output)
}

fn section(output: &mut String, title: &str, text: &str) {
    output.push_str("\n    ");
    output.push_str(title);
    output.push_str(":\n");
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str("      ");
        output.push_str(line);
    }
}

static PATTERNS: LazyLock<Result<Vec<(regex::Regex, &'static str)>, String>> =
    LazyLock::new(|| {
        [
            (r"(?m)^\s*```[^\n]*$", ""),
            (r"\*\*([^\n]*?)\*\*", "$1"),
            (r"__([^\n]*?)__", "$1"),
            (r"`([^`\n]+)`", "$1"),
            (r"(?m)^\s*[-*]\s+", "- "),
            (r"\n{3,}", "\n\n"),
        ]
        .into_iter()
        .map(|(pattern, replacement)| {
            regex::Regex::new(pattern)
                .map(|regex| (regex, replacement))
                .map_err(|_| String::from("HTML help formatter has an invalid built-in pattern"))
        })
        .collect()
    });

fn plain(help: &str) -> Result<String, String> {
    let mut text = help.replace("\r\n", "\n").replace('\r', "\n");
    for (regex, replacement) in PATTERNS.as_ref().map_err(Clone::clone)? {
        text = regex.replace_all(&text, *replacement).into_owned();
    }
    Ok(text.trim().into())
}
