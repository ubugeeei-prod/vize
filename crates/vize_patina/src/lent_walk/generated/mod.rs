//! Private lifetime-carrying immutable AST traversal.
//!
//! Adapted from OXC fc702c1fa9f0412d06ec6908b58cd395b826cf7f, `crates/oxc_ast_visit/src/generated/visit.rs`.
//! Original source SHA-256: `5440cd4c4a2e10effd6b9eafc8d7a283258884b91eae1feef76988ca6760490e`.
//! The walk order and AST matches come from that pinned source. Only the
//! three LentNodes hooks are observable; OXC's other hooks were no-ops for
//! the former Lending visitor. Every node input now borrows for `'a`, so
//! rustc proves that the hooks may retain it for the root's borrow.
//!
//! OXC copyright (c) 2024-present VoidZero Inc. & Contributors; (c) 2023
//! Boshen. The complete MIT license is in the adjacent LICENSE file.

#![allow(
    unused_variables,
    unused_imports,
    clippy::match_same_arms,
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_ref_mut,
    clippy::trivially_copy_pass_by_ref
)]

use super::LentNodes;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_syntax::scope::ScopeFlags;

mod part_01;
pub(super) use part_01::*;
mod part_02;
pub(super) use part_02::*;
mod part_03;
pub(super) use part_03::*;
mod part_04;
pub(super) use part_04::*;
mod part_05;
pub(super) use part_05::*;
mod part_06;
pub(super) use part_06::*;
mod part_07;
pub(super) use part_07::*;
