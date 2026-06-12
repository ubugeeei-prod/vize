//! JSX/TSX source language selection.

use oxc_span::SourceType;

/// The flavor of JSX source being lowered.
///
/// Vize supports both plain `.jsx` (JavaScript + JSX) and `.tsx`
/// (TypeScript + JSX). The distinction only affects how OXC parses the
/// surrounding script; the lowering of JSX nodes themselves is identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsxLang {
    /// JavaScript with JSX (`.jsx`).
    Jsx,
    /// TypeScript with JSX (`.tsx`).
    Tsx,
}

impl JsxLang {
    /// Infer the language from a file extension or `lang` attribute value.
    ///
    /// Anything that is not recognized as TypeScript falls back to [`JsxLang::Jsx`].
    pub fn from_lang(lang: Option<&str>) -> Self {
        match lang.map(str::trim) {
            Some("tsx") | Some("ts") | Some("typescript") => Self::Tsx,
            _ => Self::Jsx,
        }
    }

    /// Infer the language from a file path by its extension.
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".tsx") {
            Self::Tsx
        } else {
            Self::Jsx
        }
    }

    /// Whether this language carries TypeScript syntax.
    pub fn is_typescript(self) -> bool {
        matches!(self, Self::Tsx)
    }

    /// The OXC [`SourceType`] used to parse this language as an ES module.
    pub fn source_type(self) -> SourceType {
        match self {
            Self::Jsx => SourceType::jsx().with_module(true),
            Self::Tsx => SourceType::tsx().with_module(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_lang_maps_typescript_variants() {
        assert_eq!(JsxLang::from_lang(Some("tsx")), JsxLang::Tsx);
        assert_eq!(JsxLang::from_lang(Some("ts")), JsxLang::Tsx);
        assert_eq!(JsxLang::from_lang(Some("typescript")), JsxLang::Tsx);
        assert_eq!(JsxLang::from_lang(Some(" tsx ")), JsxLang::Tsx);
    }

    #[test]
    fn from_lang_defaults_to_jsx() {
        assert_eq!(JsxLang::from_lang(Some("jsx")), JsxLang::Jsx);
        assert_eq!(JsxLang::from_lang(None), JsxLang::Jsx);
        assert_eq!(JsxLang::from_lang(Some("js")), JsxLang::Jsx);
    }

    #[test]
    fn from_path_uses_extension() {
        assert_eq!(JsxLang::from_path("App.tsx"), JsxLang::Tsx);
        assert_eq!(JsxLang::from_path("App.jsx"), JsxLang::Jsx);
        assert_eq!(JsxLang::from_path("App.js"), JsxLang::Jsx);
    }

    #[test]
    fn is_typescript_flag() {
        assert!(JsxLang::Tsx.is_typescript());
        assert!(!JsxLang::Jsx.is_typescript());
    }
}
