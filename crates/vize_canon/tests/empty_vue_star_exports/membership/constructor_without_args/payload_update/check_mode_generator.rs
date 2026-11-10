//! Invert the diagnostics-only projection's generator threading (#12431) before older custody.
use super::support;

const PATH: &str = "crates/vize_canon/src/virtual_ts/generator.rs";
const AFTER: (usize, &str) = (
    37789,
    "3eb5b8a7c0e30e866a9cafe482955945f50bc35b9ac33396f52d3b97ce33acdf",
);
const BEFORE: (usize, &str) = (
    37699,
    "31ffef289d05da9ba517f41367ff6d21e46ef76e40b09a26a4389ceb7bdd6655",
);
/// Each successor beside its predecessor. The blank line keeps the generator
/// within its source length budget after the new scope option.
const SUCCESSORS: [(&str, &str); 2] = [
    (
        "                props_const_assertions.splice_output_line(&mut output_line, line_start);\n                // Strip `export` from non-import lines inside setup scope\n",
        "                props_const_assertions.splice_output_line(&mut output_line, line_start);\n\n                // Strip `export` from non-import lines inside setup scope\n",
    ),
    (
        "                preserve_event_navigation: generation_options.preserve_event_navigation,\n                omit_navigation_references: generation_options.omit_navigation_references,\n",
        "                preserve_event_navigation: generation_options.preserve_event_navigation,\n",
    ),
];

fn restore(mut current: String) -> String {
    assert_eq!(current.len(), AFTER.0);
    for (successor, predecessor) in SUCCESSORS {
        assert_eq!(current.matches(successor).count(), 1);
        current = current.replacen(successor, predecessor, 1);
    }
    assert_eq!(current.len(), BEFORE.0);
    assert_eq!(support::hash(current.as_bytes()), BEFORE.1);
    current
}

/// The exact pre-check-mode generator bytes; other paths and images pass through.
pub(super) fn source_before(path: &str, current: String) -> String {
    if path == PATH && support::hash(current.as_bytes()) == AFTER.1 {
        restore(current)
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn check_mode_generator_restores_its_exact_predecessor() {
        let current = fs::read_to_string(support::repository().join(PATH)).unwrap();
        assert_eq!(support::hash(current.as_bytes()), AFTER.1);
        let previous = source_before(PATH, current.clone());
        assert_eq!(support::hash(previous.as_bytes()), BEFORE.1);
        assert_eq!(source_before(PATH, previous.clone()), previous);
        assert_eq!(source_before("other.rs", current.clone()), current);
        let mutated = current.replacen(SUCCESSORS[1].0, "", 1);
        assert_eq!(source_before(PATH, mutated.clone()), mutated);
        assert!(std::panic::catch_unwind(|| restore(mutated)).is_err());
    }
}
