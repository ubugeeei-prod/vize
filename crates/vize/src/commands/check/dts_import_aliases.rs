use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{FxHashMap, String, ToCompactString, cstr};

use super::dts_rewrite::rewrite_relative_specifier;

#[derive(Default)]
pub(super) struct ImportTypeAliases {
    aliases: FxHashMap<String, ImportTypeAlias>,
    /// Value imports, rewritten only as a `typeof` operand: `typeof Button`
    /// for `import Button from './Button.vue'` keeps naming the component
    /// once the annotation is copied out of its file.
    values: FxHashMap<String, ImportTypeAlias>,
}

enum ImportTypeAlias {
    Default { module: String },
    Named { module: String, imported: String },
    Namespace { module: String },
}

pub(super) fn collect_import_type_aliases(content: &str, source_dir: &Path) -> ImportTypeAliases {
    let mut aliases = ImportTypeAliases::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if ret.panicked {
        return aliases;
    }

    for statement in &ret.program.body {
        if let Statement::ImportDeclaration(import) = statement {
            collect_import_declaration(import, source_dir, &mut aliases);
        }
    }

    aliases
}

pub(super) fn rewrite_import_type_aliases(
    type_annotation: &str,
    aliases: &ImportTypeAliases,
) -> String {
    if (aliases.aliases.is_empty() && aliases.values.is_empty())
        || !super::dts_type_references::may_reference(type_annotation, |name| {
            aliases.aliases.contains_key(name) || aliases.values.contains_key(name)
        })
    {
        return type_annotation.to_compact_string();
    }

    let mut out = String::with_capacity(type_annotation.len());
    let mut offset = 0;
    for reference in super::dts_type_references::references(type_annotation) {
        let start = reference.span.start as usize;
        let end = reference.span.end as usize;
        let ident = reference.name.as_str();
        let alias = aliases.aliases.get(ident).or_else(|| {
            follows_typeof(type_annotation, start)
                .then(|| aliases.values.get(ident))
                .flatten()
        });
        let Some(alias) = alias else { continue };
        let replacement = match alias {
            ImportTypeAlias::Namespace { module } => render_namespace_reference(module.as_str()),
            ImportTypeAlias::Named { module, imported } => {
                render_named_reference(module.as_str(), imported.as_str())
            }
            ImportTypeAlias::Default { module } => {
                render_named_reference(module.as_str(), "default")
            }
        };
        out.push_str(type_annotation.get(offset..start).unwrap_or_default());
        out.push_str(&replacement);
        offset = end;
    }
    out.push_str(type_annotation.get(offset..).unwrap_or_default());
    out
}

fn collect_import_declaration(
    declaration: &ImportDeclaration,
    source_dir: &Path,
    aliases: &mut ImportTypeAliases,
) {
    let module = rewrite_relative_specifier(declaration.source.value.as_str(), source_dir);
    let Some(specifiers) = &declaration.specifiers else {
        return;
    };
    let declaration_is_type = declaration.import_kind.is_type();

    for specifier in specifiers {
        if !declaration_is_type {
            collect_value_specifier(specifier, module.as_str(), aliases);
        }
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if declaration_is_type || specifier.import_kind.is_type() =>
            {
                let Some(imported) = module_export_name(&specifier.imported) else {
                    continue;
                };
                push_named_alias(
                    specifier.local.name.as_str(),
                    module.as_str(),
                    imported,
                    aliases,
                );
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier)
                if declaration_is_type =>
            {
                push_default_alias(specifier.local.name.as_str(), module.as_str(), aliases);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier)
                if declaration_is_type =>
            {
                push_namespace_alias(specifier.local.name.as_str(), module.as_str(), aliases);
            }
            _ => {}
        }
    }
}

fn collect_value_specifier(
    specifier: &ImportDeclarationSpecifier<'_>,
    module: &str,
    aliases: &mut ImportTypeAliases,
) {
    let (local, alias) = match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(specifier)
            if !specifier.import_kind.is_type() =>
        {
            let Some(imported) =
                module_export_name(&specifier.imported).filter(|name| is_identifier(name))
            else {
                return;
            };
            (
                specifier.local.name.as_str(),
                ImportTypeAlias::Named {
                    module: module.into(),
                    imported: imported.into(),
                },
            )
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => (
            specifier.local.name.as_str(),
            ImportTypeAlias::Default {
                module: module.into(),
            },
        ),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => (
            specifier.local.name.as_str(),
            ImportTypeAlias::Namespace {
                module: module.into(),
            },
        ),
        ImportDeclarationSpecifier::ImportSpecifier(_) => return,
    };
    if is_identifier(local) {
        aliases.values.insert(local.to_compact_string(), alias);
    }
}

/// Whether the identifier at `start` is the operand of a `typeof` query.
fn follows_typeof(input: &str, start: usize) -> bool {
    input
        .get(..start)
        .unwrap_or_default()
        .trim_end()
        .strip_suffix("typeof")
        .is_some_and(|rest| !rest.chars().next_back().is_some_and(is_identifier_char))
}

fn push_named_alias(local: &str, module: &str, imported: &str, aliases: &mut ImportTypeAliases) {
    if is_identifier(local) && is_identifier(imported) {
        aliases.aliases.insert(
            local.to_compact_string(),
            ImportTypeAlias::Named {
                module: module.into(),
                imported: imported.into(),
            },
        );
    }
}

fn push_default_alias(local: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if is_identifier(local) {
        aliases.aliases.insert(
            local.to_compact_string(),
            ImportTypeAlias::Default {
                module: module.into(),
            },
        );
    }
}

fn push_namespace_alias(local: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if is_identifier(local) {
        aliases.aliases.insert(
            local.to_compact_string(),
            ImportTypeAlias::Namespace {
                module: module.into(),
            },
        );
    }
}

fn render_named_reference(module: &str, imported: &str) -> String {
    cstr!("import('{module}').{imported}")
}

fn render_namespace_reference(module: &str) -> String {
    cstr!("import('{module}')")
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_char)
}

fn is_identifier_start(ch: char) -> bool {
    oxc_syntax::identifier::is_identifier_start(ch)
}

fn is_identifier_char(ch: char) -> bool {
    oxc_syntax::identifier::is_identifier_part(ch)
}

fn module_export_name<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::StringLiteral(literal) => Some(literal.value.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{collect_import_type_aliases, rewrite_import_type_aliases};

    #[test]
    fn rewrites_import_type_aliases() {
        let content = r#"
import type { User, Wrapped as AliasedWrapped } from '../../app/types/user'
import type DefaultThing from '../../app/types/default'
import type * as Shared from '../../app/types/shared'
"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/.nuxt/types"));
        let rewritten = rewrite_import_type_aliases(
            "() => User | AliasedWrapped | DefaultThing | Shared.Thing",
            &aliases,
        );

        assert_eq!(
            rewritten.as_str(),
            "() => import('/workspace/app/types/user').User | import('/workspace/app/types/user').Wrapped | import('/workspace/app/types/default').default | import('/workspace/app/types/shared').Thing"
        );
    }

    #[test]
    fn ignores_value_imports_and_quoted_literals() {
        let content = r#"import { type User, value } from './types'"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/.nuxt/types"));
        let rewritten = rewrite_import_type_aliases("Record<'User', User> | value", &aliases);

        assert_eq!(
            rewritten.as_str(),
            "Record<'User', import('/workspace/.nuxt/types/types').User> | value"
        );
    }

    #[test]
    fn rewrites_value_imports_only_as_typeof_operands() {
        let content = r#"
import Button from './Button.vue'
import { Card as AliasedCard } from './card'
import * as Icons from './icons'
"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/src"));
        let rewritten = rewrite_import_type_aliases(
            "typeof Button | typeof AliasedCard | typeof Icons.Star | Button | typeofButton",
            &aliases,
        );

        assert_eq!(
            rewritten.as_str(),
            "typeof import('/workspace/src/Button.vue').default | typeof import('/workspace/src/card').Card | typeof import('/workspace/src/icons').Star | Button | typeofButton"
        );
    }

    #[test]
    fn collects_multiline_type_import_declarations() {
        let content = r#"
import type DefaultThing, {
  User,
  Wrapped as AliasedWrapped,
} from '../../app/types/user'
"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/.nuxt/types"));
        let rewritten =
            rewrite_import_type_aliases("DefaultThing | User | AliasedWrapped", &aliases);

        assert_eq!(
            rewritten.as_str(),
            "import('/workspace/app/types/user').default | import('/workspace/app/types/user').User | import('/workspace/app/types/user').Wrapped"
        );
    }

    #[test]
    fn lexical_rejection_keeps_escaped_import_bindings() {
        let aliases = collect_import_type_aliases(
            r"import type { Item as \u0041lias } from './types'",
            Path::new("/workspace"),
        );
        let rewritten = rewrite_import_type_aliases(r"Box<\u0041lias> & { Alias: 1 }", &aliases);
        assert_eq!(
            rewritten.as_str(),
            "Box<import('/workspace/types').Item> & { Alias: 1 }"
        );
    }
}
