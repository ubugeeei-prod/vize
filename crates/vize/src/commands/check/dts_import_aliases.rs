use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{FxHashMap, String, ToCompactString, cstr};

use super::dts_rewrite::rewrite_relative_specifier;

#[derive(Default)]
pub(super) struct ImportTypeAliases {
    aliases: FxHashMap<String, ImportTypeAlias>,
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
    if aliases.aliases.is_empty() {
        return type_annotation.to_compact_string();
    }

    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;
    while i < type_annotation.len() {
        let ch = type_annotation[i..].chars().next().unwrap();
        if ch == '\'' || ch == '"' || ch == '`' {
            i = copy_quoted(type_annotation, i, &mut out);
            continue;
        }
        if !is_identifier_start(ch) {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let start = i;
        i += ch.len_utf8();
        while i < type_annotation.len() {
            let ch = type_annotation[i..].chars().next().unwrap();
            if !is_identifier_char(ch) {
                break;
            }
            i += ch.len_utf8();
        }

        let ident = &type_annotation[start..i];
        let next = type_annotation[i..].chars().next();
        if let Some(alias) = aliases.aliases.get(ident) {
            match alias {
                ImportTypeAlias::Namespace { module } if next == Some('.') => {
                    out.push_str(&render_namespace_reference(module.as_str()));
                    continue;
                }
                ImportTypeAlias::Named { module, imported }
                    if previous_non_whitespace(type_annotation, start) != Some('.') =>
                {
                    out.push_str(&render_named_reference(module.as_str(), imported.as_str()));
                    continue;
                }
                ImportTypeAlias::Default { module }
                    if previous_non_whitespace(type_annotation, start) != Some('.') =>
                {
                    out.push_str(&render_named_reference(module.as_str(), "default"));
                    continue;
                }
                _ => {}
            }
        }
        out.push_str(ident);
    }

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

fn copy_quoted(input: &str, start: usize, out: &mut String) -> usize {
    let quote = input[start..].chars().next().unwrap();
    let mut i = start;
    while i < input.len() {
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
        if ch == '\\' && i < input.len() {
            let escaped = input[i..].chars().next().unwrap();
            out.push(escaped);
            i += escaped.len_utf8();
            continue;
        }
        if ch == quote && i > start + quote.len_utf8() {
            break;
        }
    }
    i
}

fn render_named_reference(module: &str, imported: &str) -> String {
    cstr!("import('{module}').{imported}")
}

fn render_namespace_reference(module: &str) -> String {
    cstr!("import('{module}')")
}

fn previous_non_whitespace(input: &str, index: usize) -> Option<char> {
    input[..index].chars().rev().find(|ch| !ch.is_whitespace())
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_char)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_identifier_char(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
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
}
