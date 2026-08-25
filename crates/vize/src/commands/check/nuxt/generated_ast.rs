use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Statement, TSImportType};
use oxc_ast_visit::{Visit, walk::walk_ts_import_type};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{String, ToCompactString, cstr};

use super::super::dts::rewrite_relative_specifier;
use super::parsing::module_export_name;

pub(super) struct ImportManifestExport {
    pub(super) local_name: String,
    pub(super) exported_name: String,
    pub(super) module_specifier: String,
}

pub(super) fn parse_import_manifest_exports(
    content: &str,
    base_dir: &Path,
) -> Vec<ImportManifestExport> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if ret.panicked {
        return Vec::new();
    }

    let mut exports = Vec::new();
    for statement in &ret.program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        let Some(source) = &export.source else {
            continue;
        };
        let module_specifier = rewrite_relative_specifier(source.value.as_str(), base_dir);

        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() {
                continue;
            }
            let Some(local_name) = module_export_name(&specifier.local) else {
                continue;
            };
            let Some(exported_name) = module_export_name(&specifier.exported) else {
                continue;
            };
            exports.push(ImportManifestExport {
                local_name: local_name.to_compact_string(),
                exported_name: exported_name.to_compact_string(),
                module_specifier: module_specifier.clone(),
            });
        }
    }

    exports
}

pub(super) fn collect_import_type_specifiers(type_annotation: &str) -> Vec<String> {
    collect_import_types(type_annotation)
        .unwrap_or_default()
        .into_iter()
        .map(|import| import.specifier)
        .collect()
}

pub(super) fn rewrite_component_imports_for_virtual_project(
    type_annotation: &str,
    project_root: &Path,
) -> String {
    rewrite_import_type_specifiers(type_annotation, |specifier| {
        virtual_project_specifier(specifier, project_root)
    })
    .unwrap_or_else(|| type_annotation.to_compact_string())
}

fn virtual_project_specifier(specifier: &str, project_root: &Path) -> String {
    if !specifier.ends_with(".vue") {
        return specifier.to_compact_string();
    }

    let specifier_path = Path::new(specifier);
    let relative = if specifier_path.is_absolute() {
        specifier_path.strip_prefix(project_root).ok()
    } else {
        None
    };

    if let Some(relative) = relative {
        let mut rendered = cstr!("./{}", relative.display());
        rendered.push_str(".ts");
        return rendered;
    }

    cstr!("{specifier}.ts")
}

const TYPE_ANNOTATION_PREFIX: &str = "type __VizeGeneratedType = ";

fn rewrite_import_type_specifiers(
    type_annotation: &str,
    rewrite: impl Fn(&str) -> String,
) -> Option<String> {
    let imports = collect_import_types(type_annotation)?;
    let prefix_len = TYPE_ANNOTATION_PREFIX.len();

    let mut replacements = imports
        .into_iter()
        .filter_map(|import| {
            let start = import.source_start.checked_sub(prefix_len)?;
            let end = import.source_end.checked_sub(prefix_len)?;
            if start > end || end > type_annotation.len() {
                return None;
            }

            let quote = import.quote.unwrap_or('\'');
            let rewritten = rewrite(import.specifier.as_str());
            Some(ImportTypeReplacement {
                start,
                end,
                replacement: cstr!("{quote}{rewritten}{quote}"),
            })
        })
        .collect::<Vec<_>>();

    if replacements.is_empty() {
        return Some(type_annotation.to_compact_string());
    }

    replacements.sort_by_key(|replacement| replacement.start);
    let mut out = type_annotation.to_compact_string();
    for replacement in replacements.into_iter().rev() {
        out.replace_range(
            replacement.start..replacement.end,
            replacement.replacement.as_str(),
        );
    }
    Some(out)
}

fn collect_import_types(type_annotation: &str) -> Option<Vec<ImportTypeSource>> {
    let wrapped = cstr!("{TYPE_ANNOTATION_PREFIX}{type_annotation};");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, wrapped.as_str(), SourceType::d_ts()).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        return None;
    }

    let mut collector = ImportTypeCollector::default();
    collector.visit_program(&ret.program);
    Some(collector.imports)
}

#[derive(Default)]
struct ImportTypeCollector {
    imports: Vec<ImportTypeSource>,
}

struct ImportTypeSource {
    specifier: String,
    source_start: usize,
    source_end: usize,
    quote: Option<char>,
}

struct ImportTypeReplacement {
    start: usize,
    end: usize,
    replacement: String,
}

impl<'a> Visit<'a> for ImportTypeCollector {
    fn visit_ts_import_type(&mut self, it: &TSImportType<'a>) {
        self.imports.push(ImportTypeSource {
            specifier: it.source.value.as_str().to_compact_string(),
            source_start: it.source.span.start as usize,
            source_end: it.source.span.end as usize,
            quote: it
                .source
                .raw
                .as_ref()
                .and_then(|raw| raw.as_str().chars().next())
                .filter(|quote| *quote == '\'' || *quote == '"'),
        });
        walk_ts_import_type(self, it);
    }
}
