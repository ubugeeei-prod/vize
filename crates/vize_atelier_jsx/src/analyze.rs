//! Running Croquis semantic analysis over a parsed JSX/TSX module.
//!
//! Croquis normally parses script text itself with a TypeScript [`SourceType`],
//! which would reject JSX. We sidestep that by feeding it the program we have
//! *already* parsed with the correct JSX/TSX dialect via its parse-free entry
//! point, so the binding/scope/reactivity analysis runs without a second parse.
//!
//! [`SourceType`]: oxc_span::SourceType

use oxc_ast::ast::Program;
use vize_croquis::{Croquis, Drawer};

/// Analyze a parsed JSX/TSX program, returning the Croquis binding metadata,
/// scope chain, reactivity tracking, and macro/import information.
///
/// Exposed (re-exported as [`crate::analyze_jsx_program`]) so consumers that
/// parse the program themselves — e.g. Patina's zero-cost JSX lint path, which
/// drives rules straight over the OXC AST without lowering — can attach the same
/// semantic analysis the lowering lane produces without a second parse.
pub fn analyze_program(program: &Program<'_>, source: &str) -> Croquis {
    let mut drawer = Drawer::for_compile();
    drawer.draw_script_setup_program(program, source, None);
    drawer.finish()
}
