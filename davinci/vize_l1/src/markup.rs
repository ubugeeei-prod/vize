//! L1 markup: the concrete syntax of markup grammars (#6835).
//!
//! Markup is a grammar × profile matrix. The grammar is the markup language
//! family (Vue now; Svelte, Angular and others later). The profile selects
//! the host rules: [`profile::Document`] for HTML and in-DOM templates
//! (petite-vue) and [`profile::Component`] for SFC `<template>` content.
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
//! The root parse APIs and compiler products retain
//! their existing parser routes through the published tokenizer facade;
//! that facade executes this same lexer while preserving callback policy.
//! Replacing product parser routes still requires fix-history and parity gates.
//! The Vue dialect provides allocation-free directive decomposition with checked
//! source admission; typed Shape dispatch remains unfinished.
//! Design: <https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929>.

pub mod directive;
pub mod document;
pub mod entity;
pub mod grammar;
pub mod lex;
mod native;
pub mod parse;
pub mod profile;
pub mod token;

pub use directive::{
    ArgSyntax, DirectiveName, DirectiveNameError, DirectivePrefix, DirectiveSyntax, VueDirectives,
};
pub use grammar::{MarkupGrammar, Vue};
pub use lex::{Delimiters, LexOptions, Lexer};
pub use native::{
    NativeAttribute, NativeAttributeExpression, NativeAttributeExpressionFailure,
    NativeAttributeExpressionView, NativeAttributeForHead, NativeAttributeForHeadFailure,
    NativeAttributeForHeadView, NativeAttributeHandler, NativeAttributeHandlerFailure,
    NativeAttributeHandlerView, NativeAttributeOperandError, NativeAttributes, NativeChild,
    NativeChildren, NativeComponent, NativeConditionKind, NativeElement, NativeInterpolationError,
    NativeInterpolationFailure, NativeInterpolationOperand, NativeInterpolationView,
    NativeScriptSelection, NativeTemplateComponent, NativeTemplateGrammar,
};
pub use parse::{
    ComponentParse, ComponentSourceError, DirectiveAdmission, parse_component,
    parse_component_with_authored, parse_component_with_options,
};
pub use profile::{Component, Document, Profile, ProfileKind};
pub use token::{LexErrorCode, LexMode, Namespace, QuoteType, Sink};

pub use native::{NativeLintTag, NativeLintTagKind, NativeLintTagRefusal};
