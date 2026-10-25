//! Hygienic lexical declarations from actual admitted retained resolutions.
//! Property names and rewritten instance accesses are not lexical occupants.

use super::{Vue2Access, Vue2Error, Vue2ErrorKind};
use crate::expr::{EmitError, ResolvedExpressions};
use vize_l0::{FxHashSet, Span, String, cstr};
use vize_l2::{artifact::Artifact, expr::ExprRef, op::Op, walk::NodeRef};

pub(super) struct Names {
    pub(super) receiver: String,
    pub(super) render: String,
    pub(super) statics: String,
}

impl Names {
    pub(super) fn checked<'a>(
        artifact: &Artifact<'a>,
        expressions: ResolvedExpressions<'_, 'a>,
        access: &Vue2Access<'_>,
    ) -> Result<Self, Vue2Error> {
        let mut collision = false;
        visit_verbatim(artifact, expressions, access, &mut |name| {
            collision |= matches!(name, "_vm" | "render" | "staticRenderFns");
        })?;
        if !collision {
            // No name set is allocated on the common path; defaults fit inline.
            return Ok(Self {
                receiver: "_vm".into(),
                render: "render".into(),
                statics: "staticRenderFns".into(),
            });
        }
        let mut occupied = FxHashSet::default();
        visit_verbatim(artifact, expressions, access, &mut |name| {
            occupied.insert(name);
        })?;
        let fail = || Vue2Error {
            node: None,
            span: Span::new(0, artifact.source().len() as u32),
            kind: Vue2ErrorKind::OutputTooLarge,
        };
        // These three bases have disjoint numeric-suffix families. Public
        // export aliases introduce no additional lexical declarations.
        Ok(Self {
            receiver: fresh("_vm", &occupied).ok_or_else(fail)?,
            render: fresh("render", &occupied).ok_or_else(fail)?,
            statics: fresh("staticRenderFns", &occupied).ok_or_else(fail)?,
        })
    }
}

fn fresh(base: &str, occupied: &FxHashSet<&str>) -> Option<String> {
    if !occupied.contains(base) {
        return Some(base.into());
    }
    // A set of N occupants containing the base can occupy at most N-1 of
    // these N suffixes. Selection depends only on membership, never hash order.
    (1..=occupied.len()).find_map(|suffix| {
        let candidate = cstr!("{base}{suffix}");
        (!occupied.contains(candidate.as_str())).then_some(candidate)
    })
}

fn visit_verbatim<'a>(
    artifact: &Artifact<'a>,
    expressions: ResolvedExpressions<'_, 'a>,
    access: &Vue2Access<'_>,
    visit: &mut impl FnMut(&'a str),
) -> Result<(), Vue2Error> {
    let mut failure = None;
    artifact
        .visit_nodes(&mut |id, node| {
            if failure.is_some() {
                return;
            }
            let error = |kind| Vue2Error {
                node: Some(id),
                span: node.span(),
                kind,
            };
            let NodeRef::Op(op) = node else {
                return;
            };
            if let Op::Element(element) = op {
                if !element.bindings.is_empty() {
                    failure = Some(error(Vue2ErrorKind::UnsupportedBindings));
                }
                return;
            }
            let Op::Interpolation(interpolation) = op else {
                return;
            };
            let ExprRef::Js(expression) = interpolation.expression else {
                failure = Some(error(Vue2ErrorKind::UnsupportedExpression));
                return;
            };
            let table = match expressions.table_for_retained(expression) {
                Ok(table) => table,
                Err(problem) => {
                    failure = Some(error(Vue2ErrorKind::Expression(problem)));
                    return;
                }
            };
            if !table
                .expression()
                .matches_authored_source(artifact.source())
            {
                failure = Some(error(Vue2ErrorKind::Expression(EmitError {
                    span: table.expression().span,
                    kind: crate::expr::EmitErrorKind::SourceMismatch,
                })));
                return;
            }
            for occurrence in table.occurrences() {
                if access.is_verbatim(occurrence.binding) {
                    visit(occurrence.name);
                }
            }
        })
        .map_err(|_| Vue2Error {
            node: None,
            span: Span::new(0, artifact.source().len() as u32),
            kind: Vue2ErrorKind::MissingNode,
        })?;
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
