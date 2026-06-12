//! Scoped-style support for JSX/TSX components (#1495).
//!
//! Reuses the SFC scoped-style infrastructure (`vize_atelier_sfc`) rather than
//! reimplementing CSS rewriting or scope-id hashing:
//!
//! - **Scope id**: [`vize_atelier_sfc::generate_bundler_scope_id`] — the same
//!   SHA-256-prefix hasher the SFC bundler path uses. JSX has no SFC filename at
//!   this layer, so the component name (falling back to a stable placeholder)
//!   plus the CSS content seeds the hash, giving a deterministic 8-char id.
//! - **CSS rewrite**: [`vize_atelier_sfc::style::apply_scoped_css`] — the same
//!   selector rewriter that turns `.box { … }` into `.box[data-v-<hash>] { … }`
//!   and handles `:deep()` / `:slotted()` / `:global()` / `@keyframes`.
//!
//! The resulting `data-v-<hash>` attribute is then injected into the component's
//! rendered elements by the VDOM codegen (via `CodegenOptions.scope_id`) and the
//! Vapor backend (via post-generation template rewriting), mirroring the two SFC
//! scope-injection paths.

use vize_carton::String;

/// A JSX component's scoped style, after rewriting.
#[derive(Debug, Clone)]
pub struct ScopedStyle {
    /// The generated scope id, e.g. `data-v-1a2b3c4d` (the value injected as a
    /// `data-v-…` attribute onto rendered elements).
    pub scope_id: String,
    /// The scoped-rewritten CSS, e.g. `.box[data-v-1a2b3c4d] { color: red }`.
    pub css: String,
}

/// Generate the scope id and scoped-rewritten CSS for a JSX component's raw
/// `<style scoped>` content, reusing the SFC scope infrastructure.
///
/// `component_name` seeds the scope-id hash (with the CSS content) so two
/// components in one module get distinct ids; it falls back to a placeholder
/// when the enclosing component name could not be resolved.
pub(crate) fn build_scoped_style(component_name: Option<&str>, raw_css: &str) -> ScopedStyle {
    let scope_id = generate_scope_id(component_name, raw_css);
    let css = vize_atelier_sfc::style::apply_scoped_css(raw_css, &scope_id);
    ScopedStyle { scope_id, css }
}

/// The `data-v-<hash>` scope id for a component, reusing the SFC bundler hasher.
///
/// The hash input is `"<component>\n<css>"`: including the CSS keeps ids stable
/// and distinct across components while staying purely content-derived (no real
/// filename exists at this layer).
fn generate_scope_id(component_name: Option<&str>, raw_css: &str) -> String {
    let name = component_name.unwrap_or("JsxComponent");
    let mut seed = String::with_capacity(name.len() + raw_css.len() + 1);
    seed.push_str(name);
    seed.push('\n');
    seed.push_str(raw_css);

    // `generate_bundler_scope_id(filename, root=None, is_production=false, src)`
    // hashes the normalized filename; passing the content-seed as the filename
    // yields a deterministic 8-char hex digest.
    let hash = vize_atelier_sfc::generate_bundler_scope_id(&seed, None, false, None);

    let mut scope_id = String::with_capacity(hash.len() + 7);
    scope_id.push_str("data-v-");
    scope_id.push_str(&hash);
    scope_id
}
