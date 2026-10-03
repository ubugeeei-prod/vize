use super::support::{block, script};
use vize_l0::{Allocator, Vec};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::NativeComponent;
use vize_l1_to_l2::vue_file::{VueFileIssueKind, VueFileProducer};
use vize_l2::artifact::{ComponentBody, ComponentFactory};
use vize_l2::lang::js::ProgramInput;
use vize_l2::op::Namespace;

#[test]
fn original_js_and_ts_empty_default_complete_with_directives_comments_and_zero_bindings() {
    let arena = Allocator::default();
    for (content, lang, comments) in [
        ("export default {};", Lang::Js, 0),
        (
            "'use strict'; ; export /*間*/ default /*before*/ { /*inside*/ }; ;",
            Lang::Js,
            3,
        ),
        (
            "\r\n/*before*/'雪';\r\nexport default {}\r\n/*tail*/",
            Lang::Ts,
            2,
        ),
    ] {
        let source =
            format!("🌸<script>{content}</script><template><p>kept<!--tail--></p></template>");
        let (syntax, script_block) = script(&arena, &source, content, lang).unwrap();
        let original = syntax.program().unwrap();
        let mut producer = VueFileProducer::new(&arena, &source).unwrap();
        let unit = producer
            .ordinary(
                ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap(),
            )
            .unwrap();
        let component =
            NativeComponent::parse_in(&arena, block(&source, "<p>kept<!--tail--></p>").unwrap())
                .unwrap();
        let native = component
            .construct_vue_file_in(&mut producer.template_region().unwrap())
            .unwrap();
        assert!(native.produced().is_supported());
        let vue = producer.finish().unwrap();
        let file = vue.file();
        assert!(file.is_complete());
        assert_eq!(file.units().len(), 1);
        assert_eq!(vue.ordinary().unwrap().unit(), unit);
        assert_eq!(vue.ordinary().unwrap().span(), script_block.span());
        assert_eq!(
            vue.ordinary().unwrap().profile().typescript,
            lang == Lang::Ts
        );
        assert!(vue.setup().is_none());
        assert!(vue.declarations().is_empty());
        assert_eq!(file.bindings().count(), 0);
        assert!(file.imports().is_empty());
        assert!(file.references().is_empty());
        assert_eq!(file.exports().len(), 1);
        assert_eq!(file.exports()[0].name.as_str(), "default");
        assert_eq!(file.exports()[0].unit, unit);
        assert!(file.ordinary_empty_script().is_some());
        assert_eq!(file.artifact().node_count(), 3);
        assert!(core::ptr::eq(file.artifact().source(), source.as_str()));
        assert!(core::ptr::eq(
            file.artifact().source(),
            native.produced().component.block().root_source()
        ));
        assert!(core::ptr::eq(syntax.program().unwrap(), original));
        assert_eq!(syntax.comments().count(), comments);
        assert_eq!(syntax.diagnostics().count(), 0);
    }
}

#[test]
fn similar_options_families_keep_the_original_staged_issue_and_never_start_template() {
    let arena = Allocator::default();
    for (content, lang, defaults) in [
        ("export default {name: 'options'};", Lang::Js, 1),
        ("export default ({});", Lang::Js, 1),
        ("export default {} as const;", Lang::Ts, 1),
        ("export default {} satisfies object;", Lang::Ts, 1),
        ("import 'dep'; export default {};", Lang::Js, 1),
        ("export default {}; export default {};", Lang::Ts, 2),
        ("export default {}; export {};", Lang::Js, 1),
        ("; 'post'; export default {};", Lang::Js, 1),
        ("const local=1; export default {};", Lang::Js, 1),
    ] {
        let source = format!("<script>{content}</script><template>kept</template>");
        let (syntax, script_block) = script(&arena, &source, content, lang).unwrap();
        assert_eq!(syntax.diagnostics().count(), 0, "{content}");
        let mut producer = VueFileProducer::new(&arena, &source).unwrap();
        let unit = producer
            .ordinary(
                ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap(),
            )
            .unwrap();
        assert_eq!(
            producer.template_region().err().unwrap().kind,
            VueFileIssueKind::PreviousIssues,
            "{content}"
        );
        let rejected = producer.finish().unwrap_err();
        let staged = rejected
            .issues()
            .iter()
            .filter(|issue| issue.kind == VueFileIssueKind::UnsupportedOptions)
            .collect::<std::vec::Vec<_>>();
        assert_eq!(staged.len(), defaults, "{content}");
        for issue in staged {
            assert_eq!(issue.unit, Some(unit));
            assert_eq!(issue.scope, Some(rejected.ordinary().unwrap().scope()));
            assert!(
                source[issue.span.start as usize..issue.span.end as usize]
                    .starts_with("export default")
            );
        }
        let file = rejected.file().unwrap();
        assert!(file.ordinary_empty_script().is_none());
        assert_eq!(file.artifact().node_count(), 0);
        assert_eq!(file.units().len(), 1);
        assert!(core::ptr::eq(file.artifact().source(), source.as_str()));
    }
}

#[test]
fn mixed_setup_never_resolves_the_ordinary_empty_default_options_issue() {
    let arena = Allocator::default();
    let source = "<script>export default {};</script><script setup>const value=1;</script><template>kept</template>";
    let (ordinary, ordinary_block) =
        script(&arena, source, "export default {};", Lang::Js).unwrap();
    let (setup, setup_block) = script(&arena, source, "const value=1;", Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let unit = producer
        .ordinary(
            ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 0).unwrap(),
        )
        .unwrap();
    producer
        .setup(ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 1).unwrap())
        .unwrap();
    assert_eq!(
        producer.template_region().err().unwrap().kind,
        VueFileIssueKind::PreviousIssues
    );
    let rejected = producer.finish().unwrap_err();
    let issue = rejected
        .issues()
        .iter()
        .find(|issue| issue.kind == VueFileIssueKind::UnsupportedOptions)
        .unwrap();
    assert_eq!(issue.unit, Some(unit));
    assert!(rejected.setup().is_some());
    let file = rejected.file().unwrap();
    assert!(file.is_complete());
    assert_eq!(file.units().len(), 2);
    assert_eq!(file.bindings().count(), 1);
    assert!(file.ordinary_empty_script().is_none());
    assert_eq!(file.artifact().node_count(), 0);
}

struct Unwind;
impl<'a> ComponentBody<'a> for Unwind {
    fn run<R: ComponentFactory<'a>>(self, _: &mut R, _: vize_l0::id::NodeId) {
        std::panic::resume_unwind(Box::new("ordinary template interruption"));
    }
}

#[test]
fn template_interruption_keeps_the_staged_options_issue_and_all_original_owners() {
    let arena = Allocator::default();
    let source =
        "<script>/*original*/ export default {};</script><template><div>kept</div></template>";
    let (syntax, script_block) =
        script(&arena, source, "/*original*/ export default {};", Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let unit = producer
        .ordinary(
            ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap(),
        )
        .unwrap();
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer
            .template_region()
            .unwrap()
            .element(
                "div",
                Namespace::Html,
                Vec::new_in(&&arena),
                block(source, "<div>kept</div>").unwrap().span(),
                Unwind,
            )
            .unwrap();
    }));
    assert!(interrupted.is_err());
    let rejected = producer.finish().unwrap_err();
    let issue = rejected
        .issues()
        .iter()
        .find(|issue| issue.kind == VueFileIssueKind::UnsupportedOptions)
        .unwrap();
    assert_eq!(issue.unit, Some(unit));
    assert_eq!(
        &source[issue.span.start as usize..issue.span.end as usize],
        "export default {};"
    );
    let partial = rejected.rejected_file().unwrap();
    assert_eq!(partial.units().len(), 1);
    assert!(partial.units()[0].interruption().is_none());
    assert!(partial.declarations().is_empty());
    assert_eq!(partial.artifact().parts.root.ops.len(), 1);
    assert!(core::ptr::eq(partial.source(), source));
    assert_eq!(syntax.comments().count(), 1);
}
