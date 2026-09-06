use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::DocumentLink;

use super::DocumentLinkService;

#[derive(Debug, PartialEq)]
struct ModuleSpecifier {
    path: String,
    start: usize,
    end: usize,
}

/// Collect import statement links from script content.
pub(super) fn collect_import_links(
    script: &str,
    source_type: SourceType,
    base_offset: usize,
    full_content: &str,
    base_path: Option<&Path>,
    links: &mut Vec<DocumentLink>,
) {
    for specifier in collect_static_module_specifiers(script, source_type) {
        if (specifier.path.starts_with('.') || specifier.path.starts_with('/'))
            && let Some(target) = DocumentLinkService::resolve_path(&specifier.path, base_path)
        {
            let abs_start = base_offset + specifier.start;
            let abs_end = base_offset + specifier.end;
            links.push(DocumentLinkService::create_link(
                full_content,
                abs_start,
                abs_end,
                target,
            ));
        }
    }
}

pub(super) fn script_source_type(lang: Option<&str>) -> SourceType {
    match lang.unwrap_or("js") {
        "ts" => SourceType::ts().with_module(true),
        "tsx" => SourceType::tsx().with_module(true),
        "jsx" => SourceType::jsx().with_module(true),
        _ => SourceType::mjs(),
    }
}

fn collect_static_module_specifiers(script: &str, source_type: SourceType) -> Vec<ModuleSpecifier> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, source_type).parse();
    if parsed.panicked {
        return Vec::new();
    }

    parsed
        .program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ImportDeclaration(decl) => Some(module_specifier(
                decl.source.value.as_str(),
                decl.source.span.start,
                decl.source.span.end,
            )),
            Statement::ExportNamedDeclaration(decl) => decl.source.as_ref().map(|source| {
                module_specifier(source.value.as_str(), source.span.start, source.span.end)
            }),
            Statement::ExportAllDeclaration(decl) => Some(module_specifier(
                decl.source.value.as_str(),
                decl.source.span.start,
                decl.source.span.end,
            )),
            _ => None,
        })
        .collect()
}

fn module_specifier(path: &str, start: u32, end: u32) -> ModuleSpecifier {
    ModuleSpecifier {
        path: path.to_string(),
        start: start as usize,
        end: end as usize,
    }
}

/// Extract string literal from text.
/// Returns (content, start, end_offset) where offsets include quotes.
pub(super) fn extract_string_literal(text: &str) -> Option<(String, usize, usize)> {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let quote = bytes[0] as char;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == quote as u8 && (i == 1 || bytes[i - 1] != b'\\') {
            let content = text[1..i].to_string();
            return Some((content, 0, i + 1));
        }
        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{collect_static_module_specifiers, extract_string_literal, script_source_type};

    #[test]
    fn extracts_static_module_specifier_ranges() {
        let source = concat!(
            "import { ref } from \"./utils\"\n",
            "import \"./styles.css\"\n",
            "export { foo } from './foo'\n",
            "export * from './all'\n",
        );

        let specifiers = collect_static_module_specifiers(source, script_source_type(Some("ts")));
        assert_eq!(
            specifiers
                .iter()
                .map(|specifier| &specifier.path)
                .collect::<Vec<_>>(),
            vec!["./utils", "./styles.css", "./foo", "./all"]
        );
        assert_eq!(
            specifiers
                .iter()
                .map(|specifier| &source[specifier.start..specifier.end])
                .collect::<Vec<_>>(),
            vec![r#""./utils""#, r#""./styles.css""#, "'./foo'", "'./all'"]
        );
    }

    #[test]
    fn extracts_string_literals() {
        let (content, start, end) = extract_string_literal(r#""hello""#).unwrap();
        assert_eq!(content, "hello");
        assert_eq!(start, 0);
        assert_eq!(end, 7);

        let (content, _, _) = extract_string_literal("'world'").unwrap();
        assert_eq!(content, "world");
    }

    #[test]
    fn extracts_multiline_imports_and_skips_inactive_specifiers() {
        let script = concat!(
            "import {\n",
            "  ref, /* from './ghost' */\n",
            "} from './real'\n",
            "const note = \"export { Hidden } from './hidden'\"\n",
            "// export { Line } from './line'\n",
            "export {\n",
            "  ref as value,\n",
            "} from './exported'\n",
        );

        let specifiers = collect_static_module_specifiers(script, script_source_type(Some("ts")));
        assert_eq!(
            specifiers
                .iter()
                .map(|specifier| &specifier.path)
                .collect::<Vec<_>>(),
            vec!["./real", "./exported"]
        );
        assert_eq!(
            specifiers
                .iter()
                .map(|specifier| &script[specifier.start..specifier.end])
                .collect::<Vec<_>>(),
            vec!["'./real'", "'./exported'"]
        );
    }
}
