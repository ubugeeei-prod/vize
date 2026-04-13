//! VDom code generation.
//!
//! This module generates JavaScript render function code from the transformed AST.

mod children;
mod context;
mod element;
mod expression;
mod generate;
mod helpers;
mod node;
mod patch_flag;
mod props;
mod root;
mod slots;
mod v_for;
mod v_if;

use crate::{
    ast::{RootNode, RuntimeHelper, TemplateChildNode},
    options::CodegenOptions,
};
use vize_carton::profile;

use children::is_directive_comment;
pub use context::{CodegenContext, CodegenResult};
use element::generate_root_node;
use generate::{collect_hoist_helpers, generate_hoists};
use node::generate_node;
use root::{
    generate_assets, generate_function_signature, generate_preamble_from_helpers,
    is_ignorable_root_text,
};

/// Generate code from root AST.
pub fn generate(root: &RootNode<'_>, options: CodegenOptions) -> CodegenResult {
    let mut ctx = CodegenContext::new(options);
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
    profile!("atelier.codegen.assets", generate_assets(&mut ctx, root));

    // Generate return statement
    ctx.push("return ");

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
        ctx.push("], 64 /* STABLE_FRAGMENT */))");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");

    // Now generate preamble after we know all used helpers
    // Only include specific helpers from root.helpers that are known to be
    // added during transform but not tracked during codegen (like Unref)
    // We don't merge ALL root.helpers because transform may add helpers that
    // get optimized away during codegen (e.g., createElementVNode -> createElementBlock)
    let mut all_helpers: Vec<RuntimeHelper> = ctx.used_helpers.iter().copied().collect();
    if root.helpers.contains(&RuntimeHelper::Unref) && !all_helpers.contains(&RuntimeHelper::Unref)
    {
        all_helpers.push(RuntimeHelper::Unref);
    }
    // Collect helpers from hoisted nodes - generate_hoists() takes &CodegenContext (immutable)
    // so helpers used in hoisted VNodes aren't tracked via use_helper(). Pre-scan them here.
    profile!(
        "atelier.codegen.collect_hoist_helpers",
        collect_hoist_helpers(root, &mut all_helpers)
    );
    // Sort helpers for consistent output order
    all_helpers.sort();
    all_helpers.dedup();

    let mut preamble = profile!(
        "atelier.codegen.preamble",
        generate_preamble_from_helpers(&ctx, &all_helpers)
    );

    // Generate hoisted variable declarations (appended to preamble)
    let hoists_code = profile!("atelier.codegen.hoists", generate_hoists(&ctx, root));
    if !hoists_code.is_empty() {
        preamble.push('\n');
        preamble.push_str(&hoists_code);
    }

    CodegenResult {
        code: ctx.into_code(),
        preamble,
        map: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::compile;

    fn result_output(result: &super::CodegenResult) -> vize_carton::String {
        let mut output =
            vize_carton::String::with_capacity(result.preamble.len() + result.code.len() + 1);
        output.push_str(&result.preamble);
        output.push('\n');
        output.push_str(&result.code);
        output
    }

    macro_rules! assert_codegen_snapshot {
        ($result:expr) => {{
            let output = result_output(&$result);
            insta::assert_snapshot!(output.as_str());
        }};
    }

    #[test]
    fn test_codegen_simple_element() {
        let result = compile!("<div>hello</div>");
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_interpolation() {
        // When prefix_identifiers is false (default), expressions are not prefixed with _ctx.
        let result = compile!("<div>{{ msg }}</div>");
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_with_props() {
        let result = compile!(r#"<div id="app" class="container"></div>"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_component() {
        let result = compile!("<MyComponent />");
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_root_directive_comment_does_not_create_fragment_hole() {
        let result =
            compile!("<!-- @vize:forget sections are labeled by their headings --><section />");

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_root_only_directive_comment_compiles_to_null() {
        let result = compile!("<!-- @vize:forget no render output -->");

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_pascal_case_dynamic_component() {
        let result = compile!(r#"<Component :is="current" :active-class="klass" />"#);

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_pascal_case_dynamic_component_inside_v_for() {
        let result =
            compile!(r#"<Component :is="item.component" v-for="item in items" :key="item.id" />"#);

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_if_template_fragment_wraps_interpolation_in_text_vnode() {
        let result = compile!(
            r#"<p><template v-if="ready">{{ count }}</template><span v-if="pending">updating</span></p>"#
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_if_template_fragment_wraps_static_text_in_text_vnode() {
        let result = compile!(
            r#"<div><template v-if="ready">Found packages</template><span v-if="pending">updating</span></div>"#
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_preamble_module() {
        use crate::options::CodegenMode;
        let options = super::CodegenOptions {
            mode: CodegenMode::Module,
            ..Default::default()
        };
        let result = compile!("<div>hello</div>", options);
        insta::assert_snapshot!(result.preamble.as_str());
    }

    #[test]
    fn test_codegen_v_model_on_component() {
        let result = compile!(r#"<MyComponent v-model="msg" />"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_model_with_arg() {
        let result = compile!(r#"<MyComponent v-model:title="pageTitle" />"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_model_on_input() {
        let result = compile!(r#"<input v-model="inputValue" />"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_model_with_other_props() {
        // v-model with other props should not produce comments
        let result = compile!(r#"<MonacoEditor v-model="source" :language="editorLanguage" />"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_slot_fallback() {
        let result = compile!(r#"<slot name="label">{{ label }}</slot>"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_slot_without_fallback() {
        // Slot element without fallback should not have empty object or function
        let result = compile!(r#"<slot name="header"></slot>"#);
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_codegen_conditional_slot_with_else_does_not_append_undefined() {
        let result = compile!(
            r#"<MyDialog>
  <template v-if="step === 1" #header>First</template>
  <template v-else #header>Second</template>
</MyDialog>"#
        );
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_default_slot_with_v_if_is_marked_dynamic() {
        let result = compile!(
            r#"<PageWithHeader>
  <div v-if="tab === 'overview'">Overview</div>
  <div v-else-if="tab === 'emojis'">Emojis</div>
  <div v-else>Charts</div>
</PageWithHeader>"#
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_forwarded_default_slot_is_marked_forwarded() {
        let result = compile!(r#"<MkSwiper><slot /></MkSwiper>"#);

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_if_branch_mixed_children_wrap_interpolations_in_text_vnodes() {
        let result = compile!(
            r#"<p v-if="speaker.affiliation || speaker.title">{{ speaker.affiliation }}<br v-if="speaker.affiliation && speaker.title" />{{ speaker.title }}</p>"#
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_for_aliases_without_parentheses_stay_local() {
        use crate::options::{CodegenOptions, TransformOptions};
        use crate::parser::parse;
        use crate::transform::transform;
        use bumpalo::Bump;

        let allocator = Bump::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<div><template v-for="item, index of items" :key="index"><UserCard :user="item" :data-index="index" /></template></div>"#,
        );

        transform(
            &allocator,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
            None,
        );

        let result = super::generate(
            &root,
            CodegenOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_v_for_scope_handlers_are_not_cached() {
        use crate::options::{CodegenOptions, TransformOptions};
        use crate::parser::parse;
        use crate::transform::transform;
        use bumpalo::Bump;

        let allocator = Bump::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<button v-for="tab in tabs" :key="tab.id" @click="select(tab)">{{ tab.label }}</button>"#,
        );

        transform(
            &allocator,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
            None,
        );

        let result = super::generate(
            &root,
            CodegenOptions {
                prefix_identifiers: true,
                cache_handlers: true,
                ..Default::default()
            },
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_scoped_slot_params_stay_local_in_handlers() {
        use crate::options::{CodegenOptions, TransformOptions};
        use crate::parser::parse;
        use crate::transform::transform;
        use bumpalo::Bump;

        let allocator = Bump::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<CommonPaginator>
  <template #default="{ item, index }">
    <button @click="showHistory(item)">{{ index }}</button>
    <button @click="() => edit(item.id)">{{ item.id }}</button>
  </template>
</CommonPaginator>"#,
        );

        transform(
            &allocator,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
            None,
        );

        let result = super::generate(
            &root,
            CodegenOptions {
                prefix_identifiers: true,
                cache_handlers: true,
                ..Default::default()
            },
        );

        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_escape_newline_in_attribute() {
        // Attribute values containing newlines should be properly escaped
        let result = compile!(
            r#"<div style="
            color: red;
            background: blue;
        "></div>"#
        );
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_escape_special_chars_in_attribute() {
        // Attribute values should escape backslashes and quotes
        let result = compile!(r#"<div data-value="line1\nline2"></div>"#);
        assert_codegen_snapshot!(result);
    }

    #[test]
    fn test_codegen_escape_multiline_style_attribute() {
        // Complex multiline style attribute (real-world case from Discord issue)
        let result = compile!(
            r#"<div style="
            display: flex;
            flex-direction: column;
        "></div>"#
        );
        assert_codegen_snapshot!(result);
    }
}
