//! Link generated aliases to authored declarations through verbatim source spans.

use std::ops::Range;

use crate::virtual_ts::VizeMapping;

pub(in crate::virtual_ts::generator) fn mapped_binding_range(
    mappings: &[VizeMapping],
    declaration: &Range<usize>,
) -> Option<Range<usize>> {
    mappings
        .iter()
        .flat_map(|mapping| {
            mapping
                .sub_spans
                .iter()
                .map(|span| (&span.src_range, &span.gen_range))
                .chain(std::iter::once((&mapping.src_range, &mapping.gen_range)))
        })
        .filter(|(source, generated)| {
            source.start <= declaration.start
                && declaration.end <= source.end
                && source.len() == generated.len()
        })
        .min_by_key(|(source, _)| source.len())
        .map(|(source, generated)| {
            let start = generated.start + declaration.start - source.start;
            start..start + declaration.len()
        })
}

#[cfg(test)]
mod tests {
    use super::{VizeMapping, mapped_binding_range};
    use crate::virtual_ts::VizeSubSpan;

    #[test]
    fn exact_sub_spans_take_precedence_over_containing_lines() {
        let mapping = VizeMapping {
            gen_range: 20..55,
            src_range: 100..142,
            sub_spans: vec![VizeSubSpan {
                gen_range: 22..55,
                src_range: 109..142,
            }],
        };
        assert_eq!(mapped_binding_range(&[mapping], &(115..121)), Some(28..34));
    }

    #[test]
    fn nonlinear_or_missing_mappings_do_not_invent_binding_links() {
        let mapping = VizeMapping {
            gen_range: 20..55,
            src_range: 100..142,
            sub_spans: vec![],
        };
        assert_eq!(mapped_binding_range(&[mapping], &(115..121)), None);
        assert_eq!(mapped_binding_range(&[], &(115..121)), None);
    }
}
