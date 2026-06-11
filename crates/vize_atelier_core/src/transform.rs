//! Transform infrastructure for Vue template AST.
//!
//! This module provides the transform context, traversal, and base transform traits.

mod context;
pub mod element;
pub mod structural;
pub mod traverse;

use vize_carton::{Box, Bump, SmallVec, String, Vec, profile};
use vize_croquis::{Croquis, ScopeChain};

use crate::ast::*;
use crate::errors::CompilerError;
use crate::options::TransformOptions;
use crate::runtime_helpers::RuntimeHelpers;

use traverse::traverse_children;

/// Transform function for nodes - returns optional exit function(s)
pub type NodeTransform<'a> =
    fn(&mut TransformContext<'a>, &mut TemplateChildNode<'a>) -> Option<ExitFns<'a>>;

/// Exit function called after children are processed
pub type ExitFn<'a> = std::boxed::Box<dyn FnOnce(&mut TransformContext<'a>) + 'a>;
pub type ExitFns<'a> = SmallVec<[ExitFn<'a>; 2]>;

/// Transform function for directives
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

/// Structural directive transform (v-if, v-for)
pub type StructuralDirectiveTransform<'a> =
    fn(&mut TransformContext<'a>, &mut ElementNode<'a>, &DirectiveNode<'a>) -> Option<ExitFn<'a>>;

/// Transform context for AST traversal
pub struct TransformContext<'a> {
    /// Arena allocator
    pub allocator: &'a Bump,
    /// Transform options
    pub options: TransformOptions,
    /// Source code
    pub source: String,
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
    /// Flushed to [`RootNode::filters`](crate::ast::RootNode) and emitted as
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
    /// Node was removed flag
    pub(crate) node_removed: bool,
    /// Semantic analysis summary (optional, for enhanced transforms)
    pub(crate) analysis: Option<&'a Croquis>,
    /// Scope ID to bake into static VNodes hoisted outside render scope.
    pub(crate) hoisted_scope_id: Option<String>,
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
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(allocator, root, options, analysis, false, None)
}

/// Transform the root AST node with template syntax quirk compatibility enabled.
pub fn transform_with_template_syntax_quirks<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(allocator, root, options, analysis, true, None)
}

/// Transform the root AST node with Vue parser quirk compatibility enabled.
#[deprecated(note = "use transform_with_template_syntax_quirks instead")]
pub fn transform_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_with_template_syntax_quirks(allocator, root, options, analysis)
}

/// Transform the root AST node with an explicit scope ID for hoisted VNodes.
#[doc(hidden)]
pub fn transform_with_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(allocator, root, options, analysis, false, hoisted_scope_id)
}

/// Transform the root AST node with template syntax quirks and an explicit hoisted scope ID.
#[doc(hidden)]
pub fn transform_with_template_syntax_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(allocator, root, options, analysis, true, hoisted_scope_id)
}

/// Transform the root AST node with Vue parser quirks and an explicit hoisted scope ID.
#[doc(hidden)]
#[deprecated(note = "use transform_with_template_syntax_quirks_and_hoisted_scope_id instead")]
pub fn transform_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_with_template_syntax_quirks_and_hoisted_scope_id(
        allocator,
        root,
        options,
        analysis,
        hoisted_scope_id,
    )
}

fn transform_inner<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    template_syntax_quirks: bool,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    let source = root.source.clone();
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
    ctx.hoisted_scope_id = hoisted_scope_id;
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
        crate::transforms::legacy::desugar_legacy_template(allocator, root, caps);
    }

    // Transform the root children
    profile!(
        "atelier.transform.traverse_children",
        traverse_children(&mut ctx, ParentNode::Root(root as *mut _))
    );

    // Apply static hoisting after traversal (before codegen)
    use crate::transforms::hoist_static::hoist_static;
    profile!(
        "atelier.transform.hoist_static",
        hoist_static(&mut ctx, &mut root.children)
    );

    // Create root codegen node
    profile!(
        "atelier.transform.create_root_codegen",
        create_root_codegen(&mut ctx, root)
    );

    // Update root with context results
    for helper in ctx.helpers.iter() {
        root.helpers.push(helper);
    }
    for component in ctx.components.into_iter() {
        root.components.push(component);
    }
    for directive in ctx.directives.into_iter() {
        root.directives.push(directive);
    }
    // Transfer Vue 2 pipe filters (legacy-only; empty for Vue 3).
    #[cfg(feature = "legacy")]
    for filter in ctx.filters.into_iter() {
        root.filters.push(filter);
    }
    // Transfer hoisted nodes to root
    for hoist in ctx.hoists.into_iter() {
        root.hoists.push(hoist);
    }
    root.temps = ctx.temps;
    root.transformed = true;

    ctx.errors
}

/// Create codegen node for root
fn create_root_codegen<'a>(ctx: &mut TransformContext<'a>, root: &mut RootNode<'a>) {
    if root.children.is_empty() {
        return;
    }

    if root.children.len() > 1 {
        // Multiple root children need to be wrapped in a fragment
        ctx.helper(RuntimeHelper::OpenBlock);
        ctx.helper(RuntimeHelper::CreateElementBlock);
        ctx.helper(RuntimeHelper::Fragment);
    }

    // Root codegen node is handled in codegen directly for now
    root.codegen_node = None;
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{transform, transform_with_template_syntax_quirks};
    use crate::codegen::generate;
    use crate::options::{CodegenOptions, TransformOptions};
    use crate::parser::parse;
    use bumpalo::Bump;

    #[test]
    fn test_transform_simple_element() {
        assert_transform!("<div>hello</div>" => helpers: [CreateElementVNode]);
    }

    #[test]
    fn test_transform_interpolation() {
        assert_transform!("{{ msg }}" => helpers: [ToDisplayString]);
    }

    #[test]
    fn test_transform_component() {
        assert_transform!("<MyComponent></MyComponent>" => components: ["MyComponent"]);
        assert_transform!("<MyComponent></MyComponent>" => helpers: [ResolveComponent]);
    }

    #[test]
    fn test_transform_pascal_case_dynamic_component() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<Component :is="current" />"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform(&allocator, &mut root, TransformOptions::default(), None);

        assert!(
            !root
                .components
                .iter()
                .any(|component| component.as_str() == "Component"),
            "Dynamic component special tag should not be tracked as a resolved component"
        );
        assert!(
            !root
                .helpers
                .iter()
                .any(|helper| matches!(helper, crate::ast::RuntimeHelper::ResolveComponent)),
            "Dynamic component special tag should not request resolveComponent"
        );
    }

    #[test]
    fn test_transform_v_if() {
        assert_transform!("<div v-if=\"show\">hello</div>" => helpers: [OpenBlock, CreateBlock, Fragment, CreateComment]);
    }

    #[test]
    fn test_transform_v_for() {
        assert_transform!("<div v-for=\"item in items\">{{ item }}</div>" => helpers: [RenderList, OpenBlock, CreateBlock, Fragment]);
    }

    #[test]
    fn test_transform_v_for_rejects_unmatched_edge_parens_by_default() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<div v-for="item) in items"></div>"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform(&allocator, &mut root, TransformOptions::default(), None);

        assert!(
            !matches!(&root.children[0], crate::ast::TemplateChildNode::For(_)),
            "strict parser mode should not accept unmatched v-for alias parens"
        );
    }

    #[test]
    fn test_transform_v_for_template_syntax_quirks_accepts_unmatched_edge_parens() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<div v-for="item) in items"></div>"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform_with_template_syntax_quirks(
            &allocator,
            &mut root,
            TransformOptions::default(),
            None,
        );

        match &root.children[0] {
            crate::ast::TemplateChildNode::For(for_node) => match &for_node.value_alias {
                Some(crate::ast::ExpressionNode::Simple(value)) => {
                    assert_eq!(value.content.as_str(), "item");
                }
                _ => panic!("expected value alias"),
            },
            other => panic!("expected ForNode, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_v_if_creates_if_node() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<div v-if="show">visible</div>"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform(&allocator, &mut root, TransformOptions::default(), None);

        // After transform, root should have 1 child: an IfNode
        assert_eq!(
            root.children.len(),
            1,
            "Should have 1 child after transform"
        );

        match &root.children[0] {
            crate::ast::TemplateChildNode::If(if_node) => {
                assert_eq!(if_node.branches.len(), 1, "Should have 1 branch");
                // First branch should have condition "show"
                let branch = &if_node.branches[0];
                assert!(branch.condition.is_some(), "Branch should have condition");
            }
            other => panic!("Expected IfNode, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_v_if_else_creates_branches() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(
            &allocator,
            r#"<div v-if="show">yes</div><div v-else>no</div>"#,
        );
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform(&allocator, &mut root, TransformOptions::default(), None);

        // After transform, should have 1 IfNode with 2 branches
        assert_eq!(
            root.children.len(),
            1,
            "Should have 1 child (IfNode) after transform, got {}",
            root.children.len()
        );

        match &root.children[0] {
            crate::ast::TemplateChildNode::If(if_node) => {
                assert_eq!(
                    if_node.branches.len(),
                    2,
                    "Should have 2 branches (if + else)"
                );
                // First branch has condition, second doesn't (v-else)
                assert!(
                    if_node.branches[0].condition.is_some(),
                    "First branch should have condition"
                );
                assert!(
                    if_node.branches[1].condition.is_none(),
                    "Second branch (else) should not have condition"
                );
            }
            other => panic!("Expected IfNode, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_v_for_creates_for_node() {
        let allocator = Bump::new();
        let (mut root, errors) =
            parse(&allocator, r#"<div v-for="item in items">{{ item }}</div>"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        transform(&allocator, &mut root, TransformOptions::default(), None);

        // After transform, root should have 1 child: a ForNode
        assert_eq!(
            root.children.len(),
            1,
            "Should have 1 child after transform"
        );

        match &root.children[0] {
            crate::ast::TemplateChildNode::For(for_node) => {
                // Check source is "items"
                match &for_node.source {
                    crate::ast::ExpressionNode::Simple(exp) => {
                        assert_eq!(exp.content.as_str(), "items", "Source should be 'items'");
                    }
                    _ => panic!("Expected Simple expression for source"),
                }
                // Check value alias is "item"
                assert!(for_node.value_alias.is_some(), "Should have value alias");
                match for_node.value_alias.as_ref().unwrap() {
                    crate::ast::ExpressionNode::Simple(exp) => {
                        assert_eq!(exp.content.as_str(), "item", "Value alias should be 'item'");
                    }
                    _ => panic!("Expected Simple expression for value alias"),
                }
            }
            other => panic!("Expected ForNode, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_codegen_v_if() {
        let allocator = Bump::new();
        let (mut root, _) = parse(&allocator, r#"<div v-if="show">visible</div>"#);
        transform(&allocator, &mut root, TransformOptions::default(), None);

        let result = generate(&root, CodegenOptions::default());
        insta::assert_snapshot!(result.code.as_str());
    }
}
