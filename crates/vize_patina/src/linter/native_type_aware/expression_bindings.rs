//! Type probes read `const __expr_N`, the binding the croquis generator used
//! to name one template expression. Canon emits `void (expr); // kind` and
//! event handlers as `__vize_cb)((expr))`. This pass adds the binding and
//! keeps each expression's mapping on its text.

use vize_s0::String as VizeString;

use super::document::TypeAwareDocument;

const VOID_OPEN: &str = "void (";
const HANDLER_MARK: &str = "__vize_cb)((";

pub(super) fn binding_offset(generated: &str, expression_offset: u32) -> Option<u32> {
    let offset = usize::min(expression_offset as usize, generated.len());
    binding_on_line(generated, offset).or_else(|| {
        let before = generated.get(..offset)?;
        let previous_end = before.rfind('\n')?;
        let previous_start = before[..previous_end]
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
    (name_end > name_start).then_some((line_start + name_end - 1) as u32)
}

pub(super) fn hoist_type_imports(document: &mut TypeAwareDocument, script: &str) {
    let mut prelude = std::string::String::new();
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
    document.content = VizeString::from(prelude.as_str());
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
    super::has_unsafe_template_type(probe)
}

pub(super) fn bind_template_expressions(document: &mut TypeAwareDocument) {
    let mut text = std::string::String::from(document.content.as_str());
    let mut edits = Vec::new();
    let mut index = 0u32;
    collect_void_wrappers(&text, &mut edits, &mut index);
    collect_handler_arguments(&text, &mut edits, &mut index);
    edits.sort_by_key(|(start, _, _, _, _)| *start);
    for (start, end, expr_start, expr_end, new_stmt) in edits.into_iter().rev() {
        if start == end {
            document
                .mapping
                .note_generated_replacement(start, 0, new_stmt.len());
        } else {
            let total_delta = new_stmt.len() as isize - (end - start) as isize;
            document.mapping.retarget_expression_binding(
                start..end,
                expr_start..expr_end,
                total_delta + 1,
                total_delta,
            );
        }
        text.replace_range(start..end, &new_stmt);
    }
    document.content = VizeString::from(text.as_str());
}

fn collect_void_wrappers(
    text: &str,
    edits: &mut Vec<(usize, usize, usize, usize, String)>,
    index: &mut u32,
) {
    let mut search = 0usize;
    while let Some(relative) = text[search..].find(VOID_OPEN) {
        let void_at = search + relative;
        let line_end = text[void_at..]
            .find('\n')
            .map_or(text.len(), |offset| void_at + offset);
        let line = &text[void_at..line_end];
        let Some(close) = line.rfind("); // ") else {
            search = line_end;
            continue;
        };
        let expr_start = void_at + VOID_OPEN.len();
        let expr_end = void_at + close;
        if expr_end < expr_start {
            search = line_end;
            continue;
        }
        let expr = &text[expr_start..expr_end];
        let new_stmt = format!("const __expr_{index} = {expr}");
        let old_end = void_at + VOID_OPEN.len() + expr.len() + 1;
        edits.push((void_at, old_end, expr_start, expr_end, new_stmt));
        *index += 1;
        search = line_end;
    }
}

fn collect_handler_arguments(
    text: &str,
    edits: &mut Vec<(usize, usize, usize, usize, String)>,
    index: &mut u32,
) {
    let mut search = 0usize;
    while let Some(relative) = text[search..].find(HANDLER_MARK) {
        let expr_start = search + relative + HANDLER_MARK.len();
        let Some(relative_end) = text[expr_start..].find("))") else {
            break;
        };
        let expr_end = expr_start + relative_end;
        let expr = &text[expr_start..expr_end];
        let line_start = text[..expr_start].rfind('\n').map_or(0, |at| at + 1);
        let indent = " ".repeat(
            text[line_start..]
                .chars()
                .take_while(|character| *character == ' ')
                .count(),
        );
        let line = format!("{indent}const __expr_{index} = {expr};\n");
        edits.push((line_start, line_start, expr_start, expr_end, line));
        *index += 1;
        search = expr_end;
    }
}

#[cfg(test)]
mod tests {
    use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping};
    use vize_s0::String as VizeString;

    use super::bind_template_expressions;
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
            "  const __expr_0 = actions[method]; // VOn\n"
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
            "  const __expr_0 = actions[method];\n  const __vize_handler_1 = ((__vize_cb: unknown) => __vize_cb)((actions[method]));\n"
        );
        let generated = document.mapping.to_generated(3).expect("row");
        assert_eq!(
            &document.content[generated..generated + "actions[method]".len()],
            "actions[method]"
        );
    }
}
