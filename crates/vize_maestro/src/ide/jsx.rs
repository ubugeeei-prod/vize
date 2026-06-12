//! Type-aware LSP support for `.jsx`/`.tsx` Vue components (#1498).
//!
//! Today maestro surfaces JSX **compiler** diagnostics only (parse/lowering
//! errors) and generates no virtual TypeScript for JSX, so hover, completion,
//! go-to-definition, and type diagnostics don't work on `.jsx`/`.tsx`. This
//! module is the type-aware slice: it lowers a JSX/TSX document to plain
//! virtual TypeScript (matching the `vize check` type-checker — standing
//! directive that JSX virtual TS stays plain `.ts`, never a TSX-format virtual
//! document), maps editor positions into that virtual TS, and reuses the SFC
//! Corsa machinery to answer requests.
//!
//! The whole surface is gated on the opt-in `typeChecker.jsxTypecheck` flag
//! (default off) so React `.tsx` files are never type-checked as Vue JSX.

mod position;
pub mod virtual_ts;

#[cfg(feature = "native")]
mod service;

#[cfg(feature = "native")]
pub use service::JsxService;
