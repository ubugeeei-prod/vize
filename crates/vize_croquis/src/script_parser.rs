//! OXC-based script parser for high-performance AST analysis.
//!
//! Uses OXC parser to extract:
//! - Compiler macros (defineProps, defineEmits, etc.)
//! - Top-level bindings (const, let, function, class)
//! - Import statements
//! - Reactivity wrappers (ref, computed, reactive)
//! - Invalid exports in script setup
//! - Nested function scopes (arrow functions, callbacks)
//!
//! ## Module Structure
//!
//! - [`parse`] - Public parse entry points (script setup / plain)
//! - [`result`] - [`ScriptParseResult`] and option/metadata types
//! - [`globals`] - Global-name tables and scope hierarchy setup
//! - [`process`] - Statement and variable processing
//! - [`extract`] - Props/emits extraction and reactivity detection
//! - [`walk`] - Scope walking functions

mod extract;
mod globals;
mod parse;
mod process;
mod result;
mod typeof_refs;
mod walk;

pub use parse::{
    analyze_script_setup_program, parse_script, parse_script_setup,
    parse_script_setup_with_generic, parse_script_with_options,
};
pub use process::{collect_options_descriptor, process_statement};
pub(crate) use result::{ReactiveGetterContext, ReactiveValueOrigin, RuntimeObjectLiteral};
pub use result::{ScriptParseResult, ScriptParserOptions};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod props_destructure_tests;

#[cfg(test)]
mod options_api_emits_tests;
