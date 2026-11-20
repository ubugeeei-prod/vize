//! Every name the type-check target writes that the authored code does not.
//!
//! The target never declares a helper. Each ambient name in [`HELPERS`] is
//! declared in the published ambient set, `npm/cli/virtual-ts-globals.d.ts`,
//! which `tests/tooling/l4-ts-helpers.test.ts` asserts. A construct that needs
//! an unpublished helper is refused until the helper-asset change lands.

/// Checks one bound native element prop: `<Tag, Prop>(value)`.
pub const ELEMENT_PROP: &str = "__vizeNativeElementProp";

/// Every ambient helper the target writes.
pub const HELPERS: &[&str] = &[ELEMENT_PROP];

/// The value type of a re-render root, from Vue's public types.
pub const RE_RENDER_ROOT: &str = "import(\"vue\").VNode";
