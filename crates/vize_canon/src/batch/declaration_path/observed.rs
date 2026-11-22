//! Project declarations and inspect Vue re-exports in the same statement pass.
use super::{DefaultExport, TextEdit, ambient_default_export, ambient_insertion, keyword_end};
use crate::batch::{
    declaration_products::{DeclarationCheckerProducts, DeclarationProjectionConflict},
    vue_template_ref_global::is_vue_package,
};
use oxc_ast::ast::{
    ExportDefaultDeclaration, ExportDefaultDeclarationKind, ImportOrExportKind, Program, Statement,
};
use vize_l0::ToCompactString;

pub(in crate::batch) fn products(
    text: &str,
    program: &Program<'_>,
    inspect_exports: bool,
) -> Result<DeclarationCheckerProducts, DeclarationProjectionConflict> {
    let mut edits = Vec::new();
    let mut sequence = 0;
    let mut adjacent = false;
    let mut exported_names = Vec::new();
    let mut reexports_vue_template_ref = false;
    for statement in &program.body {
        if inspect_exports && !reexports_vue_template_ref {
            reexports_vue_template_ref = vue_template_ref_export(statement);
        }
        if let Statement::ExportDefaultDeclaration(export) = statement {
            if let Some(declaration) = default_declaration(text, export) {
                ambient_default_export(
                    text,
                    declaration,
                    &mut sequence,
                    &mut exported_names,
                    &mut edits,
                );
            }
        } else {
            let declaration = match statement {
                Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
                other => other.as_declaration(),
            };
            if let Some(start) = declaration.and_then(ambient_insertion) {
                adjacent |= edits
                    .last()
                    .is_some_and(|edit| edit.start == start as usize);
                edits.push(TextEdit::insertion(start, "declare "));
            }
        }
    }
    edits.sort_unstable_by(|left, right| {
        right
            .start
            .cmp(&left.start)
            .then_with(|| right.end.cmp(&left.end))
    });
    let conflict = adjacent
        .then(|| order_adjacent_insertions(&mut edits))
        .flatten();
    let mut typescript = text.to_compact_string();
    for edit in edits {
        typescript.replace_range(edit.start..edit.end, &edit.replacement);
    }
    let projected = DeclarationCheckerProducts {
        typescript,
        reexports_vue_template_ref,
    };
    match conflict {
        Some(offset) => Err(DeclarationProjectionConflict {
            offset,
            corrected: projected,
        }),
        None => Ok(projected),
    }
}

// Edits are applied backwards. At a shared offset the following declaration’s
// modifier must be inserted first, then the preceding export assignment.
fn order_adjacent_insertions(edits: &mut [TextEdit]) -> Option<usize> {
    let mut conflict = None;
    for index in 1..edits.len() {
        let (left, right) = (&edits[index - 1], &edits[index]);
        if left.start == right.start
            && left.end == left.start
            && right.end == right.start
            && right.replacement == "declare "
            && left.replacement.starts_with("\nexport default ")
        {
            conflict = Some(left.start);
            edits.swap(index - 1, index);
        }
    }
    conflict
}

fn default_declaration<'a>(
    text: &str,
    export: &'a ExportDefaultDeclaration<'_>,
) -> Option<DefaultExport<'a>> {
    let (span, id, anonymous_name_offset) = match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => (
            function.span,
            function.id.as_ref(),
            Some(
                function
                    .type_parameters
                    .as_ref()
                    .map_or(function.params.span.start, |p| p.span.start),
            ),
        ),
        ExportDefaultDeclarationKind::ClassDeclaration(class) => (
            class.span,
            class.id.as_ref(),
            class
                .type_parameters
                .as_ref()
                .map(|p| p.span.start)
                .or_else(|| {
                    keyword_end(text, class.span.start, class.span.end, "class")
                        .map(|end| end as u32)
                }),
        ),
        _ => return None,
    };
    Some(DefaultExport {
        export_start: export.span.start,
        declaration_start: span.start,
        declaration_end: span.end,
        declared_name: id.map(|id| id.name.as_str()),
        anonymous_name_offset,
    })
}

pub(in crate::batch) fn vue_template_ref_export(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ExportNamedDeclaration(export) => {
            export.export_kind != ImportOrExportKind::Type
                && export
                    .source
                    .as_ref()
                    .is_some_and(|source| is_vue_package(&source.value))
                && export.specifiers.iter().any(|specifier| {
                    specifier.export_kind != ImportOrExportKind::Type
                        && specifier.local.name().as_str() == "useTemplateRef"
                        && specifier.exported.name().as_str() == "useTemplateRef"
                })
        }
        Statement::ExportAllDeclaration(export) => {
            export.exported.is_none()
                && export.export_kind != ImportOrExportKind::Type
                && is_vue_package(&export.source.value)
        }
        _ => false,
    }
}
