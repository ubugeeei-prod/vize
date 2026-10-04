mod setup;

use super::{VaporPart, VaporUnsupported, build_vapor_file_decisions};
use crate::decision::{DecisionBuildError, build_decisions, policy::TargetPolicy};
use alloc::format;
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::{
    artifact::ComponentFactory,
    file::{Declaration, TemplatePolicy, TemplateScope},
    lang::js::FileProducer,
    op::Op,
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn genuine_sfc_file_retains_original_order_nodes_and_target_isolation() {
    let arena = Allocator::default();
    let source = "\r\n<!--雪🌸--><template><!--x--><div id=\"a&amp;b\"><span>hello world</span><br></div></template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert_eq!(observed.issues(), []);
    let native = observed.admitted().unwrap();
    let file = native.file().file();
    let analysis = build_vapor_file_decisions(file).unwrap();
    assert!(core::ptr::eq(analysis.file(), file));
    assert!(core::ptr::eq(analysis.artifact(), file.artifact()));
    assert!(core::ptr::eq(analysis.artifact().source(), source));
    assert_eq!(
        analysis.tables().nodes.len(),
        file.artifact().node_count() as usize
    );
    let facts = analysis.vapor().unwrap();
    assert_eq!(facts.unsupported(), []);
    assert_eq!(facts.roots().len(), 2);
    let root = &facts.roots()[1];
    assert_eq!(facts.inherit_attrs(), Some(root.node()));
    let parts = facts.parts(root).unwrap();
    assert_eq!(parts.len(), 6);
    let [Op::Comment(_), Op::Element(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual original roots")
    };
    let VaporPart::Open { element, void, .. } = parts[0] else {
        panic!("original opening")
    };
    assert!(core::ptr::eq(element, original.as_ref()));
    assert!(!void);
    assert_eq!(element.attributes[0].value, Some("a&b"));
    assert!(matches!(parts[4], VaporPart::Open { void: true, .. }));
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr] {
        assert!(
            build_decisions(file.artifact(), policy)
                .unwrap()
                .vapor()
                .is_none()
        );
    }
}

#[test]
fn root_ranges_reject_equal_foreign_owners_and_text_prevents_fallthrough() {
    let arena = Allocator::default();
    let source = "<template>a<!--x--><span>b</span></template>";
    let first = lower_sfc_native(&arena, source, options());
    let second = lower_sfc_native(&arena, source, options());
    let first_native = first.admitted().unwrap();
    let second_native = second.admitted().unwrap();
    let first_analysis = build_vapor_file_decisions(first_native.file().file()).unwrap();
    let second_analysis = build_vapor_file_decisions(second_native.file().file()).unwrap();
    let facts = first_analysis.vapor().unwrap();
    assert_eq!(facts.inherit_attrs(), None);
    assert_eq!(facts.roots().len(), 3);
    assert_eq!(facts.parts(&facts.roots()[0]).unwrap().len(), 1);
    assert_eq!(facts.parts(&facts.roots()[1]).unwrap().len(), 1);
    assert_eq!(facts.parts(&facts.roots()[2]).unwrap().len(), 3);
    assert!(
        facts
            .parts(&second_analysis.vapor().unwrap().roots()[0])
            .is_none()
    );
}

#[test]
fn actual_whitespace_headers_tags_and_dynamic_ops_refuse_before_output() {
    for (template, reason) in [
        ("<span>a  b</span>", VaporUnsupported::TextNormalization),
        ("<span> a</span>", VaporUnsupported::TextNormalization),
        ("<span>a\nb</span>", VaporUnsupported::TextNormalization),
        ("<p>x</p>", VaporUnsupported::ElementSemantics),
        (
            "<span class=\"x\">ok</span>",
            VaporUnsupported::AttributeSemantics,
        ),
        ("<span :title=\"'x'\">ok</span>", VaporUnsupported::Binding),
        ("<span>{{1}}</span>", VaporUnsupported::Operation),
    ] {
        let arena = Allocator::default();
        let source = format!("<template>{template}</template>");
        let observed = lower_sfc_native(&arena, &source, options());
        assert_eq!(observed.issues(), [], "{template}");
        let native = observed.admitted().unwrap();
        let analysis = build_vapor_file_decisions(native.file().file()).unwrap();
        let rejected = analysis.vapor().unwrap().unsupported();
        assert_eq!(rejected.len(), 1, "{template}");
        assert_eq!(rejected[0].reason, reason);
        assert!(
            source
                .get(rejected[0].span.start as usize..rejected[0].span.end as usize)
                .is_some()
        );
    }
}

#[test]
fn interrupted_file_fails_before_the_shared_walk() {
    #[derive(Clone, Copy)]
    struct Hidden;
    impl TemplatePolicy for Hidden {
        fn visible(self, _: &Declaration) -> bool {
            false
        }
    }
    let arena = Allocator::default();
    let mut producer = FileProducer::new(&arena, "hello").unwrap();
    {
        let mut region = producer
            .template_region(TemplateScope::Root, Hidden)
            .unwrap();
        let mut walk = region.walk(Span::new(0, 5)).unwrap();
        walk.text("hello", Span::new(0, 5)).unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(matches!(
        build_vapor_file_decisions(&file),
        Err(DecisionBuildError::IncompleteFile)
    ));
    assert!(file.template_interruption().is_some());
}

#[test]
fn original_selected_completion_is_the_sole_target_capability() {
    use super::build_native_vapor_file_decisions;
    use vize_l1::{container::Vue, markup::NativeTemplateComponent};
    use vize_l2::lang::js::NativeTemplateOwner;
    let arena = Allocator::default();
    let source = "<template><!--original--><div><span>雪🌸</span><br></div></template>";
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected)
        .map_err(|_| "original owner")
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    let output = owner.finish();
    let analysis = build_native_vapor_file_decisions(output.view().unwrap()).unwrap();
    assert!(core::ptr::eq(analysis.owner(), &output));
    assert!(core::ptr::eq(analysis.file(), output.file().unwrap()));
    assert!(core::ptr::eq(analysis.artifact().source(), source));
    assert_eq!(analysis.tables().nodes.len(), 5);
    assert_eq!(analysis.vapor().unwrap().unsupported(), []);
    assert_eq!(analysis.vapor().unwrap().roots().len(), 2);
}
