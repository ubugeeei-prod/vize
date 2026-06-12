//! Running Croquis semantic analysis over a parsed JSX/TSX module.
//!
//! Croquis normally parses script text itself with a TypeScript [`SourceType`],
//! which would reject JSX. We sidestep that by feeding it the program we have
//! *already* parsed with the correct JSX/TSX dialect via its parse-free entry
//! point, so the binding/scope/reactivity analysis runs without a second parse.
//!
//! [`SourceType`]: oxc_span::SourceType

use oxc_ast::ast::Program;
use vize_croquis::script_parser::analyze_script_setup_program;
use vize_croquis::{Croquis, Drawer};

/// Analyze a parsed JSX/TSX program, returning the Croquis binding metadata,
/// scope chain, reactivity tracking, and macro/import information.
pub(crate) fn analyze_program(program: &Program<'_>, source: &str) -> Croquis {
    let result = analyze_script_setup_program(program, source, None);
    let mut croquis = Drawer::new().finish();
    result.apply_to_croquis(&mut croquis);
    croquis
}
