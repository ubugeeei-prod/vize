use std::process::Output;

use super::output_text;

pub(super) const DISABLED_OPTIONS_API: [&str; 4] = [
    "src/Options.vue(31,6): error TS2304: Cannot find name 'label'.",
    "src/Options.vue(31,18): error TS2304: Cannot find name 'count'.",
    "src/Options.vue(31,41): error TS2304: Cannot find name 'doubled'.",
    "src/Options.vue(31,66): error TS2304: Cannot find name 'increment'.",
];

pub(super) fn raw_lines(output: &Output) -> Vec<std::string::String> {
    assert!(output.stderr.is_empty(), "{}", output_text(output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect()
}
