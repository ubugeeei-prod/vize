//! Pure, scope-addressed public-instance rewrites, reused by every emission.

use std::rc::Rc;

use super::reserved_props::rewrite_reserved_with_segments;
use super::spread_reserved_props::{
    BindingRewrite, SpreadRewriteSegment, rewrite_instance_references,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;

pub(super) fn rewrite(
    expression: &str,
    bindings: &TemplateBindingAccess,
) -> Option<Rc<BindingRewrite>> {
    if !bindings.has_instance_globals() || !expression.contains('$') {
        return None;
    }
    bindings.cached_instance_rewrite(expression, || {
        // Preserve the existing reserved-word grammar before asking OXC for
        // actual value references. Its source segments compose with the AST
        // rewrite; synthetic receivers never acquire authored identity.
        let reserved = rewrite_reserved_with_segments(expression, bindings);
        let intermediate = reserved
            .as_ref()
            .map_or(expression, |rewrite| rewrite.code.as_str());
        let instance = vize_carton::profile!(
            "canon.instance_global_rewrite.parse",
            rewrite_instance_references(intermediate, bindings)
        );
        match (reserved, instance) {
            (None, None) => None,
            (Some(rewrite), None) | (None, Some(rewrite)) => Some(rewrite),
            (Some(reserved), Some(instance)) => Some(BindingRewrite {
                code: instance.code,
                segments: compose(&reserved.segments, &instance.segments),
            }),
        }
    })
}

/// Both rewrites map only unchanged bytes and actual property-name tokens.
/// Their segments are therefore affine, even when a shorthand maps twice.
fn compose(
    first: &[SpreadRewriteSegment],
    second: &[SpreadRewriteSegment],
) -> Vec<SpreadRewriteSegment> {
    let mut result = Vec::new();
    let mut first_index = 0;
    for outer in second {
        while first
            .get(first_index)
            .is_some_and(|inner| inner.generated.end <= outer.source.start)
        {
            first_index += 1;
        }
        for inner in first.iter().skip(first_index) {
            if inner.generated.start >= outer.source.end {
                break;
            }
            let start = outer.source.start.max(inner.generated.start);
            let end = outer.source.end.min(inner.generated.end);
            if start >= end {
                continue;
            }
            result.push(SpreadRewriteSegment {
                generated: outer.generated.start + start - outer.source.start
                    ..outer.generated.start + end - outer.source.start,
                source: inner.source.start + start - inner.generated.start
                    ..inner.source.start + end - inner.generated.start,
            });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_ts::types::VirtualTsOptions;
    use std::cell::Cell;
    use vize_croquis::{Analyzer, AnalyzerOptions, ScopeId};

    #[test]
    fn semantic_reads_preserve_keys_literals_types_and_function_bindings() {
        let analyzer = Analyzer::with_options(AnalyzerOptions::full());
        let summary = analyzer.finish();
        let options = VirtualTsOptions::default();
        let bindings = TemplateBindingAccess::default().with_instance_globals(
            &summary,
            &options,
            &[],
            false,
            true,
        );
        let expression = "({ $ghost, value: $ghost, [$ghost]: 1, key: item.$ghost, text: '$ghost', local: (($ghost) => $ghost)(1), typed: (value as $ghost), query: (() => { type T = typeof $ghost; return $ghost })() })";
        let actual = rewrite(expression, &bindings).expect("free value references");
        assert_eq!(
            actual.code,
            "({ $ghost: __ctx.$ghost, value: __ctx.$ghost, [__ctx.$ghost]: 1, key: item.$ghost, text: '$ghost', local: (($ghost) => $ghost)(1), typed: (value as $ghost), query: (() => { type T = typeof __ctx.$ghost; return __ctx.$ghost })() })"
        );
        for segment in &actual.segments {
            assert_eq!(
                actual.code.get(segment.generated.clone()),
                expression.get(segment.source.clone())
            );
        }
        assert_eq!(rewrite("'$ghost' /* $ghost */", &bindings), None);
        assert_eq!(rewrite("(($ghost) => $ghost)(1)", &bindings), None);
    }

    #[test]
    fn relative_cache_is_shared_by_views_but_isolated_by_scope_and_admission() {
        let analyzer = Analyzer::with_options(AnalyzerOptions::full());
        let summary = analyzer.finish();
        let options = VirtualTsOptions::default();
        let bindings = TemplateBindingAccess::default().with_instance_globals(
            &summary,
            &options,
            &[],
            false,
            true,
        );
        let first = bindings.for_scope(ScopeId::ROOT);
        let second = bindings.for_scope(ScopeId::ROOT);
        let a = rewrite("$ghost.known", &first).unwrap();
        let b = rewrite("$ghost.known", &second).unwrap();
        assert!(Rc::ptr_eq(&a, &b));
        let other = bindings.for_scope(ScopeId::new(42));
        let c = rewrite("$ghost.known", &other).unwrap();
        assert!(!Rc::ptr_eq(&a, &c));
        let other_file = TemplateBindingAccess::default().with_instance_globals(
            &summary,
            &options,
            &[],
            false,
            true,
        );
        assert!(!Rc::ptr_eq(
            &a,
            &rewrite("$ghost.known", &other_file).unwrap()
        ));
        let calls = Cell::new(0);
        for _ in 0..2 {
            assert_eq!(
                first.cached_instance_rewrite("not-a-reference", || {
                    calls.set(calls.get() + 1);
                    None
                }),
                None
            );
        }
        assert_eq!(calls.get(), 1);
        let disabled = TemplateBindingAccess::default();
        assert_eq!(rewrite("$ghost.known", &disabled), None);
        assert_eq!(rewrite("ordinary.known", &bindings), None);
    }
}
