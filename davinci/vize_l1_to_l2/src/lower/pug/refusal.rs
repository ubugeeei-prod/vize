//! The refusal diagnostics: why each pug construct has no static Vue
//! template, and what to write instead.

use vize_l0::{String, cstr};
use vize_l1::pug::PugRefusal;

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

/// Buffered code or `#{…}` interpolation without a build-time value.
pub(super) fn code(reason: &str) -> String {
    cstr!(
        "pug code cannot be evaluated at build time: {reason} (use `{{{{ … }}}}` for component \
         state)"
    )
}

/// Unbuffered code (`- …`) pug cannot run without an engine.
pub(super) fn statement(reason: &str) -> String {
    cstr!("pug unbuffered code (`- …`) cannot run at build time: {reason}")
}

pub(super) fn attribute(key: &str, reason: &str) -> String {
    cstr!(
        "pug attribute `{key}` cannot be evaluated at build time: {reason} (use `:{key}` for \
         bindings)"
    )
}

pub(super) fn condition(keyword: &str, reason: &str) -> String {
    cstr!(
        "pug `{keyword}` cannot be evaluated at build time: {reason} (use `v-if` for component \
         state)"
    )
}

pub(super) fn each(reason: &str) -> String {
    cstr!("pug `each` cannot run at build time: {reason} (use `v-for` for component state)")
}

pub(super) const EACH_BLOCK: &str = "pug `each` and its `else` need an indented block";

pub(super) const EACH_SAME_NAMES: &str = "pug `each` cannot use one name for the value and the key";

pub(super) const CODE_BLOCK: &str = "pug unbuffered code with a block runs JavaScript control flow \
                                     at build time; use pug `if` / `each` or `v-if` / `v-for`";

pub(super) const AND_ATTRIBUTES: &str =
    "pug `&attributes` is not supported in Vue templates: use `v-bind`";
