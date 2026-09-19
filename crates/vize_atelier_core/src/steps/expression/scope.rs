//! Names available throughout a generated template expression. Event locals
//! belong to an introduced handler scope, not this unconditional allowlist.

/// Whether a name needs no component binding outside any lexical scope.
///
/// Croquis also lists conditional event locals for consumers that model those
/// scopes themselves. Emitters must bind them only where the callback exists.
pub fn is_template_global(name: &str) -> bool {
    vize_croquis::builtins::is_global_allowed(name) && !vize_croquis::builtins::is_event_local(name)
}
