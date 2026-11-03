use super::{JsxMarkupIssue as Issue, JsxMarkupIssueKind as Kind};
use crate::lang::js::jsx_body::records::Record;
use crate::resolution::SyntaxKind as Syntax;
use alloc::vec::Vec;

pub(super) struct Children<'r, 'a> {
    records: &'r [Record<'a>],
    next: usize,
    end: usize,
}

pub(super) fn children<'r, 'a>(records: &'r [Record<'a>], index: usize) -> Children<'r, 'a> {
    Children {
        records,
        next: index + 1,
        end: records
            .get(index)
            .map_or(index + 1, |record| record.subtree_end),
    }
}

impl<'a> Iterator for Children<'_, 'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.next >= self.end {
            return None;
        }
        let index = self.next;
        self.next = self.records.get(index)?.subtree_end;
        Some(index)
    }
}

pub(super) fn roots(records: &[Record<'_>], source: &str) -> (Vec<usize>, Vec<Issue>) {
    let mut roots = Vec::new();
    let mut issues = Vec::new();
    for (index, record) in records
        .iter()
        .enumerate()
        .filter(|(_, row)| row.parent.is_none())
    {
        let subtree = records.get(index..record.subtree_end).unwrap_or(&[]);
        if !subtree
            .iter()
            .any(|row| matches!(row.kind, Syntax::Element | Syntax::Fragment))
        {
            continue;
        }
        let mut body = children(records, index);
        let first = body.next();
        let root = first.and_then(|index| records.get(index).map(|row| (index, row)));
        let result = match root {
            Some((root, row))
                if record.kind == Syntax::Expression
                    && matches!(row.kind, Syntax::Element | Syntax::Fragment)
                    && record.span == row.span
                    && body.next().is_none() =>
            {
                node(records, root, source).map(|()| root)
            }
            _ => Err(Issue {
                kind: Kind::RootExpression,
                span: record.span,
            }),
        };
        match result {
            Ok(root) => roots.push(root),
            Err(issue) => issues.push(issue),
        }
    }
    (roots, issues)
}

fn node(records: &[Record<'_>], index: usize, source: &str) -> Result<(), Issue> {
    if record(records, index)?.kind == Syntax::Fragment {
        return fragment(records, index, source);
    }
    element(records, index, source)
}

fn fragment(records: &[Record<'_>], index: usize, source: &str) -> Result<(), Issue> {
    let row = record(records, index)?;
    let mut body = children(records, index);
    let opening = body.next().and_then(|index| records.get(index));
    if opening.is_none_or(|opening| {
        opening.kind != Syntax::FragmentOpening || opening.span.start != row.span.start
    }) {
        return Err(Issue {
            kind: Kind::InvalidRecord,
            span: row.span,
        });
    }
    while let Some(index) = body.next() {
        let row_child = record(records, index)?;
        if row_child.kind == Syntax::FragmentClosing {
            return if row_child.span.end == row.span.end && body.next().is_none() {
                Ok(())
            } else {
                Err(Issue {
                    kind: Kind::InvalidRecord,
                    span: row_child.span,
                })
            };
        }
        child(records, index, source)?;
    }
    Err(Issue {
        kind: Kind::InvalidRecord,
        span: row.span,
    })
}

fn element(records: &[Record<'_>], index: usize, source: &str) -> Result<(), Issue> {
    let row = record(records, index)?;
    let mut body = children(records, index);
    let opening = body.next().ok_or(Issue {
        kind: Kind::InvalidRecord,
        span: row.span,
    })?;
    let tag = header(records, opening, source)?;
    for child in body {
        let row = record(records, child)?;
        match row.kind {
            Syntax::Closing => {
                let mut names = children(records, child);
                if names
                    .next()
                    .and_then(|name| records.get(name))
                    .is_some_and(|row| row.kind == Syntax::Intrinsic(tag))
                    && names.next().is_none()
                {
                    continue;
                }
                return Err(Issue {
                    kind: Kind::InvalidRecord,
                    span: row.span,
                });
            }
            _ => self::child(records, child, source)?,
        }
    }
    Ok(())
}

fn child(records: &[Record<'_>], index: usize, source: &str) -> Result<(), Issue> {
    let row = record(records, index)?;
    let kind = match row.kind {
        Syntax::Element | Syntax::Fragment => return node(records, index, source),
        Syntax::Text(value) => {
            let original = source.get(row.span.start as usize..row.span.end as usize);
            if super::whitespace::ignored(value) && original == Some(value) {
                return Ok(());
            }
            return normalized(row, value, source);
        }
        Syntax::Container if empty(records, index) => return Ok(()),
        Syntax::Container => Kind::DynamicChild,
        Syntax::SpreadChild => Kind::SpreadChild,
        _ => Kind::InvalidRecord,
    };
    Err(Issue {
        kind,
        span: row.span,
    })
}

fn header<'a>(records: &[Record<'a>], index: usize, source: &str) -> Result<&'a str, Issue> {
    let row = record(records, index)?;
    let mut body = children(records, index);
    let name = body
        .next()
        .and_then(|index| records.get(index))
        .ok_or(Issue {
            kind: Kind::InvalidRecord,
            span: row.span,
        })?;
    let tag = match name.kind {
        Syntax::Intrinsic(tag) if !matches!(tag, "svg" | "math") => tag,
        Syntax::Intrinsic(_) => {
            return Err(Issue {
                kind: Kind::Namespace,
                span: name.span,
            });
        }
        Syntax::Component(_) | Syntax::Member => {
            return Err(Issue {
                kind: Kind::Component,
                span: name.span,
            });
        }
        _ => {
            return Err(Issue {
                kind: Kind::InvalidRecord,
                span: name.span,
            });
        }
    };
    for child in body {
        let row = record(records, child)?;
        match row.kind {
            Syntax::Attribute(name) => attribute(records, child, name, source)?,
            Syntax::SpreadAttribute => {
                return Err(Issue {
                    kind: Kind::SpreadAttribute,
                    span: row.span,
                });
            }
            _ => {
                return Err(Issue {
                    kind: Kind::InvalidRecord,
                    span: row.span,
                });
            }
        }
    }
    Ok(tag)
}

fn attribute(records: &[Record<'_>], index: usize, name: &str, source: &str) -> Result<(), Issue> {
    let row = record(records, index)?;
    if name.starts_with("v-")
        || name.starts_with("vModel")
        || name.starts_with("vSlots")
        || name.starts_with("on")
        || matches!(
            name,
            "key" | "ref" | "is" | "innerHTML" | "textContent" | "xmlns"
        )
    {
        return Err(Issue {
            kind: Kind::SpecialAttribute,
            span: row.span,
        });
    }
    let mut parts = children(records, index);
    if parts
        .next()
        .and_then(|index| records.get(index))
        .is_none_or(|row| row.kind != Syntax::AttributeName(name))
    {
        return Err(Issue {
            kind: Kind::InvalidRecord,
            span: row.span,
        });
    }
    if let Some(value) = parts.next().and_then(|index| records.get(index)) {
        let Syntax::AttributeString(text) = value.kind else {
            return Err(Issue {
                kind: Kind::DynamicAttribute,
                span: value.span,
            });
        };
        normalized(value, text, source)?;
    }
    if parts.next().is_some() {
        return Err(Issue {
            kind: Kind::InvalidRecord,
            span: row.span,
        });
    }
    Ok(())
}

fn normalized(row: &Record<'_>, value: &str, source: &str) -> Result<(), Issue> {
    let raw = source
        .get(row.span.start as usize..row.span.end as usize)
        .unwrap_or("");
    if raw.contains('&')
        || value.contains('&')
        || raw.bytes().any(|byte| byte.is_ascii_control())
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(Issue {
            kind: Kind::ValueNormalization,
            span: row.span,
        });
    }
    Ok(())
}

pub(super) fn empty(records: &[Record<'_>], index: usize) -> bool {
    let mut parts = children(records, index);
    parts
        .next()
        .and_then(|index| records.get(index))
        .is_some_and(|row| row.kind == Syntax::Empty)
        && parts.next().is_none()
}

fn record<'r, 'a>(records: &'r [Record<'a>], index: usize) -> Result<&'r Record<'a>, Issue> {
    records.get(index).ok_or(Issue {
        kind: Kind::InvalidRecord,
        span: vize_l0::Span::new(0, 0),
    })
}
