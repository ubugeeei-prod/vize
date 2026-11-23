use super::{GraphError, OpGraph, OpId, Span, TemplateGraphError, TemplateReachBatch};

/// The retained graph, original template text and byte offset from one document revision.
pub struct TemplateSourceSites<'db> {
    graph: &'db OpGraph<'static>,
    text: &'db str,
    start: u32,
}

impl<'db> TemplateReachBatch<'db> {
    /// Read the block's source and offset once for a group of operation lookups.
    #[must_use]
    pub fn source_sites(&self) -> TemplateSourceSites<'db> {
        TemplateSourceSites {
            graph: self.graph,
            text: self.view.source().text.as_str(),
            start: self.view.start(),
        }
    }
}

impl<'db> TemplateSourceSites<'db> {
    pub(super) fn new(graph: &'db OpGraph<'static>, text: &'db str, start: u32) -> Self {
        Self { graph, text, start }
    }

    /// The original, checked graph whose operation IDs this view resolves.
    #[must_use]
    pub fn graph(&self) -> &'db OpGraph<'static> {
        self.graph
    }

    /// Look up an ID from this graph. The file-absolute byte span and the borrowed
    /// spelling both refer to the revision that opened this view.
    #[inline]
    pub fn get(&self, op: OpId) -> Result<(Span, &'db str), TemplateGraphError> {
        let relative = self
            .graph
            .operation(op)
            .ok_or(TemplateGraphError::Graph(GraphError::UnknownOp(op)))?
            .span;
        project(self.text, self.start, relative).ok_or(TemplateGraphError::OutsideBlock(relative))
    }
}

pub(super) fn project(text: &str, start: u32, span: Span) -> Option<(Span, &str)> {
    let spelling = text.get(span.start as usize..span.end as usize)?;
    let authored = Span::new(start.checked_add(span.start)?, start.checked_add(span.end)?);
    Some((authored, spelling))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_requires_utf8_boundaries_and_representable_document_offsets() {
        let text = "x\r\n雪🌸";
        let len = u32::try_from(text.len()).unwrap();
        for base in [0, 17, u32::MAX - len, u32::MAX] {
            for start in 0..=len + 1 {
                for end in 0..=len + 1 {
                    let range = start as usize..end as usize;
                    let expected = if start <= end
                        && text.is_char_boundary(range.start)
                        && text.is_char_boundary(range.end)
                        && u64::from(base) + u64::from(end) <= u64::from(u32::MAX)
                    {
                        Some((Span::new(base + start, base + end), &text[range]))
                    } else {
                        None
                    };
                    assert_eq!(project(text, base, Span::new(start, end)), expected);
                }
            }
        }
    }
}
