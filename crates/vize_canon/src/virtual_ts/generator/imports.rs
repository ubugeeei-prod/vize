//! Import-name extraction and `/// <reference types>` directives.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    IdentifierReference, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
    TSEnumDeclaration, TSTypeName, TSTypeQueryExprName, TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::identifier::is_identifier_name;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::{CompactString, FxHashSet, String};
use vize_croquis::Croquis;

use super::super::types::VirtualTsOptions;

pub(super) fn collect_component_imported_names<'a>(
    summary: &Croquis,
    script_content: Option<&'a str>,
    options: &VirtualTsOptions,
    global_components_enabled: bool,
    type_only: &FxHashSet<CompactString>,
) -> FxHashSet<&'a str> {
    // The template-scope unwrap set also needs it: an auto-import already
    // imported by a plain `<script>` must not gain a second shadow.
    let needed = !options.auto_import_stubs.is_empty()
        || !options.auto_import_bindings.is_empty()
        || (global_components_enabled
            && !vize_croquis::facts::component_usage_list(summary).is_empty())
        || super::super::component_aliases::needs_imported_names(summary, type_only);
    if needed {
        vize_carton::profile!(
            "canon.virtual_ts.extract_imported_names",
            collect_imported_names(summary, script_content)
        )
    } else {
        FxHashSet::default()
    }
}

pub(super) fn emit_reference_path_directives(ts: &mut String, paths: &[String]) {
    super::super::reference_paths::emit(ts, paths);
}

pub(super) fn emit_reference_type_directives(
    ts: &mut String,
    script_content: Option<&str>,
) -> bool {
    let Some(script) = script_content else {
        return false;
    };

    let mut seen = FxHashSet::default();
    for line in script.lines() {
        let Some(package) = reference_types_attribute(line) else {
            continue;
        };
        if seen.insert(package) {
            append!(*ts, "/// <reference types=\"{package}\" />\n");
        }
    }
    !seen.is_empty()
}

fn reference_types_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "types")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = cstr!("{name}=");
    let start = line.find(needle.as_str())? + needle.len();
    let quote = line.get(start..)?.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line.get(value_start..)?.find(quote)? + value_start;
    line.get(value_start..value_end)
}

pub(super) fn collect_imported_names<'a>(
    summary: &Croquis,
    script_content: Option<&'a str>,
) -> FxHashSet<&'a str> {
    let Some(script) = script_content else {
        return FxHashSet::default();
    };

    summary
        .import_statements
        .iter()
        .flat_map(|imp| {
            let text = script
                .get(imp.start as usize..imp.end as usize)
                .unwrap_or("");
            extract_import_names(text)
        })
        .collect()
}

/// Syntactic value imports can still resolve to type-only exports when
/// `verbatimModuleSyntax` is disabled. Until either the script or template
/// proves a runtime read, generated code must not use those locals as values.
/// Keep one generation's parser result for its read-only consumers.
/// Names keep their authored spelling and the existing binding-table filter.
#[derive(Default)]
pub(super) struct ValueImportNames {
    pub(super) bindings: FxHashSet<CompactString>,
    pub(super) without_script_value_use: FxHashSet<String>,
}

pub(super) struct TypeOnlyImportPlan<'a> {
    pub(super) script_content: Option<&'a str>,
    /// Some(empty) is a completed extraction, distinct from the late fallback.
    pub(super) prepared: Option<FxHashSet<CompactString>>,
}

pub(super) struct ImportNames<'a> {
    pub(super) values: ValueImportNames,
    pub(super) type_only: TypeOnlyImportPlan<'a>,
}

impl<'a> ImportNames<'a> {
    pub(super) fn collect(
        summary: &Croquis,
        script_content: Option<&'a str>,
        usage: &IdentifierUsage,
    ) -> Self {
        // This existing macro fact guarantees the later reader gate. Other
        // readers keep their original late extraction and admission order.
        let mut prepared = summary
            .macros
            .define_props()
            .is_some()
            .then(FxHashSet::default);
        let bindings = script_content
            .map(|script| collect_value_import_binding_names(summary, script, prepared.as_mut()))
            .unwrap_or_default();
        let without_script_value_use = bindings
            .iter()
            .filter(|name| !usage.value_refs.contains(*name))
            .map(|name| String::from(name.as_str()))
            .collect();
        Self {
            values: ValueImportNames {
                bindings,
                without_script_value_use,
            },
            type_only: TypeOnlyImportPlan {
                script_content,
                prepared,
            },
        }
    }
}

pub(super) fn collect_setup_binding_anchor_names(
    summary: &Croquis,
    imports: &ValueImportNames,
    usage: &IdentifierUsage,
    template_referenced_names: Option<&FxHashSet<String>>,
    template_value_names: &FxHashSet<String>,
) -> Vec<String> {
    let mut binding_names: Vec<String> =
        crate::virtual_ts::script_facts::with_bindings(summary, |bindings| {
            bindings
                .typed()
                .map(|(name, _)| String::from(name))
                .filter(|name| {
                    // Typed bindings include quoted prop keys. Only lexical
                    // identifiers can be read by the generated `void name` anchor.
                    is_identifier_name(name)
                        && template_referenced_names
                            .is_none_or(|names| names.contains(name.as_str()))
                })
                .collect()
        });
    // Croquis records a reactive-props rest binding on the macro summary,
    // outside the typed binding table. Anchor it only when the template reads
    // that local so unused rest bindings still produce TS6133.
    if let Some(template_referenced_names) = template_referenced_names
        && let Some(rest_id) = summary
            .macros
            .props_destructure()
            .and_then(|destructure| destructure.rest_id.as_deref())
        && template_referenced_names.contains(rest_id)
        && is_identifier_name(rest_id)
    {
        binding_names.push(String::from(rest_id));
    }
    binding_names.retain(|name| {
        !usage.const_enums.contains(name.as_str())
            && (!imports.without_script_value_use.contains(name.as_str())
                || template_value_names.contains(name.as_str()))
    });
    binding_names.sort_unstable();
    binding_names.dedup();
    binding_names
}

pub(super) fn collect_value_import_binding_names(
    summary: &Croquis,
    script: &str,
    mut type_only: Option<&mut FxHashSet<CompactString>>,
) -> FxHashSet<CompactString> {
    let mut names = summary
        .import_statements
        .iter()
        .flat_map(|imp| {
            let text = script
                .get(imp.start as usize..imp.end as usize)
                .unwrap_or("");
            extract_import_names_with_types(text, type_only.as_deref_mut())
                .into_iter()
                .map(CompactString::new)
                .collect::<Vec<_>>()
        })
        .peekable();
    // Preserve the allocation-free fact path for scripts with no value imports.
    if names.peek().is_none() {
        return FxHashSet::default();
    }
    crate::virtual_ts::script_facts::with_bindings(summary, |bindings| {
        names
            .filter(|name| bindings.contains_binding(name))
            .collect()
    })
}

#[derive(Default)]
pub(super) struct IdentifierUsage {
    value_refs: FxHashSet<CompactString>,
    const_enums: FxHashSet<CompactString>,
    type_depth: u32,
}

impl<'a> Visit<'a> for IdentifierUsage {
    fn visit_ts_enum_declaration(&mut self, declaration: &TSEnumDeclaration<'a>) {
        if declaration.r#const {
            self.const_enums
                .insert(CompactString::new(declaration.id.name.as_str()));
        }
        walk::walk_ts_enum_declaration(self, declaration);
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.type_depth == 0 {
            self.value_refs
                .insert(CompactString::new(ident.name.as_str()));
        }
    }

    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        self.type_depth += 1;
        walk::walk_ts_type_reference(self, ty);
        self.type_depth -= 1;
    }

    fn visit_ts_type_query_expr_name(&mut self, name: &TSTypeQueryExprName<'a>) {
        record_type_query_root(name, &mut self.value_refs);
        walk::walk_ts_type_query_expr_name(self, name);
    }
}

pub(super) fn collect_identifier_usage(program: &Program<'_>) -> IdentifierUsage {
    let mut usage = IdentifierUsage::default();
    usage.visit_program(program);
    usage
}

fn record_type_name_root(name: &TSTypeName<'_>, refs: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeName::IdentifierReference(ident) => {
            refs.insert(CompactString::new(ident.name.as_str()));
        }
        TSTypeName::QualifiedName(qualified) => record_type_name_root(&qualified.left, refs),
        TSTypeName::ThisExpression(_) => {}
    }
}

fn record_type_query_root(name: &TSTypeQueryExprName<'_>, refs: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeQueryExprName::IdentifierReference(ident) => {
            refs.insert(CompactString::new(ident.name.as_str()));
        }
        TSTypeQueryExprName::QualifiedName(qualified) => {
            record_type_name_root(&qualified.left, refs);
        }
        TSTypeQueryExprName::TSImportType(_) => {}
        _ => {}
    }
}

/// Extract imported identifier names from an import statement string.
/// Handles `import { a, b as c } from "..."` and `import D from "..."`.
/// Returns the local names (e.g., `["a", "c", "D"]`).
pub(crate) fn extract_declared_name(stub: &str) -> Option<&str> {
    let stub = stub.trim_start();
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '<' | '(' | ':' | '=' | ';')
            })
            .unwrap_or(rest.len());
        let name = rest.get(..end).unwrap_or_default().trim();
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}

fn extract_import_names(import_text: &str) -> Vec<&str> {
    extract_import_names_with_types(import_text, None)
}

fn extract_import_names_with_types<'a>(
    import_text: &'a str,
    mut type_only: Option<&mut FxHashSet<CompactString>>,
) -> Vec<&'a str> {
    #[cfg(test)]
    import_parse_count(true);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, import_text, SourceType::ts()).parse();
    let Some(Statement::ImportDeclaration(declaration)) = parsed.program.body.first() else {
        return Vec::new();
    };
    let whole_type = declaration.import_kind == ImportOrExportKind::Type;
    if whole_type && type_only.is_none() {
        return Vec::new();
    }
    let mut values = Vec::new();
    for specifier in declaration.specifiers.iter().flatten() {
        let (span, is_type) = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if whole_type || specifier.import_kind == ImportOrExportKind::Type =>
            {
                (specifier.local.span, true)
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if specifier.import_kind == ImportOrExportKind::Value =>
            {
                (specifier.local.span, false)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                (specifier.local.span, whole_type)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                (specifier.local.span, whole_type)
            }
            ImportDeclarationSpecifier::ImportSpecifier(_) => continue,
        };
        if let Some(name) = import_text.get(span.start as usize..span.end as usize) {
            if is_type {
                if let Some(names) = type_only.as_mut() {
                    names.insert(CompactString::new(name));
                }
            } else {
                values.push(name);
            }
        }
    }
    values
}

#[cfg(test)]
pub(super) fn extract_import_facts_for_test(source: &str) -> (Vec<&str>, FxHashSet<CompactString>) {
    let mut types = FxHashSet::default();
    let values = extract_import_names_with_types(source, Some(&mut types));
    (values, types)
}

#[cfg(test)]
pub(super) fn import_parse_count(increment: bool) -> usize {
    thread_local! {
        static COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    COUNT.with(|count| {
        if increment {
            count.set(count.get() + 1);
        }
        count.get()
    })
}

#[cfg(test)]
mod tests {
    use super::extract_import_names;

    #[test]
    fn mixed_default_and_named_imports_keep_every_value_binding() {
        assert_eq!(
            extract_import_names("import Badge, { helper as local, type Props } from 'pkg'"),
            ["Badge", "local"]
        );
    }

    #[test]
    fn mixed_default_and_namespace_imports_keep_both_bindings() {
        assert_eq!(
            extract_import_names("import Badge, * as badgeNs from 'pkg'"),
            ["Badge", "badgeNs"]
        );
        assert!(extract_import_names("import type Badge from 'pkg'").is_empty());
    }

    #[test]
    fn import_syntax_keeps_complete_authored_local_name_lists() {
        for (source, expected) in [
            ("", vec![]),
            ("import 'side-effect'", vec![]),
            ("import type * as Types from 'pkg'", vec![]),
            (
                "import { type Model, value as local } from 'pkg'",
                vec!["local"],
            ),
            (
                "import Value, { type Model, value } from 'pkg'",
                vec!["Value", "value"],
            ),
            (
                "import Value, * as space from 'pkg'",
                vec!["Value", "space"],
            ),
            (
                "import { café as résumé, $value } from 'pkg'",
                vec!["résumé", "$value"],
            ),
            (
                r"import { value as \u006cocal } from 'pkg'",
                vec![r"\u006cocal"],
            ),
            (
                "import data from './data.json' with { type: 'json' }",
                vec!["data"],
            ),
            ("export { value } from 'pkg'", vec![]),
        ] {
            assert_eq!(extract_import_names(source), expected, "{source}");
        }
    }
}
