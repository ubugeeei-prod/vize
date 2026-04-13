//! Source code transformation for destructured props.
//!
//! Rewrites identifier references to destructured props
//! (e.g., `foo` becomes `__props.foo`) using AST-based analysis
//! with a text-based fallback.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashMap;

use super::collector::collect_identifier_rewrites;
use super::helpers::{transform_props_text_based, PROPS_REST_SENTINEL};
use super::PropsDestructuredBindings;
use vize_carton::{profile, String, ToCompactString};

/// Transform destructured props references in source code.
/// Rewrites `foo` to `__props.foo` for destructured props.
pub fn transform_destructured_props(
    source: &str,
    destructured: &PropsDestructuredBindings,
) -> String {
    if destructured.is_empty() {
        return source.to_compact_string();
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

    // Try AST-based transformation first
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = profile!(
        "atelier.props_destructure.parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if !ret.panicked {
        // Collect rewrites: (start, end, replacement)
        let mut rewrites: Vec<(usize, usize, String)> = Vec::new();

        // Walk the AST to find identifier references
        profile!(
            "atelier.props_destructure.collect_rewrites",
            collect_identifier_rewrites(&ret.program, source, &local_to_key, &mut rewrites)
        );

        // Apply rewrites if any found (empty rewrites means all props are shadowed or unused)
        if !rewrites.is_empty() {
            // Apply rewrites in reverse order to preserve positions
            rewrites.sort_by(|a, b| b.0.cmp(&a.0));

            let mut result = source.to_compact_string();
            for (start, end, replacement) in rewrites {
                result.replace_range(start..end, &replacement);
            }
            return result;
        }

        // AST parsing succeeded but no rewrites needed (props are shadowed or unused)
        return source.to_compact_string();
    }

    // Fallback: Simple text-based transformation
    // This handles cases where AST parsing failed
    profile!(
        "atelier.props_destructure.text_fallback",
        transform_props_text_based(source, &local_to_key)
    )
}
