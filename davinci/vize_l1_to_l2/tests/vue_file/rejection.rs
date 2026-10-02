use super::support::{construct, script, whole_span};
use vize_l0::{Allocator, Vec};
use vize_l1::embed::Lang;
use vize_l1_to_l2::vue_file::{VueFileIssueKind, VueFileProducer};
use vize_l2::artifact::{ComponentBody, ComponentFactory};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l2::op::Namespace;

#[test]
fn ordinary_only_local_is_not_template_context_and_failure_retains_original_owners() {
    let arena = Allocator::default();
    let source = "<script>const local = 1;</script><template>{{local}}</template>";
    let (syntax, block) = script(&arena, source, "const local = 1;", Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .ordinary(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let native = construct(&arena, source, "{{local}}", &mut producer).unwrap();
    assert!(!native.is_supported());
    assert_eq!(native.embeds.len(), 1);
    assert!(native.embeds.first().unwrap().node.is_none());
    let rejected = producer.finish().unwrap_err();
    let file = rejected.file().unwrap();
    assert!(!file.is_complete());
    assert_eq!(
        file.template_issues().first().unwrap().kind,
        FileIssueKind::UnresolvedReference
    );
    assert_eq!(file.bindings().count(), 1);
    assert!(core::ptr::eq(
        file.artifact().source(),
        native.component.block().root_source()
    ));
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn genuine_call_refusal_rolls_back_whole_reference_batch_while_neutral_calls_stay_supported() {
    let arena = Allocator::default();
    let source =
        "import {fn} from 'dep'; const accepted = 1; const value = accepted + fn(() => missing);";
    let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
    let mut vue = VueFileProducer::new(&arena, source).unwrap();
    vue.setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    assert!(vue.template_region().is_err());
    let rejected = vue.finish().unwrap_err();
    assert!(rejected.issues().iter().any(|issue| matches!(
        issue.kind,
        VueFileIssueKind::UnsupportedCall { optional: false }
    ) && issue.unit.is_some()
        && issue.scope.is_some()
        && source.get(issue.span.start as usize..issue.span.end as usize)
            == Some("fn(() => missing)")));
    assert_eq!(rejected.file().unwrap().references().len(), 0);
    assert_eq!(rejected.file().unwrap().bindings().count(), 3);
    let positive = "import {fn} from 'dep'; const value = fn(1);";
    let (syntax, block) = script(&arena, positive, positive, Lang::Js).unwrap();
    let mut neutral = FileProducer::new(&arena, positive).unwrap();
    neutral
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = neutral.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(file.references().len(), 1);

    // Template calls keep syntax references; this grants no runtime access kind.
    let source = "<script setup>import {fn} from 'dep'; const value = 1;</script><template>{{fn(value)}}</template>";
    let (syntax, block) = script(
        &arena,
        source,
        "import {fn} from 'dep'; const value = 1;",
        Lang::Js,
    )
    .unwrap();
    let mut vue = VueFileProducer::new(&arena, source).unwrap();
    vue.setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let native = construct(&arena, source, "{{fn(value)}}", &mut vue).unwrap();
    assert!(native.is_supported());
    let vue = vue.finish().unwrap();
    let embed = native.embeds.first().unwrap();
    let resolution = vue.file().expression(embed.node.unwrap()).unwrap();
    assert_eq!(resolution.scope(), Some(vue.setup().unwrap().scope()));
    let table = resolution.table().unwrap();
    assert!(core::ptr::eq(
        table.expression().ast,
        embed.syntax.expression().unwrap()
    ));
    assert_eq!(
        table
            .occurrences()
            .iter()
            .map(|occurrence| occurrence.name)
            .collect::<std::vec::Vec<_>>(),
        ["fn", "value"]
    );
    for occurrence in table.occurrences() {
        assert!(
            vue.exposure(resolution.binding(occurrence.binding).unwrap())
                .is_some()
        );
    }
}

#[test]
fn actual_setup_exports_options_and_macros_remain_typed_unfinished() {
    let arena = Allocator::default();
    for (source, setup, expected) in [
        (
            "export const value = 1;",
            true,
            VueFileIssueKind::SetupExport,
        ),
        (
            "export default {};",
            false,
            VueFileIssueKind::UnsupportedOptions,
        ),
        (
            "const props = defineProps({});",
            true,
            VueFileIssueKind::UnsupportedCall { optional: false },
        ),
    ] {
        let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        let input = ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap();
        if setup {
            producer.setup(input).unwrap();
        } else {
            producer.ordinary(input).unwrap();
        }
        let rejected = producer.finish().unwrap_err();
        assert!(rejected.issues().iter().any(|issue| issue.kind == expected));
        assert!(core::ptr::eq(
            rejected.file().unwrap().artifact().source(),
            source
        ));
    }
}

#[test]
fn component_node_is_retained_but_tag_exposure_cannot_be_claimed_complete() {
    let arena = Allocator::default();
    let source = "<MyComponent><span>kept</span></MyComponent>";
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer).unwrap();
    assert!(native.is_supported());
    let rejected = producer.finish().unwrap_err();
    let file = rejected.file().unwrap();
    assert_eq!(file.artifact().node_count(), 3);
    let issue = file.template_issues().first().unwrap();
    assert_eq!(issue.kind, FileIssueKind::UnsupportedComponent);
    assert!(issue.node.is_some());
}

struct Unwind;
impl<'a> ComponentBody<'a> for Unwind {
    fn run<R: ComponentFactory<'a>>(self, _: &mut R, _: vize_l0::id::NodeId) {
        std::panic::resume_unwind(Box::new("kept owner"));
    }
}
#[test]
fn actual_callback_unwind_retains_canonical_owner_and_private_file_declarations() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut region = producer.template_region().unwrap();
        region
            .element(
                "div",
                Namespace::Html,
                Vec::new_in(&&arena),
                whole_span(source),
                Unwind,
            )
            .unwrap();
    }));
    assert!(result.is_err());
    let rejected = producer.finish().unwrap_err();
    let partial = rejected.rejected_file().unwrap();
    assert_eq!(partial.declarations().len(), 1);
    assert_eq!(partial.units().len(), 1);
    assert!(core::ptr::eq(partial.source(), source));
    assert_eq!(partial.artifact().parts.root.ops.len(), 1);
}
