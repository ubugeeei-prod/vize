extern crate std;

use super::{NativeSelectedSfcIssueKind as Kind, lower_selected_sfc_native};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::vue::{DescriptorOptions, ScriptRole},
};
use vize_l2::{
    file::RejectedFileHandler,
    op::{BindingOp, OnOp, Op, Region},
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
fn on<'o, 'a>(region: &'o Region<'a>) -> &'o OnOp<'a> {
    for op in &region.ops {
        if let Op::Element(element) = op {
            for binding in &element.bindings {
                if let BindingOp::On(on) = binding {
                    return on;
                }
            }
            if !element.children.ops.is_empty() {
                return on(&element.children);
            }
        }
    }
    panic!("actual original On")
}

#[test]
fn whole_original_descriptor_and_handler_file_survive_movement_with_full_source() {
    let arena = Allocator::default();
    let source = "<!--whole--><template>\n<button @click=\"const unused=&quot;雪🌸&quot;; $event.count+=2;\">go</button>\n</template>";
    let observation = lower_selected_sfc_native(&arena, source, options());
    assert!(observation.issues().is_empty());
    let original = observation.template().unwrap();
    let stock = original
        .selected()
        .component()
        .carrier()
        .tree
        .children
        .as_ptr();
    let file = original.file().unwrap();
    let body = file
        .handler_for(on(file.artifact().root()))
        .unwrap()
        .resolution()
        .unwrap()
        .input()
        .body();
    let body_ptr = core::ptr::from_ref(body);
    let moved = core::hint::black_box(observation);
    let admitted = moved.admitted().unwrap();
    assert!(core::ptr::eq(admitted.observation(), &moved));
    let view = admitted.into_template_view();
    assert!(core::ptr::eq(view.owner(), moved.template().unwrap()));
    let file = view.file().unwrap();
    assert!(file.is_complete());
    assert!(file.units().is_empty());
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert_eq!(
        view.owner()
            .selected()
            .component()
            .carrier()
            .tree
            .children
            .as_ptr(),
        stock
    );
    assert_eq!(view.owner().selected().children().len(), 3);
    assert_eq!(file.artifact().root().ops.len(), 1);
    let original_on = on(file.artifact().root());
    let handler = file.handler_for(original_on).unwrap();
    let resolution = handler.resolution().unwrap();
    assert!(handler.accepts_on(original_on));
    assert_eq!(core::ptr::from_ref(resolution.input().body()), body_ptr);
    let syntax = resolution.input().operand().syntax();
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(
        syntax.source().text(),
        "const unused=\"雪🌸\"; $event.count+=2;"
    );
    assert_eq!(
        syntax.source().span().slice(source),
        "const unused=&quot;雪🌸&quot;; $event.count+=2;"
    );
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(resolution.references().len(), 1);
    let descriptor = moved.descriptor().admitted().unwrap();
    assert_eq!(
        view.owner().selected().template_index(),
        descriptor.template().unwrap().container_index()
    );
    assert_eq!(
        view.owner().selected().component().block().span(),
        descriptor.template().unwrap().block().span()
    );
}

#[test]
fn equal_text_foreign_descriptor_or_file_cannot_join_after_internal_mispairing() {
    let arena = Allocator::default();
    let first = vize_l0::String::from("<template><button @click='var unused=$event;'/></template>");
    let second = first.clone();
    let mut left = lower_selected_sfc_native(&arena, &first, options());
    let mut right = lower_selected_sfc_native(&arena, &second, options());
    assert!(left.admitted().is_some());
    assert!(right.admitted().is_some());
    core::mem::swap(&mut left.template, &mut right.template);
    assert!(left.admitted().is_none());
    assert!(right.admitted().is_none());
    assert!(left.issues().is_empty());
    assert!(right.issues().is_empty());
}

#[test]
fn every_script_role_and_style_sibling_is_refused_before_template_admission() {
    for (suffix, expected) in [
        ("<script></script>", Kind::Script(ScriptRole::Ordinary)),
        (
            "<script>/*empty*/</script>",
            Kind::Script(ScriptRole::Ordinary),
        ),
        (
            "<script lang=ts>export default {}</script>",
            Kind::Script(ScriptRole::Ordinary),
        ),
        ("<script setup></script>", Kind::Script(ScriptRole::Setup)),
        (
            "<script setup>const value=1;</script>",
            Kind::Script(ScriptRole::Setup),
        ),
        (
            "<script setup lang=ts>const value:number=1;</script>",
            Kind::Script(ScriptRole::Setup),
        ),
        ("<style></style>", Kind::Style),
        ("<style scoped>.go{color:red}</style>", Kind::Style),
        ("<style module>.go{}</style>", Kind::Style),
    ] {
        let arena = Allocator::default();
        let source =
            std::format!("<template><button @click='var unused=$event;'/></template>{suffix}");
        let observation = lower_selected_sfc_native(&arena, &source, options());
        assert!(observation.admitted().is_none());
        assert!(observation.template().is_none());
        assert_eq!(observation.issues().len(), 1);
        let issue = observation.issues()[0];
        assert_eq!(issue.kind, expected);
        assert_eq!(issue.container_index, Some(1));
        assert!(observation.descriptor().admitted().is_ok());
        assert_eq!(observation.descriptor().container().blocks.len(), 2);
        assert!(
            source
                .get(issue.span.start as usize..issue.span.end as usize)
                .is_some()
        );
        assert!(core::ptr::eq(
            observation.descriptor().source(),
            source.as_str()
        ));
    }
}

#[test]
fn unsupported_profiles_external_custom_and_malformed_envelopes_keep_descriptor_evidence() {
    for source in [
        "<template src='outside.vue'/>",
        "<template><button/></template><script src='outside.js'/>",
        "<template><button/></template><style src='outside.css'/>",
        "<template><button/></template><docs>custom</docs>",
        "<template><button/></template><template><p/></template>",
        "<template lang=pug>button</template>",
    ] {
        let arena = Allocator::default();
        let observation = lower_selected_sfc_native(&arena, source, options());
        assert!(observation.admitted().is_none());
        assert!(observation.template().is_none());
        assert_eq!(observation.issues()[0].kind, Kind::Descriptor);
        assert!(
            !observation.descriptor().issues().is_empty()
                || !observation.descriptor().container().errors.is_empty()
        );
        assert!(core::ptr::eq(observation.descriptor().source(), source));
    }
    for options in [
        DescriptorOptions {
            version: VueVersion::V2,
            ..options()
        },
        DescriptorOptions {
            dialect: VueDialect::PetiteVue,
            ..options()
        },
        DescriptorOptions {
            template: SurfaceParseOptions {
                experimental_in_tag_comments: true,
                ..SurfaceParseOptions::default()
            },
            ..options()
        },
    ] {
        let arena = Allocator::default();
        let observation =
            lower_selected_sfc_native(&arena, "<template><button/></template>", options);
        assert!(observation.admitted().is_none());
        assert_eq!(observation.descriptor().options(), options);
        assert_eq!(observation.issues()[0].kind, Kind::Descriptor);
    }
}

#[test]
fn original_refused_handler_syntax_and_late_header_inputs_remain_owned() {
    for (body, tail, syntax_refusal) in [
        ("return (", "", true),
        ("var unused=$event;", "v-if=true", false),
    ] {
        let arena = Allocator::default();
        let source = std::format!("<template><button @click='{body}' {tail}/></template>");
        let observation = lower_selected_sfc_native(&arena, &source, options());
        assert!(observation.admitted().is_none());
        assert!(matches!(observation.issues()[0].kind, Kind::Template(_)));
        let original = observation.template().unwrap();
        assert!(original.view().is_err());
        let file = original.file().unwrap();
        assert!(core::ptr::eq(file.artifact().source(), source.as_str()));
        if syntax_refusal {
            let [RejectedFileHandler::Syntax(rejected)] = file.rejected_handlers() else {
                panic!("whole retained handler syntax");
            };
            assert_eq!(rejected.operand().raw_value(), body);
            assert!(rejected.operand().syntax().hole().is_some());
        } else {
            let input = file.unattached_handlers().next().unwrap();
            assert_eq!(input.operand().raw_value(), body);
            assert!(input.operand().syntax().admitted_body().is_some());
        }
    }
}

#[test]
fn root_text_and_interpolation_use_original_visits_and_retained_input() {
    let arena = Allocator::default();
    let source = "<!--SFC--><template>\n{{ /*kept*/ '雪&amp;🌸' }}\n</template>";
    let observation = lower_selected_sfc_native(&arena, source, options());
    let view = observation.admitted().unwrap().into_template_view();
    let file = view.file().unwrap();
    assert_eq!(view.owner().selected().children().len(), 3);
    let [Op::Interpolation(interpolation)] = file.artifact().root().ops.as_slice() else {
        panic!("one genuine interpolation; original root blanks are omitted");
    };
    let [record] = file.native_interpolations() else {
        panic!("whole original input");
    };
    let syntax = record.input().operand().syntax();
    assert_eq!(record.input().operand().full_span(), interpolation.span);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap(),
        "/*kept*/"
    );
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(file.artifact().node_count(), 1);
}
