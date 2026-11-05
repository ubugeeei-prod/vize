//! Pure, scope-addressed public-instance rewrites, reused by every emission.

use std::cell::Ref;

use super::reserved_props::rewrite_reserved_with_segments;
use super::spread_reserved_props::{
    BindingRewrite, ParsedInstanceReferences, SpreadRewriteSegment,
    materialize_instance_references, prepare_instance_references,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;

/// Receiver-independent parse result for one actual scope/text input.
#[derive(Debug)]
pub(crate) struct PreparedInstanceRewrite {
    reserved: Option<BindingRewrite>,
    instance: ParsedInstanceReferences,
}

impl PreparedInstanceRewrite {
    pub(crate) fn normalized_names(&self) -> &vize_carton::FxHashSet<vize_carton::String> {
        &self.instance.normalized_names
    }

    pub(crate) fn has_instance_references(&self) -> bool {
        self.instance.has_instance_references()
    }

    pub(crate) fn receiver_is_shadowed(&self) -> bool {
        self.instance.receiver_is_shadowed()
    }

    pub(crate) fn materialize(
        self,
        expression: &str,
        bindings: &TemplateBindingAccess,
    ) -> Option<BindingRewrite> {
        let intermediate = self
            .reserved
            .as_ref()
            .map_or(expression, |row| row.code.as_str());
        let instance = materialize_instance_references(intermediate, self.instance, bindings);
        match (self.reserved, instance) {
            (None, None) => None,
            (Some(rewrite), None) | (None, Some(rewrite)) => Some(rewrite),
            (Some(reserved), Some(instance)) => Some(BindingRewrite {
                code: instance.code,
                segments: compose(&reserved.segments, &instance.segments),
                diagnostic_prefixes: compose_diagnostic_prefixes(
                    &reserved.segments,
                    &reserved.diagnostic_prefixes,
                    &instance.segments,
                    &instance.diagnostic_prefixes,
                ),
            }),
        }
    }
}

pub(crate) fn prepare_instance_rewrite(
    expression: &str,
    bindings: &TemplateBindingAccess,
) -> PreparedInstanceRewrite {
    let reserved = rewrite_reserved_with_segments(expression, bindings);
    let intermediate = reserved
        .as_ref()
        .map_or(expression, |row| row.code.as_str());
    let instance = vize_carton::profile!(
        "canon.instance_global_rewrite.parse",
        prepare_instance_references(intermediate, bindings)
    );
    PreparedInstanceRewrite { reserved, instance }
}

pub(super) fn rewrite<'a>(
    expression: &str,
    bindings: &'a TemplateBindingAccess,
) -> Option<Ref<'a, BindingRewrite>> {
    if !bindings.has_instance_globals()
        || (!bindings.has_authored_instance_calls() && !expression.contains('$'))
    {
        return None;
    }
    bindings.cached_instance_rewrite(expression, || {
        prepare_instance_rewrite(expression, bindings).materialize(expression, bindings)
    })
}

/// Both rewrites map only unchanged bytes and actual property-name tokens.
/// Their segments are therefore affine, even when a shorthand maps twice.
pub(super) fn compose(
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

/// Diagnostic anchors are not affine: preserve their entire receiver range
/// while transporting its position through unchanged navigation segments.
pub(super) fn compose_diagnostic_prefixes(
    first: &[SpreadRewriteSegment],
    first_prefixes: &[SpreadRewriteSegment],
    second: &[SpreadRewriteSegment],
    second_prefixes: &[SpreadRewriteSegment],
) -> Vec<SpreadRewriteSegment> {
    let mut index = 0;
    let mut second_anchors = second_prefixes
        .iter()
        .filter_map(|prefix| {
            while first
                .get(index)
                .is_some_and(|row| row.generated.end <= prefix.source.start)
            {
                index += 1;
            }
            first
                .get(index)
                .filter(|row| {
                    row.generated.start <= prefix.source.start
                        && prefix.source.end <= row.generated.end
                })
                .map(|row| SpreadRewriteSegment {
                    generated: prefix.generated.clone(),
                    source: row.source.start + prefix.source.start - row.generated.start
                        ..row.source.start + prefix.source.end - row.generated.start,
                })
        })
        .peekable();
    let mut index = 0;
    let mut first_anchors = first_prefixes
        .iter()
        .filter_map(|prefix| {
            while second
                .get(index)
                .is_some_and(|row| row.source.end <= prefix.generated.start)
            {
                index += 1;
            }
            second
                .get(index)
                .filter(|row| {
                    row.source.start <= prefix.generated.start
                        && prefix.generated.end <= row.source.end
                })
                .map(|row| SpreadRewriteSegment {
                    generated: row.generated.start + prefix.generated.start - row.source.start
                        ..row.generated.start + prefix.generated.end - row.source.start,
                    source: prefix.source.clone(),
                })
        })
        .peekable();
    let mut result = Vec::with_capacity(first_prefixes.len() + second_prefixes.len());
    // Keep the next composition's cursor monotonic without sorting or rescanning.
    while let (Some(first), Some(second)) = (first_anchors.peek(), second_anchors.peek()) {
        if first.generated.start <= second.generated.start {
            result.extend(first_anchors.next());
        } else {
            result.extend(second_anchors.next());
        }
    }
    result.extend(first_anchors);
    result.extend(second_anchors);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_ts::types::VirtualTsOptions;
    use std::cell::Cell;
    use vize_croquis::{Analyzer, AnalyzerOptions, ScopeId};

    mod receiver_plan;

    #[test]
    fn lexical_contextual_and_instance_diagnostic_anchors_all_survive() {
        let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
        let options = VirtualTsOptions::default();
        let bindings = TemplateBindingAccess::from_props(
            ["static".into(), "from".into()].into_iter().collect(),
        )
        .with_instance_globals(&summary, &options, &[], false, true);
        for (expression, expected) in [
            ("static + from + $ghost", vec!["static", "from", "$ghost"]),
            ("static + $ghost + from", vec!["static", "$ghost", "from"]),
        ] {
            let actual = rewrite(expression, &bindings).unwrap();
            assert_eq!(
                actual
                    .diagnostic_prefixes
                    .iter()
                    .map(|prefix| expression.get(prefix.source.clone()).unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

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
        drop(actual);
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
        assert_eq!(actual.diagnostic_prefixes.len(), 2);
        let prefix = actual.diagnostic_prefixes.last().unwrap();
        assert_eq!(actual.code.get(prefix.generated.clone()), Some("__ctx."));
        assert_eq!(expression.get(prefix.source.clone()), Some("$ghost"));
        let reserved_prefix = actual.diagnostic_prefixes.first().unwrap();
        assert_eq!(
            actual.code.get(reserved_prefix.generated.clone()),
            Some("props[\"")
        );
        assert_eq!(
            expression.get(reserved_prefix.source.clone()),
            Some("class")
        );
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
