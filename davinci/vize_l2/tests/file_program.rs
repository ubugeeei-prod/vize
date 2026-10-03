use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l2::file::{DeclarationKind, FileIssueKind, InitializerKind, Namespace};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

#[test]
fn actual_declaration_sites_keep_container_index_and_full_file_spans() {
    let arena = Allocator::default();
    let source = "<!--é--><script>export const 日本語 = 1; let tip = 'x';</script>";
    let start = source.find("export").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let input = ProgramInput::checked(parsed.admitted().unwrap(), block, 3).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let unit = producer.program(input, ProgramScope::Module).unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(unit.index(), 3);
    assert_eq!(file.units()[0].span, block.span());
    let bindings: Vec<_> = file.bindings().collect();
    assert_eq!(bindings.len(), 2);
    let declaration = bindings[0].declaration().unwrap();
    assert_eq!(declaration.id.index(), 0);
    assert_eq!(declaration.name, "日本語");
    assert_eq!(
        declaration.span.start as usize,
        source.find("日本語").unwrap()
    );
    assert_eq!(declaration.kind, DeclarationKind::Const);
    assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
    assert_eq!(file.exports()[0].local, Some(declaration.id));
    assert_eq!(file.exports()[0].unit, unit);
    assert!(file.references().is_empty());
}

#[test]
fn type_only_imports_occupy_the_type_namespace_without_runtime_binding() {
    let arena = Allocator::default();
    let source = "import type {T} from 'types'; import make, {value as local} from 'dep';";
    let source_type = SourceType::ts().with_module(true);
    let parsed = Parser::new(&arena, source, source_type).parse_observed();
    assert!(parsed.diagnostics().is_empty());
    let block = SourceRoot::new(source).unwrap().whole_block();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(parsed.admitted().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    let scope = file.units()[0].scope;
    assert!(file.is_complete());
    assert!(file.lookup(scope, "T", Namespace::Value).is_none());
    assert_eq!(
        file.lookup(scope, "T", Namespace::Type)
            .unwrap()
            .id()
            .index(),
        0
    );
    let local = file.lookup(scope, "local", Namespace::Value).unwrap();
    assert_eq!(
        local.declaration().unwrap().imported_name.as_deref(),
        Some("value")
    );
    assert_eq!(
        local.declaration().unwrap().import_source.as_deref(),
        Some("dep")
    );
    assert_eq!(file.imports().len(), 2);
}

#[test]
fn borrowed_handles_distinguish_files_even_when_numeric_ids_match() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let build = || {
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(parsed.admitted().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        producer.finish().unwrap()
    };
    let first = build();
    let second = build();
    let a = first.bindings().next().unwrap();
    let b = second.bindings().next().unwrap();
    assert_eq!(a.id(), b.id());
    assert!(a.same_owner(a));
    assert!(!a.same_owner(b));
    assert!(core::ptr::eq(a.file(), &first));
}

#[test]
fn whole_program_admission_rejects_foreign_source_and_builder_ownership() {
    let arena = Allocator::default();
    let source = String::from("const value = 1;");
    let copy = source.clone();
    let parsed = Parser::new(&arena, &source, SourceType::mjs()).parse_observed();
    let foreign = SourceRoot::new(&copy).unwrap().whole_block();
    assert!(
        matches!(ProgramInput::checked(parsed.admitted().unwrap(), foreign, 0),
        Err(error) if error.kind == FileIssueKind::InvalidSource)
    );
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let mut producer = FileProducer::new(&arena, &copy).unwrap();
    assert_eq!(
        producer
            .program(
                ProgramInput::checked(parsed.admitted().unwrap(), block, 0).unwrap(),
                ProgramScope::Module
            )
            .unwrap_err()
            .kind,
        FileIssueKind::InvalidSource
    );
}

#[test]
fn actual_nested_unit_shadowing_preserves_distinct_declaration_sites() {
    let arena = Allocator::default();
    let source = "const value = 1; const value = 2;";
    let split = source.find(" const").unwrap() + 1;
    let root = SourceRoot::new(source).unwrap();
    let first = root.block(source.get(..split).unwrap(), 0).unwrap();
    let second = root
        .block(source.get(split..).unwrap(), split as u32)
        .unwrap();
    let a = Parser::new(&arena, first.source(), SourceType::mjs()).parse_observed();
    let b = Parser::new(&arena, second.source(), SourceType::mjs()).parse_observed();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(a.admitted().unwrap(), first, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    producer
        .program(
            ProgramInput::checked(b.admitted().unwrap(), second, 2).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    let outer = file.units()[0].scope;
    let inner = file.units()[1].scope;
    assert!(file.is_complete());
    assert_ne!(outer, inner);
    assert_eq!(file.scopes()[1].parent, Some(outer));
    assert_ne!(
        file.lookup(outer, "value", Namespace::Value).unwrap().id(),
        file.lookup(inner, "value", Namespace::Value).unwrap().id()
    );
}

#[test]
fn unsupported_subtrees_never_produce_false_complete_script_units() {
    let arena = Allocator::default();
    for source in [
        "const value = () => 1;",
        "const {value} = obj;",
        "function f() { function nested() {} }",
        "class C {}",
        "const value: number = 1;",
    ] {
        let parsed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(parsed.diagnostics().is_empty(), "{source}");
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
        assert!(!file.is_complete(), "{source}");
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax)
        );
        assert!(file.references().is_empty());
    }
}

#[test]
fn duplicate_container_indices_are_rejected_without_duplicate_bindings() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(parsed.admitted().unwrap(), block, 7).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    assert_eq!(
        producer
            .program(
                ProgramInput::checked(parsed.admitted().unwrap(), block, 7).unwrap(),
                ProgramScope::Module
            )
            .unwrap_err()
            .kind,
        FileIssueKind::DuplicateUnit
    );
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.units().len(), 1);
    assert_eq!(file.bindings().count(), 1);
    assert_eq!(file.issues()[0].span, Span::new(0, source.len() as u32));
}

#[test]
fn declaration_names_come_from_real_ast_semantics_with_authored_escape_ranges() {
    let arena = Allocator::default();
    let source = "const \\u0076alue = 1;";
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
    let binding = file.bindings().next().unwrap();
    assert!(file.is_complete());
    assert_eq!(binding.declaration().unwrap().name, "value");
    assert_eq!(binding.declaration().unwrap().span, Span::new(6, 16));
}

#[test]
fn declaration_profiles_outside_the_admitted_module_family_stay_incomplete() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    for source_type in [
        SourceType::cjs(),
        SourceType::ts()
            .with_module(true)
            .with_typescript_definition(true),
    ] {
        let parsed = Parser::new(&arena, source, source_type).parse_observed();
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
        assert_eq!(file.bindings().count(), 0);
        assert_eq!(file.issues()[0].kind, FileIssueKind::InvalidProfile);
    }
}
