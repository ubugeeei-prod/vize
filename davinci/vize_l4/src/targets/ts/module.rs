//! A whole JSX/TSX module with each outermost markup root replaced in place.
//!
//! Every byte outside the roots is authored script, written verbatim as one
//! linked run per gap. Each root becomes one expression from its L2
//! [`ComponentDef`] (decision record `2026-11-10-6849-l4-ts-projection.md`).
//! The document is a plain `.ts` module whatever the input language.

use alloc::vec::Vec;

use oxc_ast::ast::{JSXElement, JSXFragment, Program};
use oxc_ast_visit::{Visit, walk};
use vize_l0::Span;
use vize_l2::component::{ComponentDef, Exec, RenderScope};

use super::MODULE_SUFFIX;
use super::ops;
use super::output::{LinkRole, Out, TsDocument};

/// Why a module has no projection. A refusal produces no document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxModuleRefusal {
    /// The module does not fit 32-bit offsets.
    SourceTooLarge,
    /// A root's L2 body indexes another source.
    ForeignSource,
    /// The roots are not exactly the module's outermost markup roots, in order.
    Roots,
    /// A run-once root: its element or block value type is not projected yet.
    RunOnce,
    /// A root resolves names through an instance, which a module has not.
    InstanceScope,
    /// The op (by mnemonic) needs a helper that is not published.
    UnpublishedHelper(&'static str),
    /// The op (by mnemonic) or this shape of it is not projected yet.
    Unsupported(&'static str),
    /// An expression is not authored JavaScript at its own span.
    Expression,
    /// A name a string literal cannot spell verbatim.
    Name,
    /// A verbatim run would carry authored markup.
    NestedMarkup,
}

/// Project a JSX/TSX module and the L2 definitions of its outermost roots.
///
/// `roots` are in authored order and lowered from `program`'s own source.
pub fn project_jsx_module(
    program: &Program<'_>,
    roots: &[ComponentDef<'_>],
) -> Result<TsDocument, JsxModuleRefusal> {
    let source = program.source_text;
    let length = u32::try_from(source.len()).map_err(|_| JsxModuleRefusal::SourceTooLarge)?;
    let markup = Markup::of(program);
    if markup.roots.len() != roots.len()
        || markup
            .roots
            .iter()
            .zip(roots)
            .any(|(span, root)| *span != root.span)
    {
        return Err(JsxModuleRefusal::Roots);
    }
    for root in roots {
        if !core::ptr::eq(root.body.source, source) {
            return Err(JsxModuleRefusal::ForeignSource);
        }
        if root.exec != Exec::ReRender {
            return Err(JsxModuleRefusal::RunOnce);
        }
        if root.scope != RenderScope::Lexical {
            return Err(JsxModuleRefusal::InstanceScope);
        }
    }
    let mut cx = Cx {
        source,
        markup: &markup.starts,
        out: Out::with_capacity(source.len().saturating_mul(2)),
    };
    let mut cursor = 0;
    for root in roots {
        cx.script(Span::new(cursor, root.span.start))?;
        ops::root(&mut cx, root)?;
        cursor = root.span.end;
    }
    cx.script(Span::new(cursor, length))?;
    cx.out.push(MODULE_SUFFIX);
    Ok(cx.out.finish())
}

/// The projection state: the module source, its markup starts and the output.
pub(super) struct Cx<'s> {
    pub(super) source: &'s str,
    markup: &'s [u32],
    pub(super) out: Out,
}

impl Cx<'_> {
    /// Write the authored bytes at `span` as one verbatim link. They never
    /// hold a markup node: markup is projected, not copied.
    pub(super) fn verbatim(&mut self, span: Span, role: LinkRole) -> Result<(), JsxModuleRefusal> {
        let text = self
            .source
            .get(span.start as usize..span.end as usize)
            .ok_or(JsxModuleRefusal::Expression)?;
        let first = self.markup.partition_point(|start| *start < span.start);
        if self
            .markup
            .get(first)
            .is_some_and(|start| *start < span.end)
        {
            return Err(JsxModuleRefusal::NestedMarkup);
        }
        self.out.verbatim(text, span, role);
        Ok(())
    }

    fn script(&mut self, span: Span) -> Result<(), JsxModuleRefusal> {
        if span.start >= span.end {
            return Ok(());
        }
        self.verbatim(span, LinkRole::Script)
    }

    /// Write one anchor unit owned by `authored` through `write`.
    pub(super) fn anchor(
        &mut self,
        authored: Span,
        role: LinkRole,
        write: impl FnOnce(&mut Self) -> Result<(), JsxModuleRefusal>,
    ) -> Result<(), JsxModuleRefusal> {
        let capacity = (authored.end.saturating_sub(authored.start) as usize).saturating_mul(2);
        let outer = core::mem::replace(&mut self.out, Out::with_capacity(capacity));
        let written = write(self);
        let fragment = core::mem::replace(&mut self.out, outer);
        written?;
        self.out.append_anchor(fragment, authored, role);
        Ok(())
    }
}

/// The module's outermost markup roots and the start of every markup node.
struct Markup {
    roots: Vec<Span>,
    starts: Vec<u32>,
    depth: u32,
}

impl Markup {
    fn of(program: &Program<'_>) -> Self {
        let mut markup = Self {
            roots: Vec::new(),
            starts: Vec::new(),
            depth: 0,
        };
        markup.visit_program(program);
        markup.starts.sort_unstable();
        markup
    }

    fn enter(&mut self, span: oxc_span::Span) {
        if self.depth == 0 {
            self.roots.push(Span::new(span.start, span.end));
        }
        self.starts.push(span.start);
        self.depth = self.depth.saturating_add(1);
    }
}

impl<'a> Visit<'a> for Markup {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        self.enter(element.span);
        walk::walk_jsx_element(self, element);
        self.depth = self.depth.saturating_sub(1);
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        self.enter(fragment.span);
        walk::walk_jsx_fragment(self, fragment);
        self.depth = self.depth.saturating_sub(1);
    }
}
