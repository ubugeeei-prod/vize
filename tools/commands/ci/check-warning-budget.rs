#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    io::{self, Write},
    process::{self, Command},
};

const WARNING_BUDGET_ERROR: &str = "JS/TS warning budget is 0 for v1 alpha CI";
const WARNING_MARKER: &str = "warn: Lint warnings found";

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args = command_args(env::args().skip(1).collect::<Vec<_>>());
    if args.is_empty() {
        eprintln!(
            "Usage: rust-script tools/commands/ci/check-warning-budget.rs -- <command> [args...]"
        );
        return 2;
    }

    let command = &args[0];
    let child_args = &args[1..];
    let output = match Command::new(command).args(child_args).output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("failed to run {command}: {error}");
            return 1;
        }
    };

    write_all(&mut io::stdout(), &output.stdout);
    write_all(&mut io::stderr(), &output.stderr);

    if !output.status.success() {
        return output.status.code().unwrap_or(1);
    }

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stripped = strip_ansi(&combined);
    let warning_count = count_warnings(&stripped);
    let has_marker = has_unparsed_warning_marker(&stripped);

    if warning_count > 0 {
        eprintln!("{WARNING_BUDGET_ERROR}; found {warning_count} warnings.");
        return 1;
    }
    if has_marker {
        eprintln!("{WARNING_BUDGET_ERROR}; found unparsed warnings.");
        return 1;
    }
    0
}

fn command_args(args: Vec<String>) -> Vec<String> {
    match args.first().map(String::as_str) {
        Some("--") => args.into_iter().skip(1).collect(),
        _ => args,
    }
}

fn write_all(writer: &mut dyn Write, bytes: &[u8]) {
    if let Err(error) = writer.write_all(bytes) {
        eprintln!("failed to write child process output: {error}");
    }
}

fn strip_ansi(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(next) = chars.next() {
                if next == 'm' {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn count_warnings(text: &str) -> usize {
    text.lines().map(count_line_warnings).sum()
}

fn count_line_warnings(line: &str) -> usize {
    let Some(start) = line.find("Found ") else {
        return 0;
    };
    let rest = &line[start + "Found ".len()..];
    let Some(after_error_count) = strip_number(rest) else {
        return 0;
    };
    let after_error_label = after_error_count
        .strip_prefix(" error and ")
        .or_else(|| after_error_count.strip_prefix(" errors and "));
    let Some(after_error_label) = after_error_label else {
        return 0;
    };
    let Some((warning_count, after_warning_count)) = parse_number(after_error_label) else {
        return 0;
    };
    if after_warning_count == " warning" || after_warning_count == " warnings" {
        warning_count
    } else {
        0
    }
}

fn strip_number(text: &str) -> Option<&str> {
    parse_number(text).map(|(_, rest)| rest)
}

fn parse_number(text: &str) -> Option<(usize, &str)> {
    let digit_count = text.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digit_count == 0 {
        return None;
    }
    let (digits, rest) = text.split_at(digit_count);
    digits.parse::<usize>().ok().map(|count| (count, rest))
}

fn has_unparsed_warning_marker(text: &str) -> bool {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .contains(WARNING_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_sgr_sequences() {
        assert_eq!(
            strip_ansi("\x1b[31mFound 0 errors and 2 warnings\x1b[0m"),
            "Found 0 errors and 2 warnings"
        );
    }

    #[test]
    fn counts_warning_summary_lines() {
        assert_eq!(count_warnings("Found 1 error and 1 warning"), 1);
        assert_eq!(count_warnings("Found 0 errors and 12 warnings"), 12);
        assert_eq!(count_warnings("prefix Found 0 errors and 3 warnings"), 3);
        assert_eq!(count_warnings("Found no warnings"), 0);
    }

    #[test]
    fn detects_warning_marker_with_spacing() {
        assert!(has_unparsed_warning_marker("warn: Lint warnings found"));
        assert!(has_unparsed_warning_marker("warn:\nLint warnings found"));
    }
}
