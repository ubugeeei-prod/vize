//! Legacy storage compatibility and host-runtime integration.
//! Canonical implementations live in `vize_l0`.
pub use vize_l0::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod corsa_api_mode;
#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::disallowed_types,
    reason = "legacy host discovery retains its environment and path collections"
)]
pub mod corsa_resolver;

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host JSON-RPC records retain the existing std String and Vec contract"
)]
pub mod lsp;
