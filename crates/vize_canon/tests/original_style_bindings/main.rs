//! Original CSS emission, maps, L2 expressions and compiler/checker consumers.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "compare the actual original fixtures and outputs"
)]
use oxc_span::{ContentEq, GetSpan};
use std::borrow::Cow;
use vize_atelier_sfc::{
    compile_sfc, parse_sfc,
    vite_plugin::{
        transform_css_vars_for_pipeline_with_context, transform_original_style_bindings,
    },
};
use vize_canon::virtual_ts::{StyleBindingInputError, StyleBindingReferences};
use vize_carton::FxHashSet;
use vize_croquis::sfc::__internal::extract_and_transform_v_bind_with_edits;
use vize_l0::Allocator;
use vize_l1::container::vue::{AdmittedDescriptor, Vue};
use vize_l1_to_l2::{lower::OriginalStyleBindings, lower_style_block_in};
use vize_l2::op::{BindingOp, Op};
mod input;
use input::{options, originals};

#[derive(Default, Debug)]
struct Totals {
    inputs: usize,
    invalid_utf8: usize,
    containers: usize,
    complete: usize,
    styles: usize,
    calls: usize,
    references: usize,
    edits: usize,
    compiled: usize,
    errors: usize,
    parser_errors: usize,
}

fn same_carriers(left: &Op<'_>, right: &Op<'_>) {
    let (Op::Element(a), Op::Element(b)) = (left, right) else {
        panic!("style carrier expected")
    };
    assert_eq!((a.tag, a.span, a.namespace), (b.tag, b.span, b.namespace));
    assert_eq!(a.attributes.len(), b.attributes.len());
    assert_eq!(a.children.ops.len(), b.children.ops.len());
    assert_eq!(a.bindings.len(), b.bindings.len());
    for (x, y) in a.bindings.iter().zip(&b.bindings) {
        let (BindingOp::VueCssBind(x), BindingOp::VueCssBind(y)) = (x, y) else {
            panic!("style call expected")
        };
        assert_eq!(
            (x.span, x.value.span(), x.value.source()),
            (y.span, y.value.span(), y.value.source())
        );
        assert_eq!(
            std::mem::discriminant(&x.value),
            std::mem::discriminant(&y.value)
        );
        match (x.value, y.value) {
            (vize_l2::expr::ExprRef::Opaque(a), vize_l2::expr::ExprRef::Opaque(b)) => {
                assert_eq!(a.reason, b.reason)
            }
            (vize_l2::expr::ExprRef::Js(a), vize_l2::expr::ExprRef::Js(b)) => {
                assert!(a.ast.content_eq(b.ast));
                assert_eq!(a.ast.span(), b.ast.span());
            }
            _ => {}
        }
    }
}

fn compare_style(style: &OriginalStyleBindings<'_>, scale: &mut Totals) {
    let block = style.block();
    for production in [None, Some("/original/project/花.vue")] {
        let id = "data-v-a73c19";
        let mut published = Vec::new();
        let result = transform_original_style_bindings(style, id, production, |edit| {
            published.push((edit.source, edit.generated))
        });
        assert_eq!(
            result,
            transform_css_vars_for_pipeline_with_context(block.source(), id, production)
        );
        let arena = vize_carton::pool::acquire();
        let mut expected = Vec::new();
        let (code, vars) = extract_and_transform_v_bind_with_edits(
            &arena,
            block.source(),
            Some(id),
            production.map(|_| "a73c19"),
            |edit| {
                expected.push((
                    vize_l0::Span::new(
                        block.start() + edit.source.start as u32,
                        block.start() + edit.source.end as u32,
                    ),
                    edit.generated,
                ));
            },
        );
        assert_eq!(published, expected);
        assert_eq!(result.as_str(), code);
        assert!(
            style
                .calls()
                .map(|c| c.value.source())
                .eq(vars.iter().map(|s| s.as_str()))
        );
        scale.edits += expected.len();
    }
}

fn compare<'a>(
    arena: &'a Allocator,
    source: &'a str,
    owner: Option<AdmittedDescriptor<'_, 'a>>,
    scale: &mut Totals,
) {
    let baseline = parse_sfc(source, Default::default());
    let mut styles = owner.map(|o| o.styles());
    let original = vize_l0::SourceRoot::new(source).unwrap();
    let before_styles = scale.styles;
    let prepared = StyleBindingReferences::prepare(arena, source, Default::default(), |style| {
        let expected = &baseline.as_ref().unwrap().styles[scale.styles - before_styles];
        let block = original
            .block(
                &source[expected.loc.start..expected.loc.end],
                expected.loc.start as u32,
            )
            .unwrap();
        assert_eq!(style.block(), block);
        if let Some(retained) = styles.as_mut() {
            assert_eq!(retained.next().unwrap().block(), block);
        }
        scale.styles += 1;
        scale.calls += style.calls().count();
        compare_style(&style, scale);
        same_carriers(&lower_style_block_in(arena, block), &style.into_op(arena));
    });
    let (baseline, prepared, facts) = match (baseline, prepared) {
        (Err(old), Err(StyleBindingInputError::Descriptor(new))) => {
            assert_eq!(
                serde_json::to_vec(&old).unwrap(),
                serde_json::to_vec(&new).unwrap()
            );
            assert_eq!(scale.styles, before_styles);
            scale.parser_errors += 1;
            return;
        }
        (Ok(old), Ok((new, facts))) => (old, new, facts),
        _ => panic!("compiler and shared file admission disagree: {source}"),
    };
    assert!(styles.is_none_or(|mut selected| selected.next().is_none()));
    assert_eq!(scale.styles - before_styles, baseline.styles.len());
    assert_eq!(
        serde_json::to_vec(&prepared).unwrap(),
        serde_json::to_vec(&baseline).unwrap(),
        "complete compiler descriptor for the original file"
    );
    assert!(
        facts
            .expressions()
            .iter()
            .copied()
            .eq(baseline.css_vars.iter().map(Cow::as_ref))
    );
    let names: FxHashSet<_> = baseline
        .css_vars
        .iter()
        .flat_map(|expr| vize_croquis::drawer::extract_identifiers_oxc(expr))
        .collect();
    assert_eq!(facts.names(), &names);
    scale.references += names.len();
    if !facts.expressions().is_empty() {
        let before = compile_sfc(&baseline, Default::default());
        let after = compile_sfc(&prepared, Default::default());
        assert_eq!(
            serde_json::to_vec(&before).unwrap(),
            serde_json::to_vec(&after).unwrap()
        );
        scale.compiled += usize::from(before.is_ok());
        scale.errors += usize::from(before.is_err());
    }
    scale.complete += 1;
}

#[test]
fn all_style_languages_keep_edits_expressions_and_whole_compilations() {
    for lang in ["css", "scss", "sass", "less", "stylus", "styl"] {
        let source = format!(
            r#"<script setup lang='ts'>const tone='r', count=1, hidden=12</script><template><i/></template>
<style lang='{lang}'>/* v-bind(fake) */ .猫 {{ color:v-bind(tone); width:v-bind("Math.min(count, 2) + 'px'"); content:'v-bind(ignored)'; background:v-bind(tone); }}</style>
<style lang='{lang}'>.a{{color:v-bind(tone);width:v-bind(not +);height:v-bind(unclosed}}</style>"#
        );
        let arena = Allocator::new();
        let desc = Vue.observe_descriptor(&arena, &source, options());
        let mut count = Totals::default();
        compare(&arena, &source, Some(desc.admitted().unwrap()), &mut count);
        assert_eq!((count.styles, count.calls, count.edits), (2, 5, 10));
        assert!(count.references >= 3);
    }
}

#[test]
fn retained_styles_publish_completed_calls_and_visit_empty_blocks() {
    for (source, has_calls) in [
        ("<template></template>", None),
        ("<template></template><style></style>", Some(false)),
        ("<template></template><style>v-bind(</style>", Some(false)),
        ("<template></template><style>v-bind()</style>", Some(true)),
        (
            "<template></template><style>v-bind(x);v-bind(</style>",
            Some(true),
        ),
    ] {
        let arena = Allocator::new();
        let owner = Vue.observe_descriptor(&arena, source, options());
        let mut visits = 0;
        let result = StyleBindingReferences::observe(&arena, owner.admitted().unwrap(), |style| {
            visits += 1;
            assert_eq!(Some(style.has_calls()), has_calls);
            assert_eq!(style.has_calls(), style.calls().next().is_some());
        });
        assert_eq!(result.is_some(), has_calls.is_some());
        assert_eq!(visits, usize::from(has_calls.is_some()));
    }
}

#[test]
fn filename_maps_and_parse_options_keep_the_complete_compiler_descriptor() {
    use vize_atelier_sfc::{PadOption, SfcParseOptions};
    let source = "<template>雪🌸</template>\r\n<style scoped module lang='scss'>i{color:v-bind(tone)}</style><style></style>";
    let arena = Allocator::new();
    let observed = Vue.observe_descriptor(&arena, source, options());
    assert!(observed.admitted().is_ok());
    for pad in [PadOption::None, PadOption::Line, PadOption::Space] {
        for ignore_empty in [false, true] {
            let compiler_options = SfcParseOptions {
                filename: "花🌸.vue".into(),
                source_map: true,
                pad,
                ignore_empty,
                template_parse_options: Some(Default::default()),
            };
            let expected = parse_sfc(source, compiler_options.clone()).unwrap();
            let (descriptor, refs) =
                StyleBindingReferences::prepare(&arena, source, compiler_options, |_| {}).unwrap();
            assert_eq!(
                serde_json::to_vec(&descriptor).unwrap(),
                serde_json::to_vec(&expected).unwrap()
            );
            assert_eq!(refs.expressions(), ["tone"]);
        }
    }
}

#[test]
#[ignore = "compare tracked and external original style products"]
fn original_vue_styles() {
    let mut counts = Totals::default();
    (counts.inputs, counts.invalid_utf8) = originals(|arena, source, owner, _| {
        counts.containers += usize::from(owner.is_none());
        compare(arena, source, owner, &mut counts);
    });
    assert_eq!(
        counts.inputs,
        counts.invalid_utf8 + counts.complete + counts.parser_errors
    );
    assert!(counts.styles > 0);
    eprintln!("original_style_bindings {counts:?}");
}
