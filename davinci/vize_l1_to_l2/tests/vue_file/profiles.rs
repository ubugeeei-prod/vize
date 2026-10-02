use super::support::{block, script};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::NativeComponent;
use vize_l1_to_l2::vue_file::{VueFileIssueKind, VueFileProducer};
use vize_l2::lang::js::ProgramInput;

#[test]
fn no_script_uses_vue_js_and_retains_the_original_component_and_expression() {
    let arena = Allocator::default();
    let source = "<p>{{ 1 }}</p>";
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let component =
        NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let native = component
        .construct_vue_file_in(&mut producer.template_region().unwrap())
        .unwrap();
    let native = native.produced();
    assert!(native.is_supported());
    assert_eq!(native.embeds.len(), 1);
    assert_eq!(native.embeds[0].syntax.grammar().lang, Lang::Js);
    let vue = producer.finish().unwrap();
    assert!(vue.ordinary().is_none());
    assert!(vue.setup().is_none());
    let table = vue
        .file()
        .expression(native.embeds[0].node.unwrap())
        .unwrap()
        .table()
        .unwrap();
    assert!(core::ptr::eq(
        table.expression().ast,
        native.embeds[0].syntax.expression().unwrap()
    ));
    assert!(core::ptr::eq(
        native.component.block().root_source(),
        source
    ));
}

#[test]
fn setup_receipt_selects_actual_js_or_ts_without_a_template_language_argument() {
    let arena = Allocator::default();
    let source = "é<script setup>const value = 1;</script><template><p>{{ value }}</p></template>";
    for lang in [Lang::Js, Lang::Ts] {
        let (syntax, script_block) = script(&arena, source, "const value = 1;", lang).unwrap();
        let input =
            ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 7).unwrap();
        assert_eq!(input.source_type(), syntax.source_type());
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer.setup(input).unwrap();
        let component =
            NativeComponent::parse_in(&arena, block(source, "<p>{{ value }}</p>").unwrap())
                .unwrap();
        let native = component
            .construct_vue_file_in(&mut producer.template_region().unwrap())
            .unwrap();
        let native = native.produced();
        assert!(native.is_supported());
        assert_eq!(native.embeds[0].syntax.grammar().lang, lang);
        let vue = producer.finish().unwrap();
        let receipt = vue.setup().unwrap();
        assert_eq!(receipt.source_type(), syntax.source_type());
        assert_eq!(receipt.span(), script_block.span());
        assert_eq!(receipt.unit().index(), 7);
        let resolution = vue
            .file()
            .expression(native.embeds[0].node.unwrap())
            .unwrap();
        assert_eq!(resolution.scope(), Some(receipt.scope()));
        assert!(core::ptr::eq(
            resolution.table().unwrap().expression().ast,
            native.embeds[0].syntax.expression().unwrap()
        ));
        assert_eq!(syntax.diagnostics().count(), 0);
    }
}

#[test]
fn compatible_combined_profiles_use_real_receipts_despite_reversed_authored_order() {
    let arena = Allocator::default();
    let source = "<script setup>const local = ordinary;</script><script>const ordinary = 1;</script><p>{{local}}</p>";
    for lang in [Lang::Js, Lang::Ts] {
        let (ordinary, ordinary_block) =
            script(&arena, source, "const ordinary = 1;", lang).unwrap();
        let (setup, setup_block) = script(&arena, source, "const local = ordinary;", lang).unwrap();
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer
            .ordinary(
                ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 1)
                    .unwrap(),
            )
            .unwrap();
        producer
            .setup(
                ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 0).unwrap(),
            )
            .unwrap();
        let component =
            NativeComponent::parse_in(&arena, block(source, "<p>{{local}}</p>").unwrap()).unwrap();
        let native = component
            .construct_vue_file_in(&mut producer.template_region().unwrap())
            .unwrap();
        let native = native.produced();
        assert_eq!(native.embeds[0].syntax.grammar().lang, lang);
        let vue = producer.finish().unwrap();
        assert_eq!(
            vue.ordinary().unwrap().source_type(),
            ordinary.source_type()
        );
        assert_eq!(vue.setup().unwrap().source_type(), setup.source_type());
        assert_eq!(vue.ordinary().unwrap().unit().index(), 1);
        assert_eq!(vue.setup().unwrap().unit().index(), 0);
    }
}

#[test]
fn conflicting_script_only_profiles_refuse_with_the_actual_setup_origin() {
    let arena = Allocator::default();
    let source = "<script setup>const local = 2;</script><script>const ordinary = 1;</script>";
    for (ordinary_lang, setup_lang) in [(Lang::Js, Lang::Ts), (Lang::Ts, Lang::Js)] {
        let (ordinary, ordinary_block) =
            script(&arena, source, "const ordinary = 1;", ordinary_lang).unwrap();
        let (setup, setup_block) = script(&arena, source, "const local = 2;", setup_lang).unwrap();
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer
            .ordinary(
                ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 1)
                    .unwrap(),
            )
            .unwrap();
        producer
            .setup(
                ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 0).unwrap(),
            )
            .unwrap();
        let rejected = producer.finish().unwrap_err();
        let receipt = rejected.setup().unwrap();
        assert_eq!(rejected.issues().len(), 1);
        let issue = rejected.issues()[0];
        assert_eq!(issue.kind, VueFileIssueKind::ConflictingScriptProfiles);
        assert_eq!(issue.unit, Some(receipt.unit()));
        assert_eq!(issue.scope, Some(receipt.scope()));
        assert_eq!(issue.span, setup_block.span());
        assert!(rejected.file().unwrap().is_complete());
        assert_eq!(rejected.file().unwrap().units().len(), 2);
        assert_eq!(rejected.file().unwrap().artifact().node_count(), 0);
        assert_eq!(
            ordinary.diagnostics().count() + setup.diagnostics().count(),
            0
        );
    }
}

#[test]
fn invalid_observed_profiles_refuse_template_and_script_only_finish_without_losing_units() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    for profile in [
        SourceType::cjs(),
        SourceType::jsx().with_module(true),
        SourceType::d_ts().with_module(true),
    ] {
        let observation = Parser::new(&arena, source, profile).parse_observed();
        let admitted = observation.admitted().unwrap();
        let original = admitted.program();
        let input =
            ProgramInput::checked(admitted, SourceRoot::new(source).unwrap().whole_block(), 9)
                .unwrap();
        assert_eq!(input.source_type(), profile);
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer.setup(input).unwrap();
        let error = producer.template_region().err().unwrap();
        assert_eq!(error.kind, VueFileIssueKind::InvalidTemplateProfile);
        assert_eq!(producer.template_region().err(), Some(error));
        let rejected = producer.finish().unwrap_err();
        let receipt = rejected.setup().unwrap();
        assert_eq!(receipt.source_type(), profile);
        assert_eq!(error.unit, Some(receipt.unit()));
        assert_eq!(error.scope, Some(receipt.scope()));
        assert_eq!(error.span, receipt.span());
        assert_eq!(rejected.issues(), &[error]);
        assert_eq!(rejected.file().unwrap().units().len(), 1);
        assert!(!rejected.file().unwrap().is_complete());
        assert!(core::ptr::eq(
            observation.admitted().unwrap().program(),
            original
        ));
        assert_eq!(observation.diagnostics().len(), 0);
        let mut script_only = VueFileProducer::new(&arena, source).unwrap();
        script_only
            .setup(
                ProgramInput::checked(
                    observation.admitted().unwrap(),
                    SourceRoot::new(source).unwrap().whole_block(),
                    9,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(script_only.finish().unwrap_err().issues(), &[error]);
    }
}

#[test]
fn ts_profile_reaches_actual_retained_typed_ast_without_claiming_typed_resolution() {
    let arena = Allocator::default();
    let source = "<script setup>const value = 1;</script><p>{{value as number}}</p>";
    let (syntax, script_block) = script(&arena, source, "const value = 1;", Lang::Ts).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap())
        .unwrap();
    let component =
        NativeComponent::parse_in(&arena, block(source, "<p>{{value as number}}</p>").unwrap())
            .unwrap();
    let native = component
        .construct_vue_file_in(&mut producer.template_region().unwrap())
        .unwrap();
    let native = native.produced();
    assert!(!native.is_supported());
    let expression_span = block(source, "value as number").unwrap().span();
    assert_eq!(native.holes.len(), 1);
    assert_eq!(
        native.holes[0].span,
        block(source, "{{value as number}}").unwrap().span()
    );
    assert_eq!(
        native.holes[0].kind,
        vize_l1_to_l2::native::NativeHoleKind::Construction(
            vize_l2::artifact::ArtifactError::InvalidSpan {
                node: None,
                span: expression_span,
            }
        )
    );
    assert_eq!(native.diagnostics.len(), 1);
    assert_eq!(native.embeds.len(), 1);
    assert!(native.embeds[0].node.is_none());
    assert_eq!(native.embeds[0].syntax.grammar().lang, Lang::Ts);
    assert!(matches!(
        native.embeds[0].syntax.expression(),
        Some(oxc_ast::ast::Expression::TSAsExpression(_))
    ));
    let rejected = producer.finish().unwrap_err();
    let file = rejected.file().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.artifact().node_count(), 1);
    assert_eq!(file.template_issues().len(), 1);
    assert!(file.template_issues()[0].node.is_none());
    assert_eq!(file.template_issues()[0].span, expression_span);
    assert_eq!(
        file.template_issues()[0].kind,
        vize_l2::file::FileIssueKind::UnsupportedSyntax
    );
    assert!(core::ptr::eq(
        native.component.block().root_source(),
        source
    ));
}

#[test]
fn intrinsic_entry_retains_foreign_equal_source_component_before_any_factory_mint() {
    let arena = Allocator::default();
    let source = "<p>{{1}}</p>";
    let foreign = source.to_owned();
    let component =
        NativeComponent::parse_in(&arena, SourceRoot::new(&foreign).unwrap().whole_block())
            .unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let rejected = component
        .construct_vue_file_in(&mut producer.template_region().unwrap())
        .unwrap_err();
    assert!(core::ptr::eq(
        rejected.component().block().root_source(),
        foreign.as_str()
    ));
    assert_eq!(
        rejected.error(),
        vize_l1_to_l2::native::NativeVueConstructionError::ForeignSource
    );
    let vue = producer.finish().unwrap_err();
    let file = vue.file().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.artifact().node_count(), 0);
    assert!(core::ptr::eq(file.artifact().source(), source));
}
