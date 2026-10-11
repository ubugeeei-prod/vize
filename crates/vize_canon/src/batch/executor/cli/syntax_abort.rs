//! Distinguish native parser aborts before diagnostic mapping and suppression.

use std::{path::PathBuf, process::Output};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashMap;

use super::{VirtualProject, is_cli_diagnostic_line, normalize_cli_path};

pub(super) fn output_aborted_on_syntax(output: &Output, project: &VirtualProject) -> bool {
    if output.status.success() {
        return false;
    }
    let mut parsed: FxHashMap<PathBuf, bool> = FxHashMap::default();
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
            if !suffix.starts_with("error TS") {
                return false;
            }
            let Some((path, _)) = prefix.rsplit_once('(') else {
                return false;
            };
            let path = normalize_cli_path(path, project.virtual_root());
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
