//! Style languages: one sibling per `<style lang>` over shared readers.
//!
//! Preprocessors themselves stay upstream. `structure` reads where rules,
//! declarations and comments are in any language, `classes` reads selector
//! preludes, and `nesting` resolves nested selectors. Each language module
//! supplies its lexis; `css` also owns the one production cssparser parse.


pub mod css;
pub mod language;
pub(crate) mod classes;
pub(crate) mod nesting;
pub(crate) mod sass;
pub(crate) mod structure;
