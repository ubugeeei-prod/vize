//! AST-backed import extraction for inspector graphs.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ImportDeclaration, ImportDeclarationSpecifier, ImportExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::parser::parse;
use vize_atelier_sfc::{
    SfcParseOptions, SfcScriptBlock, parse_sfc, script::resolve_template_used_identifiers,
};
use vize_s0::{FxHashSet, String, ToCompactString};

#[derive(Debug)]
pub(super) struct ImportEdge {
    pub specifier: String,
    pub kind: &'static str,
    pub locals: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct FileAnalysis {
    pub imports: Vec<ImportEdge>,
    pub template_used_ids: FxHashSet<String>,
}

pub(super) fn analyze_file(path: &str, source: &str) -> FileAnalysis {
    if path.ends_with(".vue") {
        return analyze_sfc_file(path, source);
    }

    FileAnalysis {
        imports: extract_script_imports(source, source_type_for_path(path)),
        template_used_ids: FxHashSet::default(),
    }
}

fn analyze_sfc_file(path: &str, source: &str) -> FileAnalysis {
    let Ok(descriptor) = parse_sfc(
        source,
        SfcParseOptions {
            filename: path.into(),
            ..Default::default()
        },
    ) else {
        return FileAnalysis::default();
    };

    let mut imports = Vec::new();
    if let Some(script) = descriptor.script.as_ref() {
        imports.extend(extract_script_block_imports(script));
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        imports.extend(extract_script_block_imports(script_setup));
    }

    let template_used_ids = descriptor
        .template
        .as_ref()
        .map(|template| {
            let allocator = vize_s0::Allocator::new();
            let (root, _) = parse(&allocator, template.content.as_ref());
            resolve_template_used_identifiers(&root).used_ids
        })
        .unwrap_or_default();

    FileAnalysis {
        imports,
        template_used_ids,
    }
}

fn extract_script_block_imports(script: &SfcScriptBlock<'_>) -> Vec<ImportEdge> {
    extract_script_imports(
        script.content.as_ref(),
        source_type_for_script_lang(script.lang.as_deref()),
    )
}

fn extract_script_imports(source: &str, source_type: SourceType) -> Vec<ImportEdge> {
    let allocator = Allocator::default();
    let result = Parser::new(&allocator, source, source_type).parse();
    if result.panicked {
        return Vec::new();
    }

    let mut collector = ImportCollector::default();
    collector.visit_program(&result.program);
    collector.imports
}

#[derive(Default)]
struct ImportCollector {
    imports: Vec<ImportEdge>,
}

impl ImportCollector {
    fn collect_static_import(&mut self, import: &ImportDeclaration<'_>) {
        let Some(locals) = runtime_import_locals(import) else {
            return;
        };

        self.imports.push(ImportEdge {
            specifier: import.source.value.as_str().to_compact_string(),
            kind: "import",
            locals,
        });
    }
}

impl<'a> Visit<'a> for ImportCollector {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        self.collect_static_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Expression::StringLiteral(literal) = &import.source {
            self.imports.push(ImportEdge {
                specifier: literal.value.as_str().to_compact_string(),
                kind: "dynamic-import",
                locals: Vec::new(),
            });
        }

        walk::walk_import_expression(self, import);
    }
}

fn runtime_import_locals(import: &ImportDeclaration<'_>) -> Option<Vec<String>> {
    if import.import_kind.is_type() {
        return None;
    }

    let Some(specifiers) = import.specifiers.as_ref() else {
        return Some(Vec::new());
    };

    let mut locals = Vec::new();
    let mut saw_type_only_specifier = false;
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                if specifier.import_kind.is_type() {
                    saw_type_only_specifier = true;
                } else {
                    locals.push(specifier.local.name.as_str().to_compact_string());
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                locals.push(specifier.local.name.as_str().to_compact_string());
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                locals.push(specifier.local.name.as_str().to_compact_string());
            }
        }
    }

    if specifiers.is_empty() || !locals.is_empty() || !saw_type_only_specifier {
        Some(locals)
    } else {
        None
    }
}

fn source_type_for_path(path: &str) -> SourceType {
    let path_without_query = path.split(['?', '#']).next().unwrap_or(path);
    match path_without_query
        .rsplit_once('.')
        .map(|(_, extension)| extension)
    {
        Some("tsx" | "jsx") => SourceType::tsx().with_module(true),
        Some("js" | "mjs" | "cjs") => SourceType::from_path("module.js")
            .unwrap_or_else(|_| SourceType::default())
            .with_module(true),
        _ => SourceType::ts().with_module(true),
    }
}

fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("tsx" | "jsx") => SourceType::tsx().with_module(true),
        Some("js" | "mjs" | "cjs") => SourceType::from_path("module.js")
            .unwrap_or_else(|_| SourceType::default())
            .with_module(true),
        _ => SourceType::ts().with_module(true),
    }
}
