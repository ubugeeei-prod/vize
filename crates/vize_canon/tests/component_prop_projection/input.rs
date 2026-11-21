//! Ordinary public construction of a prepared JSX module.
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::Allocator;
use vize_l1_to_l2::framework::vue::jsx::{ModuleOptions, VIZE, lower_module_def};
use vize_l2::{component::Exec, module::ModuleDef};

pub(super) fn lower<'a>(arena: &'a Allocator, source: &'a str) -> ModuleDef<'a> {
    let parsed = Parser::new(arena.as_oxc(), source, SourceType::tsx()).parse_observed();
    let observed = parsed.admitted().unwrap_or_else(|| panic!("{source}"));
    let scoping = SemanticBuilder::new()
        .build(observed.program())
        .semantic
        .into_scoping();
    let options = ModuleOptions {
        dialect: &VIZE,
        default_exec: Exec::ReRender,
    };
    lower_module_def(arena, &observed, Some(&scoping), options)
        .unwrap()
        .module
}

pub(super) fn repeated(roots: usize) -> String {
    let mut source =
        String::from("declare function Widget(p: { title: string; active?: boolean }): unknown;\n");
    for n in 0..roots {
        use std::fmt::Write;
        writeln!(&mut source, "export function view{n}(bag: {{ title: string }}) {{ return <Widget {{...bag}} active title={{bag.title}}/>; }}").unwrap();
    }
    source
}
