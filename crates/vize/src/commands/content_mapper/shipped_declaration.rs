//! Packaged SFCs are typed by their shipped declarations, not compiled JavaScript.

use std::io;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_canon::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperTransform,
};

#[derive(Debug)]
struct TextEdit {
    start: usize,
    end: usize,
    replacement: String,
}

impl TextEdit {
    fn insertion(offset: u32, text: &str) -> Self {
        Self {
            start: offset as usize,
            end: offset as usize,
            replacement: text.into(),
        }
    }
}

fn ambient_insertion(declaration: &Declaration<'_>) -> Option<u32> {
    match declaration {
        Declaration::VariableDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::FunctionDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::ClassDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::TSEnumDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::TSModuleDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        _ => None,
    }
}

fn default_export_name(text: &str, sequence: &mut usize) -> String {
    loop {
        let name = format!("__vize_shipped_default_{}", *sequence);
        *sequence += 1;
        if !text.contains(&name) {
            return name;
        }
    }
}

fn keyword_end(text: &str, span_start: u32, span_end: u32, keyword: &str) -> Option<usize> {
    let start = span_start as usize;
    let end = span_end as usize;
    let relative = text.get(start..end)?.find(keyword)?;
    Some(start + relative + keyword.len())
}

fn ambient_default_export(
    text: &str,
    export_start: u32,
    declaration_start: u32,
    declaration_end: u32,
    declared_name: Option<&str>,
    anonymous_name_offset: Option<u32>,
    sequence: &mut usize,
    edits: &mut Vec<TextEdit>,
) {
    let export_start = export_start as usize;
    let declaration_start = declaration_start as usize;
    let prefix = &text[export_start..declaration_start];
    let Some(default_start) = prefix.find("default") else {
        return;
    };
    let after_default = default_start + "default".len();
    let delete_end = if prefix[after_default..].trim().is_empty() {
        declaration_start
    } else {
        export_start + after_default
    };
    edits.push(TextEdit {
        start: export_start,
        end: delete_end,
        replacement: String::new(),
    });
    edits.push(TextEdit::insertion(declaration_start as u32, "declare "));

    let name = declared_name.map_or_else(|| default_export_name(text, sequence), str::to_owned);
    if declared_name.is_none()
        && let Some(offset) = anonymous_name_offset
    {
        edits.push(TextEdit::insertion(offset, &format!(" {name}")));
    }
    edits.push(TextEdit::insertion(
        declaration_end,
        &format!("\nexport default {name};"),
    ));
}

/// The content-mapper protocol only accepts ordinary TypeScript extensions.
/// Make the value declarations that are implicitly ambient in a `.d.ts`
/// explicitly ambient before the host parses them as `.ts`.
fn project_declaration_as_typescript(text: &str) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::d_ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return text.into();
    }

    let mut edits = Vec::new();
    let mut default_sequence = 0;
    for statement in &parsed.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::FunctionDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::ClassDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::TSEnumDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::TSModuleDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(offset) = export.declaration.as_ref().and_then(ambient_insertion) {
                    edits.push(TextEdit::insertion(offset, "declare "));
                }
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(declaration) => {
                    let anonymous_name_offset = declaration
                        .type_parameters
                        .as_ref()
                        .map_or(declaration.params.span.start, |parameters| {
                            parameters.span.start
                        });
                    ambient_default_export(
                        text,
                        export.span.start,
                        declaration.span.start,
                        declaration.span.end,
                        declaration.id.as_ref().map(|id| id.name.as_str()),
                        Some(anonymous_name_offset),
                        &mut default_sequence,
                        &mut edits,
                    );
                }
                ExportDefaultDeclarationKind::ClassDeclaration(declaration) => {
                    let anonymous_name_offset = declaration
                        .type_parameters
                        .as_ref()
                        .map(|parameters| parameters.span.start)
                        .or_else(|| {
                            keyword_end(text, declaration.span.start, declaration.span.end, "class")
                                .map(|offset| offset as u32)
                        });
                    ambient_default_export(
                        text,
                        export.span.start,
                        declaration.span.start,
                        declaration.span.end,
                        declaration.id.as_ref().map(|id| id.name.as_str()),
                        anonymous_name_offset,
                        &mut default_sequence,
                        &mut edits,
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }

    edits.sort_unstable_by(|left, right| {
        right
            .start
            .cmp(&left.start)
            .then_with(|| right.end.cmp(&left.end))
    });
    let mut projected = text.to_owned();
    for edit in edits {
        projected.replace_range(edit.start..edit.end, &edit.replacement);
    }
    projected
}

/// Read the declaration shipped beside a packaged `.vue` file.
///
/// Application source is deliberately excluded: an adjacent declaration can
/// be stale while the author edits the SFC. Published packages are different;
/// their declarations are the package's public type surface, while their Vue
/// files may contain compiled JavaScript with no named types or slot metadata.
pub(super) fn read_shipped_declaration(
    file: &Path,
    skip_lib_check: bool,
) -> io::Result<Option<ContentMapperTransform>> {
    if file.extension().is_none_or(|extension| extension != "vue")
        || !file
            .components()
            .any(|component| component.as_os_str() == "node_modules")
    {
        return Ok(None);
    }

    for declaration in [
        file.with_extension("d.vue.ts"),
        file.with_extension("vue.d.ts"),
    ] {
        let text = match std::fs::read_to_string(declaration) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let text = project_declaration_as_typescript(text.trim_start_matches('\u{feff}'));
        // Reproduce skipLibCheck for this virtual declaration only; diagnostics
        // in importing application files stay enabled.
        let diagnostic_directives = skip_lib_check.then(|| ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics: Vec::new(),
            directives: vec![ContentMapperDiagnosticDirective([
                0,
                0,
                0,
                text.len(),
                0,
                0,
            ])],
        });
        return Ok(Some(ContentMapperTransform {
            text: text.into(),
            extension: ".ts",
            mappings: Vec::new(),
            semantic_links: Vec::new(),
            diagnostic_directives,
            diagnostics: Vec::new(),
        }));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{project_declaration_as_typescript, read_shipped_declaration};
    use crate::commands::content_mapper::test_support::{
        exchange, frames, initialize_request, open_project_request, transform_request,
    };
    use serde_json::{Value, json};
    use std::path::Path;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn packaged_declaration_order_and_diagnostic_policy_are_explicit() {
        let root = tempfile::tempdir().unwrap();
        let component = root.path().join("node_modules/ui/Badge.vue");
        write(
            &component.with_extension("vue.d.ts"),
            "export type Fallback = 1;\n",
        );
        write(
            &component.with_extension("d.vue.ts"),
            "\u{feff}export type Preferred = 1;\n",
        );

        let checked = read_shipped_declaration(&component, false)
            .unwrap()
            .unwrap();
        assert_eq!(checked.text, "export type Preferred = 1;\n");
        assert_eq!(checked.extension, ".ts");
        assert_eq!(checked.diagnostic_directives, None);

        let skipped = read_shipped_declaration(&component, true).unwrap().unwrap();
        let table = skipped.diagnostic_directives.unwrap();
        assert_eq!(table.unused_expect_directive_diagnostics, []);
        assert_eq!(table.directives[0].0, [0, 0, 0, skipped.text.len(), 0, 0]);
    }

    #[test]
    fn local_sources_and_packages_without_declarations_fall_through() {
        let root = tempfile::tempdir().unwrap();
        let local = root.path().join("src/Badge.vue");
        write(
            &local.with_extension("d.vue.ts"),
            "export type Stale = 1;\n",
        );
        assert!(read_shipped_declaration(&local, true).unwrap().is_none());

        let packaged = root.path().join("node_modules/ui/Badge.vue");
        write(&packaged, "<template />\n");
        assert!(read_shipped_declaration(&packaged, true).unwrap().is_none());
    }

    #[test]
    fn declaration_only_values_are_explicitly_ambient_in_virtual_typescript() {
        let declaration = r#"export function format(value: string): string;
export const palette: readonly string[];
export class Theme { current(): string; }
export enum Tone { Dark }
export namespace colors { const current: string; }
"#;
        assert_eq!(
            project_declaration_as_typescript(declaration),
            r#"export declare function format(value: string): string;
export declare const palette: readonly string[];
export declare class Theme { current(): string; }
export declare enum Tone { Dark }
export declare namespace colors { const current: string; }
"#
        );
    }

    #[test]
    fn default_declarations_keep_their_module_export() {
        assert_eq!(
            project_declaration_as_typescript(
                "export default function format(value: string): string;\n"
            ),
            "declare function format(value: string): string;\nexport default format;\n"
        );
        assert_eq!(
            project_declaration_as_typescript("export default class { value: string; }\n"),
            "declare class __vize_shipped_default_0 { value: string; }\nexport default __vize_shipped_default_0;\n"
        );
        assert_eq!(
            project_declaration_as_typescript("export default function<T>(): T;\n"),
            "declare function __vize_shipped_default_0<T>(): T;\nexport default __vize_shipped_default_0;\n"
        );
    }

    #[test]
    fn server_returns_the_declaration_and_scopes_skip_lib_check_to_it() {
        let root = tempfile::tempdir().unwrap();
        let component = root.path().join("node_modules/ui/Badge.vue");
        let declaration = "export interface BadgeProps { label: string }\n";
        write(&component.with_extension("d.vue.ts"), declaration);

        let transform = |id, skip_lib_check| {
            let mut request = transform_request(id, "<script>export default {}</script>");
            request["params"]["fileName"] = json!(component);
            exchange(&frames(&[
                initialize_request(),
                open_project_request(2, Value::Null, json!({ "skipLibCheck": skip_lib_check })),
                request,
            ]))
        };

        let skipped = transform(3, true);
        let result = &skipped[2]["result"];
        assert_eq!(result["text"], declaration);
        assert_eq!(result["extension"], ".ts");
        assert_eq!(result["mappings"], json!([]));
        assert_eq!(
            result["diagnosticDirectives"]["directives"],
            json!([[0, 0, 0, declaration.len(), 0]])
        );

        let checked = transform(3, false);
        assert_eq!(checked[2]["result"]["text"], declaration);
        assert!(checked[2]["result"].get("diagnosticDirectives").is_none());
    }
}
