//! Original Document-profile lexical observations, without browser tree meaning.
//!
//! One private producer runs the shared lexer once over the complete checked
//! source. Token views and lexical completion borrow that original owner;
//! neither a Component nor caller-built token vectors can mint them. The
//! browser tree policies remain explicitly unfinished under #6835/#6843.

use vize_l0::{Allocator, SourceRoot, Span, Vec};

use super::{Document, LexErrorCode, LexOptions, Lexer, ProfileKind, QuoteType};
use crate::markup::entity::DecodedEntity;

mod sink;
mod structure;

pub use structure::{
    DocumentHtmlAttribute, DocumentHtmlAttributes, DocumentHtmlChildNodes, DocumentHtmlComment,
    DocumentHtmlElement, DocumentHtmlNode, DocumentHtmlRefusal, DocumentHtmlStructure,
    DocumentHtmlText,
};

/// The callbacks actually emitted by the Document-profile lexer.
/// End-of-tag callbacks carry a zero-width coordinate, including recovered
/// EOF coordinates. Names and values preserve authored case and entity bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTokenKind {
    Text,
    TextEntity,
    Interpolation,
    OpenTagName,
    OpenTagEnd,
    SelfClosingTag,
    CloseTagName,
    AttributeName,
    AttributeNameEnd,
    AttributeData,
    AttributeEntity,
    AttributeEnd(QuoteType),
    DirectiveName,
    DirectiveArgument,
    DirectiveModifier,
    Comment,
    Cdata,
    ProcessingInstruction,
    Declaration {
        terminated: bool,
    },
    /// Original skipped recovery window, not an exact declaration frame.
    DeclarationRecovery {
        terminated: bool,
    },
}

#[derive(Debug)]
struct Event {
    kind: RecordedKind,
    span: Span,
}

#[derive(Debug)]
enum RecordedKind {
    Token(DocumentTokenKind),
    TextEntity(DecodedEntity),
    AttributeEntity(DecodedEntity),
}

impl Event {
    fn kind(&self) -> DocumentTokenKind {
        match self.kind {
            RecordedKind::Token(kind) => kind,
            RecordedKind::TextEntity(_) => DocumentTokenKind::TextEntity,
            RecordedKind::AttributeEntity(_) => DocumentTokenKind::AttributeEntity,
        }
    }

    fn decoded_entity(&self) -> Option<DecodedEntity> {
        match self.kind {
            RecordedKind::TextEntity(value) | RecordedKind::AttributeEntity(value) => Some(value),
            RecordedKind::Token(_) => None,
        }
    }
}

/// A retained lexical diagnostic. Its byte coordinate is not a tree span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentLexicalError {
    pub code: LexErrorCode,
    pub offset: u32,
}

/// Browser tree semantics which this lexical owner cannot certify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTreePolicy {
    FoldNameCase,
    HtmlSelfClosing,
    ImpliedEndTags,
    TableContentModel,
}

const UNFINISHED_TREE_POLICIES: [DocumentTreePolicy; 4] = [
    DocumentTreePolicy::FoldNameCase,
    DocumentTreePolicy::HtmlSelfClosing,
    DocumentTreePolicy::ImpliedEndTags,
    DocumentTreePolicy::TableContentModel,
];

/// The whole original source and its sole completed Document lexer run.
///
/// This is lexical custody, not a DOM tree, petite-vue dialect admission,
/// Descriptor selection, semantic File or native product completion. It uses
/// default delimiters and no dialect-controlled verbatim scopes.
///
/// ```compile_fail
/// use vize_l1::markup::document::NativeDocument;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeDocument<'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::NativeDocument;
/// fn forge<'a>(original: NativeDocument<'a>) -> NativeDocument<'a> {
///     NativeDocument { ..original }
/// }
/// ```
pub struct NativeDocument<'a> {
    allocator: &'a Allocator,
    root: SourceRoot<'a>,
    events: Vec<'a, Event>,
    errors: Vec<'a, DocumentLexicalError>,
    normal_end: bool,
    declaration_refusal: Option<DocumentLexicalRefusal>,
}

impl core::fmt::Debug for NativeDocument<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeDocument")
            .field("root", &self.root)
            .field("events", &self.events)
            .field("errors", &self.errors)
            .field("normal_end", &self.normal_end)
            .field("declaration_refusal", &self.declaration_refusal)
            .finish_non_exhaustive()
    }
}

impl<'a> NativeDocument<'a> {
    /// Run the actual Document lexer once over the complete original source.
    /// The checked root supplies the source limit and base-zero coordinates.
    #[must_use]
    pub fn lex_in(allocator: &'a Allocator, root: SourceRoot<'a>) -> Self {
        let mut events = Vec::new_in(&allocator);
        let mut errors = Vec::new_in(&allocator);
        let recorder = sink::Recorder {
            events: &mut events,
            errors: &mut errors,
            end_calls: 0,
            declaration_refusal: None,
        };
        let mut lexer = Lexer::<Document, _>::new(root.source(), recorder, LexOptions::default());
        lexer.run();
        let normal_end = lexer.normal_end_state();
        // The sink is private; no caller can insert events or assert EOF.
        let recorder = lexer.into_sink();
        assert_eq!(recorder.end_calls, 1);
        let declaration_refusal = recorder.declaration_refusal;
        Self {
            allocator,
            root,
            events,
            errors,
            normal_end,
            declaration_refusal,
        }
    }

    #[must_use]
    pub fn root(&self) -> SourceRoot<'a> {
        self.root
    }

    #[must_use]
    pub fn source(&self) -> &'a str {
        self.root.source()
    }

    #[must_use]
    pub fn tokens(&self) -> impl ExactSizeIterator<Item = DocumentToken<'_, 'a>> {
        self.events
            .iter()
            .map(|event| DocumentToken { owner: self, event })
    }

    #[must_use]
    pub fn errors(&self) -> &[DocumentLexicalError] {
        &self.errors
    }

    /// Construct only the admitted explicit HTML envelope's element ancestry.
    /// The original retained events are consumed directly, without lexing again.
    /// Direct child text/comment readback is scoped to the admitted body subtree;
    /// this does not certify general HTML tree or petite-vue semantics.
    pub fn html_structure(&self) -> Result<DocumentHtmlStructure<'_, 'a>, DocumentHtmlRefusal> {
        structure::construct(self)
    }

    /// Declaration tolerance is implemented; these four tree policies are
    /// only declarations on `Profile`, never completion credit for this run.
    #[must_use]
    pub fn unfinished_tree_policies(&self) -> &[DocumentTreePolicy; 4] {
        &UNFINISHED_TREE_POLICIES
    }

    /// Certify only a normal complete lexical run of this original source.
    /// Recoverable errors, declaration recovery and silent pending EOF states
    /// retain their owner and observations but cannot mint this readonly view.
    pub fn normal_completion(
        &self,
    ) -> Result<DocumentLexicalCompletion<'_, 'a>, DocumentLexicalRefusal> {
        if !self.errors.is_empty() {
            return Err(DocumentLexicalRefusal::RecoveredSyntax);
        }
        if let Some(refusal) = self.declaration_refusal {
            return Err(refusal);
        }
        if !self.normal_end {
            return Err(DocumentLexicalRefusal::PendingSyntax);
        }
        Ok(DocumentLexicalCompletion { owner: self })
    }
}

/// Why original observations cannot certify normal lexical completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentLexicalRefusal {
    RecoveredSyntax,
    UnterminatedDeclaration,
    RecoveredDeclaration,
    PendingSyntax,
}

/// A normal lexical run borrowing its actual original Document owner.
/// This provides no browser tree, dialect or semantic completion capability.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentLexicalCompletion;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentLexicalCompletion<'static, 'static>>();
/// ```
#[derive(Debug)]
pub struct DocumentLexicalCompletion<'o, 'a> {
    owner: &'o NativeDocument<'a>,
}

impl<'o, 'a> DocumentLexicalCompletion<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    #[must_use]
    pub fn profile(&self) -> ProfileKind {
        ProfileKind::Document
    }
}

/// One immutable callback observation borrowing its genuine original run.
#[derive(Debug)]
pub struct DocumentToken<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    event: &'o Event,
}

impl<'o, 'a> DocumentToken<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    #[must_use]
    pub fn kind(&self) -> DocumentTokenKind {
        self.event.kind()
    }

    /// The actual scalar expansion retained from this original lexer callback.
    /// Named references may contain two scalars; their authored source span
    /// remains one event. This performs no second decode or DOM normalization.
    #[must_use]
    pub fn decoded_entity(&self) -> Option<DecodedEntity> {
        self.event.decoded_entity()
    }

    /// Raw callback coordinates; a recovered zero-width EOF coordinate can
    /// point inside a UTF-8 character. `source` checks that boundary explicitly.
    #[must_use]
    pub fn span(&self) -> Span {
        self.event.span
    }

    /// The exact original slice, when both callback coordinates are UTF-8
    /// boundaries. No replacement buffer or decoded text is manufactured.
    #[must_use]
    pub fn source(&self) -> Option<&'a str> {
        self.owner
            .source()
            .get(self.event.span.start as usize..self.event.span.end as usize)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod entity_tests;
