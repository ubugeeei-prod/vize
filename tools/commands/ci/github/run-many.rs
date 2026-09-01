#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    process::{Command, ExitCode, ExitStatus},
};

fn main() -> ExitCode {
    let groups = split_groups(env::args().skip(1));
    if groups.is_empty() {
        println!("Usage: rust-script tools/commands/ci/github/run-many.rs <cmd...> -- <cmd...>");
        return ExitCode::from(1);
    }

    for group in groups {
        let command = &group[0];
        let args = &group[1..];
        let status = match Command::new(command).args(args).status() {
            Ok(status) => status,
            Err(error) => {
                eprintln!("run-many: failed to run {command}: {error}");
                return ExitCode::from(1);
            }
        };
        if !status.success() {
            return exit_code(status);
        }
    }

    ExitCode::SUCCESS
}

fn split_groups(args: impl IntoIterator<Item = String>) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for arg in args {
        if arg == "--" {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(arg);
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn exit_code(status: ExitStatus) -> ExitCode {
    match status.code().and_then(|code| u8::try_from(code).ok()) {
        Some(code) => ExitCode::from(code),
        None => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::split_groups;

    #[test]
    fn splits_commands_on_separator() {
        let groups = split_groups(
            ["vp", "run", "build", "--", "cargo", "test"]
                .into_iter()
                .map(String::from),
        );

        assert_eq!(
            groups,
            vec![
                vec!["vp".to_string(), "run".to_string(), "build".to_string()],
                vec!["cargo".to_string(), "test".to_string()]
            ]
        );
    }

    #[test]
    fn ignores_empty_separator_groups() {
        let groups = split_groups(
            ["--", "vp", "install", "--", "--"]
                .into_iter()
                .map(String::from),
        );

        assert_eq!(groups, vec![vec!["vp".to_string(), "install".to_string()]]);
    }
}
