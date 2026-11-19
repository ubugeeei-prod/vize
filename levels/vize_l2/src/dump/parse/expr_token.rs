//! Expression-payload token grammar (P2-5b; Vue 2 filters P2-9):
//! `js(<quoted> @s:e)`, `opaque(<reason> <quoted> @s:e)`,
//! `foreign(<dialect> <quoted> @s:e)`, `vue.filter(<quoted> @s:e)`.
//!
//! The payload is owned text + span, never an AST - a `js(...)` token
//! re-parses into the arena on load (see `crate::expr::js`). Reasons are
//! the closed [`OpaqueReason`] spellings; dialect ids are bare non-empty
//! words (quotes, spaces and `)` in a dialect id are outside the
//! contract, the same documented-edges rule the quoted-string grammar
//! uses).

include!("../../lang/dump/parse.rs");
