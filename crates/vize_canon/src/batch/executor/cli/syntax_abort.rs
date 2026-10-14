//! Distinguish native parser aborts before diagnostic mapping and suppression.

use std::{path::PathBuf, process::Output};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashMap;

use super::{CliPathResolver, VirtualProject, is_cli_diagnostic_line};

pub(super) fn output_aborted_on_syntax(output: &Output, project: &VirtualProject) -> bool {
    if output.status.success() {
        return false;
    }
    let mut parsed: FxHashMap<PathBuf, bool> = FxHashMap::default();
    let mut paths = CliPathResolver::new(project.virtual_root());
    [&output.stdout, &output.stderr].into_iter().any(|stream| {
        #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
        let text = std::string::String::from_utf8_lossy(stream);
        text.lines().any(|line| {
            if !is_cli_diagnostic_line(line) {
                return false;
            }
            let Some((prefix, suffix)) = line.split_once("): ") else {
                return false;
            };
            // A parser error is always reported with a syntactic TS1xxx code,
            // so only files with such a line can have aborted the semantic
            // pass. Re-parsing every file with a semantic finding would read
            // and parse most of a large project on every failing run.
            if !is_syntactic_error(suffix) {
                return false;
            }
            let Some((path, _)) = prefix.rsplit_once('(') else {
                return false;
            };
            let path = paths.resolve(path);
            *parsed.entry(path.clone()).or_insert_with(|| {
                let Ok(source_type) = SourceType::from_path(&path) else {
                    return false;
                };
                let Ok(source) = std::fs::read_to_string(&path) else {
                    return false;
                };
                !Parser::new(&Allocator::default(), &source, source_type)
                    .parse()
                    .diagnostics
                    .is_empty()
            })
        })
    })
}

/// `error TS1xxx: …`: TypeScript numbers parser diagnostics 1000–1999.
fn is_syntactic_error(suffix: &str) -> bool {
    suffix
        .strip_prefix("error TS")
        .and_then(|rest| rest.split_once(':'))
        .and_then(|(code, _)| code.parse::<u32>().ok())
        .is_some_and(|code| (1000..2000).contains(&code))
}

#[cfg(test)]
mod tests {
    use super::is_syntactic_error;

    #[test]
    fn only_parser_error_codes_can_abort_the_semantic_pass() {
        assert!(is_syntactic_error("error TS1005: ';' expected."));
        assert!(is_syntactic_error(
            "error TS1128: Declaration or statement expected."
        ));
        assert!(!is_syntactic_error(
            "error TS2322: Type 'string' is not assignable."
        ));
        assert!(!is_syntactic_error(
            "error TS7006: Parameter 'x' implicitly has an 'any' type."
        ));
        assert!(!is_syntactic_error("warning TS1005: ';' expected."));
        assert!(!is_syntactic_error("error TS: malformed"));
    }
}
