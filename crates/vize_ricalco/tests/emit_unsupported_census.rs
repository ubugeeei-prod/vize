//! P2-11 refusal census: every unsupported source fixture has a typed,
//! span-carrying reason, and the committed counts are stable.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use support::with_transformed_caps;
use vize_ricalco::{
    EmitError, LegacyCaps, UnsupportedReason as Reason, UnsupportedRefusal, emit_dom,
    emit_dom_source,
};
use vize_s0::Allocator;

struct Case {
    name: &'static str,
    source: &'static str,
    caps: LegacyCaps,
    reason: Reason,
}

const VUE3: LegacyCaps = LegacyCaps::VUE3;

const SOURCE_CASES: &[Case] = &[
    case(
        "array_builtin_slots",
        r#"<KeepAlive v-slots="slots"></KeepAlive>"#,
        VUE3,
        Reason::ArrayBuiltinCannotUseSlotObject,
    ),
    case(
        "bare_style_attr",
        r#"<div style :style="s"></div>"#,
        VUE3,
        Reason::BareStyleAttributeWithDynamicStyle,
    ),
    case(
        "bad_bind_name",
        r#"<div :[a.]="x"></div>"#,
        VUE3,
        Reason::BindNameNotJs,
    ),
    case(
        "bad_bind_value",
        r#"<div :id="a."></div>"#,
        VUE3,
        Reason::BindValueNotJs,
    ),
    case(
        "bad_custom_directive",
        r#"<div v-pin="a."></div>"#,
        VUE3,
        Reason::CustomDirectiveExprNotJs,
    ),
    case(
        "duplicate_class",
        r#"<div :class="a" :class="b"></div>"#,
        VUE3,
        Reason::DuplicateClassBinding,
    ),
    case(
        "duplicate_style",
        r#"<div :style="a" :style="b"></div>"#,
        VUE3,
        Reason::DuplicateStyleBinding,
    ),
    case(
        "bad_for_source",
        r#"<div v-for="item in list."></div>"#,
        VUE3,
        Reason::ForSourceNotJs,
    ),
    case(
        "bad_html_expression",
        r#"<div v-html="%"></div>"#,
        VUE3,
        Reason::HtmlExpressionNotJs,
    ),
    case(
        "bad_text_directive_expression",
        r#"<div v-text="%"></div>"#,
        VUE3,
        Reason::TextDirectiveExpressionNotJs,
    ),
    case(
        "bad_if_condition",
        r#"<div v-if="ok."></div>"#,
        VUE3,
        Reason::IfConditionNotJs,
    ),
    case(
        "bad_memo_expression",
        r#"<div v-memo="%"></div>"#,
        VUE3,
        Reason::MemoExpressionNotJs,
    ),
    case(
        "bad_model_argument",
        r#"<Foo v-model:[a.]="x" />"#,
        VUE3,
        Reason::ModelArgumentNotJs,
    ),
    case(
        "object_bind_mod",
        r#"<div v-bind.prop="obj"></div>"#,
        VUE3,
        Reason::ObjectBindHasModifiers,
    ),
    case(
        "object_on_bad_handler",
        r#"<div v-on="handlers."></div>"#,
        VUE3,
        Reason::ObjectOnHandlerNotJs,
    ),
    case(
        "object_on_mod",
        r#"<div v-on.once="handlers"></div>"#,
        VUE3,
        Reason::ObjectOnHasModifiers,
    ),
    case(
        "bad_on_handler",
        r#"<div @click="handler."></div>"#,
        VUE3,
        Reason::OnHandlerNotJs,
    ),
    case(
        "bad_on_name",
        r#"<div @[event.]="handler"></div>"#,
        VUE3,
        Reason::OnNameNotJs,
    ),
    case(
        "bad_show_expression",
        r#"<div v-show="%"></div>"#,
        VUE3,
        Reason::ShowExpressionNotJs,
    ),
    case(
        "slot_template_extra_binding",
        r#"<Foo><template #header v-pin>x</template></Foo>"#,
        VUE3,
        Reason::SlotDefaultShape,
    ),
    case(
        "underscore_slot",
        r#"<Foo><template #_>x</template></Foo>"#,
        VUE3,
        Reason::SlotNameUnderscore,
    ),
    case(
        "bad_outlet_name",
        r#"<slot :name="slot."></slot>"#,
        VUE3,
        Reason::SlotOutletNameNotJs,
    ),
    case(
        "outlet_event_prop",
        r#"<slot @click="handler"></slot>"#,
        VUE3,
        Reason::SlotOutletPropKind,
    ),
    case(
        "slots_spread_arg",
        r#"<Foo v-slots:foo="slots"></Foo>"#,
        VUE3,
        Reason::SlotsSpreadShape,
    ),
    case(
        "slots_spread_bad_value",
        r#"<Foo v-slots="slots."></Foo>"#,
        VUE3,
        Reason::SlotsSpreadValueNotJs,
    ),
    case(
        "bad_text_expression",
        r#"<div>{{ value. }}</div>"#,
        VUE3,
        Reason::TextExpressionNotEmittable,
    ),
];

const fn case(name: &'static str, source: &'static str, caps: LegacyCaps, reason: Reason) -> Case {
    Case {
        name,
        source,
        caps,
        reason,
    }
}

#[test]
fn committed_fixture_refusal_census_is_pinned() {
    let mut counts = BTreeMap::new();
    for fixture in SOURCE_CASES {
        let refusal = source_refusal(fixture);
        assert_eq!(
            refusal.reason, fixture.reason,
            "{} classified into the wrong bucket",
            fixture.name
        );
        assert_source_span(fixture, refusal);
        *counts.entry(refusal.reason.code()).or_insert(0u64) += 1;
    }

    assert_eq!(
        counts.into_iter().collect::<Vec<_>>(),
        vec![
            ("array_builtin_cannot_use_slot_object", 1),
            ("bare_style_attr_with_dynamic_style", 1),
            ("bind_name_not_js", 1),
            ("bind_value_not_js", 1),
            ("custom_directive_expr_not_js", 1),
            ("duplicate_class_binding", 1),
            ("duplicate_style_binding", 1),
            ("for_source_not_js", 1),
            ("html_expression_not_js", 1),
            ("if_condition_not_js", 1),
            ("memo_expression_not_js", 1),
            ("model_argument_not_js", 1),
            ("object_bind_has_modifiers", 1),
            ("object_on_handler_not_js", 1),
            ("object_on_has_modifiers", 1),
            ("on_handler_not_js", 1),
            ("on_name_not_js", 1),
            ("show_expression_not_js", 1),
            ("slot_default_shape", 1),
            ("slot_name_underscore", 1),
            ("slot_outlet_name_not_js", 1),
            ("slot_outlet_prop_kind", 1),
            ("slots_spread_shape", 1),
            ("slots_spread_value_not_js", 1),
            ("text_directive_expression_not_js", 1),
            ("text_expression_not_emittable", 1),
        ]
    );
}

#[test]
fn optional_hydrated_corpus_census_is_deterministic() {
    let Some(root) = std::env::var_os("VIZE_DAVINCI_UNSUPPORTED_CENSUS_CORPUS") else {
        return;
    };
    let root = PathBuf::from(root);
    let mut files = Vec::new();
    collect_vue_files(&root, &mut files);
    assert!(
        !files.is_empty(),
        "corpus root contains no .vue files: {root:?}"
    );

    let first = corpus_counts(&files);
    let second = corpus_counts(&files);
    assert_eq!(first, second, "corpus census must not depend on host order");
}

fn source_refusal(case: &Case) -> UnsupportedRefusal {
    with_transformed_caps(
        case.source,
        case.caps,
        |lowered, _, facts, _| match emit_dom(lowered, facts).expect_err(case.name) {
            EmitError::Diagnostics => panic!("{} produced diagnostics", case.name),
            EmitError::Unsupported(refusal) => refusal,
        },
    )
}

fn assert_source_span(case: &Case, refusal: UnsupportedRefusal) {
    let span = refusal
        .span
        .unwrap_or_else(|| panic!("{} did not carry an authored span", case.name));
    let start = span.start as usize;
    let end = span.end as usize;
    assert!(start <= end, "{} has an inverted span", case.name);
    assert!(
        end <= case.source.len(),
        "{} span exceeds source",
        case.name
    );
    assert!(
        case.source.is_char_boundary(start),
        "{} start splits UTF-8",
        case.name
    );
    assert!(
        case.source.is_char_boundary(end),
        "{} end splits UTF-8",
        case.name
    );
}

fn corpus_counts(files: &[PathBuf]) -> BTreeMap<&'static str, u64> {
    let mut counts = BTreeMap::new();
    for file in files {
        let source = fs::read_to_string(file).unwrap_or_else(|error| {
            panic!("failed to read corpus fixture {}: {error}", file.display())
        });
        let allocator = Allocator::new();
        if let Err(EmitError::Unsupported(refusal)) = emit_dom_source(&allocator, &source) {
            *counts.entry(refusal.reason.code()).or_insert(0) += 1;
        }
    }
    counts
}

fn collect_vue_files(path: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read corpus directory {path:?}: {error}"));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read corpus entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_vue_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            out.push(path);
        }
    }
    out.sort();
}
