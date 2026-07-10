//! Owned JSX/TSX syntax product built from OXC without constructing Relief.

#[path = "syntax/build.rs"]
mod build;
#[path = "syntax/control.rs"]
mod control;
#[path = "syntax/text.rs"]
pub(crate) mod text;

use crate::{JsxDiagnostic, JsxLang};
use vize_carton::source_anchor::SourceAnchor;

/// Inclusive-start, exclusive-end byte range in the original module.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct JsxSyntaxSpan {
    pub start: u32,
    pub end: u32,
}

impl JsxSyntaxSpan {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

impl From<oxc_span::Span> for JsxSyntaxSpan {
    fn from(span: oxc_span::Span) -> Self {
        Self::new(span.start, span.end)
    }
}

/// Opaque expression code and its exact authored range.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxSyntaxExpression {
    pub code: Box<str>,
    pub span: JsxSyntaxSpan,
    /// True when the snapshot normalized source syntax into an equivalent
    /// condition, such as the condition for `left || <Fallback />`.
    pub synthetic: bool,
}

impl JsxSyntaxExpression {
    fn authored(code: impl Into<Box<str>>, span: oxc_span::Span) -> Self {
        Self {
            code: code.into(),
            span: span.into(),
            synthetic: false,
        }
    }

    fn synthetic(code: impl Into<Box<str>>, span: oxc_span::Span) -> Self {
        Self {
            code: code.into(),
            span: span.into(),
            synthetic: true,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxSyntaxBinding {
    pub pattern: Box<str>,
    pub span: JsxSyntaxSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JsxSyntaxAttributeValue {
    Presence,
    Static {
        value: Box<str>,
        span: JsxSyntaxSpan,
    },
    Expression(JsxSyntaxExpression),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JsxSyntaxAttribute {
    Attribute {
        name: Box<str>,
        name_span: JsxSyntaxSpan,
        value: JsxSyntaxAttributeValue,
        span: JsxSyntaxSpan,
    },
    Spread {
        expression: JsxSyntaxExpression,
        span: JsxSyntaxSpan,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxSyntaxElement {
    pub name: Box<str>,
    pub component: bool,
    pub attributes: Vec<JsxSyntaxAttribute>,
    pub children: Vec<JsxSyntaxNode>,
    pub span: JsxSyntaxSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxSyntaxBranch {
    pub condition: Option<JsxSyntaxExpression>,
    pub body: Vec<JsxSyntaxNode>,
    pub span: JsxSyntaxSpan,
}

/// Owned render-relevant syntax extracted from OXC.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JsxSyntaxNode {
    Element(JsxSyntaxElement),
    Fragment {
        children: Vec<JsxSyntaxNode>,
        span: JsxSyntaxSpan,
    },
    Text {
        value: Box<str>,
        span: JsxSyntaxSpan,
    },
    Expression {
        expression: JsxSyntaxExpression,
        span: JsxSyntaxSpan,
    },
    Comment {
        value: Box<str>,
        span: JsxSyntaxSpan,
    },
    If {
        branches: Vec<JsxSyntaxBranch>,
        span: JsxSyntaxSpan,
    },
    For {
        source: JsxSyntaxExpression,
        value: Option<JsxSyntaxBinding>,
        index: Option<JsxSyntaxBinding>,
        body: Vec<JsxSyntaxNode>,
        span: JsxSyntaxSpan,
    },
}

impl JsxSyntaxNode {
    pub const fn span(&self) -> JsxSyntaxSpan {
        match self {
            Self::Element(element) => element.span,
            Self::Fragment { span, .. }
            | Self::Text { span, .. }
            | Self::Expression { span, .. }
            | Self::Comment { span, .. }
            | Self::If { span, .. }
            | Self::For { span, .. } => *span,
        }
    }
}

/// Parser-independent owned JSX/TSX product.
///
/// No OXC or Relief node escapes construction, so this value is
/// `Send + Sync + 'static` and may be cached by the compilation graph.
#[derive(Debug, Clone, PartialEq)]
pub struct JsxSyntaxSnapshot {
    pub filename: Option<Box<str>>,
    pub source: Box<str>,
    pub lang: JsxLang,
    pub roots: Vec<JsxSyntaxNode>,
    pub diagnostics: Vec<JsxDiagnostic>,
    pub panicked: bool,
    /// Stable compilation source identity when constructed by an Atlas provider.
    pub source_anchor: Option<SourceAnchor>,
}

impl JsxSyntaxSnapshot {
    pub fn has_errors(&self) -> bool {
        self.panicked || self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }

    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}

/// Parse an anonymous module into an owned syntax snapshot.
pub fn snapshot_jsx(source: &str, lang: JsxLang) -> JsxSyntaxSnapshot {
    build::snapshot(None, source, lang)
}

/// Parse a named module into an owned syntax snapshot.
pub fn snapshot_jsx_named(
    filename: impl Into<Box<str>>,
    source: &str,
    lang: JsxLang,
) -> JsxSyntaxSnapshot {
    build::snapshot(Some(filename.into()), source, lang)
}
