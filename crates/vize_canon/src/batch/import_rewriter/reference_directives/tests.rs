use std::path::Path;

use super::{escaping_reference_occurrences, relative_reference_occurrences};

fn values(source: &str) -> Vec<&str> {
    relative_reference_occurrences(source)
        .iter()
        .map(|(start, end, value, _)| {
            let span = source
                .get(*start as usize..*end as usize)
                .unwrap_or_default();
            assert_eq!(span, value.as_str());
            span
        })
        .collect()
}

#[test]
fn relative_path_and_types_references_in_the_prologue_are_collected() {
    let source = concat!(
        "// header\n",
        "/* block\n comment */\n",
        "/// <reference types=\"vite/client\" />\n",
        "/// <reference types=\"../../.histoire/components\" />\n",
        "///<reference path='./env.d.ts'/>\n",
        "/// <reference lib=\"dom\" />\n",
        "\n",
        "import type { X } from './x'\n",
        "/// <reference path=\"./after.d.ts\" />\n",
    );
    assert_eq!(values(source), ["../../.histoire/components", "./env.d.ts"]);
}

#[test]
fn attribute_names_need_a_boundary_and_a_quoted_value() {
    for source in [
        "/// <reference xtypes=\"./a\" />\n",
        "/// <reference types=./a />\n",
        "/// <amd-module name=\"./a\" />\n",
        "// <reference types=\"./a\" />\n",
    ] {
        assert_eq!(values(source), Vec::<&str>::new(), "{source}");
    }
}

#[test]
fn sources_without_a_reference_directive_are_skipped() {
    assert!(relative_reference_occurrences("import './a'\n").is_empty());
}

#[test]
fn only_targets_outside_the_project_root_are_rewritten() {
    let source = concat!(
        "/// <reference types=\"../../types/components\" />\n",
        "/// <reference path=\"./env.d.ts\" />\n",
        "/// <reference path=\"../shared/env.d.ts\" />\n",
    );
    let escaping = escaping_reference_occurrences(
        source,
        Some(Path::new("/vize-reference-case/app/src")),
        Path::new("/vize-reference-case/app"),
    );
    let values = escaping
        .iter()
        .map(|(_, _, value, _)| value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(values, ["../../types/components"]);
    assert!(
        escaping_reference_occurrences(source, None, Path::new("/vize-reference-case")).is_empty()
    );
}
