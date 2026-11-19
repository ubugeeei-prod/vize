//! What a source path holds, selected once from its file name.
//!
//! Products route a path by its [`FileKind`] and never match file extensions
//! themselves; the extension table lives in [`crate::registry`]. The kind is
//! the path-level selector of the input axes: the container and markup a file
//! is parsed with. The axes that need configuration or file content are
//! resolved later into a [`FileDialect`](super::FileDialect).

use std::path::Path;

use super::Lang;

/// The input a source path holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// A Vue single-file component.
    Sfc,
    /// A standalone HTML document: in-DOM Vue or petite-vue.
    Document,
    /// A TSRX module: TypeScript whose expression positions may hold markup.
    Tsrx,
    /// A JavaScript or TypeScript module.
    Script(ScriptKind),
}

/// The language and module shape a script file name selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptKind {
    pub lang: Lang,
    /// JSX syntax is enabled.
    pub jsx: bool,
    /// A TypeScript declaration file.
    pub declaration: bool,
    pub module: ModuleFormat,
}

/// How a script file's module system is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleFormat {
    /// The nearest `package.json` `type` field decides.
    Package,
    /// Always an ES module.
    Esm,
    /// Always CommonJS.
    CommonJs,
}

impl FileKind {
    /// The kind a file name selects, or `None` when it names no Vize source
    /// input. Matching is case-sensitive and needs a non-empty stem.
    #[must_use]
    pub fn of_name(name: &str) -> Option<Self> {
        crate::registry::file_kind(name)
    }

    /// The kind of the file `path` names.
    #[must_use]
    pub fn of_path(path: &Path) -> Option<Self> {
        path.file_name()?.to_str().and_then(Self::of_name)
    }

    /// Every recognized extension, without its dot, with the kind it selects
    /// for a non-declaration file. Display and glob lists derive from this.
    #[must_use]
    pub const fn extensions() -> &'static [(&'static str, Self)] {
        crate::registry::FILE_KINDS
    }

    /// The script shape, for a script file.
    #[must_use]
    pub const fn script(self) -> Option<ScriptKind> {
        match self {
            Self::Script(script) => Some(script),
            Self::Sfc | Self::Document | Self::Tsrx => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileKind, Lang, ModuleFormat, ScriptKind};
    use std::path::Path;

    const fn script(lang: Lang, jsx: bool, declaration: bool, module: ModuleFormat) -> FileKind {
        FileKind::Script(ScriptKind {
            lang,
            jsx,
            declaration,
            module,
        })
    }

    #[test]
    fn every_source_name_selects_one_kind() {
        let cases = [
            ("App.vue", Some(FileKind::Sfc)),
            ("App.ce.vue", Some(FileKind::Sfc)),
            ("index.html", Some(FileKind::Document)),
            ("index.htm", Some(FileKind::Document)),
            ("Counter.tsrx", Some(FileKind::Tsrx)),
            (
                "main.js",
                Some(script(Lang::Js, false, false, ModuleFormat::Package)),
            ),
            (
                "main.mjs",
                Some(script(Lang::Js, false, false, ModuleFormat::Esm)),
            ),
            (
                "main.cjs",
                Some(script(Lang::Js, false, false, ModuleFormat::CommonJs)),
            ),
            (
                "View.jsx",
                Some(script(Lang::Js, true, false, ModuleFormat::Package)),
            ),
            (
                "main.ts",
                Some(script(Lang::Ts, false, false, ModuleFormat::Package)),
            ),
            (
                "main.mts",
                Some(script(Lang::Ts, false, false, ModuleFormat::Esm)),
            ),
            (
                "main.cts",
                Some(script(Lang::Ts, false, false, ModuleFormat::CommonJs)),
            ),
            (
                "View.tsx",
                Some(script(Lang::Ts, true, false, ModuleFormat::Package)),
            ),
            (
                "env.d.ts",
                Some(script(Lang::Ts, false, true, ModuleFormat::Package)),
            ),
            (
                "env.d.mts",
                Some(script(Lang::Ts, false, true, ModuleFormat::Esm)),
            ),
            (
                "env.d.cts",
                Some(script(Lang::Ts, false, true, ModuleFormat::CommonJs)),
            ),
            (
                "view.d.tsx",
                Some(script(Lang::Ts, true, false, ModuleFormat::Package)),
            ),
            ("App.VUE", None),
            ("style.css", None),
            ("data.json", None),
            (".vue", None),
            ("vue", None),
            ("main.", None),
        ];
        for (name, expected) in cases {
            assert_eq!(FileKind::of_name(name), expected, "{name}");
        }
    }

    #[test]
    fn paths_use_their_file_name_only() {
        assert_eq!(
            FileKind::of_path(Path::new("src/components.vue/Counter.tsrx")),
            Some(FileKind::Tsrx)
        );
        assert_eq!(FileKind::of_path(Path::new("src.tsrx/notes.txt")), None);
        assert_eq!(FileKind::of_path(Path::new("/")), None);
    }

    #[test]
    fn the_extension_table_agrees_with_name_resolution() {
        let extensions: std::vec::Vec<&str> = FileKind::extensions()
            .iter()
            .map(|(extension, _)| *extension)
            .collect();
        assert_eq!(
            extensions,
            [
                "vue", "html", "htm", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "tsrx"
            ]
        );
        for &(extension, kind) in FileKind::extensions() {
            assert_eq!(
                FileKind::of_name(&crate::cstr!("a.{extension}")),
                Some(kind)
            );
            assert_eq!(kind.script().filter(|script| script.declaration), None);
        }
        assert_eq!(FileKind::Tsrx.script(), None);
    }
}
