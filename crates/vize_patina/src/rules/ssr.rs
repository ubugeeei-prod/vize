//! SSR-specific lint rules.
//!
//! These rules help detect SSR-unfriendly code patterns that would cause
//! errors when running on the server (Node.js, Deno, Bun).

mod no_browser_globals_in_ssr;
mod no_hydration_mismatch;

pub use no_browser_globals_in_ssr::NoBrowserGlobalsInSsr;
pub use no_hydration_mismatch::NoHydrationMismatch;
