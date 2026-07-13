//! Cross-file projections from the neutral owned module snapshot.
//!
//! The OXC program is consumed by `vize_module` while its allocator is alive.
//! This adapter interprets only owned operations and import bindings; it must
//! never parse `ModuleSyntax::source` again.

#[path = "module_facts/croquis.rs"]
mod croquis;
#[path = "module_facts/effect.rs"]
mod effect;

use vize_carton::{CompactString, FxHashMap};
use vize_module::{
    ModuleDocument, ModuleExpression, ModuleExpressionKind, ModuleImportBindingKind, ModulePattern,
    ModuleSyntax,
};

pub(super) use croquis::project_raw_croquis;
pub(super) use effect::module_effect_summary;

use crate::rules::cross_file_reactivity::store_detection::StoreFactories;

pub(super) fn module_store_factories(document: &ModuleDocument) -> StoreFactories {
    let mut stores = StoreFactories::default();
    for module in &document.modules {
        let aliases = ModuleAliases::new(module);
        for operation in &module.operations.operations {
            let vize_module::ModuleOperationKind::Binding {
                pattern,
                initializer: Some(initializer),
                ..
            } = &operation.kind
            else {
                continue;
            };
            let Some(call) = CallView::from_expression(initializer, &aliases) else {
                continue;
            };
            if aliases.is_define_store_call(&call) {
                for name in pattern_names(pattern) {
                    stores.insert(name);
                }
            }
        }
    }
    stores
}

#[derive(Debug, Default)]
struct ModuleAliases {
    imported: FxHashMap<CompactString, ImportedName>,
    define_store_imported: bool,
}

#[derive(Debug)]
struct ImportedName {
    source: CompactString,
    imported: Option<CompactString>,
    namespace: bool,
}

impl ModuleAliases {
    fn new(module: &ModuleSyntax) -> Self {
        let mut aliases = Self::default();
        for import in &module.imports {
            for binding in &import.bindings {
                if binding.type_only {
                    continue;
                }
                let imported = binding.imported.as_deref().map(CompactString::new);
                if import.specifier.as_ref() == "pinia"
                    && imported.as_deref() == Some("defineStore")
                {
                    aliases.define_store_imported = true;
                }
                aliases.imported.insert(
                    CompactString::new(binding.local.as_ref()),
                    ImportedName {
                        source: CompactString::new(import.specifier.as_ref()),
                        imported,
                        namespace: binding.kind == ModuleImportBindingKind::Namespace,
                    },
                );
            }
        }
        aliases
    }

    fn canonical_path(&self, expression: &ModuleExpression) -> Option<Vec<CompactString>> {
        let raw = expression_path(expression)?;
        let first = raw.first()?;
        let Some(import) = self.imported.get(first.as_str()) else {
            return Some(raw);
        };
        if import.namespace {
            return Some(raw.into_iter().skip(1).collect());
        }
        let mut canonical = Vec::with_capacity(raw.len());
        canonical.push(import.imported.clone().unwrap_or_else(|| first.clone()));
        canonical.extend(raw.into_iter().skip(1));
        Some(canonical)
    }

    fn source_for_local(&self, local: &str) -> Option<&str> {
        self.imported
            .get(local)
            .map(|import| import.source.as_str())
    }

    fn is_define_store_call(&self, call: &CallView<'_>) -> bool {
        if self.define_store_imported {
            return self
                .imported
                .get(call.raw_callee.as_str())
                .is_some_and(|import| {
                    import.source == "pinia" && import.imported.as_deref() == Some("defineStore")
                });
        }
        call.raw_callee == "defineStore"
    }
}

struct CallView<'a> {
    callee: CompactString,
    raw_callee: CompactString,
    arguments: &'a [ModuleExpression],
}

impl<'a> CallView<'a> {
    fn from_expression(expression: &'a ModuleExpression, aliases: &ModuleAliases) -> Option<Self> {
        let ModuleExpressionKind::Call {
            callee, arguments, ..
        } = &expression.kind
        else {
            return None;
        };
        let raw = expression_path(callee)?;
        let canonical = aliases.canonical_path(callee)?;
        Some(Self {
            callee: canonical.last()?.clone(),
            raw_callee: raw.last()?.clone(),
            arguments,
        })
    }
}

fn expression_path(expression: &ModuleExpression) -> Option<Vec<CompactString>> {
    match &expression.kind {
        ModuleExpressionKind::Identifier(name) => Some(vec![CompactString::new(name.as_ref())]),
        ModuleExpressionKind::Path(path) => Some(
            path.iter()
                .map(|segment| CompactString::new(segment.as_ref()))
                .collect(),
        ),
        _ => None,
    }
}

fn pattern_names(pattern: &ModulePattern) -> Vec<CompactString> {
    let mut names = Vec::new();
    collect_pattern_names(pattern, &mut names);
    names
}

fn collect_pattern_names(pattern: &ModulePattern, names: &mut Vec<CompactString>) {
    match pattern {
        ModulePattern::Identifier(name) => names.push(CompactString::new(name.as_ref())),
        ModulePattern::Path(_) | ModulePattern::Unknown { .. } => {}
        ModulePattern::Object(properties) => {
            for property in properties {
                collect_pattern_names(&property.value, names);
            }
        }
        ModulePattern::Array(items) => {
            for item in items.iter().flatten() {
                collect_pattern_names(item, names);
            }
        }
        ModulePattern::Rest(pattern) => collect_pattern_names(pattern, names),
        ModulePattern::Assignment { binding, .. } => collect_pattern_names(binding, names),
    }
}

fn first_pattern_name(pattern: &ModulePattern) -> Option<CompactString> {
    pattern_names(pattern).into_iter().next()
}

fn expression_text(expression: &ModuleExpression) -> CompactString {
    match &expression.kind {
        ModuleExpressionKind::Identifier(name) => CompactString::new(name.as_ref()),
        ModuleExpressionKind::Path(path) => CompactString::new(
            path.iter()
                .map(|part| part.as_ref())
                .collect::<Vec<_>>()
                .join("."),
        ),
        ModuleExpressionKind::Literal { text, .. } | ModuleExpressionKind::Unknown(text) => {
            CompactString::new(text.as_ref())
        }
        _ => CompactString::new("<expression>"),
    }
}
