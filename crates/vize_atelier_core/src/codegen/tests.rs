//! Snapshot and assertion tests for the codegen pipeline.

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
fn test_codegen_component_name_with_colon_uses_valid_identifier() {
    let allocator = bumpalo::Bump::new();
    let parser_opts = crate::ParserOptions {
        is_native_tag: Some(vize_carton::is_native_tag),
        ..Default::default()
    };
    let (mut root, errors) = crate::parse_with_options(
        &allocator,
        r#"<global:head title="Page Title" />"#,
        parser_opts,
    );
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    crate::transform::transform(
        &allocator,
        &mut root,
        crate::TransformOptions::default(),
        None,
    );
    let output = result_output(&super::generate(&root, crate::CodegenOptions::default()));

    // Vue encodes non-word characters by char code (`:` -> 58), matching
    // `toValidAssetId` (issue #4422).
    assert!(output.contains(r#"const _component_global58head = _resolveComponent("global:head")"#));
    assert!(output.contains("_createBlock(_component_global58head"));
    assert!(!output.contains("_component_global:head"));
}

#[test]
fn test_codegen_self_component_resolve_marks_maybe_self_reference() {
    let result = compile!(
        "<FileTree />",
        super::CodegenOptions {
            component_name: Some("FileTree".into()),
            ..Default::default()
        }
    );
    let output = result_output(&result);

    assert!(
        output.contains(r#"const _component_FileTree = _resolveComponent("FileTree", true)"#),
        "self component resolution should pass maybeSelfReference. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_inline_setup_ref_component_prop_uses_value() {
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = crate::parse(&allocator, r#"<Child :initialText="initialText" />"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let mut bindings = vize_carton::FxHashMap::default();
    bindings.insert("Child".into(), crate::BindingType::SetupConst);
    bindings.insert("initialText".into(), crate::BindingType::SetupRef);
    let binding_metadata = crate::BindingMetadata {
        bindings,
        props_aliases: vize_carton::FxHashMap::default(),
        is_script_setup: true,
    };

    crate::transform::transform(
        &allocator,
        &mut root,
        crate::TransformOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(binding_metadata.clone()),
            ..Default::default()
        },
        None,
    );

    let output = result_output(&super::generate(
        &root,
        crate::CodegenOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(binding_metadata),
            ..Default::default()
        },
    ));

    assert!(
        output.contains("initialText: initialText.value"),
        "component prop should unwrap setup refs in inline mode. Got:\n{}",
        output
    );
}

#[test]
fn test_root_directive_comment_does_not_create_fragment_hole() {
    let result =
        compile!("<!-- @vize:forget sections are labeled by their headings --><section></section>");

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
fn test_codegen_numeric_component_v_for_uses_component_block() {
    let result = compile!(r#"<Child v-for="(id, index) in 4" :key="id" :label="String(index)" />"#);

    assert!(
        result.code.contains("_createBlock(_component_Child"),
        "numeric component v-for should render a component block. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains(r#"_createElementVNode("Child""#),
        "numeric component v-for must not render Child as a native element. Got:\n{}",
        result.code
    );
}

#[test]
fn test_codegen_duplicate_attribute_keeps_first_occurrence() {
    // Regression for #958: a `<div id="a" id="b">x</div>` template
    // used to produce a 0-byte module marked as success because the
    // parser pushed a fatal-looking diagnostic and the SFC pipeline
    // discarded the template output. Codegen now dedupes by
    // attribute name (Vue parity: first wins); the parser
    // diagnostic is classified as recoverable so downstream
    // continues. The compile macro bails on parse errors, so this
    // test drives the pipeline by hand.
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = crate::parser::parse(&allocator, r#"<div id="a" id="b">x</div>"#);
    assert!(
        errors
            .iter()
            .any(|e| e.code == vize_relief::errors::ErrorCode::DuplicateAttribute),
        "expected a DuplicateAttribute diagnostic, got {errors:?}"
    );
    assert!(errors.iter().all(|e| e.is_recoverable()));
    crate::transform::transform(
        &allocator,
        &mut root,
        crate::options::TransformOptions::default(),
        None,
    );
    let result = crate::codegen::generate(&root, crate::options::CodegenOptions::default());
    assert!(!result.code.is_empty(), "compiled output must not be empty");
    assert!(
        result.code.contains(r#"id: "a""#),
        "expected first `id` to be retained, got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains(r#"id: "b""#),
        "expected duplicate `id` to be dropped, got:\n{}",
        result.code
    );
}

#[test]
fn test_codegen_v_if_nested_branch_keys_reset_per_scope() {
    // Regression for #961 (Vue-parity): a nested v-if (inside another
    // v-if's branch) starts its key counter at 0 again, matching Vue's
    // recursive transform. Without the per-branch reset, sibling
    // sub-chains would consume keys from the outer counter and drift
    // from `@vue/compiler-sfc`.
    let result = compile!(r#"<div v-if="a"><span v-if="b">B</span><span v-else>C</span></div>"#);
    // Outer key 0, inner keys 0 and 1.
    let key_count_0 = result.code.matches("{ key: 0 }").count();
    assert!(
        key_count_0 >= 2,
        "expected outer + inner key 0 (>=2 occurrences), got {key_count_0}:\n{}",
        result.code
    );
    assert!(
        result.code.contains("{ key: 1 }"),
        "missing inner key 1:\n{}",
        result.code
    );
}

#[test]
fn test_codegen_v_if_sibling_chains_allocate_unique_branch_keys() {
    // Regression for #961: sibling `v-if`/`v-else` blocks must get keys
    // from a template-wide counter (Vue parity: 0,1,2,3 — not the
    // per-chain 0,1,0,1 vize used to emit), so a patch can't reuse a
    // first-block element for a second-block element.
    let result = compile!(
        r#"<div><div v-if="a">A</div><div v-else>B</div><div v-if="c">C</div><div v-else>D</div></div>"#
    );

    assert!(
        result.code.contains("{ key: 0 }"),
        "missing key 0:\n{}",
        result.code
    );
    assert!(
        result.code.contains("{ key: 1 }"),
        "missing key 1:\n{}",
        result.code
    );
    assert!(
        result.code.contains("{ key: 2 }"),
        "missing key 2:\n{}",
        result.code
    );
    assert!(
        result.code.contains("{ key: 3 }"),
        "missing key 3:\n{}",
        result.code
    );
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
fn test_codegen_v_model_without_expression_omits_empty_directive_binding() {
    let result = compile!(r#"<input v-model />"#);
    let output = result_output(&result);

    assert!(
        !output.contains("_vModelText, ]"),
        "value-less v-model must not emit malformed directive bindings:\n{}",
        output
    );
    assert!(
        !output.contains("_withDirectives"),
        "value-less native v-model should be removed before directive codegen:\n{}",
        output
    );
    assert!(output.contains(r#"_createElementBlock("input")"#));
}

#[test]
fn test_codegen_v_model_on_input_with_custom_directive() {
    let result = compile!(r#"<input v-model="inputValue" v-example />"#);
    assert_codegen_snapshot!(result);
}

#[test]
fn test_codegen_nested_v_model_on_input_with_custom_directive() {
    let result = compile!(r#"<div><input v-model="inputValue" v-example /></div>"#);
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
fn test_codegen_conditional_slot_outlet_with_bound_props_uses_render_slot() {
    let result = compile!(r#"<slot v-if="show" name="updater" v-bind="{ number, update }" />"#);
    let output = result_output(&result);

    assert!(
        output.contains(r#"_renderSlot(_ctx.$slots, "updater""#),
        "conditional slot outlet should use renderSlot. Got:\n{}",
        output
    );
    assert!(
        output.contains(r#"_mergeProps({ number, update }, { key: 0 })"#),
        "v-bind object props should be merged with the branch key. Got:\n{}",
        output
    );
    assert!(
        !output.contains(r#"_createElementBlock("slot""#)
            && !output.contains(r#"_createElementVNode("slot""#),
        "slot outlets should not be emitted as literal slot elements. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_v_for_slot_outlet_with_bound_props_uses_render_slot() {
    let result = compile!(
        r#"<slot v-for="(item, index) of items" v-bind="{ key: item.id }" :item="item" :index="index" />"#
    );
    let output = result_output(&result);

    assert!(
        output.contains(r#"_renderSlot(_ctx.$slots, "default""#),
        "v-for slot outlet should use renderSlot. Got:\n{}",
        output
    );
    assert!(
        output.contains(r#"_mergeProps({ key: item.id }, { item: item, index: index })"#),
        "slot v-bind object props should be preserved with explicit props. Got:\n{}",
        output
    );
    assert!(
        !output.contains(r#"_createElementBlock("slot""#)
            && !output.contains(r#"_createElementVNode("slot""#),
        "slot outlets should not be emitted as literal slot elements. Got:\n{}",
        output
    );
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
fn test_codegen_conditional_named_slot_preserves_implicit_default_slot() {
    let result = compile!(
        r#"<Parent>
  Not rendering!
  <template v-if="showNamed" #named>
    Named content
  </template>
</Parent>"#
    );
    let output = result_output(&result);

    assert!(
        output.contains("default: _withCtx(() => ["),
        "implicit default slot should be generated when createSlots is used:\n{}",
        output
    );
    assert!(
        output.contains("Not rendering!"),
        "default slot text should be preserved:\n{}",
        output
    );
    assert!(
        output.contains("name: \"named\""),
        "conditional named slot should still be dynamic:\n{}",
        output
    );
}

#[test]
fn test_codegen_looped_slot_key_and_index_aliases_stay_local_in_dynamic_args() {
    use crate::options::{CodegenOptions, TransformOptions};
    use crate::parser::parse;
    use crate::transform::transform;
    use bumpalo::Bump;

    let allocator = Bump::new();
    let (mut root, _) = parse(
        &allocator,
        r#"<Comp>
  <template v-for="(item, idx) in list" #default>
    <div :[idx]="item"></div>
  </template>
  <template v-for="(val, key) in obj" #item>
    <button @[key]="val"></button>
  </template>
</Comp>"#,
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
    let output = result_output(&result);

    assert!(
        output.contains("[idx || \"\"]"),
        "looped slot index alias should remain local in dynamic prop args:\n{}",
        output
    );
    assert!(
        output.contains("_toHandlerKey(key)"),
        "looped slot key alias should remain local in dynamic event args:\n{}",
        output
    );
    assert!(
        !output.contains("_ctx.idx") && !output.contains("_ctx.key"),
        "looped slot key/index aliases must not be prefixed as outer scope refs:\n{}",
        output
    );
    assert_codegen_snapshot!(result);
}

#[test]
fn test_codegen_default_slot_with_v_if_is_stable() {
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
fn test_codegen_dynamic_keyed_slot_child_uses_block() {
    let result = compile!(
        r#"<PageWithHeader>
  <div :key="tab">
    <MkPagination :paginator="paginator">
      <template #default="{ items }">{{ items.length }}</template>
    </MkPagination>
  </div>
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
fn test_codegen_if_branch_mixed_children_wraps_interpolation_in_text_vnode() {
    let result = compile!(
        r#"<div><label v-if="show">{{ msg }}<span v-if="required">*</span></label></div>"#
    );

    assert!(
        result
            .code
            .contains("_createTextVNode(_toDisplayString(msg), 1 /* TEXT */)"),
        "mixed children inside v-if branch should wrap interpolation in createTextVNode. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("[_toDisplayString(msg),"),
        "v-if branch should not emit raw string children inside arrays. Got:\n{}",
        result.code
    );
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
fn test_codegen_numeric_template_v_for_uses_fragment() {
    let result = compile!(
        r#"<div><template v-for="n in 4" :key="`set-${n}`"><button></button><span v-for="(icon, i) in icons" :key="`${n}-${i}`" :class="icon"></span></template></div>"#
    );

    assert!(
        !result.code.contains("\"template\""),
        "template v-for must not create a DOM template element. Got:\n{}",
        result.code
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
fn test_codegen_merged_v_on_handlers_are_cached() {
    use crate::options::{CodegenOptions, TransformOptions};
    use crate::parser::parse;
    use crate::transform::transform;
    use bumpalo::Bump;

    let allocator = Bump::new();
    let (mut root, _) = parse(
        &allocator,
        r#"<div @click="() => x++" @click.stop="() => y++"></div>"#,
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
    let output = result_output(&result);

    assert!(
            output.contains("onClick: [_cache[0] || (_cache[0] = () => _ctx.x++), _cache[1] || (_cache[1] = _withModifiers(() => _ctx.y++, [\"stop\"]))]"),
            "merged same-event handlers should each be cached. Got:\n{}",
            output
        );
    insta::assert_snapshot!(output.as_str());
}

#[test]
fn test_codegen_v_on_option_modifier_events_are_not_merged() {
    // Issue #1172: events differing only by an option modifier
    // (.once/.capture/.passive) must compile to distinct props and never
    // be merged under one key.
    let once = result_output(&compile!(r#"<div @click="a" @click.once="b"></div>"#));
    assert!(
        once.contains("onClick: a") && once.contains("onClickOnce: b"),
        "@click and @click.once should be distinct props. Got:\n{}",
        once
    );
    assert!(
        !once.contains("onClick: ["),
        "@click and @click.once must not be merged into an array. Got:\n{}",
        once
    );

    let capture = result_output(&compile!(r#"<div @click.capture="a" @click="b"></div>"#));
    assert!(
        capture.contains("onClickCapture: a") && capture.contains("onClick: b"),
        "@click.capture and @click should be distinct props. Got:\n{}",
        capture
    );
    assert!(
        !capture.contains("onClick: ["),
        "@click.capture and @click must not be merged into an array. Got:\n{}",
        capture
    );
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

fn compile_prefixed(source: &str) -> vize_carton::String {
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = crate::parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    crate::transform::transform(
        &allocator,
        &mut root,
        crate::TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    result_output(&super::generate(
        &root,
        crate::CodegenOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    ))
}

#[test]
fn test_codegen_looped_slot_index_alias_is_slot_param_for_dynamic_arg() {
    // Issue #1173: the index alias of a v-for on a slot template must be
    // registered as a slot param so a dynamic arg derived from it is not
    // wrongly _ctx-prefixed.
    let output = compile_prefixed(
        r#"<Comp><template v-for="(item, idx) in list" #default><div :[idx]="item"></div></template></Comp>"#,
    );
    assert!(
        output.contains("[idx || \"\"]"),
        "index alias should be a local slot param, not _ctx-prefixed. Got:\n{}",
        output
    );
    assert!(
        !output.contains("_ctx.idx"),
        "index alias must not be _ctx-prefixed. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_looped_slot_key_alias_is_slot_param_for_dynamic_arg() {
    // Issue #1173: the key alias (object iteration) of a v-for on a slot
    // template must be registered as a slot param.
    let output = compile_prefixed(
        r#"<Comp><template v-for="(val, key) in obj" #default><div :[key]="val"></div></template></Comp>"#,
    );
    assert!(
        output.contains("[key || \"\"]"),
        "key alias should be a local slot param, not _ctx-prefixed. Got:\n{}",
        output
    );
    assert!(
        !output.contains("_ctx.key"),
        "key alias must not be _ctx-prefixed. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_static_style_merged_with_dynamic_escapes_values() {
    // Issue #1171: a static `style` merged with `:style` must JSON-escape
    // key/value so a `"` does not terminate the JS string early.
    let output = result_output(&compile!(r#"<div style='content:"x"' :style="s"></div>"#));
    assert!(
        output.contains(r#"_normalizeStyle([{"content":"\"x\""}, s])"#),
        "static style values must be escaped. Got:\n{}",
        output
    );
}

#[test]
fn test_codegen_static_style_merged_with_dynamic_does_not_split_inside_parens() {
    // Issue #1171: a `;` inside `url(...)` must not be treated as a
    // declaration separator, and no orphan double comma must appear.
    let output = result_output(&compile!(
        r#"<div style="background:url(a;b);color:red" :style="s"></div>"#
    ));
    assert!(
        output.contains(r#"_normalizeStyle([{"background":"url(a;b)","color":"red"}, s])"#),
        "`;` inside parens must not split the declaration. Got:\n{}",
        output
    );
    assert!(
        !output.contains(",,"),
        "orphan parts must not produce a double comma. Got:\n{}",
        output
    );
}
