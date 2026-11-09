//! JSX/TSX render roots lowered from the oxc Program (L1) to the shared L2
//! ops (decision record `2026-11-10-6885-jsx-l2-lowering.md`, #6885).
//!
//! A dialect is resolved once per file from the L0 descriptor and is a const
//! composition of [`JsxFeatures`]. Every admitted construct produces the op
//! the equivalent template produces; a construct whose lowering has not
//! landed yet is a typed [`JsxRefusal`], never a degraded op.

mod attributes;
mod children;
#[cfg(test)]
mod tests;

pub use children::clean_text;

use oxc_ast::ast::{JSXElement, JSXFragment};
use oxc_semantic::Scoping;
use vize_l0::dialect::JsxSemantics;
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, Span};
use vize_l2::artifact::{ArtifactParts, LoweringFeatures, OpFamily};
use vize_l2::component::{ComponentDef, Exec, RenderScope};
use vize_l2::op::Region;

/// How deeply a dialect interprets JavaScript around JSX.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// JS control flow stays plain expressions (`@vue/babel-plugin-jsx`).
    Opaque,
    /// Expressions are tracked as effects (`vue-jsx-vapor`).
    ReactiveGetter,
    /// `&&`, `?:` and `.map()` lower to `ui.if` / `ui.for` (Vize).
    Structural,
}

/// The attribute and child spellings a dialect accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxFeatures {
    /// `onClickCapture` / `Once` / `Passive` name option modifiers.
    pub on_option_suffixes: bool,
    /// `onClick_stop`: `_` modifiers on event props.
    pub event_underscore_modifiers: bool,
}

/// One JSX dialect: an interpretation depth and its accepted spellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxDialect {
    pub depth: Depth,
    pub features: JsxFeatures,
}

/// Vize's own JSX semantics, for either execution model.
pub const VIZE: JsxDialect = JsxDialect {
    depth: Depth::Structural,
    features: JsxFeatures {
        on_option_suffixes: true,
        // #12313 turns this on.
        event_underscore_modifiers: false,
    },
};

/// `@vue/babel-plugin-jsx` semantics (re-render only).
pub const BABEL: JsxDialect = JsxDialect {
    depth: Depth::Opaque,
    features: JsxFeatures {
        on_option_suffixes: true,
        event_underscore_modifiers: false,
    },
};

/// `vue-jsx-vapor` semantics (run-once only).
pub const VUE_JSX_VAPOR: JsxDialect = JsxDialect {
    depth: Depth::ReactiveGetter,
    features: JsxFeatures {
        on_option_suffixes: false,
        event_underscore_modifiers: true,
    },
};

impl JsxDialect {
    /// The composition selected by a file's resolved JSX semantics.
    #[must_use]
    pub const fn for_semantics(semantics: JsxSemantics) -> &'static Self {
        match semantics {
            JsxSemantics::Vize => &VIZE,
            JsxSemantics::Babel => &BABEL,
            JsxSemantics::VueJsxVapor => &VUE_JSX_VAPOR,
        }
    }
}

/// An outermost JSX render root.
#[derive(Debug, Clone, Copy)]
pub enum JsxRoot<'r, 'x> {
    Element(&'r JSXElement<'x>),
    Fragment(&'r JSXFragment<'x>),
}

/// The component a render root belongs to.
#[derive(Debug, Clone, Copy)]
pub struct JsxComponent<'a> {
    pub name: Option<&'a str>,
    pub exec: Exec,
}

/// A construct this lowering does not admit yet. The caller keeps its
/// existing path for the whole root; nothing is lowered partially.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxRefusal {
    /// The dialect or execution model has no lowering yet.
    Dialect,
    /// A namespaced, `this` or `<style>` tag.
    Tag,
    /// A `v-*` directive, `v-slots` or an attribute value holding JSX.
    Attribute,
    /// A child shape whose op has not landed (expression children outside
    /// the sole-child position, control flow, spreads, slot idioms).
    Child,
    /// A span outside the source.
    Span,
}

struct Cx<'a, 's> {
    allocator: &'a Allocator,
    dialect: JsxDialect,
    source: &'a str,
    scoping: Option<&'s Scoping>,
    op_count: u32,
    features: LoweringFeatures,
}

impl<'a> Cx<'a, '_> {
    fn mint(&mut self) {
        self.op_count = self.op_count.saturating_add(1);
    }

    fn observe(&mut self, family: OpFamily) {
        self.features = self.features.observing(family);
    }

    fn slice(&self, span: oxc_span::Span) -> Result<&'a str, JsxRefusal> {
        self.source
            .get(span.start as usize..span.end as usize)
            .ok_or(JsxRefusal::Span)
    }
}

const fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

/// Lower one render root into its component definition.
///
/// `source` is the whole module the root's spans index; `scoping` resolves
/// component tags to lexical bindings, as the Program's semantic pass built
/// it.
pub fn lower_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    scoping: Option<&Scoping>,
    dialect: &JsxDialect,
    component: JsxComponent<'a>,
    root: JsxRoot<'_, '_>,
) -> Result<ComponentDef<'a>, JsxRefusal> {
    if *dialect != VIZE || component.exec != Exec::ReRender {
        return Err(JsxRefusal::Dialect);
    }
    let mut cx = Cx {
        allocator,
        dialect: *dialect,
        source,
        scoping,
        op_count: 0,
        features: LoweringFeatures::EMPTY,
    };
    let (ops, root_span) = match root {
        JsxRoot::Element(element) => {
            let mut ops = vize_l0::Vec::new_in(&allocator);
            ops.push(children::element(&mut cx, element)?);
            (ops, element.span)
        }
        JsxRoot::Fragment(fragment) => {
            let mut ops = vize_l0::Vec::new_in(&allocator);
            children::push_children(&mut cx, &fragment.children, &mut ops)?;
            (ops, fragment.span)
        }
    };
    Ok(ComponentDef {
        name: component.name,
        exec: component.exec,
        scope: RenderScope::Lexical,
        span: span(root_span),
        body: ArtifactParts {
            scopes: SideTable::new(),
            features: cx.features,
            ..ArtifactParts::new(source, Region { ops })
        },
        op_count: cx.op_count,
    })
}
