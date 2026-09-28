//! Triple-slash path dependencies are not JavaScript import declarations.

use vize_carton::{String, ToCompactString};

pub(super) fn path_references(source: &str) -> Vec<String> {
    if !source.contains("///") {
        return Vec::new();
    }
    source
        .lines()
        .filter_map(|line| {
            let comment = line.trim_start().strip_prefix("///")?.trim_start();
            let directive = comment.strip_prefix("<reference")?;
            let path = directive.find("path=")?;
            let value = directive.get(path + "path=".len()..)?;
            let quote = value.as_bytes().first().copied()?;
            if quote != b'\'' && quote != b'"' {
                return None;
            }
            let rest = value.get(1..)?;
            let end = rest.find(quote as char)?;
            Some(rest.get(..end)?.to_compact_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::path_references;

    #[test]
    fn extracts_only_path_references() {
        let source = "/// <reference types=\"vite/client\" />\n/// <reference path='./types/env.d.ts' />\n/// <reference path=\"../shared.d.ts\" />\n";
        assert_eq!(
            path_references(source),
            ["./types/env.d.ts", "../shared.d.ts"]
        );
    }
}
