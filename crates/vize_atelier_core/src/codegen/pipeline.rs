//! Code generation entry point and runtime-helper ordering.
//!
//! Drives the top-level `generate()` pipeline and computes the deduplicated,
//! import-ranked helper list used to build the module preamble.

use crate::{RootNode, RuntimeHelper, TemplateChildNode, options::CodegenOptions};
use vize_carton::profile;

use super::children::is_directive_comment;
use super::context::{CodegenContext, CodegenResult, CodegenResultWithSections, CodegenSections};
use super::element::generate_root_node;
use super::generate::{collect_hoist_helpers, generate_hoists};
use super::node::generate_node;
use super::root::{
    generate_assets, generate_function_signature, generate_preamble_from_helpers,
    is_ignorable_root_text,
};

/// Generate code from root AST.
pub fn generate(root: &RootNode<'_>, options: CodegenOptions) -> CodegenResult {
    generate_with_sections(root, options).into_result()
}

/// Generate code from root AST and return emission-recorded section boundaries.
pub fn generate_with_sections(
    root: &RootNode<'_>,
    options: CodegenOptions,
) -> CodegenResultWithSections {
    let mut ctx = CodegenContext::new(options);
    ctx.static_cache = ctx.options.inline || !root.hoists.is_empty();
    let root_children: std::vec::Vec<&TemplateChildNode<'_>> = root
        .children
        .iter()
        .filter(|child| !is_ignorable_root_text(child) && !is_directive_comment(child))
        .collect();

    // Generate function signature
    profile!(
        "atelier.codegen.function_signature",
        generate_function_signature(&mut ctx)
    );

    // Generate body
    ctx.indent();
    ctx.newline();

    // Generate component/directive resolution
    let assets_start = ctx.code.len();
    profile!("atelier.codegen.assets", generate_assets(&mut ctx, root));
    let assets_end = ctx.code.len();

    // Generate return statement
    ctx.push("return ");
    let return_expr_start = ctx.code.len();

    // Generate root node
    if root_children.is_empty() {
        ctx.push("null");
    } else if root_children.len() == 1 {
        // Single root child - wrap in block
        profile!(
            "atelier.codegen.root_node",
            generate_root_node(&mut ctx, root_children[0])
        );
    } else {
        // Multiple root children - wrap in fragment block
        ctx.use_helper(RuntimeHelper::OpenBlock);
        ctx.use_helper(RuntimeHelper::CreateElementBlock);
        ctx.use_helper(RuntimeHelper::Fragment);
        ctx.push("(");
        ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
        ctx.push("(), ");
        ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
        ctx.push("(");
        ctx.push(ctx.helper(RuntimeHelper::Fragment));
        ctx.push(", null, [");
        ctx.indent();
        for (i, child) in root_children.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.newline();
            profile!(
                "atelier.codegen.fragment_child",
                generate_node(&mut ctx, child)
            );
        }
        ctx.deindent();
        ctx.newline();
        // Vue tags a root fragment as DEV_ROOT_FRAGMENT when it wraps a single
        // real node plus comment siblings, so dev tooling treats it as a root.
        let non_comment_children = root_children
            .iter()
            .filter(|child| !matches!(child, TemplateChildNode::Comment(_)))
            .count();
        if non_comment_children == 1 {
            ctx.push("], 2112 /* STABLE_FRAGMENT, DEV_ROOT_FRAGMENT */))");
        } else {
            ctx.push("], 64 /* STABLE_FRAGMENT */))");
        }
    }
    let return_expr_end = ctx.code.len();

    ctx.deindent();
    ctx.newline();
    ctx.push("}");

    // Now generate preamble after we know all used helpers
    // Only include specific helpers from root.helpers that are known to be
    // added during transform but not tracked during codegen (like Unref)
    // We don't merge ALL root.helpers because transform may add helpers that
    // get optimized away during codegen (e.g., createElementVNode -> createElementBlock)
    let mut all_helpers: Vec<RuntimeHelper> = ctx.used_helpers.iter().collect();
    let mut all_helper_bits = retain_unique_helpers(&mut all_helpers);
    if root.helpers.contains(&RuntimeHelper::Unref) {
        push_unique_helper(RuntimeHelper::Unref, &mut all_helpers, &mut all_helper_bits);
    }
    // Collect helpers from hoisted nodes - generate_hoists() takes &CodegenContext (immutable)
    // so helpers used in hoisted VNodes aren't tracked via use_helper(). Pre-scan them here.
    profile!(
        "atelier.codegen.collect_hoist_helpers",
        collect_hoist_helpers(root, &mut all_helpers)
    );
    all_helper_bits = retain_unique_helpers(&mut all_helpers);

    let mut ordered_helpers = Vec::with_capacity(all_helpers.len());
    let mut ordered_helper_bits = 0;
    for helper in root.helpers.iter().copied() {
        if has_helper(all_helper_bits, helper) {
            push_unique_helper(helper, &mut ordered_helpers, &mut ordered_helper_bits);
        }
    }
    for helper in all_helpers {
        push_unique_helper(helper, &mut ordered_helpers, &mut ordered_helper_bits);
    }
    ordered_helpers.sort_by_key(|helper| vue_helper_import_rank(*helper));

    let mut preamble = profile!(
        "atelier.codegen.preamble",
        generate_preamble_from_helpers(&ctx, &ordered_helpers)
    );
    let imports_len = preamble.len();

    // Generate hoisted variable declarations (appended to preamble)
    let hoists_code = profile!("atelier.codegen.hoists", generate_hoists(&ctx, root));
    if !hoists_code.is_empty() {
        preamble.push('\n');
        preamble.push_str(&hoists_code);
    }

    // Assemble the source map (only populated when the `source_map` flag is on,
    // in which case `take_map_builder` returns `Some`). Segments were recorded
    // against byte offsets into `ctx.code` during emission, so resolve them
    // against the final code buffer before it is moved out. The render `code`
    // string itself is unchanged whether or not a map is produced.
    let map = ctx.take_map_builder().map(|builder| {
        let filename = ctx.options.filename.as_str();
        builder.finish(ctx.code_as_str(), filename, root.source.as_str())
    });

    CodegenResultWithSections {
        result: CodegenResult {
            code: ctx.into_code(),
            preamble,
            map,
        },
        sections: Some(CodegenSections {
            imports_len,
            assets_start,
            assets_end,
            return_expr_start,
            return_expr_end,
        }),
    }
}

fn retain_unique_helpers(helpers: &mut Vec<RuntimeHelper>) -> u128 {
    let mut helper_bits = 0;
    helpers.retain(|helper| push_helper_bit(*helper, &mut helper_bits));
    helper_bits
}

fn push_unique_helper(
    helper: RuntimeHelper,
    helpers: &mut Vec<RuntimeHelper>,
    helper_bits: &mut u128,
) {
    if push_helper_bit(helper, helper_bits) {
        helpers.push(helper);
    }
}

fn push_helper_bit(helper: RuntimeHelper, helper_bits: &mut u128) -> bool {
    let bit = helper_bit(helper);
    if *helper_bits & bit != 0 {
        return false;
    }
    *helper_bits |= bit;
    true
}

fn has_helper(helper_bits: u128, helper: RuntimeHelper) -> bool {
    helper_bits & helper_bit(helper) != 0
}

fn helper_bit(helper: RuntimeHelper) -> u128 {
    let index = helper as u8;
    debug_assert!(index < 128);
    1u128 << index
}

fn vue_helper_import_rank(helper: RuntimeHelper) -> u8 {
    match helper {
        RuntimeHelper::ResolveComponent
        | RuntimeHelper::ResolveDynamicComponent
        | RuntimeHelper::ResolveDirective
        | RuntimeHelper::ResolveFilter => 0,
        RuntimeHelper::VModelRadio
        | RuntimeHelper::VModelCheckbox
        | RuntimeHelper::VModelText
        | RuntimeHelper::VModelSelect
        | RuntimeHelper::VModelDynamic => 1,
        RuntimeHelper::WithDirectives | RuntimeHelper::WithModifiers | RuntimeHelper::WithKeys => 2,
        RuntimeHelper::ToDisplayString => 3,
        RuntimeHelper::CreateElementVNode
        | RuntimeHelper::CreateVNode
        | RuntimeHelper::RenderSlot => 4,
        RuntimeHelper::NormalizeClass
        | RuntimeHelper::NormalizeStyle
        | RuntimeHelper::NormalizeProps
        | RuntimeHelper::GuardReactiveProps
        | RuntimeHelper::MergeProps
        | RuntimeHelper::ToHandlers
        | RuntimeHelper::Camelize
        | RuntimeHelper::Capitalize
        | RuntimeHelper::ToHandlerKey => 5,
        RuntimeHelper::OpenBlock => 6,
        RuntimeHelper::CreateElementBlock | RuntimeHelper::CreateBlock => 7,
        RuntimeHelper::Fragment => 8,
        RuntimeHelper::CreateComment | RuntimeHelper::CreateText | RuntimeHelper::CreateStatic => 9,
        RuntimeHelper::RenderList
        | RuntimeHelper::CreateSlots
        | RuntimeHelper::SetBlockTracking
        | RuntimeHelper::PushScopeId
        | RuntimeHelper::PopScopeId
        | RuntimeHelper::WithCtx
        | RuntimeHelper::Unref
        | RuntimeHelper::IsRef
        | RuntimeHelper::WithMemo
        | RuntimeHelper::IsMemoSame
        | RuntimeHelper::VShow
        | RuntimeHelper::Teleport
        | RuntimeHelper::Suspense
        | RuntimeHelper::KeepAlive
        | RuntimeHelper::BaseTransition
        | RuntimeHelper::Transition
        | RuntimeHelper::TransitionGroup => 10,
        RuntimeHelper::SsrInterpolate
        | RuntimeHelper::SsrRenderVNode
        | RuntimeHelper::SsrRenderComponent
        | RuntimeHelper::SsrRenderSlot
        | RuntimeHelper::SsrRenderSlotInner
        | RuntimeHelper::SsrRenderAttrs
        | RuntimeHelper::SsrRenderAttr
        | RuntimeHelper::SsrRenderDynamicAttr
        | RuntimeHelper::SsrIncludeBooleanAttr
        | RuntimeHelper::SsrRenderClass
        | RuntimeHelper::SsrRenderStyle
        | RuntimeHelper::SsrRenderDynamicModel
        | RuntimeHelper::SsrGetDynamicModelProps
        | RuntimeHelper::SsrRenderList
        | RuntimeHelper::SsrLooseEqual
        | RuntimeHelper::SsrLooseContain
        | RuntimeHelper::SsrGetDirectiveProps
        | RuntimeHelper::SsrRenderTeleport
        | RuntimeHelper::SsrRenderSuspense => 11,
    }
}
