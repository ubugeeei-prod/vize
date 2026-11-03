use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::{FileIssueKind, Namespace, ReferenceTarget};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l2::resolution::Usage;

#[test]
fn forward_uses_and_export_leaves_resolve_at_the_actual_unit_scope_closure() {
    let arena = Allocator::default();
    let source = "const first = later; const later = 1; export {first as out};";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(
                parsed.admitted().unwrap(),
                SourceRoot::new(source).unwrap().whole_block(),
                2,
            )
            .unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    let scope = file.units()[0].scope;
    let first = file.lookup(scope, "first", Namespace::Value).unwrap().id();
    let later = file.lookup(scope, "later", Namespace::Value).unwrap().id();
    assert!(file.is_complete());
    assert_eq!(file.references().len(), 2);
    assert_eq!(file.references()[0].name, "later");
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(later)
    );
    assert_eq!(
        file.references()[1].target,
        ReferenceTarget::Resolved(first)
    );
    assert_eq!(file.exports()[0].name, "out");
    assert_eq!(file.exports()[0].local, Some(first));
}

#[test]
fn unresolved_runtime_names_keep_their_authored_site_without_context_or_global_fallback() {
    let arena = Allocator::default();
    let source = "é:const value = Math.random() + missing;";
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(3..).unwrap(), 3)
        .unwrap();
    let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(parsed.admitted().unwrap(), block, 5).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.references().len(), 2);
    assert!(
        file.references()
            .iter()
            .all(|reference| reference.target == ReferenceTarget::Unresolved)
    );
    assert_eq!(file.references()[0].span, Span::new(17, 21));
    assert_eq!(file.issues().len(), 2);
    assert!(
        file.issues()
            .iter()
            .all(|issue| issue.kind == FileIssueKind::UnresolvedReference)
    );
    assert_eq!(file.issues()[0].span, file.references()[0].span);
    assert_eq!(file.issues()[0].unit.index(), 5);
}

#[test]
fn unsupported_expression_rolls_back_every_preceding_use_in_that_subtree() {
    let arena = Allocator::default();
    let source = "const known = 1; const value = known + (() => missing);";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(
                parsed.admitted().unwrap(),
                SourceRoot::new(source).unwrap().whole_block(),
                0,
            )
            .unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert!(file.references().is_empty());
    assert_eq!(file.bindings().count(), 2);
    assert_eq!(file.issues().len(), 1);
    assert_eq!(file.issues()[0].kind, FileIssueKind::UnsupportedSyntax);
}

#[test]
fn real_use_events_preserve_write_shorthand_and_constructor_roles() {
    let arena = Allocator::default();
    let source = "import Maker from 'dep'; let value = 1; value += 2; const obj = {value}; const result = new Maker(value);";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(
                parsed.admitted().unwrap(),
                SourceRoot::new(source).unwrap().whole_block(),
                0,
            )
            .unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    let uses = file.references();
    assert_eq!(uses.len(), 4);
    assert_eq!(uses[0].usage, Usage::ReadWrite);
    assert!(uses[1].shorthand);
    assert_eq!(uses[2].name, "Maker");
    assert!(uses[2].constructor);
    assert!(!uses[3].constructor);
    assert_eq!(uses[0].target, uses[1].target);
    assert_eq!(uses[0].target, uses[3].target);
}

#[test]
fn type_export_local_resolves_in_its_namespace_while_reexports_create_no_local_use() {
    let arena = Allocator::default();
    let source = "import type {Thing} from 'types'; export type {Thing}; export {remote as external} from 'dep';";
    let source_type = SourceType::ts().with_module(true);
    let parsed = Parser::new(&arena, source, source_type).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(
                parsed.admitted().unwrap(),
                SourceRoot::new(source).unwrap().whole_block(),
                0,
            )
            .unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(file.references().len(), 1);
    assert_eq!(file.references()[0].namespace, Namespace::Type);
    assert!(matches!(
        file.references()[0].target,
        ReferenceTarget::Resolved(_)
    ));
    assert_eq!(file.exports()[0].namespace, Namespace::Type);
    assert!(file.exports()[0].local.is_some());
    assert_eq!(file.exports()[1].name, "external");
    assert_eq!(file.exports()[1].source.as_deref(), Some("dep"));
    assert!(file.exports()[1].local.is_none());
}
