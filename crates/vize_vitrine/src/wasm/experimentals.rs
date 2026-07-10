use crate::CompilerOptions;
use vize_atelier_core::options::ParserOptions;
use vize_atelier_dom::DomCompilerOptions;

pub(super) fn experimental_flags(opts: &CompilerOptions) -> (bool, bool) {
    (
        opts.experimental_in_tag_comments.unwrap_or(false),
        opts.experimental_patterned_template.unwrap_or(false),
    )
}

pub(super) fn experimental_parser_options(opts: &CompilerOptions) -> ParserOptions {
    ParserOptions {
        experimental_in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
        ..Default::default()
    }
}

pub(super) fn experimental_dom_options(opts: &CompilerOptions) -> DomCompilerOptions {
    let (experimental_in_tag_comments, experimental_patterned_template) = experimental_flags(opts);
    DomCompilerOptions {
        experimental_in_tag_comments,
        experimental_patterned_template,
        ..Default::default()
    }
}
