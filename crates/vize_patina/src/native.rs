//! Opt-in syntax linting over an original selected L1 template owner.
//!
//! Callers supply the authentic selected owner and one of its readonly element
//! projections. This lane neither parses source nor walks the component body.
//! Existing linter entry points and default rule registration are unchanged.

use vize_l0::{
    Span,
    diag::{Diagnostic, MessageLookup},
};
use vize_l1::markup::{NativeElement, NativeTemplateComponent};

mod attribute;
mod header;
mod iframe_has_title;
mod img_alt;
mod tabindex_no_positive;

/// The existing rule code; the native entry is separately opt-in.
pub const IMG_ALT_RULE: &str = "a11y/img-alt";

/// The existing iframe title rule code; the native entry is separately opt-in.
pub const IFRAME_HAS_TITLE_RULE: &str = "a11y/iframe-has-title";

/// The existing positive tabindex rule code; the native entry is separately opt-in.
pub const TABINDEX_NO_POSITIVE_RULE: &str = "a11y/tabindex-no-positive";

/// A refusal retains the caller's original component and observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLintRefusal {
    Recovered { offset: u32 },
    UnsupportedComponent,
    ForeignElement,
    SourceMismatch,
    Hole,
    UnresolvedBinding { span: Span },
    UnsupportedDirective { span: Span },
    DuplicateAttribute { span: Span },
}

/// A finished L0 diagnostic and its original stable rule code.
/// Diagnostic text is owned and can outlive the parsing arena.
#[derive(Debug)]
pub struct NativeLintFinding {
    rule_name: &'static str,
    diagnostic: Diagnostic,
}

impl NativeLintFinding {
    #[must_use]
    pub fn rule_name(&self) -> &'static str {
        self.rule_name
    }

    #[must_use]
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    #[must_use]
    pub fn into_diagnostic(self) -> Diagnostic {
        self.diagnostic
    }
}

/// One admitted original selected template, borrowed without replacing it.
/// Recovery status is read from that owner, never supplied by the caller.
///
/// Detached compatibility trees cannot establish admission:
/// ```compile_fail
/// use vize_l1::markup::ComponentParse;
/// use vize_patina::native::NativeSyntaxLint;
/// fn detached(tree: &ComponentParse<'_>) {
///     let _ = NativeSyntaxLint::new(tree);
/// }
/// ```
pub struct NativeSyntaxLint<'o, 'a> {
    owner: &'o NativeTemplateComponent<'a>,
}

impl<'o, 'a> NativeSyntaxLint<'o, 'a> {
    pub fn new(owner: &'o NativeTemplateComponent<'a>) -> Result<Self, NativeLintRefusal> {
        let component = owner.component();
        let carrier = component.carrier();
        if let Some(error) = carrier.errors.first() {
            let offset = component
                .block()
                .start()
                .checked_add(error.offset)
                .ok_or(NativeLintRefusal::SourceMismatch)?;
            return Err(NativeLintRefusal::Recovered { offset });
        }
        if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
            return Err(NativeLintRefusal::UnsupportedComponent);
        }
        if !core::ptr::eq(carrier.tree.source, component.block().source()) {
            return Err(NativeLintRefusal::SourceMismatch);
        }
        Ok(Self { owner })
    }

    #[must_use]
    pub fn owner(&self) -> &'o NativeTemplateComponent<'a> {
        self.owner
    }

    /// Check one genuine element's original header for `a11y/img-alt`.
    ///
    /// Static attributes and typed static binding arguments are supported.
    /// Dynamic arguments, object bindings and holes refuse before any result.
    /// Values remain opaque; this does not certify expression semantics.
    pub fn img_alt(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        img_alt::check(element, messages)
    }

    /// Check one genuine element's original header for `a11y/iframe-has-title`.
    ///
    /// Static title text must contain a non-whitespace decoded scalar. Typed
    /// static title bindings are accepted without parsing their opaque values.
    /// Every original header is checked before a result, as in `img_alt`.
    pub fn iframe_has_title(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        iframe_has_title::check(element, messages)
    }

    /// Check static authored tabindex values without parsing binding expressions.
    /// Findings retain original attribute order and complete attribute spans.
    /// A later unsupported header refuses the entire result.
    pub fn tabindex_no_positive(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        tabindex_no_positive::check(element, messages)
    }
}
