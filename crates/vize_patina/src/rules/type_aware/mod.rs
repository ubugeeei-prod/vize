//! Type-aware lint rules.
//!
//! These rules require semantic analysis (Croquis) and/or type
//! information from external corsa-backed type checkers to function properly.
//!
//! ## Rule Categories
//!
//! ### Script Rules
//! - `type/require-typed-props` - Require type definition for defineProps
//! - `type/require-typed-emits` - Require type definition for defineEmits
//! - `type/no-floating-promises` - Disallow unhandled Promise results
//!
//! ### Template Rules
//! - `type/no-unsafe-template-binding` - Disallow template expressions that
//!   stay `any`/`unknown` when resolved through Corsa

#[cfg(not(target_arch = "wasm32"))]
mod require_typed_emits;
#[cfg(not(target_arch = "wasm32"))]
mod require_typed_props;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::rules::opinionated::type_aware::NoFloatingPromises;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::rules::opinionated::type_aware::NoUnsafeTemplateBinding;
#[cfg(not(target_arch = "wasm32"))]
pub use require_typed_emits::RequireTypedEmits;
#[cfg(not(target_arch = "wasm32"))]
pub use require_typed_props::RequireTypedProps;
