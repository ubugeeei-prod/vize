//! Opt-in syntax linting over an original selected L1 template owner.
//!
//! Callers supply the authentic selected owner and one of its readonly element
//! projections. This lane neither parses source nor walks the component body.
//! Existing linter entry points and default rule registration are unchanged.

#![deny(missing_docs)]

use vize_l0::{
    Span,
    diag::{Diagnostic, MessageLookup},
};
use vize_l1::markup::{NativeElement, NativeLintTagRefusal, NativeTemplateComponent};

mod admission;
mod aria_unsupported_elements;
mod attribute;
pub mod canonical;
pub mod child_facts;
mod deprecated_attr;
mod deprecated_element;
mod header;
pub mod header_facts;
mod header_rules;
mod iframe_has_title;
mod img_alt;
mod inline_style;
mod no_v_html;
pub mod sfc;
mod tabindex_no_positive;
pub mod template;
mod textarea_mustache;

/// The existing rule code; the native entry is separately opt-in.
pub const IMG_ALT_RULE: &str = "a11y/img-alt";

/// The existing iframe title rule code; the native entry is separately opt-in.
pub const IFRAME_HAS_TITLE_RULE: &str = "a11y/iframe-has-title";

/// The existing positive tabindex rule code; the native entry is separately opt-in.
pub const TABINDEX_NO_POSITIVE_RULE: &str = "a11y/tabindex-no-positive";

/// The existing autofocus rule code; the native entry is separately opt-in.
pub const NO_AUTOFOCUS_RULE: &str = "a11y/no-autofocus";
/// The existing access-key rule code; the native entry is separately opt-in.
pub const NO_ACCESS_KEY_RULE: &str = "a11y/no-access-key";
/// The existing distracting-elements rule code; the native entry is separately opt-in.
pub const NO_DISTRACTING_ELEMENTS_RULE: &str = "a11y/no-distracting-elements";
/// The existing inline-style rule code; the native entry is separately opt-in.
pub const NO_INLINE_STYLE_RULE: &str = "vue/no-inline-style";
/// The existing HTML-injection rule code; the native entry is separately opt-in.
pub const NO_V_HTML_RULE: &str = "vue/no-v-html";
/// The existing textarea interpolation rule code; the native entry is separately opt-in.
pub const NO_TEXTAREA_MUSTACHE_RULE: &str = "vue/no-textarea-mustache";

/// The existing unsupported-ARIA rule code; the native entry is separately opt-in.
pub const ARIA_UNSUPPORTED_ELEMENTS_RULE: &str = "a11y/aria-unsupported-elements";
/// The existing deprecated-attribute rule code; the native entry is separately opt-in.
pub const DEPRECATED_ATTR_RULE: &str = "html/deprecated-attr";
/// The existing deprecated-element rule code; the native entry is separately opt-in.
pub const DEPRECATED_ELEMENT_RULE: &str = "html/deprecated-element";

pub use aria_unsupported_elements::NativeAriaLintError;
pub use no_v_html::NativeVHtmlLintError;
pub use textarea_mustache::NativeTextareaLintError;

/// A refusal retains the caller's original component and observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLintRefusal {
    /// The original parser recovered from an error at the root-source offset.
    Recovered {
        /// Root-source byte offset of the first recovery.
        offset: u32,
    },
    /// The selected component uses syntax outside the native lint contract.
    UnsupportedComponent,
    /// The supplied element belongs to a different selected component.
    ForeignElement,
    /// The retained source and the admitted component source do not match.
    SourceMismatch,
    /// The producer recorded a syntax hole that prevents a complete claim.
    Hole,
    /// A binding could not be resolved without reconstructing syntax.
    UnresolvedBinding {
        /// Original binding range that could not be resolved.
        span: Span,
    },
    /// A directive is not represented by the native header contract.
    UnsupportedDirective {
        /// Original directive range.
        span: Span,
    },
    /// Duplicate attributes make the original header ambiguous.
    DuplicateAttribute {
        /// Original duplicate attribute range.
        span: Span,
    },
    /// A literal-context lint tag prevents an authenticated interpretation.
    LintTag {
        /// Producer-supplied reason for refusing the lint tag.
        reason: NativeLintTagRefusal,
    },
    /// HTML table recovery makes the element's parent context ambiguous.
    TableContext {
        /// Original range whose table context is ambiguous.
        span: Span,
    },
    /// Parser recovery changes the context needed by the lint rule.
    RecoveryContext {
        /// Original range affected by recovery.
        span: Span,
    },
}

/// A finished L0 diagnostic and its original stable rule code.
/// Diagnostic text is owned and can outlive the parsing arena.
#[derive(Debug)]
pub struct NativeLintFinding {
    rule_name: &'static str,
    diagnostic: Diagnostic,
}

impl NativeLintFinding {
    /// Returns the stable registered rule code that produced the finding.
    #[must_use]
    pub fn rule_name(&self) -> &'static str {
        self.rule_name
    }

    /// Borrows the completed diagnostic for rendering or transport.
    #[must_use]
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    /// Consumes the finding and returns its completed diagnostic.
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
    /// Admits an original selected template when its source is complete and supported.
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

    /// Returns the original selected template that granted native lint admission.
    #[must_use]
    pub fn owner(&self) -> &'o NativeTemplateComponent<'a> {
        self.owner
    }

    /// Retain SDK facts from one genuine, completely checked authored header.
    /// No detached evidence, caller key, source, span or tag kind is accepted.
    pub fn header_facts<'e>(
        &'e self,
        element: &NativeElement<'e, 'a>,
    ) -> Result<header_facts::NativeHeaderFacts<'e, 'a>, header_facts::NativeHeaderFactError> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(header_facts::NativeHeaderFactError::Header(
                NativeLintRefusal::ForeignElement,
            ));
        }
        header_facts::NativeHeaderFacts::new(self.owner, element)
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

    /// Check exact static autofocus attribute/binding names, retaining original
    /// full attribute ranges and actual default lint component exemptions.
    pub fn no_autofocus(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        header_rules::no_autofocus(element, messages)
    }

    /// Check exact static accesskey attribute/binding names without evaluating
    /// values. Any unsupported original header refuses the entire result.
    pub fn no_access_key(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        header_rules::no_access_key(element, messages)
    }

    /// Check authored marquee/blink tag names with original opening ranges.
    /// The original provider's ambiguous lint category remains a typed refusal.
    pub fn no_distracting_elements(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        header_rules::no_distracting_elements(element, messages)
    }

    /// Check exact authored obsolete tag spellings with original opening ranges.
    /// Every original header is strictly admitted before component exemption or
    /// catalog lookup. This opt-in rule does not evaluate values or inspect body.
    pub fn deprecated_element(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        deprecated_element::check(element, messages)
    }

    /// Check exact static deprecated attributes on original non-component headers.
    /// Values remain opaque. Findings retain authored order and full attribute
    /// ranges; a later unsupported head refuses before any catalog lookup.
    pub fn deprecated_attr(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        deprecated_attr::check(element, messages)
    }

    /// Check original full `v-html` and static Bind/Prop HTML sink arguments.
    /// Every header is strictly checked once before any catalog lookup. Values
    /// remain opaque; host comment suppression and expression-dependent forms
    /// require separate authority before whole-file/default integration.
    pub fn no_v_html(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeVHtmlLintError> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement.into());
        }
        no_v_html::check(element, messages)
    }

    /// Check exact static `style` attributes, including component attributes.
    /// Bind/Prop expressions and all attribute values remain opaque. The whole
    /// original header must be admitted before any lookup or finding.
    pub fn no_inline_style(
        &self,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        if !core::ptr::eq(self.owner.component(), element.component()) {
            return Err(NativeLintRefusal::ForeignElement);
        }
        inline_style::check(element, messages)
    }
}
