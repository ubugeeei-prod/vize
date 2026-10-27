//! The refusal diagnostics: why each pug construct has no static Vue
//! template, and what to write instead.

use vize_l0::{String, cstr};
use vize_l1::pug::PugRefusal;

/// The error message for a refused keyword construct.
pub(super) fn message(refusal: PugRefusal) -> String {
    let reason = match refusal {
        PugRefusal::Doctype => "a Vue template renders a fragment, not a document",
        PugRefusal::Extends | PugRefusal::Block | PugRefusal::Include | PugRefusal::Yield => {
            "it links other pug files at build time"
        }
        // Run by the mixin lowering; never refused as a keyword.
        PugRefusal::Mixin | PugRefusal::MixinBlock => "mixins expand at build time",
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

pub(super) fn and_attributes(reason: &str) -> String {
    cstr!(
        "pug `&attributes` cannot be evaluated at build time: {reason} (use `v-bind` for \
         component state)"
    )
}

/// A mixin call whose name, arguments or attributes have no value.
pub(super) fn call(reason: &str) -> String {
    cstr!(
        "pug mixin call cannot be evaluated at build time: {reason} (mixins expand with \
         build-time values; use a Vue component for component state)"
    )
}

pub(super) fn parameters(name: &str, reason: &str) -> String {
    cstr!("pug mixin `{name}` cannot bind its parameters at build time: {reason}")
}

pub(super) fn undefined_mixin(name: &str) -> String {
    cstr!("pug mixin `{name}` is not defined before this call")
}

pub(super) fn without_body(name: &str) -> String {
    cstr!("Mixin {name} declared without body")
}

pub(super) const MIXIN_HEAD: &str =
    "pug cannot parse this mixin head: expected `mixin name` or `mixin name(parameters)`";

pub(super) const MIXIN_PROTO: &str =
    "pug mixin `__proto__` would replace the prototype of pug's mixin table";

pub(super) const MIXIN_BLOCK_LINE: &str = "pug `block` takes no text or nested block";

pub(super) const MIXIN_BLOCK_VALUE: &str =
    "pug `block` is not a mixin block here: `block()` would throw";

pub(super) const MIXIN_DEPTH: &str = "pug mixin calls nest too deeply to expand at build time";

pub(super) const MIXIN_BUDGET: &str = "pug mixins expand to too many calls at build time";

/// Why a value holding a mixin `block` cannot be printed.
pub(super) const FUNCTION_VALUE: &str =
    "it holds a mixin `block` function, whose text is pug's generated code";
