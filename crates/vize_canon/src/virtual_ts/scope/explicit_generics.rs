//! `<!-- @vue-generic {T, U} -->`: explicit type arguments for the component
//! usage the comment precedes.
//!
//! The usage resolves through a synthetic binding: the component instantiated
//! with the authored arguments (an instantiation expression), reduced to its
//! call signature. Props, listeners and slots are then checked against that
//! one signature, exactly like a functional component's, instead of inferring
//! the type parameters from the usage. The arguments map to the comment, so a
//! constraint violation reports where it was written.

use crate::virtual_ts::types::VizeMapping;
use vize_carton::{FxHashMap, String, append, cstr};
use vize_croquis::Croquis;

const DIRECTIVE: &str = "@vue-generic";

#[derive(Default)]
pub(crate) struct ExplicitGenerics {
    by_usage_start: FxHashMap<u32, ExplicitGeneric>,
}

struct ExplicitGeneric {
    alias: String,
    /// Template-relative range of the text between the comment's braces.
    arguments: std::ops::Range<usize>,
}

impl ExplicitGenerics {
    pub(crate) fn collect(summary: &Croquis, template_source: Option<&str>) -> Self {
        let Some(source) = template_source.filter(|source| source.contains(DIRECTIVE)) else {
            return Self::default();
        };
        let by_usage_start = summary
            .component_usages
            .iter()
            .filter_map(|usage| {
                let arguments = preceding_arguments(source, usage.start as usize)?;
                Some((
                    usage.start,
                    ExplicitGeneric {
                        alias: cstr!("__vize_generic_{}", usage.start),
                        arguments,
                    },
                ))
            })
            .collect();
        Self { by_usage_start }
    }

    /// The binding the usage starting at `usage_start` resolves through.
    pub(crate) fn alias(&self, usage_start: u32) -> Option<&str> {
        self.by_usage_start
            .get(&usage_start)
            .map(|generic| generic.alias.as_str())
    }

    /// `resolved`, unless the usage carries explicit type arguments.
    pub(crate) fn usage_reference(&self, usage_start: u32, resolved: String) -> String {
        self.alias(usage_start).map_or(resolved, String::from)
    }

    /// The binding of the `component` usage whose start tag contains `offset`.
    pub(crate) fn alias_at(&self, summary: &Croquis, component: &str, offset: u32) -> Option<&str> {
        if self.by_usage_start.is_empty() {
            return None;
        }
        summary
            .component_usages
            .iter()
            .find(|usage| {
                usage.start <= offset && offset < usage.end && usage.name.as_str() == component
            })
            .and_then(|usage| self.alias(usage.start))
    }

    /// Declare every alias. `component_ref` resolves a usage's tag to the
    /// value expression the rest of the template checks it through.
    pub(crate) fn emit(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        summary: &Croquis,
        template_source: Option<&str>,
        template_offset: u32,
        component_ref: impl Fn(&str) -> String,
    ) {
        let Some(source) = template_source.filter(|_| !self.by_usage_start.is_empty()) else {
            return;
        };
        ts.push_str(
            "  // Explicit `@vue-generic` arguments: the instantiated call signature.\n  const __vizeExplicitGeneric = <C>(component: C) => component as unknown as (C extends (...args: infer __A) => infer __R ? (...args: __A) => __R : C);\n",
        );
        for usage in &summary.component_usages {
            let Some(generic) = self.by_usage_start.get(&usage.start) else {
                continue;
            };
            append!(
                *ts,
                "  const {} = __vizeExplicitGeneric({}<",
                generic.alias,
                component_ref(usage.name.as_str())
            );
            let generated_start = ts.len();
            ts.push_str(&source[generic.arguments.clone()]);
            mappings.push(VizeMapping {
                gen_range: generated_start..ts.len(),
                src_range: template_offset as usize + generic.arguments.start
                    ..template_offset as usize + generic.arguments.end,
                sub_spans: Vec::new(),
            });
            ts.push_str(">);\n");
        }
    }
}

/// The braces' content of the `@vue-generic` comment among the comments that
/// directly precede the element at `element_start`; other directive comments
/// (`@vue-expect-error`) may sit between the two.
fn preceding_arguments(source: &str, element_start: usize) -> Option<std::ops::Range<usize>> {
    let mut end = element_start;
    loop {
        let before = source.get(..end)?.trim_end();
        let comment_end = before.strip_suffix("-->")?.len();
        let comment_start = before[..comment_end].rfind("<!--")?;
        let content_start = comment_start + "<!--".len();
        let content = &before[content_start..comment_end];
        if let Some(rest) = content.trim_start().strip_prefix(DIRECTIVE) {
            let rest_start = comment_end - rest.len();
            let text = rest.trim();
            if !(text.starts_with('{') && text.ends_with('}')) {
                return None;
            }
            let open = rest_start + rest.find('{')?;
            let close = rest_start + rest.rfind('}')?;
            return Some(open + 1..close);
        }
        end = comment_start;
    }
}

#[cfg(test)]
mod tests {
    use super::preceding_arguments;

    #[test]
    fn reads_the_generic_comment_directly_preceding_an_element() {
        let source =
            "<a />\n<!-- @vue-generic {string, number} -->\n<!-- @vue-expect-error -->\n  <Comp />";
        let element = source.find("<Comp").unwrap();
        assert_eq!(
            preceding_arguments(source, element).map(|range| &source[range]),
            Some("string, number")
        );
        let first = source.find("<a").unwrap();
        assert_eq!(preceding_arguments(source, first), None);
        let detached = "<!-- @vue-generic {string} --><b /><Comp />";
        assert_eq!(
            preceding_arguments(detached, detached.find("<Comp").unwrap()),
            None
        );
        let malformed = "<!-- @vue-generic string --><Comp />";
        assert_eq!(
            preceding_arguments(malformed, malformed.find("<Comp").unwrap()),
            None
        );
    }
}
