//! htmlish: the HTML-tokenizer family, one grammar for two hosts.
//!
//! An `.html` document (also in-DOM templates and petite-vue) is read under
//! the Document profile, and an SFC `<template>` under the component profile
//! with Vue's attribute rule. The profiles in [`super::profile`] are the only
//! place the two differ, and `docs/davinci/plan/htmlish-divergences.json`
//! records every difference against each dialect's oracle. This module holds
//! the profile-generic lexer, the lossless surface tree and its builder, the
//! document and component parses, and rendering. Shared markup contracts
//! stay in [`super`].

pub(crate) mod build;
pub mod document;
pub(crate) mod event;
pub mod lex;
pub(crate) mod native;
pub mod parse;
pub mod render;
pub mod surface;
