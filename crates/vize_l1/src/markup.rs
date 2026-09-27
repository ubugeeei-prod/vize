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
//! The lexer is the one Vue template tokenizer: `vize_armature` and the L1
//! surface tree both drive it. The directive hook is still a skeleton.
//! Design: <https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929>.

pub mod directive;
pub mod entity;
pub mod grammar;
pub mod lex;
pub mod profile;
pub mod token;

pub use directive::{ArgSyntax, DirectiveName, DirectivePrefix, DirectiveSyntax, VueDirectives};
pub use grammar::{MarkupGrammar, Vue};
pub use lex::{Delimiters, LexOptions, Lexer};
pub use profile::{Component, Document, Profile, ProfileKind};
pub use token::{LexErrorCode, LexMode, Namespace, QuoteType, Sink};
