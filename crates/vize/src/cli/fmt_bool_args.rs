//! Accept a literal spaced bool without changing Clap's bare-flag/file grammar.
//!
//! Only the actual formatter command's declared optional boolean values qualify.
//! Required argument values stay opaque, and `--` ends option interpretation.

use clap::{Arg, Command, CommandFactory};
use std::ffi::OsString;

use super::Cli;

pub(super) fn normalize(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let args: Vec<_> = args.into_iter().collect();
    if !matches!(
        args.get(1).and_then(|arg| arg.to_str()),
        Some("fmt" | "glyph")
    ) || args.iter().any(|arg| arg.to_str().is_none())
    {
        return args;
    }
    let cli = Cli::command();
    let Some(mut command) = args
        .get(1)
        .and_then(|name| cli.find_subcommand(name))
        .cloned()
    else {
        return args;
    };
    if command.get_name() != "fmt" {
        return args;
    }
    command.build();
    rewrite(&command, &args).unwrap_or(args)
}

fn optional_bool(arg: &Arg) -> bool {
    arg.is_require_equals_set()
        && arg
            .get_num_args()
            .is_some_and(|range| range.min_values() == 0 && range.max_values() == 1)
        && arg.get_value_parser().type_id() == clap::builder::ValueParser::bool().type_id()
}

fn required_value(arg: &Arg) -> bool {
    arg.get_num_args()
        .is_some_and(|range| range.min_values() == 1 && range.max_values() == 1)
}

fn rewrite(command: &Command, args: &[OsString]) -> Option<Vec<OsString>> {
    let mut output: Vec<_> = args.iter().take(2).cloned().collect();
    let mut remaining = args.iter().skip(2).peekable();
    while let Some(token) = remaining.next() {
        let text = token.to_str()?;
        if text == "--" {
            output.push(token.clone());
            output.extend(remaining.cloned());
            break;
        }
        let mut take_value = false;
        if let Some(long) = text.strip_prefix("--") {
            let (name, attached) = long
                .split_once('=')
                .map_or((long, false), |(name, _)| (name, true));
            let arg = command.get_arguments().find(|arg| {
                arg.get_long() == Some(name)
                    || arg
                        .get_all_aliases()
                        .is_some_and(|aliases| aliases.contains(&name))
            })?;
            if !attached && optional_bool(arg) {
                if matches!(
                    remaining.peek().and_then(|value| value.to_str()),
                    Some("true" | "false")
                ) {
                    let mut joined = token.clone();
                    joined.push("=");
                    joined.push(remaining.next()?);
                    output.push(joined);
                    continue;
                }
            } else if !attached && arg.get_action().takes_values() {
                // Unsupported future arities remain entirely with Clap.
                if !required_value(arg) {
                    return None;
                }
                take_value = true;
            }
        } else if let Some(shorts) = text.strip_prefix('-').filter(|shorts| !shorts.is_empty()) {
            let mut shorts = shorts.chars().peekable();
            while let Some(short) = shorts.next() {
                let arg = command
                    .get_arguments()
                    .find(|arg| arg.get_short() == Some(short))?;
                if arg.get_action().takes_values() {
                    if !required_value(arg) {
                        return None;
                    }
                    take_value = shorts.peek().is_none();
                    break;
                }
            }
        }
        output.push(token.clone());
        if take_value && let Some(value) = remaining.next() {
            output.push(value.clone());
            if value == "--" {
                output.extend(remaining.cloned());
                break;
            }
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests;
