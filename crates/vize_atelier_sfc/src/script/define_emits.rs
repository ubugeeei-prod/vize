//! defineEmits macro handling.
//!
//! Handles the `defineEmits` Compiler Macro.
//!
//! Based on Vue.js official implementation:
//! https://github.com/vuejs/core/blob/main/packages/compiler-sfc/src/script/defineEmits.ts

use vize_carton::{FxHashSet, String, ToCompactString};

use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;
use std::sync::LazyLock;

use super::context::ScriptCompileContext;

pub use vize_croquis::macros::DEFINE_EMITS;

/// Result of processing defineEmits
#[derive(Debug, Clone, Default)]
pub struct DefineEmitsResult {
    /// Runtime declaration (the argument passed to defineEmits)
    pub runtime_decl: Option<String>,
    /// Type declaration (the type parameter)
    pub type_decl: Option<String>,
    /// The variable name this is assigned to (e.g., "emit")
    pub decl_id: Option<String>,
}

/// Process defineEmits call expression
///
/// Returns true if this was a defineEmits call, false otherwise.
/// Mutates ctx to store the emits information.
pub fn process_define_emits(
    ctx: &mut ScriptCompileContext,
    call: &CallExpression<'_>,
    source: &str,
    decl_id: Option<String>,
) -> bool {
    if !is_call_of(call, DEFINE_EMITS) {
        return false;
    }

    ctx.has_define_emits_call = true;

    // Store runtime declaration (first argument)
    let runtime_decl = call.arguments.first().map(|arg| {
        let start = arg.span().start as usize;
        let end = arg.span().end as usize;
        String::from(source.get(start..end).unwrap_or_default().trim())
    });

    // Store type declaration (type parameter)
    let type_decl = call.type_arguments.as_ref().map(|params| {
        let start = params.span.start as usize;
        let end = params.span.end as usize;
        let type_str = source.get(start..end).unwrap_or_default();
        // Remove the < and > from type params
        let inner = type_str.strip_prefix('<').and_then(|s| s.strip_suffix('>'));
        String::from(inner.unwrap_or(type_str))
    });

    // Store emits info in macros
    ctx.emits_runtime_decl = runtime_decl;
    ctx.emits_type_decl = type_decl;
    ctx.emit_decl_id = decl_id;

    true
}

/// Generate runtime emits declaration
///
/// Returns the emits array/object as a string for use in the compiled output.
pub fn gen_runtime_emits(ctx: &ScriptCompileContext, model_names: &[String]) -> Option<String> {
    fn debug_string<T: std::fmt::Debug>(value: &T) -> String {
        let mut out = String::default();
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:?}", value);
        out
    }

    let mut emits_decl = String::default();

    if let Some(ref runtime_decl) = ctx.emits_runtime_decl {
        emits_decl = runtime_decl.trim().to_compact_string();
    } else if ctx.emits_type_decl.is_some() {
        let type_declared_emits = extract_runtime_emits(ctx);
        if !type_declared_emits.is_empty() {
            let emits: Vec<String> = type_declared_emits
                .into_iter()
                .map(|k| debug_string(&k)) // JSON.stringify equivalent
                .collect();
            let joined = emits.join(", ");
            let mut out = String::with_capacity(joined.len() + 2);
            out.push('[');
            out.push_str(&joined);
            out.push(']');
            emits_decl = out;
        }
    }

    // Merge with model emits if defineModel was called
    if !model_names.is_empty() {
        let model_emits: Vec<String> = model_names
            .iter()
            .map(|n| {
                let mut name = String::with_capacity(7 + n.len());
                name.push_str("update:");
                name.push_str(n);
                debug_string(&name)
            })
            .collect();
        let joined = model_emits.join(", ");
        let mut model_emits_decl = String::with_capacity(joined.len() + 2);
        model_emits_decl.push('[');
        model_emits_decl.push_str(&joined);
        model_emits_decl.push(']');

        if emits_decl.is_empty() {
            emits_decl = model_emits_decl;
        } else {
            // /*@__PURE__*/_mergeModels(emitsDecl, modelEmitsDecl)
            let mut merged = String::with_capacity(emits_decl.len() + model_emits_decl.len() + 26);
            merged.push_str("/*@__PURE__*/_mergeModels(");
            merged.push_str(&emits_decl);
            merged.push_str(", ");
            merged.push_str(&model_emits_decl);
            merged.push(')');
            emits_decl = merged;
        }
    }

    if emits_decl.is_empty() {
        None
    } else {
        Some(emits_decl)
    }
}

/// Extract runtime emits from type declaration
///
/// Parses the type declaration to extract event names.
pub fn extract_runtime_emits(ctx: &ScriptCompileContext) -> FxHashSet<String> {
    let mut emits = FxHashSet::default();

    let type_decl = match &ctx.emits_type_decl {
        Some(decl) => decl,
        None => return emits,
    };

    // Parse the type declaration to extract event names
    // This is a simplified implementation - the full implementation would need
    // to use OXC's type resolver to properly resolve union types, etc.

    // Handle TSFunctionType: (e: 'click') => void
    if type_decl.contains("=>") && !type_decl.contains('{') {
        if let Some(event_name) = extract_event_name_from_function_type(type_decl) {
            emits.insert(event_name);
        }
        return emits;
    }

    // Handle object/interface type: { (e: 'click'): void } or { click: [...] }
    extract_events_from_type_literal(type_decl, &mut emits);

    emits
}

/// Extract event name from a function type like (e: 'click') => void
fn extract_event_name_from_function_type(type_str: &str) -> Option<String> {
    // Look for pattern like (e: 'eventName') or (e: "eventName")
    let re = regex::Regex::new(r#"\(\s*\w+\s*:\s*['"]([^'"]+)['"]\s*[,)]"#).ok()?;
    re.captures(type_str)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_compact_string()))
}

/// Extract events from a type literal (object type)
fn extract_events_from_type_literal(type_str: &str, emits: &mut FxHashSet<String>) {
    static CALL_SIG_RE: LazyLock<Result<regex::Regex, regex::Error>> = LazyLock::new(|| {
        regex::Regex::new(r#"\(\s*\w+\s*:\s*['"]([^'"]+)['"]\s*(?:,\s*[^)]+)?\)\s*:"#)
    });
    static PROP_RE: LazyLock<Result<regex::Regex, regex::Error>> =
        LazyLock::new(|| regex::Regex::new(r#"(?:^|[{;,])\s*([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:"#));

    // Handle call signatures: { (e: 'click'): void; (e: 'update'): void }
    if let Ok(call_sig_re) = &*CALL_SIG_RE {
        for cap in call_sig_re.captures_iter(type_str) {
            if let Some(event_name) = cap.get(1) {
                emits.insert(event_name.as_str().to_compact_string());
            }
        }
    }

    // Handle property syntax: { click: [...], update: [...] }
    // This is for the newer emit type syntax
    if let Ok(prop_re) = &*PROP_RE {
        for cap in prop_re.captures_iter(type_str) {
            if let Some(prop_name) = cap.get(1) {
                let name = prop_name.as_str();
                // Skip common type keywords
                if !matches!(name, "type" | "required" | "default" | "validator") {
                    emits.insert(name.to_compact_string());
                }
            }
        }
    }
}

/// Check if call expression is of given name
fn is_call_of(call: &CallExpression<'_>, name: &str) -> bool {
    if let Expression::Identifier(id) = &call.callee {
        return id.name.as_str() == name;
    }
    false
}

#[cfg(test)]
mod tests {
    use vize_carton::{CompactString, FxHashSet, ToCompactString};

    use super::{
        ScriptCompileContext, extract_event_name_from_function_type,
        extract_events_from_type_literal, gen_runtime_emits,
    };

    fn snapshot_emits(emits: &FxHashSet<CompactString>) -> Vec<&str> {
        let mut emits: Vec<_> = emits.iter().map(|event| event.as_str()).collect();
        emits.sort_unstable();
        emits
    }

    #[test]
    fn test_extract_event_name_from_function_type() {
        let result = extract_event_name_from_function_type("(e: 'click') => void");
        assert_eq!(result, Some("click".to_compact_string()));

        let result = extract_event_name_from_function_type("(e: \"update\") => void");
        assert_eq!(result, Some("update".to_compact_string()));
    }

    #[test]
    fn test_extract_events_from_type_literal() {
        let mut emits = FxHashSet::default();
        extract_events_from_type_literal("{ (e: 'click'): void; (e: 'update'): void }", &mut emits);
        insta::assert_debug_snapshot!(snapshot_emits(&emits));
    }

    #[test]
    fn test_extract_events_call_signature_with_payload() {
        let mut emits = FxHashSet::default();
        extract_events_from_type_literal("{ (e: 'click', payload: MouseEvent): void }", &mut emits);
        insta::assert_debug_snapshot!(snapshot_emits(&emits));
    }

    #[test]
    fn test_gen_runtime_emits_empty() {
        let ctx = ScriptCompileContext::new("");
        let result = gen_runtime_emits(&ctx, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_gen_runtime_emits_with_models() {
        let ctx = ScriptCompileContext::new("");
        let result = gen_runtime_emits(
            &ctx,
            &[
                "modelValue".to_compact_string(),
                "count".to_compact_string(),
            ],
        );
        assert!(result.is_some());
        let emits = result.unwrap();
        insta::assert_snapshot!(emits.as_str());
    }
}
