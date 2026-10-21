//! Type probes read `const __expr_N`, the binding the croquis generator used
//! to name one template expression. Canon emits `void (expr); // kind` and
//! event handlers as `__vize_cb)((expr))`. This pass adds the binding and
//! keeps each expression's mapping on its text.

use vize_l0::{String as VizeString, cstr};

use super::document::TypeAwareDocument;

mod mapped;
#[cfg(test)]
mod ownership_tests;

pub(super) fn inferred_binding_offset(
    document: &TypeAwareDocument,
    authored: &std::ops::Range<usize>,
) -> Option<u32> {
    mapped::inferred_binding_offset(document, authored)
}

const VOID_OPEN: &str = "void (";
const HANDLER_MARK: &str = "__vize_cb)((";

pub(super) fn binding_offset(generated: &str, expression_offset: u32) -> Option<u32> {
    let offset = usize::min(expression_offset as usize, generated.len());
    binding_on_line(generated, offset).or_else(|| {
        let line_end = generated
            .get(offset..)?
            .find('\n')
            .map_or(generated.len(), |at| offset + at);
        let line_start = generated.get(..offset)?.rfind('\n').map_or(0, |at| at + 1);
        if !generated.get(line_start..line_end)?.contains(HANDLER_MARK) {
            return None;
        }
        let before = generated.get(..offset)?;
        let previous_end = before.rfind('\n')?;
        let previous_start = before
            .get(..previous_end)
            .unwrap_or_default()
            .rfind('\n')
            .map_or(0, |index| index + 1);
        binding_on_line(generated, previous_start)
    })
}

fn binding_on_line(generated: &str, offset: usize) -> Option<u32> {
    let before_offset = generated.get(..offset)?;
    let after_offset = generated.get(offset..)?;
    let line_start = before_offset.rfind('\n').map_or(0, |index| index + 1);
    let line_end = after_offset
        .find('\n')
        .map_or(generated.len(), |index| offset + index);
    let line = generated.get(line_start..line_end)?;
    let const_start = line.find("const __expr_")?;
    let name_start = const_start + "const ".len();
    let name_end = line.get(name_start..)?.find(" = ")? + name_start;
    let name_end = line
        .get(name_start..name_end)?
        .find(':')
        .map_or(name_end, |at| name_start + at);
    (name_end > name_start).then_some((line_start + name_end - 1) as u32)
}

pub(super) fn hoist_type_imports(document: &mut TypeAwareDocument, script: &str) {
    let mut prelude = VizeString::new("");
    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import type ") && !document.content.contains(trimmed) {
            prelude.push_str(trimmed);
            prelude.push('\n');
        }
    }
    if prelude.is_empty() {
        return;
    }
    document
        .mapping
        .note_generated_replacement(0, 0, prelude.len());
    prelude.push_str(document.content.as_str());
    document.content = prelude;
}

pub(super) fn template_binding_is_unsafe(
    source: &str,
    query: &super::template_queries::TemplateQuery,
    probe: Option<&super::TypeProbe>,
) -> bool {
    if source
        .get(query.source_start as usize..query.source_end as usize)
        .is_some_and(|text| text.trim() == "$attrs")
    {
        return false;
    }
    if super::has_unsafe_template_type(probe) {
        return true;
    }
    if query.kind == super::template_queries::TemplateQueryKind::CallReturn {
        return probe.is_some_and(|probe| {
            probe.return_types.iter().any(|types| {
                corsa::utils::is_any_like_type_texts(types)
                    || corsa::utils::is_unknown_like_type_texts(types)
            })
        });
    }
    matches!(
        query.kind,
        super::template_queries::TemplateQueryKind::ForSource
    ) && probe.is_some_and(|probe| {
        probe.type_texts.iter().any(|text| {
            text.contains("[item: any,")
                || text.contains("[item: unknown,")
                || text.contains("[item: any]")
                || text.contains("[item: unknown]")
        })
    })
}

pub(super) fn bind_template_expressions(document: &mut TypeAwareDocument) {
    let mut text = document.content.clone();
    let props_object = super::options_prop_shape::options_props_object(&text).map(VizeString::from);
    let mut edits = Vec::new();
    let mut index = 0u32;
    collect_void_wrappers(
        document,
        &text,
        props_object.as_deref(),
        &mut edits,
        &mut index,
    );
    collect_handler_arguments(document, &text, &mut edits, &mut index);
    edits.sort_by_key(|(start, _, _, _, _)| *start);
    edits.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);
    for (start, end, expr_start, expr_end, new_stmt) in edits.into_iter().rev() {
        let authored = document.mapping.rows().find_map(|row| {
            std::iter::once((&row.span.gen_range, &row.span.src_range))
                .chain(
                    row.span
                        .sub_spans
                        .iter()
                        .map(|sub| (&sub.gen_range, &sub.src_range)),
                )
                .find_map(|(generated, authored)| {
                    (generated.start == expr_start && generated.end == expr_end)
                        .then(|| authored.clone())
                })
        });
        if start == end {
            document
                .mapping
                .note_generated_replacement(start, 0, new_stmt.len());
        } else {
            let total_delta = new_stmt.len() as isize - (end - start) as isize;
            document.mapping.retarget_expression_binding(
                start..end,
                expr_start..expr_end,
                total_delta,
                total_delta,
            );
        }
        if let Some(authored) = authored {
            mapped::mark_inferred_binding(document, start, &new_stmt, authored);
        }
        text.replace_range(start..end, &new_stmt);
    }
    document.content = text;
    mapped::bind_checked_expressions(document, &mut index);
}

fn mapped_expression_ranges(
    document: &TypeAwareDocument,
) -> impl Iterator<Item = &std::ops::Range<usize>> {
    document.mapping.rows().flat_map(|row| {
        std::iter::once(&row.span.gen_range)
            .chain(row.span.sub_spans.iter().map(|sub| &sub.gen_range))
    })
}

fn collect_void_wrappers(
    document: &TypeAwareDocument,
    text: &str,
    props_object: Option<&str>,
    edits: &mut Vec<(usize, usize, usize, usize, VizeString)>,
    index: &mut u32,
) {
    for generated in mapped_expression_ranges(document) {
        let expr_start = generated.start;
        let expr_end = generated.end;
        let Some(void_at) = expr_start.checked_sub(VOID_OPEN.len()) else {
            continue;
        };
        if text.get(void_at..expr_start) != Some(VOID_OPEN)
            || !text
                .get(expr_end..)
                .is_some_and(|rest| rest.starts_with("); // "))
        {
            continue;
        }
        let expr = text.get(expr_start..expr_end).unwrap_or_default();
        let new_stmt = match super::options_prop_shape::options_prop_annotation(props_object, expr)
        {
            Some(annotation) => cstr!("const __expr_{index}: {annotation} = ({expr})"),
            None => cstr!("const __expr_{index} = ({expr})"),
        };
        let old_end = void_at + VOID_OPEN.len() + expr.len() + 1;
        edits.push((void_at, old_end, expr_start, expr_end, new_stmt));
        *index += 1;
    }
}

fn collect_handler_arguments(
    document: &TypeAwareDocument,
    text: &str,
    edits: &mut Vec<(usize, usize, usize, usize, VizeString)>,
    index: &mut u32,
) {
    for generated in mapped_expression_ranges(document) {
        let expr_start = generated.start;
        let expr_end = generated.end;
        let Some(marker_start) = expr_start.checked_sub(HANDLER_MARK.len()) else {
            continue;
        };
        if text.get(marker_start..expr_start) != Some(HANDLER_MARK)
            || !text
                .get(expr_end..)
                .is_some_and(|rest| rest.starts_with("))"))
        {
            continue;
        }
        let expr = text.get(expr_start..expr_end).unwrap_or_default();
        let line_start = text
            .get(..expr_start)
            .unwrap_or_default()
            .rfind('\n')
            .map_or(0, |at| at + 1);
        let indent = " ".repeat(
            text.get(line_start..)
                .unwrap_or_default()
                .chars()
                .take_while(|character| *character == ' ')
                .count(),
        );
        let line = cstr!("{indent}const __expr_{index} = ({expr});\n");
        edits.push((line_start, line_start, expr_start, expr_end, line));
        *index += 1;
    }
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping};
    use vize_l0::String as VizeString;

    use super::{VOID_OPEN, bind_template_expressions};
    use crate::linter::native_type_aware::document::TypeAwareDocument;

    #[test]
    fn a_void_wrapper_becomes_an_expression_binding() {
        let source = "  void (actions[method]); // VOn\n";
        let expr_at = source.find("actions[method]").expect("expr");
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![VizeMapping::new(
                expr_at..expr_at + "actions[method]".len(),
                10..24,
            )]),
        };
        bind_template_expressions(&mut document);
        assert_eq!(
            document.content.as_str(),
            "  const __expr_0 = (actions[method]); // VOn\n"
        );
        let generated = document.mapping.to_generated(10).expect("row");
        assert_eq!(
            &document.content[generated..generated + "actions[method]".len()],
            "actions[method]"
        );
    }

    #[test]
    fn a_handler_argument_gets_a_binding_on_the_previous_line() {
        let source =
            "  const __vize_handler_1 = ((__vize_cb: unknown) => __vize_cb)((actions[method]));\n";
        let expr_at = source.find("actions[method]").expect("expr");
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![VizeMapping::new(
                expr_at..expr_at + "actions[method]".len(),
                3..17,
            )]),
        };
        bind_template_expressions(&mut document);
        assert_eq!(
            document.content.as_str(),
            "  const __expr_0 = (actions[method]);\n  const __vize_handler_1 = ((__vize_cb: unknown) => __vize_cb)((actions[method]));\n"
        );
        let generated = document.mapping.to_generated(3).expect("row");
        assert_eq!(
            &document.content[generated..generated + "actions[method]".len()],
            "actions[method]"
        );
    }

    #[test]
    fn slot_outlet_payloads_get_bindings_before_the_checked_call() {
        let source = "  __vizeSlotOutlet((undefined as unknown as __VizeSlots)[\"default\"])(\n  {\n    \"items\": getItems(),\n    \"attrs\": { id: 'x' },\n  });\n";
        let call = "getItems()";
        let object = "{ id: 'x' }";
        let call_at = source.find(call).expect("call");
        let object_at = source.find(object).expect("object");
        let call_authored = 10..10 + call.len();
        let object_authored = 30..30 + object.len();
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![
                VizeMapping::new(call_at..call_at + call.len(), call_authored.clone()),
                VizeMapping::new(object_at..object_at + object.len(), object_authored.clone()),
            ]),
        };

        bind_template_expressions(&mut document);

        assert_eq!(
            document.content.as_str(),
            "  const __expr_0 = (getItems());\n  const __expr_1 = ({ id: 'x' });\n  __vizeSlotOutlet((undefined as unknown as __VizeSlots)[\"default\"])(\n  {\n    \"items\": getItems(),\n    \"attrs\": { id: 'x' },\n  });\n"
        );
        let call_probe =
            super::mapped::inferred_binding_offset(&document, &call_authored).expect("call probe");
        let object_probe = super::mapped::inferred_binding_offset(&document, &object_authored)
            .expect("object probe");
        assert_eq!(
            &document.content[call_probe as usize..call_probe as usize + 1],
            "0"
        );
        assert_eq!(
            &document.content[object_probe as usize..object_probe as usize + 1],
            "1"
        );
    }

    #[test]
    fn an_options_prop_binding_uses_the_runtime_shape() {
        let source = "  void (isOpened); // VBind\n  void (type); // VBind\n  const __vize_options_props = ({\n    isOpened: { type: Boolean, required: true },\n  } as const);\n";
        let expr_at = source.find("isOpened").expect("expr");
        let type_at = source.find("void (type)").expect("second wrapper") + VOID_OPEN.len();
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![
                VizeMapping::new(expr_at..expr_at + "isOpened".len(), 4..11),
                VizeMapping::new(type_at..type_at + 4, 20..24),
            ]),
        };
        bind_template_expressions(&mut document);
        assert_eq!(
            document.content.as_str(),
            "  const __expr_0: __VizeOptionsPropShape<typeof __vize_options_props>[\"isOpened\"] = (isOpened); // VBind\n  const __expr_1 = (type); // VBind\n  const __vize_options_props = ({\n    isOpened: { type: Boolean, required: true },\n  } as const);\n"
        );
        let generated = document.mapping.to_generated(4).expect("row");
        assert_eq!(
            &document.content[generated..generated + "isOpened".len()],
            "isOpened"
        );
    }
}
