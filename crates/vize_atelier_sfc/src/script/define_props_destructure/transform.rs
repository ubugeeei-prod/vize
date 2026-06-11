//! Source code transformation for destructured props.
//!
//! Rewrites identifier references to destructured props
//! (e.g., `foo` becomes `__props.foo`) using AST-based analysis.
//!
//! When the source cannot be parsed by OXC, the transform does **not** fall
//! back to a text rewrite (which would silently produce wrong code); instead it
//! surfaces a structured compile diagnostic so the failure is visible.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashMap;

use super::PropsDestructuredBindings;
use super::collector::collect_identifier_rewrites;
use super::helpers::PROPS_REST_SENTINEL;
use crate::types::SfcError;
use vize_carton::{String, ToCompactString, profile};

/// Transform destructured props references in source code.
///
/// Rewrites `foo` to `__props.foo` for destructured props using AST-based
/// analysis. Returns `Err(SfcError)` (code `DEFINE_PROPS_DESTRUCTURE_PARSE`)
/// when the setup body cannot be parsed, rather than silently text-rewriting
/// identifiers and masking the real parse error.
pub fn transform_destructured_props(
    source: &str,
    destructured: &PropsDestructuredBindings,
) -> Result<String, SfcError> {
    if destructured.is_empty() {
        return Ok(source.to_compact_string());
    }

    // Build map of local name -> prop key
    let mut local_to_key: FxHashMap<&str, &str> = FxHashMap::default();
    for (key, binding) in &destructured.bindings {
        local_to_key.insert(binding.local.as_str(), key.as_str());
    }

    // Handle rest spread identifier: `const { a, ...rest } = defineProps()`
    // References to `rest` should be rewritten to `__props`
    // (e.g., `rest.foo` becomes `__props.foo`)
    if let Some(ref rest_id) = destructured.rest_id {
        local_to_key.insert(rest_id.as_str(), PROPS_REST_SENTINEL);
    }

    // AST-based transformation.
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = profile!(
        "atelier.props_destructure.parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if ret.panicked {
        // Parsing failed: emit a structured diagnostic and skip the transform
        // instead of falling back to a text rewrite that would silently emit
        // wrong code (referencing un-prefixed destructured props at runtime).
        return Err(SfcError {
            message: String::from(
                "Failed to parse <script setup> while rewriting destructured props from \
                 defineProps(). The destructured prop references could not be transformed.",
            ),
            code: Some("DEFINE_PROPS_DESTRUCTURE_PARSE".to_compact_string()),
            loc: None,
        });
    }

    // Collect rewrites: (start, end, replacement)
    let mut rewrites: Vec<(usize, usize, String)> = Vec::new();

    // Walk the AST to find identifier references
    profile!(
        "atelier.props_destructure.collect_rewrites",
        collect_identifier_rewrites(&ret.program, source, &local_to_key, &mut rewrites)
    );

    // Apply rewrites if any found (empty rewrites means all props are shadowed
    // or unused).
    if rewrites.is_empty() {
        return Ok(source.to_compact_string());
    }

    // Apply rewrites in reverse order to preserve positions
    rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

    let mut result = source.to_compact_string();
    for (start, end, replacement) in rewrites {
        result.replace_range(start..end, &replacement);
    }
    Ok(result)
}
