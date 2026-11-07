//! L0 input-dialect registry.
//!
//! This is the only L0 module that names concrete input variants. The
//! resolved [`FileDialect`] crosses every later level as one value; consumers
//! ask their own level registry for hooks instead of rematching these axes.

use crate::{
    FileDialect, Lang, ProfileKind, TemplateSyntaxMode,
    config::VueVersion,
    dialect::{VueDialect, detect_petite_vue_document},
};

/// Resolve a Vue component descriptor from its configured axes.
#[must_use]
pub const fn component(
    version: VueVersion,
    framework: VueDialect,
    syntax: TemplateSyntaxMode,
    lang: Lang,
) -> FileDialect {
    FileDialect::resolved(version, framework, syntax, ProfileKind::Component, lang)
}

/// Resolve a standalone HTML descriptor after structural petite-vue
/// detection. An explicit framework selector always wins.
#[must_use]
pub fn document(
    configured: Option<VueDialect>,
    content: &str,
    syntax: TemplateSyntaxMode,
    lang: Lang,
) -> FileDialect {
    FileDialect::resolved(
        VueVersion::V3,
        standalone_html_dialect(configured, content),
        syntax,
        ProfileKind::Document,
        lang,
    )
}

/// Resolve the framework selector for a standalone HTML document.
#[must_use]
pub fn standalone_html_dialect(configured: Option<VueDialect>, content: &str) -> VueDialect {
    match configured {
        Some(dialect) => dialect,
        None if detect_petite_vue_document(content) => VueDialect::PetiteVue,
        None => VueDialect::Vue,
    }
}

/// Whether compatibility quirks are active for this file.
#[must_use]
pub const fn is_quirks(syntax: TemplateSyntaxMode) -> bool {
    matches!(syntax, TemplateSyntaxMode::Quirks)
}

/// Whether the descriptor selects a Vue 3 component, independent of recovery
/// policy and embedded language.
#[must_use]
pub const fn is_vue3_component(dialect: FileDialect) -> bool {
    matches!(
        (dialect.version(), dialect.framework(), dialect.profile()),
        (VueVersion::V3, VueDialect::Vue, ProfileKind::Component)
    )
}

/// Whether this is the standard recovery policy for a Vue component at the
/// requested version. Language remains an independent resolved field.
#[must_use]
pub fn is_standard_vue_component(dialect: FileDialect, version: VueVersion) -> bool {
    dialect.version() == version
        && matches!(dialect.framework(), VueDialect::Vue)
        && matches!(dialect.syntax(), TemplateSyntaxMode::Standard)
        && matches!(dialect.profile(), ProfileKind::Component)
}

/// Whether this is a petite-vue standalone document.
#[must_use]
pub const fn is_petite_document(dialect: FileDialect) -> bool {
    matches!(
        (dialect.framework(), dialect.profile()),
        (VueDialect::PetiteVue, ProfileKind::Document)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_resolves_each_axis_once() {
        let vue1 = component(
            VueVersion::V1,
            VueDialect::Vue,
            TemplateSyntaxMode::Quirks,
            Lang::Ts,
        );
        assert_eq!(vue1.version(), VueVersion::V1);
        assert!(is_quirks(vue1.syntax()));
        assert!(!is_vue3_component(vue1));

        let petite = document(
            None,
            r#"<script src="petite-vue"></script>"#,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        assert!(is_petite_document(petite));
    }
}
