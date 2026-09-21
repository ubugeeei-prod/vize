//! The refusal diagnostics: why each pug construct has no static Vue
//! template, and what to write instead.

use vize_s0::{String, cstr};
use vize_s1::pug::PugRefusal;

/// The error message for a refused keyword construct.
pub(super) fn message(refusal: PugRefusal) -> String {
    let reason = match refusal {
        PugRefusal::Doctype => "a Vue template renders a fragment, not a document",
        PugRefusal::Extends
        | PugRefusal::Block
        | PugRefusal::MixinBlock
        | PugRefusal::Include
        | PugRefusal::Yield => "it links other pug files at build time",
        PugRefusal::Mixin | PugRefusal::Call => "mixins expand at build time; use a Vue component",
        PugRefusal::Filter => "filters run at build time",
        PugRefusal::Conditional
        | PugRefusal::Case
        | PugRefusal::When
        | PugRefusal::Default
        | PugRefusal::Each
        | PugRefusal::While => "it runs JavaScript at build time; use `v-if` / `v-for`",
        PugRefusal::Interpolation | PugRefusal::BlockCode => "it runs JavaScript at build time",
    };
    let keyword = refusal.keyword();
    cstr!("pug `{keyword}` is not supported in Vue templates: {reason}")
}

pub(super) const UNBUFFERED_CODE: &str = "pug unbuffered code (`- …`) is not supported in Vue templates: it runs JavaScript at build time";

pub(super) const CODE_INTERPOLATION: &str =
    "pug `#{…}` / `!{…}` interpolation is not supported in Vue templates: use `{{ … }}`";

pub(super) const EXECUTABLE_CODE: &str = "pug buffered code must be a constant literal in Vue \
                                          templates; use `{{ … }}` for expressions";

pub(super) const AND_ATTRIBUTES: &str =
    "pug `&attributes` is not supported in Vue templates: use `v-bind`";
