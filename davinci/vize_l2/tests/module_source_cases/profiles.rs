use super::{lower, observe};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::SurfaceParseOptions;
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{
    FileProducer, ModuleSourceErrorKind, ProgramInput, ProgramScope, SetupIssueKind, VueSetup,
};

#[test]
fn actual_js_ts_jsx_and_stock_parse_options_cannot_be_substituted() {
    let arena = Allocator::default();
    let source = "import 'dep';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 0);
    let ts = observe(&arena, block, SourceType::ts().with_module(true));
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(ts.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Profile)
    );
    let jsx = observe(&arena, block, SourceType::mjs().with_jsx(true));
    let jsx_file = lower(&arena, &jsx, block, 0);
    assert!(
        matches!(jsx_file.original_module_sources(ProgramInput::checked(jsx.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Profile)
    );
    for options in [
        ParseOptions {
            enable_ident_hashes: false,
            ..ParseOptions::default()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
    ] {
        let changed = Parser::new(&arena, source, SourceType::mjs())
            .with_options(options)
            .parse_observed();
        assert!(changed.diagnostics().is_empty());
        assert!(
            matches!(ProgramInput::checked(changed.admitted().unwrap(), block, 0), Err(error) if error.kind == FileIssueKind::InvalidProfile)
        );
    }
    for profile in [SourceType::unambiguous(), SourceType::ts()] {
        let inferred = observe(&arena, block, profile);
        assert!(
            inferred
                .admitted()
                .unwrap()
                .program()
                .source_type
                .is_module()
        );
        assert!(inferred.admitted().unwrap().source_type().is_unambiguous());
        assert!(
            matches!(ProgramInput::checked(inferred.admitted().unwrap(), block, 0), Err(error) if error.kind == FileIssueKind::InvalidProfile)
        );
    }
}

#[test]
fn source_attributes_remain_incomplete_and_recovered_programs_cannot_admit() {
    for source in [
        "import value from 'dep' with { type: 'json' };",
        "export * from 'dep' with { type: 'json' };",
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(!file.is_complete());
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax)
        );
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::IncompleteFile)
        );
    }
    let arena = Allocator::default();
    let recovered = Parser::new(&arena, "import value from ;", SourceType::mjs()).parse_observed();
    assert!(recovered.admitted().is_none());
    assert!(!recovered.diagnostics().is_empty());
}

#[test]
fn static_module_receipts_do_not_lift_the_existing_original_vue_setup_import_refusal() {
    let arena = Allocator::default();
    let source = "<script setup>import {value} from 'dep'; const count = 1;</script>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let script = admitted.setup().unwrap();
    let original = observe(&arena, script.block(), SourceType::mjs());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(
                original.admitted().unwrap(),
                script.block(),
                script.container_index(),
            )
            .unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    let mut count = 0;
    file.original_module_sources(
        ProgramInput::checked(
            original.admitted().unwrap(),
            script.block(),
            script.container_index(),
        )
        .unwrap(),
    )
    .unwrap()
    .for_each(|_| count += 1)
    .unwrap();
    assert_eq!(count, 1);
    assert!(
        matches!(VueSetup::checked(&file, script, original.admitted().unwrap()), Err(issue) if issue.kind == SetupIssueKind::UnsupportedSyntax)
    );
}
