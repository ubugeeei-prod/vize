//! JSX/TSX render roots lowered from the oxc Program (L1) to the shared L2
//! ops (decision record `2026-11-10-6885-jsx-l2-lowering.md`, #6885).
//!
//! A dialect is resolved once per file from the L0 descriptor and is a const
//! composition of [`JsxFeatures`]. Every admitted construct produces the op
//! the equivalent template produces; a construct whose lowering has not
//! landed yet is a typed [`JsxRefusal`], never a degraded op. A module lowers
//! through [`lower_module`], one definition per outermost root.

mod attributes;
mod boolean;
mod children;
mod control_flow;
mod directives;
mod model;
mod module;
#[cfg(test)]
mod module_tests;
mod slots;
#[cfg(test)]
mod slots_tests;
mod syntax;
#[cfg(test)]
mod tests;

pub use boolean::BooleanBindings;
pub use children::clean_text;
pub use module::{
    EnteredFunction, FunctionNode, JsxModule, ModuleOptions, ModuleRoot, ModuleVisitor,
    PrologueIssue, RootContext, lower_module, walk_module,
};
pub use syntax::JsxSyntaxFacts;

use oxc_ast::ast::{JSXElement, JSXFragment};
use oxc_semantic::Scoping;
use vize_l0::dialect::JsxSemantics;
use vize_l0::id::NodeId;
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, Span};
use vize_l2::artifact::{L2TemplateParts, LoweringFeatures, OpFamily};
use vize_l2::component::{ComponentDef, Exec, RenderScope};
use vize_l2::expr::ExprRef;
use vize_l2::op::Region;
use vize_l2::scope::{ScopeAnalysis, ScopeTag};

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

/// The facts of one module that every render root lowers against.
#[derive(Debug, Clone, Copy)]
pub struct JsxL2Ir<'a, 's> {
    /// The whole module, which every span indexes.
    pub source: &'a str,
    /// The module's semantic pass: component tags resolve through it.
    pub scoping: Option<&'s Scoping>,
    /// The module's provably boolean `const`s, for structural `&&`.
    pub booleans: &'s BooleanBindings,
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
    /// A `v-*` directive whose lowering has not landed (`v-slots`, `v-if`,
    /// `_` modifiers, …) or an attribute value holding JSX.
    Attribute,
    /// A child shape whose op has not landed (expression children outside
    /// the sole-child position, `&&` over a value, a non-element branch,
    /// spreads, slot idioms).
    Child,
    /// A span outside the source.
    Span,
}

struct Cx<'a, 's, S: syntax::SyntaxSink = syntax::NoSyntax> {
    allocator: &'a Allocator,
    dialect: JsxDialect,
    file: JsxL2Ir<'a, 's>,
    op_count: u32,
    next_scope: u32,
    scopes: SideTable<ScopeAnalysis>,
    features: LoweringFeatures,
    syntax: S,
}

impl<'a, S: syntax::SyntaxSink> Cx<'a, '_, S> {
    /// Count one op and return its page-order id.
    fn mint(&mut self) -> Option<NodeId> {
        let id = NodeId::from_index(self.op_count);
        self.op_count = self.op_count.saturating_add(1);
        id
    }

    fn mint_scope(&mut self) -> ScopeTag {
        let tag = ScopeTag::from_index(self.next_scope);
        self.next_scope = self.next_scope.saturating_add(1);
        tag
    }

    fn attach_scope(&mut self, node: Option<NodeId>, facts: ScopeAnalysis) {
        if let Some(id) = node {
            self.scopes.insert(id, facts);
        }
    }

    fn observe(&mut self, family: OpFamily) {
        self.features = self.features.observing(family);
    }

    fn slice(&self, span: oxc_span::Span) -> Result<&'a str, JsxRefusal> {
        self.file
            .source
            .get(span.start as usize..span.end as usize)
            .ok_or(JsxRefusal::Span)
    }

    /// The authored expression at `at`.
    fn expr(&self, at: oxc_span::Span) -> Result<ExprRef<'a>, JsxRefusal> {
        Ok(ExprRef::parse_js_in(
            self.allocator,
            self.slice(at)?,
            span(at),
        ))
    }
}

const fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

/// Lower one render root into its component definition.
///
/// `file` holds the facts of the module the root belongs to.
pub fn lower_component<'a>(
    allocator: &'a Allocator,
    file: &JsxL2Ir<'a, '_>,
    dialect: &JsxDialect,
    component: JsxComponent<'a>,
    root: JsxRoot<'_, '_>,
) -> Result<ComponentDef<'a>, JsxRefusal> {
    lower_component_with_sink(allocator, file, dialect, component, root, syntax::NoSyntax)
        .map(|(component, _)| component)
}

/// Retain sparse original syntax observations while lowering this root.
///
/// Facts match only the exact arena objects minted by this invocation. This
/// low-level AST entry does not authenticate a caller-constructed tree against
/// source; source-facing consumers seal these facts inside their parse factory.
pub fn lower_component_with_syntax<'a>(
    allocator: &'a Allocator,
    file: &JsxL2Ir<'a, '_>,
    dialect: &JsxDialect,
    component: JsxComponent<'a>,
    root: JsxRoot<'_, '_>,
) -> Result<(ComponentDef<'a>, JsxSyntaxFacts<'a>), JsxRefusal> {
    lower_component_with_sink(
        allocator,
        file,
        dialect,
        component,
        root,
        JsxSyntaxFacts::new(file.source),
    )
}

fn lower_component_with_sink<'a, S: syntax::SyntaxSink>(
    allocator: &'a Allocator,
    file: &JsxL2Ir<'a, '_>,
    dialect: &JsxDialect,
    component: JsxComponent<'a>,
    root: JsxRoot<'_, '_>,
    syntax: S,
) -> Result<(ComponentDef<'a>, S), JsxRefusal> {
    if *dialect != VIZE || component.exec != Exec::ReRender {
        return Err(JsxRefusal::Dialect);
    }
    let mut cx = Cx {
        allocator,
        dialect: *dialect,
        file: *file,
        op_count: 0,
        next_scope: 0,
        scopes: SideTable::new(),
        features: LoweringFeatures::EMPTY,
        syntax,
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
    cx.syntax.finish();
    let component = ComponentDef {
        name: component.name,
        exec: component.exec,
        scope: RenderScope::Lexical,
        span: span(root_span),
        body: L2TemplateParts {
            scopes: cx.scopes,
            features: cx.features,
            ..L2TemplateParts::new(file.source, Region { ops })
        },
        op_count: cx.op_count,
    };
    Ok((component, cx.syntax))
}

#[cfg(feature = "module-ir")]
mod definition;
#[cfg(feature = "module-ir")]
pub use definition::{ModuleLowering, ModuleLoweringError, lower_module_def};

#[cfg(feature = "component-sites")]
pub use definition::lower_module_with_component_sites;
