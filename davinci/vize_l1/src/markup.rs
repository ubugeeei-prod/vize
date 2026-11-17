//! L1 markup: the concrete syntax of markup grammars (#6835).
//!
//! Markup is a grammar × profile matrix. The grammar is the markup language
//! family (Vue now; Svelte, Angular and others later). The profile selects
//! the host rules: [`profile::Document`] for HTML and in-DOM templates
//! (petite-vue) and [`profile::Component`] for component templates. A
//! framework's own template profile lives with the framework
//! (`framework::vue::VueTemplate`).
//!
//! One shared state machine, [`lex::Lexer`], is generic over the profile
//! and its event [`token::Sink`], so every per-profile branch is resolved at
//! compile time. Directive-name decomposition is not part of the lexer: it
//! is a dialect syntax hook ([`directive::DirectiveSyntax`]), like an MLIR
//! custom assembly format.
//!
//! The profile-generic lexer, entity decoder and compatibility callback
//! adapter are available in default builds. [`parse_component`] constructs a
//! native surface tree with checked admission and dialect-owned `v-pre` scope.
//! The surface [`parse`] entries and compiler products retain
//! their existing parser routes through the published tokenizer facade;
//! that facade executes this same lexer while preserving callback policy.
//! Replacing product parser routes still requires fix-history and parity gates.
//! The Vue dialect provides allocation-free directive decomposition with checked
//! source admission; typed Shape dispatch remains unfinished.
//! Design: <https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929>.

pub mod directive;
pub mod entity;
pub mod grammar;
pub mod htmlish;
pub mod profile;
pub mod pug;
pub mod token;
pub mod tsrx;

pub use directive::{
    ArgSyntax, DirectiveName, DirectiveNameError, DirectivePrefix, DirectiveSyntax, VueDirectives,
};
pub use grammar::{MarkupGrammar, Vue};
pub use htmlish::lex::{Delimiters, LexOptions, Lexer};
pub use htmlish::parse::{
    ComponentParse, ComponentSourceError, DirectiveAdmission, SurfaceError, SurfaceParseOptions,
    parse, parse_component, parse_component_with_authored, parse_component_with_options,
    parse_with_authored, parse_with_authored_diagnostics, parse_with_options,
    parse_with_options_and_caps,
};
pub use htmlish::render::{HoleCounts, check_fidelity, hole_counts, render};
pub use htmlish::surface::{
    AttrValue, Attribute, CloseTag, Element, ElementClose, Interpolation, OpenTag, SurfaceChild,
    SurfaceTree, Token, TokenStatus,
};
pub use htmlish::view::{
    AttributeBindingExpression, AttributeBindingExpressionView, AttributeElse, AttributeElseView,
    AttributeExpression, AttributeExpressionFailure, AttributeExpressionView, AttributeForHead,
    AttributeForHeadFailure, AttributeForHeadView, AttributeHandler, AttributeHandlerFailure,
    AttributeHandlerView, AttributeHead, AttributeModelExpression, AttributeModelExpressionView,
    AttributeOperandError, AttributeValueFailure, AttributeValueOperand, AttributeValueView,
    AttributeView, AttributesView, ChildView, ChildrenView, ConditionKind, ElementClosingName,
    ElementNameRefusal, ElementNames, ElementView, InterpolationError, InterpolationFailure,
    InterpolationOperand, InterpolationView, ModelHead, NativeComponent, NativeTemplateComponent,
    RootText, RootTextError, RootTextProfile, RootTextView, ScriptSelection, StaticBindingHead,
    TemplateGrammar, TextValue, TextValueError, TextValueFailure, TextValueView,
};
pub use profile::{Component, Document, Profile, ProfileKind};
pub use token::{LexErrorCode, LexMode, Namespace, QuoteType, Sink};

pub use htmlish::view::{ComponentTagFacts, TagFacts, TagFactsKind, TagFactsRefusal};
