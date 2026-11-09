//! Proof of concept: a Solid component compiled through the level crates.
//!
//! Solid's markup lives inside its script, so the parsed module is the surface:
//! [`dialect`] lowers every outermost JSX element to a neutral L2 region, the
//! shared L2 to L3 lowering decides what is reactive, and [`target`] writes
//! Solid's DOM runtime calls. Everything outside the JSX is copied unchanged.
//! Not a supported product surface (2026-11-03 scope decision).

mod dialect;
mod target;

use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use poc_support::{Error, View, indent, line_indent};
use vize_l0::{Allocator, Span, String, cstr};
use vize_l4::write::{NoLinks, Writer};

/// Compile one Solid JSX module to a JavaScript module.
pub fn compile(source: &str) -> Result<String, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::jsx()).parse();
    if let Some(error) = parsed.diagnostics.iter().next() {
        return Err(Error::new(cstr!("{error}"), Span::new(0, 0)));
    }
    let mut roots = dialect::Roots::new(&allocator, source);
    roots.visit_program(&parsed.program);
    if let Some(error) = roots.error {
        return Err(error);
    }
    let mut module = target::Module::default();
    let mut body = Writer::<NoLinks>::with_capacity(source.len());
    let mut cursor = 0;
    for (span, region) in &roots.roots {
        let lowered = poc_support::lower(&allocator, region)?;
        let replacement = module.root(View::new(&lowered.program))?;
        body.push(source.get(cursor..span.start as usize).unwrap_or_default());
        body.push(&indent(&replacement, line_indent(source, span.start)));
        cursor = span.end as usize;
    }
    body.push(source.get(cursor..).unwrap_or_default());
    body.push(&module.epilogue());
    let mut preamble = Writer::<NoLinks>::default();
    preamble.push(&module.preamble());
    Ok(body.finish_with_preamble(preamble).text)
}

#[cfg(test)]
mod tests {
    use super::compile;

    const COUNTER: &str = include_str!("../input/Counter.jsx");
    const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__snapshots__/Counter.js");

    #[test]
    fn counter_compiles_to_a_solid_dom_module() {
        let output = compile(COUNTER);
        assert!(output.is_ok(), "{output:?}");
        let output = output.unwrap_or_default();
        assert_eq!(poc_support::check_module(&output), Ok(()));
        assert_eq!(poc_support::snapshot(SNAPSHOT, &output), Ok(()));
    }

    #[test]
    fn uncovered_syntax_is_refused_with_its_span() {
        let refused = compile("export const view = <><p /></>;");
        assert_eq!(refused.map_err(|error| error.span.start), Err(20));
    }
}
