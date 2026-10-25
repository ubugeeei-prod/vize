//! Pure, scope-addressed public-instance rewrites, reused by every emission.

use std::cell::Ref;

use super::reserved_props::rewrite_reserved_with_segments;
use super::spread_reserved_props::{
    BindingRewrite, SpreadRewriteSegment, rewrite_instance_references,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;

pub(super) fn rewrite<'a>(
    expression: &str,
    bindings: &'a TemplateBindingAccess,
) -> Option<Ref<'a, BindingRewrite>> {
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
                diagnostic_prefixes: instance
                    .diagnostic_prefixes
                    .into_iter()
                    .filter_map(|prefix| {
                        // A dollar token cannot be a reserved-word replacement;
                        // its complete source token lies in one unchanged segment.
                        let inner = reserved.segments.iter().find(|inner| {
                            inner.generated.start <= prefix.source.start
                                && prefix.source.end <= inner.generated.end
                        })?;
                        Some(SpreadRewriteSegment {
                            generated: prefix.generated,
                            source: inner.source.start + prefix.source.start - inner.generated.start
                                ..inner.source.start + prefix.source.end - inner.generated.start,
                        })
                    })
                    .collect(),
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
        assert_eq!(actual.diagnostic_prefixes.len(), 5);
        for prefix in &actual.diagnostic_prefixes {
            assert_eq!(actual.code.get(prefix.generated.clone()), Some("__ctx."));
            assert_eq!(expression.get(prefix.source.clone()), Some("$ghost"));
        }
        assert_eq!(rewrite("'$ghost' /* $ghost */", &bindings).as_deref(), None);
        assert_eq!(
            rewrite("(($ghost) => $ghost)(1)", &bindings).as_deref(),
            None
        );
    }

    #[test]
    fn reserved_spelling_composition_keeps_the_prefix_on_its_own_dollar_read() {
        let analyzer = Analyzer::with_options(AnalyzerOptions::full());
        let summary = analyzer.finish();
        let options = VirtualTsOptions::default();
        let bindings = TemplateBindingAccess::from_props(["class".into()].into_iter().collect())
            .with_instance_globals(&summary, &options, &[], false, true);
        let expression = "class + $ghost.known";
        let actual = rewrite(expression, &bindings).unwrap();
        assert_eq!(actual.code, "props[\"class\"] + __ctx.$ghost.known");
        assert_eq!(actual.diagnostic_prefixes.len(), 1);
        let prefix = actual.diagnostic_prefixes.first().unwrap();
        assert_eq!(actual.code.get(prefix.generated.clone()), Some("__ctx."));
        assert_eq!(expression.get(prefix.source.clone()), Some("$ghost"));
        let mut emitted = Vec::new();
        for base in [100, 1000] {
            bindings.record_instance_diagnostic_prefix(crate::virtual_ts::VizeMapping {
                gen_range: base + prefix.generated.start..base + prefix.generated.end,
                src_range: 50 + prefix.source.start..50 + prefix.source.end,
                sub_spans: Vec::new(),
            });
        }
        bindings.append_instance_diagnostic_prefixes(&mut emitted);
        assert_eq!(
            emitted
                .iter()
                .map(|row| (row.gen_range.clone(), row.src_range.clone()))
                .collect::<Vec<_>>(),
            vec![(117..123, 58..64), (1017..1023, 58..64)]
        );
        bindings.append_instance_diagnostic_prefixes(&mut emitted);
        assert_eq!(
            emitted.len(),
            2,
            "collector drains actual positions, separately from cached relative spans"
        );
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
        assert!(std::ptr::eq(&*a, &*b));
        drop((a, b));
        let other = bindings.for_scope(ScopeId::new(42));
        let c = rewrite("$ghost.known", &other).unwrap();
        assert_eq!(c.code, "__ctx.$ghost.known");
        let other_file = TemplateBindingAccess::default().with_instance_globals(
            &summary,
            &options,
            &[],
            false,
            true,
        );
        assert!(!std::ptr::eq(
            &*c,
            &*rewrite("$ghost.known", &other_file).unwrap()
        ));
        drop(c);
        let calls = Cell::new(0);
        for _ in 0..2 {
            assert_eq!(
                first
                    .cached_instance_rewrite("not-a-reference", || {
                        calls.set(calls.get() + 1);
                        None
                    })
                    .as_deref(),
                None
            );
        }
        assert_eq!(calls.get(), 1);
        let disabled = TemplateBindingAccess::default();
        assert_eq!(rewrite("$ghost.known", &disabled).as_deref(), None);
        assert_eq!(rewrite("ordinary.known", &bindings).as_deref(), None);
    }
}
