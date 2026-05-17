//! Upgrade command - Update the installed Vize package.

use clap::{Args, ValueEnum};
use std::process::Command;
use vize_carton::String as CompactString;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum UpgradeSource {
    /// Update the npm package through the project package manager.
    #[default]
    PackageManager,

    /// Update a Cargo-installed binary. This is not the v1 alpha default channel.
    Cargo,
}

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct UpgradeArgs {
    /// Update source.
    #[arg(long, value_enum, default_value = "package-manager")]
    pub source: UpgradeSource,

    /// Package to install.
    #[arg(long, default_value = "vize")]
    pub package: String,

    /// Skip `--locked` when running Cargo.
    #[arg(long)]
    pub no_locked: bool,

    /// Print the command without running it.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: UpgradeArgs) {
    let (program, command_args) = command_for_args(&args);

    if args.dry_run {
        eprintln!("{} {}", program, command_args.join(" "));
        return;
    }

    let status = Command::new(program)
        .args(command_args.iter().map(|arg| arg.as_str()))
        .status()
        .unwrap_or_else(|error| {
            eprintln!("Failed to start {}: {}", program, error);
            std::process::exit(1);
        });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn command_for_args(args: &UpgradeArgs) -> (&'static str, Vec<CompactString>) {
    match args.source {
        UpgradeSource::PackageManager => {
            let package: CompactString = if args.package == "vize" {
                "vize@latest".into()
            } else {
                args.package.as_str().into()
            };
            ("vp", vec!["install".into(), "-D".into(), package])
        }
        UpgradeSource::Cargo => {
            let mut command_args: Vec<CompactString> = vec![
                "install".into(),
                args.package.as_str().into(),
                "--force".into(),
            ];
            if !args.no_locked {
                command_args.push("--locked".into());
            }
            ("cargo", command_args)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UpgradeArgs, UpgradeSource, command_for_args};

    #[test]
    fn upgrade_defaults_to_the_npm_package_channel() {
        let args = UpgradeArgs {
            source: UpgradeSource::PackageManager,
            package: "vize".into(),
            no_locked: false,
            dry_run: true,
        };

        assert_eq!(
            command_for_args(&args),
            (
                "vp",
                vec!["install".into(), "-D".into(), "vize@latest".into()]
            )
        );
    }

    #[test]
    fn cargo_upgrade_is_explicit_and_locked() {
        let args = UpgradeArgs {
            source: UpgradeSource::Cargo,
            package: "vize".into(),
            no_locked: false,
            dry_run: true,
        };

        assert_eq!(
            command_for_args(&args),
            (
                "cargo",
                vec![
                    "install".into(),
                    "vize".into(),
                    "--force".into(),
                    "--locked".into()
                ]
            )
        );
    }
}
