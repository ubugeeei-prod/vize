//! One resolved input-dialect descriptor shared by every Davinci level.
//!
//! Configuration and structural detection may begin with separate selectors,
//! but a file crosses the L0 -> L1 boundary with exactly one of these values.
//! Later levels derive their registries and residual operations from it rather
//! than rematching independent version, framework, markup and language axes.

use crate::config::VueVersion;

use super::VueDialect;

/// Template syntax recovery policy.
///
/// Quirks is an explicit dialect feature rather than a parser-only boolean.
/// The legacy compiler re-exports this exact type for API compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TemplateSyntaxMode {
    /// Accept common recoverable syntax with warnings and repairs.
    #[default]
    Standard,
    /// Treat recoverable syntax as fatal.
    Strict,
    /// Preserve Vue-compatible recovery quirks without extra warnings.
    Quirks,
}

impl TemplateSyntaxMode {
    /// Whether compatibility quirks are active for this file.
    #[must_use]
    pub const fn is_quirks(self) -> bool {
        crate::registry::is_quirks(self)
    }
}

/// Markup host rules selected for the whole file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileKind {
    /// Standalone HTML / in-DOM rules.
    Document,
    /// Vue SFC component-template rules.
    Component,
}

/// Which JSX semantics a JSX/TSX module is compiled with, resolved once per
/// file from `compiler.jsxCompat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JsxSemantics {
    /// Vize: `&&`, `?:` and `.map()` lower structurally.
    #[default]
    Vize,
    /// `@vue/babel-plugin-jsx`.
    Babel,
    /// `vue-jsx-vapor`.
    VueJsxVapor,
}

/// Embedded source language selected from the file container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Js,
    Ts,
}

/// All input-dialect axes resolved once for one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileDialect {
    version: VueVersion,
    framework: VueDialect,
    syntax: TemplateSyntaxMode,
    profile: ProfileKind,
    lang: Lang,
}

impl FileDialect {
    /// Resolve an explicit Vue SFC descriptor.
    #[must_use]
    pub const fn component(
        version: VueVersion,
        framework: VueDialect,
        syntax: TemplateSyntaxMode,
        lang: Lang,
    ) -> Self {
        crate::registry::component(version, framework, syntax, lang)
    }

    /// Resolve a standalone HTML descriptor after structural petite-vue
    /// detection. An explicit framework selector always wins.
    #[must_use]
    pub fn document(
        configured: Option<VueDialect>,
        content: &str,
        syntax: TemplateSyntaxMode,
        lang: Lang,
    ) -> Self {
        crate::registry::document(configured, content, syntax, lang)
    }

    pub(crate) const fn resolved(
        version: VueVersion,
        framework: VueDialect,
        syntax: TemplateSyntaxMode,
        profile: ProfileKind,
        lang: Lang,
    ) -> Self {
        Self {
            version,
            framework,
            syntax,
            profile,
            lang,
        }
    }

    #[must_use]
    pub const fn version(self) -> VueVersion {
        self.version
    }

    #[must_use]
    pub const fn framework(self) -> VueDialect {
        self.framework
    }

    #[must_use]
    pub const fn syntax(self) -> TemplateSyntaxMode {
        self.syntax
    }

    #[must_use]
    pub const fn profile(self) -> ProfileKind {
        self.profile
    }

    #[must_use]
    pub const fn lang(self) -> Lang {
        self.lang
    }

    #[must_use]
    pub const fn is_vue3_component(self) -> bool {
        crate::registry::is_vue3_component(self)
    }

    /// Whether this is the standard recovery policy for a Vue component at
    /// the requested version. Language remains an independent resolved field.
    #[must_use]
    pub fn is_standard_vue_component(self, version: VueVersion) -> bool {
        crate::registry::is_standard_vue_component(self, version)
    }

    #[must_use]
    pub const fn is_petite_document(self) -> bool {
        crate::registry::is_petite_document(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_keeps_every_selected_axis() {
        let resolved = FileDialect::component(
            VueVersion::V1,
            VueDialect::Vue,
            TemplateSyntaxMode::Quirks,
            Lang::Ts,
        );
        assert_eq!(resolved.version(), VueVersion::V1);
        assert_eq!(resolved.framework(), VueDialect::Vue);
        assert_eq!(resolved.syntax(), TemplateSyntaxMode::Quirks);
        assert_eq!(resolved.profile(), ProfileKind::Component);
        assert_eq!(resolved.lang(), Lang::Ts);
        assert!(!resolved.is_vue3_component());
        assert!(resolved.syntax().is_quirks());
        assert!(!TemplateSyntaxMode::Standard.is_quirks());
    }

    #[test]
    fn document_resolves_petite_once_from_structure_or_config() {
        let detected = FileDialect::document(
            None,
            r#"<script src="petite-vue"></script>"#,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        assert!(detected.is_petite_document());

        let overridden = FileDialect::document(
            Some(VueDialect::Vue),
            r#"<script src="petite-vue"></script>"#,
            TemplateSyntaxMode::Strict,
            Lang::Js,
        );
        assert_eq!(overridden.framework(), VueDialect::Vue);
        assert_eq!(overridden.syntax(), TemplateSyntaxMode::Strict);
        assert!(!overridden.is_petite_document());
    }
}
