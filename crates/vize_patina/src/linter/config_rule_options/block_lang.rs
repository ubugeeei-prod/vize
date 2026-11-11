//! Apply `vue/block-lang` options by replacing the registered rule.

use super::super::config::Linter;
use vize_l0::config::BlockLangOptions;

pub(super) fn apply(mut linter: Linter, options: &BlockLangOptions) -> Linter {
    linter
        .registry
        .replace(Box::new(crate::rules::vue::BlockLang::from_options(
            options,
        )));
    linter
}
