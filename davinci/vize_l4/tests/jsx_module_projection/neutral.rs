//! Both projections receive independent parser observations of the same source.
//! The new module also serves ordinary readers of the authored host partition.

use super::support::Module;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};
use vize_l1::embed::regions::ModuleRegions;
use vize_l1_to_l2::framework::vue::jsx::{ModuleOptions, VIZE, lower_module_def};
use vize_l2::{
    component::Exec,
    module::{ModuleDef, ModuleError, ModulePart},
};
use vize_l4::targets::ts::{project_jsx_module, project_module};

pub(super) fn compare(arena: &Allocator, source: &str, old: &Module<'_>) {
    let parsed = Parser::new(arena.as_oxc(), source, old.program.source_type).parse_observed();
    let program = parsed.admitted().unwrap();
    let scoping = SemanticBuilder::new()
        .build(program.program())
        .semantic
        .into_scoping();
    let output = lower_module_def(
        arena,
        &program,
        Some(&scoping),
        ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        },
    )
    .unwrap();
    let module = output.module;
    let host = module.host();
    assert_eq!(host.typescript, old.program.source_type.is_typescript());
    assert_eq!(host.jsx, old.program.source_type.is_jsx());
    assert_eq!(host.module, old.program.source_type.is_module());
    let mut end = 0;
    module
        .visit(|part| {
            let span = match part {
                ModulePart::Host { span, text } => {
                    assert_eq!(
                        source.get(span.start as usize..span.end as usize),
                        Some(text)
                    );
                    span
                }
                ModulePart::Render(component) => component.span,
            };
            assert_eq!(span.start, end);
            assert!(span.end > span.start);
            end = span.end;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(end as usize, source.len());
    match (
        project_jsx_module(&old.program, &old.roots),
        project_module(&module),
    ) {
        (Ok(expected), Ok(actual)) => {
            assert_eq!(actual.document(), expected.document());
            assert_eq!(actual.link_facts(), expected.link_facts());
            assert_eq!(actual.directives(), expected.directives());
            for link in actual.document().links() {
                for span in [
                    link.generated,
                    Span::new(link.generated.end, link.generated.end),
                ] {
                    assert_eq!(actual.map_span(span), expected.map_span(span));
                }
            }
        }
        (Err(expected), Err(actual)) => assert_eq!(actual, expected),
        (old, new) => panic!("projection differs for {source:?}: {old:?}, {new:?}"),
    }
}

#[test]
fn verified_render_edits_reject_other_buffers_and_missing_or_reordered_roots() {
    let arena = Allocator::default();
    let source = "/*😀*/ const a=<div/>, b=<i/>;\r\n";
    // The same bytes held in a different allocation must never supply the body.
    let other = source.to_owned();
    let parsed = Parser::new(arena.as_oxc(), source, SourceType::jsx()).parse_observed();
    let program = parsed.admitted().unwrap();
    let regions = || ModuleRegions::from_program(&program).unwrap();
    let foreign = super::support::lower(&arena, &other, false).unwrap();
    assert!(matches!(
        ModuleDef::checked(regions(), foreign.roots),
        Err(ModuleError::ForeignSource)
    ));
    let mut local = super::support::lower(&arena, source, false).unwrap();
    local.roots.reverse();
    assert!(matches!(
        ModuleDef::checked(regions(), local.roots),
        Err(ModuleError::Roots)
    ));
    assert!(matches!(
        ModuleDef::checked(regions(), Vec::new()),
        Err(ModuleError::Roots)
    ));
}

#[test]
fn empty_plain_and_typed_hosts_preserve_their_entire_original_run() {
    let arena = Allocator::default();
    for source in [
        "",
        "/* no render */\r\n",
        "export const 日本語: number = 3;\r\n",
    ] {
        super::support::lower(&arena, source, true).unwrap();
    }
}

#[test]
fn keeps_prologue_issues_and_stops_a_visitor_on_its_first_refusal() {
    let arena = Allocator::default();
    let source = "function App(){'use vue:unknown'; return <div/>;}";
    let parsed = Parser::new(arena.as_oxc(), source, SourceType::jsx()).parse_observed();
    let program = parsed.admitted().unwrap();
    let lowered = lower_module_def(
        &arena,
        &program,
        None,
        ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        },
    )
    .unwrap();
    assert!(matches!(
        lowered.issues.as_slice(),
        [vize_l1_to_l2::framework::vue::jsx::PrologueIssue::Unknown {
            directive: "use vue:unknown",
            ..
        }]
    ));
    let mut count = 0;
    let out = lowered.module.visit(|_| {
        count += 1;
        Err(4)
    });
    assert_eq!((count, out), (1, Err(4)));
}
mod profile;
