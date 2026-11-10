use std::path::Path;

use vize_l0::dialect::FileKind;

/// Data formats the formatter reads beside Vize source kinds.
const DATA_EXTENSIONS: &[&str] = &["json", "jsonc", "yaml", "yml", "md", "markdown"];

pub(super) const FORMAT_EXTENSIONS_DISPLAY: &str = ".vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown";

pub(super) const FORMAT_PATTERN_HELP: &str = "Vue/JS/TS/JSON/JSONC globs; YAML/Markdown formatting is not implemented\n\n[default: ./**/*.vue ./**/*.js ./**/*.mjs ./**/*.cjs ./**/*.ts ./**/*.mts ./**/*.cts ./**/*.jsx ./**/*.tsx ./**/*.json ./**/*.jsonc]";

/// The source kinds Glyph formats.
const fn formats_kind(kind: FileKind) -> bool {
    match kind {
        FileKind::Sfc | FileKind::Script(_) => true,
        FileKind::Document | FileKind::Tsrx => false,
    }
}

/// A file the formatter accepts: a formatted source kind or a data format.
pub(super) fn is_format_target(path: &Path) -> bool {
    FileKind::of_path(path).map_or_else(|| data_extension(path).is_some(), formats_kind)
}

/// A data format with no formatter yet: refused when named, skipped when
/// discovered.
pub(super) fn is_unimplemented_document(path: &Path) -> bool {
    data_extension(path).is_some_and(is_unimplemented_document_extension)
}

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub(super) fn default_fmt_patterns() -> Vec<std::string::String> {
    let sources = FileKind::extensions()
        .iter()
        .filter(|(_, kind)| formats_kind(*kind))
        .map(|(extension, _)| *extension);
    let data = DATA_EXTENSIONS
        .iter()
        .copied()
        .filter(|extension| !is_unimplemented_document_extension(extension));
    sources
        .chain(data)
        .map(|extension| vize_l0::cstr!("./**/*.{extension}").into())
        .collect()
}

fn data_extension(path: &Path) -> Option<&str> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| DATA_EXTENSIONS.contains(extension))
}

fn is_unimplemented_document_extension(extension: &str) -> bool {
    matches!(extension, "yaml" | "yml" | "md" | "markdown")
}

#[cfg(test)]
mod tests {
    use super::{
        DATA_EXTENSIONS, FORMAT_EXTENSIONS_DISPLAY, FileKind, default_fmt_patterns, formats_kind,
        is_format_target, is_unimplemented_document,
    };
    use std::path::Path;

    #[test]
    fn default_patterns_cover_only_implemented_format_extensions() {
        let expected = [
            "vue", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "json", "jsonc",
        ]
        .iter()
        .map(|extension| vize_l0::cstr!("./**/*.{extension}"))
        .collect::<Vec<_>>();

        assert_eq!(default_fmt_patterns(), expected);
        for file in ["a.yaml", "a.yml", "a.md", "a.markdown"] {
            assert!(is_format_target(Path::new(file)), "{file}");
            assert!(is_unimplemented_document(Path::new(file)), "{file}");
        }
    }

    #[test]
    fn format_targets_follow_the_file_kind() {
        for file in ["App.vue", "main.ts", "env.d.ts", "View.tsx", "a.json"] {
            assert!(is_format_target(Path::new(file)), "{file}");
            assert!(!is_unimplemented_document(Path::new(file)), "{file}");
        }
        for file in ["index.html", "Counter.tsrx", "style.css", "App.VUE", ".vue"] {
            assert!(!is_format_target(Path::new(file)), "{file}");
        }
    }

    #[test]
    fn display_text_mentions_each_format_extension() {
        let display_tokens = FORMAT_EXTENSIONS_DISPLAY
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        let sources = FileKind::extensions()
            .iter()
            .filter(|(_, kind)| formats_kind(*kind))
            .map(|(extension, _)| extension);
        for extension in sources.chain(DATA_EXTENSIONS) {
            assert!(
                display_tokens.contains(extension),
                "missing .{extension} from display text"
            );
        }
    }
}
