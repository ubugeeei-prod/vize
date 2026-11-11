//! Apply `max-lines` options by replacing the registered rule.

use super::super::config::Linter;
use vize_l0::config::MaxLinesOptions;

pub(super) fn apply(mut linter: Linter, options: &MaxLinesOptions) -> Linter {
    linter
        .registry
        .replace(Box::new(crate::rules::vue::MaxLines::from_options(options)));
    linter
}
