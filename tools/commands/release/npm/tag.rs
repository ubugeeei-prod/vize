#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(version) = args.first() else {
        println!("Usage: rust-script tools/commands/release/npm/tag.rs <version>");
        return ExitCode::from(1);
    };
    println!("{}", npm_tag_for_version(version));
    ExitCode::SUCCESS
}

fn npm_tag_for_version(version: &str) -> &'static str {
    if version.contains("-alpha") {
        "alpha"
    } else if version.contains("-beta") {
        "beta"
    } else if version.contains("-rc") {
        "rc"
    } else {
        "latest"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_prerelease_versions_to_dist_tags() {
        assert_eq!(npm_tag_for_version("1.2.3-alpha.1"), "alpha");
        assert_eq!(npm_tag_for_version("1.2.3-beta.1"), "beta");
        assert_eq!(npm_tag_for_version("1.2.3-rc.1"), "rc");
        assert_eq!(npm_tag_for_version("1.2.3"), "latest");
    }
}
