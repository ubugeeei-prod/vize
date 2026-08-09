use std::path::Path;

pub(super) const CHECK_INPUTS_DISPLAY: &str =
    ".vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts";

const CHECK_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "mts", "cts"];
const JAVASCRIPT_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs"];

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct CheckFileOptions {
    pub(super) include_js: bool,
    pub(super) include_jsx: bool,
}

pub(super) fn is_supported_check_file(path: &Path, options: CheckFileOptions) -> bool {
    is_declaration_path(path)
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                CHECK_EXTENSIONS.contains(&extension)
                    || (options.include_js && JAVASCRIPT_EXTENSIONS.contains(&extension))
                    || (options.include_jsx && extension == "jsx")
            })
}

pub(super) fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

#[cfg(test)]
mod tests {
    use super::{CHECK_INPUTS_DISPLAY, CheckFileOptions, is_supported_check_file};
    use std::path::Path;

    #[test]
    fn supported_check_files_cover_ts_family_and_optional_jsx() {
        for file in [
            "App.vue",
            "main.ts",
            "view.tsx",
            "worker.mts",
            "config.cts",
            "env.d.ts",
            "env.d.mts",
            "env.d.cts",
        ] {
            assert!(
                is_supported_check_file(Path::new(file), CheckFileOptions::default()),
                "{file}"
            );
        }

        assert!(!is_supported_check_file(
            Path::new("view.jsx"),
            CheckFileOptions::default()
        ));
        assert!(is_supported_check_file(
            Path::new("view.jsx"),
            CheckFileOptions {
                include_jsx: true,
                ..Default::default()
            }
        ));
        for file in ["main.js", "view.jsx", "module.mjs", "config.cjs"] {
            assert!(
                is_supported_check_file(
                    Path::new(file),
                    CheckFileOptions {
                        include_js: true,
                        ..Default::default()
                    }
                ),
                "{file}"
            );
        }
    }

    #[test]
    fn display_text_mentions_each_supported_input_kind() {
        let display_tokens = CHECK_INPUTS_DISPLAY
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();

        for extension in ["vue", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"] {
            assert!(
                display_tokens.contains(&extension),
                "missing .{extension} from display text"
            );
        }
        assert!(
            CHECK_INPUTS_DISPLAY.contains(".d.ts"),
            "missing .d.ts from display text"
        );
        assert!(
            CHECK_INPUTS_DISPLAY.contains(".d.mts"),
            "missing .d.mts from display text"
        );
        assert!(
            CHECK_INPUTS_DISPLAY.contains(".d.cts"),
            "missing .d.cts from display text"
        );
    }
}
