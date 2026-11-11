//! Apply `vue/max-lines-per-block` options by replacing the registered rule.

use super::super::config::Linter;
use vize_l0::config::MaxLinesPerBlockOptions;

pub(super) fn apply(mut linter: Linter, options: &MaxLinesPerBlockOptions) -> Linter {
    linter
        .registry
        .replace(Box::new(crate::rules::vue::MaxLinesPerBlock::from_options(
            options,
        )));
    linter
}
