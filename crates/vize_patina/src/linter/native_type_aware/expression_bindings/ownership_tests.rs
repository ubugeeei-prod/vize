//! Exact ownership controls independent of a checker runtime.

use std::ops::Range;

use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping, VizeSubSpan};
use vize_l0::String;

use super::{TypeAwareDocument, bind_template_expressions, inferred_binding_offset};

fn mapped_document(text: &str, expressions: &[(&str, usize)]) -> TypeAwareDocument {
    TypeAwareDocument {
        content: String::from(text),
        mapping: ProjectionMapping::from_spans(
            expressions
                .iter()
                .map(|(expression, authored)| {
                    let generated = text.find(expression).expect("complete expression");
                    VizeMapping::new(
                        generated..generated + expression.len(),
                        *authored..*authored + expression.len(),
                    )
                })
                .collect(),
        ),
    }
}

fn assert_owner(document: &TypeAwareDocument, authored: Range<usize>, identifier: &str) {
    let offset = inferred_binding_offset(document, &authored).expect("owned inferred binding");
    let start = offset as usize + 1 - identifier.len();
    assert_eq!(
        document.content.get(start..offset as usize + 1),
        Some(identifier)
    );
}

#[test]
fn full_multiline_wrappers_preserve_commas_and_mapping_after_later_edits() {
    let expression = "flag\n  ? t(\"a\")\n  : t(\"b\", { name })";
    let original = format!("void ({expression}); // VBind\nvoid ((left, {{ right }})); // VBind\n");
    let mut document = mapped_document(&original, &[(expression, 30), ("(left, { right })", 100)]);
    bind_template_expressions(&mut document);
    assert_eq!(
        document.content.as_str(),
        format!(
            "const __expr_0 = ({expression}); // VBind\nconst __expr_1 = ((left, {{ right }})); // VBind\n"
        )
    );
    assert_owner(&document, 30..30 + expression.len(), "__expr_0");
    assert_owner(&document, 100..117, "__expr_1");
    assert_eq!(inferred_binding_offset(&document, &(30..34)), None);
    let prefix = "import type { Value } from './value';\n";
    document
        .mapping
        .note_generated_replacement(0, 0, prefix.len());
    document.content.insert_str(0, prefix);
    assert_owner(&document, 30..30 + expression.len(), "__expr_0");
    assert_owner(&document, 100..117, "__expr_1");
}

#[test]
fn checked_multiline_initializer_owns_only_the_complete_value() {
    let expression = "\n  `safe ${name}`\n";
    let original = format!("const __vize_prop_check_0: unknown = ({expression});\n");
    let mut document = mapped_document(&original, &[(expression, 40), ("name", 50)]);
    bind_template_expressions(&mut document);
    assert_eq!(
        document.content.as_str(),
        format!("const __expr_0 = ({expression});\n{original}")
    );
    assert_owner(&document, 40..40 + expression.len(), "__expr_0");
    assert_eq!(inferred_binding_offset(&document, &(50..54)), None);
    // A synthetic annotated name cannot become the expression's inferred owner.
    let mut anchor = mapped_document(&original, &[("__vize_prop_check_0", 40)]);
    bind_template_expressions(&mut anchor);
    assert_eq!(anchor.content.as_str(), original);
    assert_eq!(inferred_binding_offset(&anchor, &(40..59)), None);
}

#[test]
fn condition_alias_stays_in_the_same_scope_and_keeps_the_original_guard() {
    let expression = "predicate({ value: item.value })";
    for guard in [
        expression.to_string(),
        format!("({expression})"),
        format!("(({expression}))"),
    ] {
        let original = format!(
            "for (const item of items) {{\n  if ({guard}) {{\n    consume(item.value);\n  }}\n}}\n"
        );
        let mut document = mapped_document(&original, &[(expression, 12)]);
        bind_template_expressions(&mut document);
        assert_eq!(
            document.content.as_str(),
            format!(
                "for (const item of items) {{\n  const __expr_0 = ({expression});\n  if ({guard}) {{\n    consume(item.value);\n  }}\n}}\n"
            )
        );
        assert_owner(&document, 12..12 + expression.len(), "__expr_0");
    }
    // An else-if or later guard term cannot be preceded by a declaration;
    // its probe stays in place and the guard still narrows on the original.
    let probe = format!("(((__expr_0) => __expr_0)({expression}), {expression})");
    for (before, after) in [("} else if (", ") {\n"), ("if (parent && (", ")) {\n")] {
        let original = format!("{before}{expression}{after}");
        let mut document = mapped_document(&original, &[(expression, 12)]);
        bind_template_expressions(&mut document);
        assert_eq!(document.content.as_str(), format!("{before}{probe}{after}"));
        assert_owner(&document, 12..12 + expression.len(), "__expr_0");
        // Both the probe copy and the original condition keep exact rows.
        let mapped: Vec<_> = document
            .mapping
            .rows()
            .filter(|row| row.span.src_range == (12..12 + expression.len()))
            .filter_map(|row| document.content.get(row.span.gen_range.clone()))
            .collect();
        assert_eq!(mapped.iter().filter(|text| **text == expression).count(), 2);
    }
}

#[test]
fn handler_uses_the_complete_owned_span_and_preserves_inert_markers() {
    let inert = "const marker = 'void (foreign); // VBind __vize_cb)((foreign))';\n";
    let expression = "handle(\n  nested(() => ({ value: item.value }))\n)";
    let original = format!(
        "{inert}  const __vize_handler_0 = ((__vize_cb: unknown) => __vize_cb)(({expression}));\n"
    );
    let mut document = mapped_document(&original, &[(expression, 8)]);
    bind_template_expressions(&mut document);
    assert_eq!(
        document.content.as_str(),
        format!(
            "{inert}  const __expr_0 = ({expression});\n  const __vize_handler_0 = ((__vize_cb: unknown) => __vize_cb)(({expression}));\n"
        )
    );
    assert_owner(&document, 8..8 + expression.len(), "__expr_0");
    let mut unmapped = mapped_document(inert, &[]);
    bind_template_expressions(&mut unmapped);
    assert_eq!(unmapped.content.as_str(), inert);
    assert_eq!(unmapped.mapping.rows().count(), 0);
}

#[test]
fn synthetic_callback_subspan_retains_its_own_inferred_type() {
    let expression = "unsafeHandler";
    let original = "  const __vize_handler_0 = ((__vize_cb: (() => unknown) | null | undefined) => __vize_cb)((unsafeHandler));\n";
    let expr_at = original.find(expression).expect("owned value");
    let name_at = original
        .find("__vize_handler_0")
        .expect("synthetic receiver");
    let mut document = TypeAwareDocument {
        content: String::from(original),
        mapping: ProjectionMapping::from_spans(vec![VizeMapping {
            gen_range: 0..original.len(),
            src_range: 50..63,
            sub_spans: vec![
                VizeSubSpan {
                    gen_range: name_at..name_at + "__vize_handler_0".len(),
                    src_range: 40..45,
                },
                VizeSubSpan {
                    gen_range: expr_at..expr_at + expression.len(),
                    src_range: 50..63,
                },
            ],
        }]),
    };
    bind_template_expressions(&mut document);
    assert_eq!(
        document.content.as_str(),
        format!("  const __expr_0 = (unsafeHandler);\n{original}")
    );
    assert_owner(&document, 50..63, "__expr_0");
    assert_eq!(inferred_binding_offset(&document, &(40..45)), None);
    let generated = document
        .mapping
        .rows()
        .next()
        .expect("original diagnostic receiver");
    assert_eq!(generated.span.sub_spans[1].src_range, 50..63);
    assert_eq!(
        document
            .content
            .get(generated.span.sub_spans[1].gen_range.clone()),
        Some(expression)
    );
}

#[test]
fn actual_annotated_listener_initializer_owns_its_type_and_name_range() {
    // Canon scope/event_handler.rs113–145 uses this direct annotated const;
    // its final row216–243 carries separate receiver-name and value sub-spans.
    let expression = "unsafeHandler";
    let original = "  const __vize_handler_1: Listener | null | undefined = (unsafeHandler);\n";
    let expr_at = original.find(expression).expect("owned value");
    let name_at = original
        .find("__vize_handler_1")
        .expect("synthetic receiver");
    let mut document = TypeAwareDocument {
        content: String::from(original),
        mapping: ProjectionMapping::from_spans(vec![VizeMapping {
            gen_range: 0..original.len(),
            src_range: 50..63,
            sub_spans: vec![
                VizeSubSpan {
                    gen_range: name_at..name_at + "__vize_handler_1".len(),
                    src_range: 40..45,
                },
                VizeSubSpan {
                    gen_range: expr_at..expr_at + expression.len(),
                    src_range: 50..63,
                },
            ],
        }]),
    };
    bind_template_expressions(&mut document);
    assert_eq!(
        document.content.as_str(),
        format!("  const __expr_0 = (unsafeHandler);\n{original}")
    );
    assert_owner(&document, 50..63, "__expr_0");
    assert_eq!(inferred_binding_offset(&document, &(40..45)), None);
    let receiver = document
        .mapping
        .rows()
        .next()
        .expect("original diagnostic receiver");
    assert_eq!(receiver.span.sub_spans[0].src_range, 40..45);
    assert_eq!(
        document
            .content
            .get(receiver.span.sub_spans[0].gen_range.clone()),
        Some("__vize_handler_1")
    );
    assert_eq!(receiver.span.sub_spans[1].src_range, 50..63);
    assert_eq!(
        document
            .content
            .get(receiver.span.sub_spans[1].gen_range.clone()),
        Some(expression)
    );
}

#[test]
fn original_if_projections_keep_the_complete_authored_probe_owner() {
    for source in [
        include_str!(
            "../../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/original-reports/DirectiveObjectArg.vue"
        ),
        include_str!(
            "../../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/cases/SafeObjectPredicateMultiline.vue"
        ),
    ] {
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default())
                .expect("unaltered full reported SFC");
        let template = descriptor.template.as_ref().expect("template");
        let script = descriptor.script_setup.as_ref().expect("setup");
        let allocator = vize_l0::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template.content.as_ref());
        assert!(errors.is_empty(), "{errors:?}");
        let document = super::super::document::project_type_aware(
            &descriptor,
            script.content.as_ref(),
            Some(&root),
            script.loc.start as u32,
            template.loc.start as u32,
            "OriginalCondition.vue",
        );
        let start = source.find("v-if=\"").expect("first original condition") + 6;
        let end = start + source.get(start..).expect("value").find('"').expect("end");
        let offset = inferred_binding_offset(&document, &(start..end))
            .expect("complete actual Canon condition owns its inferred probe");
        let (queries, _) = super::super::template_queries::collect_template_query_sets(
            &document,
            &root,
            template.loc.start as u32,
            true,
            false,
        );
        let query = queries
            .iter()
            .find(|query| {
                query.kind == super::super::template_queries::TemplateQueryKind::Expression
                    && query.source_start as usize == start
                    && query.source_end as usize == end
            })
            .expect("whole authored condition query");
        assert_eq!(query.generated_offset, offset);
    }
}
