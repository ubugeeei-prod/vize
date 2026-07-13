//! Owned JSX/TSX syntax products built from one OXC program.

#[path = "syntax/build.rs"]
mod build;
#[path = "syntax/control.rs"]
mod control;
#[path = "syntax/roots.rs"]
mod roots;
#[path = "syntax/text.rs"]
pub(crate) mod text;
#[path = "syntax/typecheck.rs"]
mod typecheck;

#[cfg(test)]
#[path = "syntax/tests.rs"]
mod tests;

pub use typecheck::{JsxTypecheckEmit, JsxTypecheckExpression, JsxTypecheckRoot};

use crate::{ComponentSetupSpan, JsxOutputMode, StyleExprSpan};
use crate::{JsxDiagnostic, JsxLang};
use vize_atlas::Shared;
use vize_carton::source_anchor::SourceAnchor;
use vize_croquis::Croquis;

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

/// Component context retained beside one outermost owned JSX render root.
#[derive(Debug, Clone)]
pub struct JsxSyntaxRootMetadata {
    pub span: JsxSyntaxSpan,
    pub mode: Option<JsxOutputMode>,
    pub component_name: Option<Box<str>>,
    pub component_setup: Option<ComponentSetupSpan>,
    pub scoped_css: Option<Box<str>>,
    pub scoped_styles: Vec<JsxSyntaxScopedStyle>,
    pub(crate) scoped_style_exprs: Vec<StyleExprSpan>,
}

/// Raw embedded CSS and its exact authored body range.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxSyntaxScopedStyle {
    pub css: Box<str>,
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
#[derive(Debug, Clone)]
pub struct JsxSyntaxSnapshot {
    pub filename: Option<Box<str>>,
    pub source: Box<str>,
    pub lang: JsxLang,
    pub roots: Vec<JsxSyntaxNode>,
    root_metadata: Vec<JsxSyntaxRootMetadata>,
    /// Exact embedded-expression projection used by plain-TypeScript consumers.
    ///
    /// This is built while the source's single OXC program is live. Consumers
    /// never need to parse JSX again or reconstruct directives, slots, control
    /// flow, or scoped-style expressions from the generic syntax tree.
    typecheck_roots: Vec<JsxTypecheckRoot>,
    pub diagnostics: Vec<JsxDiagnostic>,
    pub panicked: bool,
    /// Stable compilation source identity when constructed by an Atlas provider.
    pub source_anchor: Option<SourceAnchor>,
    analysis: Shared<Croquis>,
}

impl JsxSyntaxSnapshot {
    pub fn has_errors(&self) -> bool {
        self.panicked || self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }

    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Complete Croquis script analysis built from the same OXC parse.
    pub fn analysis(&self) -> &Croquis {
        &self.analysis
    }

    pub fn shared_analysis(&self) -> Shared<Croquis> {
        Shared::clone(&self.analysis)
    }

    /// Outermost JSX roots and their exact type-checkable expression graph.
    pub fn typecheck_roots(&self) -> &[JsxTypecheckRoot] {
        &self.typecheck_roots
    }

    /// Metadata aligned one-for-one with [`Self::roots`].
    pub fn root_metadata(&self) -> &[JsxSyntaxRootMetadata] {
        &self.root_metadata
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

#[cfg(test)]
pub(crate) fn reset_frontend_counters() {
    crate::parse::reset_parse_count();
    typecheck::reset_lowering_counts();
}

#[cfg(test)]
pub(crate) fn frontend_counters() -> (usize, usize, usize) {
    let (lowerings, direct_fallbacks) = typecheck::lowering_counts();
    (crate::parse::parse_count(), lowerings, direct_fallbacks)
}

#[cfg(test)]
pub(crate) fn record_direct_fallback() {
    typecheck::record_direct_fallback();
}
