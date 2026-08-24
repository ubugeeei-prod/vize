//! Template transform lane for Vue template AST.
//!
//! This module provides the transform context, traversal, and base transform traits.

mod context;
pub mod element;
mod extensions;
mod options;
pub mod patterned_template;
pub mod structural;
mod structural_keys;
pub mod traverse;

use vize_carton::{Allocator, Box, SmallVec, String, Vec, interner::Interner, profile};
use vize_croquis::{Croquis, ScopeChain};

use crate::errors::CompilerError;
use crate::options::TransformOptions;
use crate::runtime_helpers::RuntimeHelpers;
use crate::{
    CacheExpression, DirectiveNode, ElementNode, ForNode, IfBranchNode, IfNode, JsChildNode,
    PropNode, RootNode, RuntimeHelper, TemplateChildNode,
};

pub(crate) use options::{JsxTransformCompat, TransformLaneOptions};
use traverse::traverse_children;

#[allow(deprecated)]
pub use extensions::{
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    transform_with_hoisted_scope_id, transform_with_jsx_compatibility,
    transform_with_plain_element_model_argument, transform_with_source_text,
    transform_with_template_syntax_quirks_and_hoisted_scope_id,
    transform_with_vue_parser_quirks_and_hoisted_scope_id,
};

/// Transform function for nodes - returns optional exit function(s)
pub type NodeTransform<'a> =
    fn(&mut TransformContext<'a>, &mut TemplateChildNode<'a>) -> Option<ExitFns<'a>>;

/// Exit function called after children are processed
pub type ExitFn<'a> = std::boxed::Box<dyn FnOnce(&mut TransformContext<'a>) + 'a>;
pub type ExitFns<'a> = SmallVec<[ExitFn<'a>; 2]>;

pub type DirectiveTransform<'a> = fn(
    &mut TransformContext<'a>,
    &mut ElementNode<'a>,
    &DirectiveNode<'a>,
) -> Option<DirectiveTransformResult<'a>>;

/// Result of a directive transform
pub struct DirectiveTransformResult<'a> {
    /// Props to add to the element
    pub props: Vec<'a, PropNode<'a>>,
    /// Whether to remove the directive
    pub remove_directive: bool,
    /// SSR tag type hint
    pub ssr_tag_type: Option<u8>,
}

pub type StructuralDirectiveTransform<'a> =
    fn(&mut TransformContext<'a>, &mut ElementNode<'a>, &DirectiveNode<'a>) -> Option<ExitFn<'a>>;

/// Transform context for AST traversal
pub struct TransformContext<'a> {
    /// Arena allocator
    pub allocator: &'a Allocator,
    /// Transform options
    pub options: TransformOptions,
    pub(crate) custom_elements: crate::options::CustomElementMatcher,
    /// Template source the spans index into, at the arena lifetime (P1-10).
    pub source: &'a str,
    /// Atoms for names this pass synthesizes rather than slices (P1-10).
    pub interner: Interner<'a>,
    /// Root node reference
    pub root: Option<*mut RootNode<'a>>,
    /// Parent node stack
    pub parent: Option<ParentNode<'a>>,
    /// Grandparent node
    pub grandparent: Option<ParentNode<'a>>,
    /// Current node being transformed
    pub current_node: Option<*mut TemplateChildNode<'a>>,
    /// Child index in parent
    pub child_index: usize,
    /// Helpers used
    pub helpers: RuntimeHelpers,
    /// Components used (Vec to maintain template order for code generation)
    pub components: std::vec::Vec<String>,
    /// Directives used (Vec to maintain template order for code generation)
    pub directives: std::vec::Vec<String>,
    /// Vue 2 pipe filters referenced by the template, first-seen order.
    /// Flushed to [`RootNode::filters`](crate::RootNode) and emitted as
    /// `_resolveFilter` asset declarations. Legacy-only and dialect-gated.
    #[cfg(feature = "legacy")]
    pub(crate) filters: std::vec::Vec<String>,
    /// Hoisted expressions
    pub hoists: Vec<'a, Option<JsChildNode<'a>>>,
    /// Cached expressions
    pub cached: Vec<'a, Option<Box<'a, CacheExpression<'a>>>>,
    /// Temp variable count
    pub temps: u32,
    /// Scope chain for tracking variable visibility
    pub scope_chain: ScopeChain,
    /// Scoped slots
    pub scoped_slots: u32,
    /// Whether in v-once
    pub in_v_once: bool,
    /// Whether in SSR
    pub in_ssr: bool,
    /// Errors collected
    pub errors: std::vec::Vec<CompilerError>,
    /// Enables compatibility for template syntax edge-case behavior.
    pub(crate) template_syntax_quirks: bool,
    pub(crate) jsx_compat: JsxTransformCompat,
    /// Node was removed flag
    pub(crate) node_removed: bool,
    /// Semantic analysis summary (optional, for enhanced transforms)
    pub(crate) analysis: Option<&'a Croquis>,
    /// Scope ID to bake into static VNodes hoisted outside render scope.
    pub(crate) hoisted_scope_id: Option<String>,
}

impl TransformContext<'_> {
    pub(crate) fn is_jsx_custom_element(&self, el: &ElementNode<'_>) -> bool {
        self.jsx_compat
            .custom_element_spans
            .contains(&(el.loc.span.start, el.loc.span.end))
    }
}

/// Enum for parent node types
#[derive(Clone, Copy)]
pub enum ParentNode<'a> {
    Root(*mut RootNode<'a>),
    Element(*mut ElementNode<'a>),
    If(*mut IfNode<'a>),
    IfBranch(*mut IfBranchNode<'a>),
    For(*mut ForNode<'a>),
}

impl<'a> ParentNode<'a> {
    /// Get mutable access to children through raw pointer.
    ///
    /// # Safety
    /// `ParentNode` stores pointers into the bump-allocated template tree so the
    /// traversal can mutate parents without cloning children on every visit. The
    /// active traversal loop owns the only mutable child slice for the current
    /// parent, and nested calls only use descendants created from that slice.
    /// Keeping the invariant here avoids an allocation-heavy zipper structure in
    /// the hottest template transform path.
    #[allow(clippy::mut_from_ref)]
    pub fn children_mut(&self) -> &mut Vec<'a, TemplateChildNode<'a>> {
        // SAFETY: every pointer is produced from a live `RootNode`, `ElementNode`,
        // `IfBranchNode`, or `ForNode` borrowed by the transform driver. The
        // transform is single-threaded and never keeps two active mutable child
        // slices for the same parent; sibling traversal advances only after the
        // previous borrow has been consumed. `ParentNode::If` intentionally has no
        // children slice, so that variant is rejected before any pointer deref.
        unsafe {
            match self {
                ParentNode::Root(r) => &mut (*(*r)).children,
                ParentNode::Element(e) => &mut (*(*e)).children,
                ParentNode::If(_) => {
                    // Panic path by design: callers must traverse concrete
                    // branches (`ParentNode::IfBranch`) because an `IfNode`
                    // itself is only a branch container and has no direct child
                    // vector to return.
                    panic!("IfNode doesn't have direct children")
                }
                ParentNode::IfBranch(b) => &mut (*(*b)).children,
                ParentNode::For(f) => &mut (*(*f)).children,
            }
        }
    }
}

/// Transform the root AST node.
///
/// Returns the diagnostics collected during the transform (e.g. invalid
/// directive usage or unparseable expressions) so callers can surface them
/// alongside parse errors instead of silently dropping them.
pub fn transform<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions::default(),
        None,
    )
}

/// Transform the root AST node with template syntax quirk compatibility enabled.
pub fn transform_with_template_syntax_quirks<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions {
            template_syntax_quirks: true,
            ..Default::default()
        },
        None,
    )
}

/// Transform the root AST node with Vue parser quirk compatibility enabled.
#[deprecated(note = "use transform_with_template_syntax_quirks instead")]
pub fn transform_with_vue_parser_quirks<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_with_template_syntax_quirks(allocator, root, options, analysis)
}

pub(crate) fn transform_inner<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    lane_options: TransformLaneOptions,
    source_text: Option<&'a str>,
) -> std::vec::Vec<CompilerError> {
    let TransformLaneOptions {
        template_syntax_quirks,
        hoisted_scope_id,
        jsx_compat,
        custom_elements,
    } = lane_options;
    let source = source_text.unwrap_or(root.source);
    let mut ctx = if let Some(analysis) = analysis {
        TransformContext::with_analysis_and_template_syntax_quirks(
            allocator,
            source,
            options,
            analysis,
            template_syntax_quirks,
        )
    } else {
        TransformContext::new_with_template_syntax_quirks(
            allocator,
            source,
            options,
            template_syntax_quirks,
        )
    };
    ctx.jsx_compat = jsx_compat;
    ctx.hoisted_scope_id = hoisted_scope_id;
    ctx.custom_elements = custom_elements;
    ctx.root = Some(root as *mut _);

    // Legacy (Vue 2 / 2.7) template-sugar pre-transform. Resolved once per file
    // from the dialect; a no-op for the default Vue 3 dialect (the resolved
    // capability set is the all-off `VUE3` set, so this returns before touching
    // the tree). Compiled only under the `legacy` cargo feature — the default
    // Vue 3 build never sees it, keeping the hot path byte-identical.
    #[cfg(feature = "legacy")]
    {
        use vize_armature::legacy::LegacyDialectCapabilities;
        let caps = LegacyDialectCapabilities::for_dialect(ctx.options.dialect);
        crate::steps::legacy::desugar_legacy_template(allocator, root, caps);
    }

    if ctx.options.experimental_patterned_template {
        patterned_template::desugar_patterned_templates(allocator, root);
    }

    // Transform the root children
    crate::walk_probe::record_walk(crate::walk_probe::WalkStage::Transform);
    profile!(
        "atelier.transform.traverse_children",
        traverse_children(&mut ctx, ParentNode::Root(root as *mut _))
    );

    // Apply static hoisting after traversal (before codegen)
    use crate::steps::hoist_static::hoist_static;
    profile!(
        "atelier.transform.hoist_static",
        hoist_static(&mut ctx, &mut root.children)
    );

    // Register helpers implied by the root shape.
    profile!(
        "atelier.transform.register_root_helpers",
        register_root_helpers(&mut ctx, root)
    );

    // Update root with context results
    for helper in ctx.helpers.iter() {
        root.helpers.push(helper);
    }
    // Asset names are computed and recur, so they land in the arena as atoms.
    for component in ctx.components.iter() {
        let atom = ctx.interner.intern(component);
        root.components.push(atom);
    }
    for directive in ctx.directives.iter() {
        let atom = ctx.interner.intern(directive);
        root.directives.push(atom);
    }
    // Transfer Vue 2 pipe filters (legacy-only; empty for Vue 3).
    #[cfg(feature = "legacy")]
    for filter in ctx.filters.iter() {
        let atom = ctx.interner.intern(filter);
        root.filters.push(atom);
    }
    // Transfer hoisted nodes to root
    for hoist in ctx.hoists.into_iter() {
        root.hoists.push(hoist);
    }
    root.temps = ctx.temps;
    root.transformed = true;

    ctx.errors
}

fn register_root_helpers<'a>(ctx: &mut TransformContext<'a>, root: &mut RootNode<'a>) {
    if root.children.is_empty() {
        return;
    }

    if root.children.len() > 1 {
        // Multiple root children need to be wrapped in a fragment
        ctx.helper(RuntimeHelper::OpenBlock);
        ctx.helper(RuntimeHelper::CreateElementBlock);
        ctx.helper(RuntimeHelper::Fragment);
    }
}

#[cfg(test)]
mod tests;
