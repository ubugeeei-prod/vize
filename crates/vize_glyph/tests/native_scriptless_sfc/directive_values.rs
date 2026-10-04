//! Embedded values require an authentic provider before complete whole-product success.

use super::{assert_fixed, options};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcOptions, NativeSfcRefusal, NativeTemplateRefusal,
    ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::SurfaceChild;

fn refused(source: &str, expected: Span, index: usize) {
    let arena = Allocator::default();
    let policy = NativeSfcOptions::default();
    let owner = observe_native_sfc_in(&arena, source, policy);
    let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
        index,
        refusal: NativeTemplateRefusal::DirectiveValue { span: expected },
    });
    assert!(core::ptr::eq(owner.source(), source));
    assert_eq!(owner.options(), policy);
    assert_eq!(owner.descriptor().options(), policy.descriptor);
    assert!(owner.descriptor().admitted().is_ok());
    assert!(owner.descriptor().issues().is_empty());
    assert!(owner.descriptor().container().errors.is_empty());
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.document().unwrap_err(), refusal);
        assert_eq!(owner.format().unwrap_err(), refusal);
    }
    assert!(owner.interpolation_failure().is_none());
    assert_eq!(owner.operands().len(), index);
    let selected = owner.selected().unwrap();
    assert!(core::ptr::eq(
        selected.component().block().root_source(),
        source
    ));
    assert_eq!(
        vize_l1::check_fidelity(&selected.component().carrier().tree),
        Ok(())
    );
    if index == 1 {
        let current = &owner.operands()[0];
        assert_eq!(current.raw_content(), "/*x*/1&#110;");
        assert_eq!(current.syntax().source().text(), "/*x*/1n");
        assert_eq!(
            current.syntax().comments().next().unwrap().text().unwrap(),
            "/*x*/"
        );
        assert!(current.syntax().source().decode_map().is_some());
        assert!(
            current
                .admitted_for(selected, selected.children().next().unwrap())
                .is_some()
        );
        assert_eq!(current.content_span().slice(source), current.raw_content());
    }
}

#[test]
fn original_bound_attribute_refuses_the_complete_sfc_at_its_exact_file_value_span() {
    let source = "<template><p :id=\"a+b\">{{1n}}</p></template>";
    let span = Span::new(18, 21);
    assert_eq!(span.slice(source), "a+b");
    refused(source, span, 0);
    // The original body interpolation remains present but is never observed after the attribute refusal.
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
    let element = owner
        .selected()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    let SurfaceChild::Interpolation(body) = element.children().next().unwrap().surface() else {
        panic!("original unvisited child")
    };
    assert_eq!(body.content.text, "1n");
    assert_eq!(
        element.surface().open.attrs[0]
            .value
            .as_ref()
            .unwrap()
            .content
            .text,
        "a+b"
    );
}

#[test]
fn every_original_valued_directive_refuses_and_late_failure_retains_the_complete_prefix() {
    for (head, value) in [
        (":id", "a+b"),
        ("@click", "f(...a)"),
        ("v-if", " a+ "),
        ("v-custom", "raw &amp; bytes"),
        ("#default", "{a}"),
        ("v-bind:[key]", "a&#43;b"),
        (":id", ""),
    ] {
        for prefix in ["", "{{/*x*/1&#110;}}"] {
            let source = format!(
                "<!--前--><template>{prefix}<p {head}=\"{value}\">{{{{later}}}}</p>{{{{tail}}}}</template><!--尾-->"
            );
            let start = source.find(&format!("\"{value}\"")).unwrap() + 1;
            let span = Span::new(start as u32, (start + value.len()) as u32);
            assert_eq!(span.slice(&source), value);
            refused(&source, span, usize::from(!prefix.is_empty()));
        }
    }
}

#[test]
fn ordinary_static_attribute_values_and_valueless_native_v_pre_keep_complete_output() {
    let source = "<template><p id=\"a+b &amp; c\" v-pre>{{raw}}</p>{{1n}}</template>";
    let expected = "<template><p id=\"a+b &amp; c\" v-pre>{{raw}}</p>{{ 1n }}</template>";
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    assert_eq!(owner.format().unwrap().code, expected);
    assert!(owner.format().unwrap().changed);
    assert_eq!(owner.operands().len(), 1);
    assert_eq!(owner.operands()[0].raw_content(), "1n");
    assert_fixed(source, policy);
}

#[test]
fn original_unquoted_value_refuses_with_exact_file_span_while_static_unquoted_bytes_stay_supported()
{
    let source = "<template><p :id=a&#43;b>{{1n}}</p></template>";
    let span = Span::new(17, 24);
    assert_eq!(span.slice(source), "a&#43;b");
    refused(source, span, 0);
    let source = "<template><p id=a&#43;b>{{1n}}</p></template>";
    let expected = "<template><p id=a&#43;b>{{ 1n }}</p></template>";
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::CrLf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    assert_eq!(owner.format().unwrap().code, expected);
    assert!(owner.format().unwrap().changed);
    assert_eq!(owner.operands().len(), 1);
    assert_fixed(source, policy);
}
