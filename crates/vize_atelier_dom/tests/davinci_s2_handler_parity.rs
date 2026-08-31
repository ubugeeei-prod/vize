//! Reduced real-project handler parity witnesses.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "native_expression_arrow_handler_is_wrapped",
        r#"<span @click="() => deleteResource(cell?.data as Node)">delete</span>"#,
    ),
    (
        "native_zero_param_arrow_handler_stays_direct",
        r#"<button @click="() => toggleDark()">toggle</button>"#,
    ),
    (
        "native_param_arrow_handler_stays_direct",
        r#"<button @contextmenu="event => openContextMenu(event, image)">open</button>"#,
    ),
    (
        "native_param_arrow_with_ts_body_is_wrapped",
        r#"<button @input="(e) => updateColor(key, (e.target as HTMLInputElement).value)"></button>"#,
    ),
    (
        "v_for_dynamic_class_and_inline_handler_keeps_compact_props_object",
        r#"<button v-for="item in items" :class="['button', { active: item.active }]" @click="select(item)">{{ item.label }}</button>"#,
    ),
    (
        "component_null_handler_stays_raw",
        r#"<VueMultiselect @change="null" />"#,
    ),
    (
        "multiline_native_inline_handler_keeps_authored_padding",
        r#"<button @click="
  themeMode.changeCompactTheme(
    themeMode.compactTheme.value === 'compact' ? '' : 'compact',
  )
">compact</button>"#,
    ),
    (
        "multiline_component_update_handler_keeps_authored_padding",
        r#"<AppCheck @update:model-value="
  value => {
    p(opt.key).value = {
      ...p(opt.key).value,
      [key]: value,
    };
  }
" />"#,
    ),
    (
        "typed_block_arrow_handler_is_wrapped",
        r#"<input @change="(e: Event) => { const v = (e.target as HTMLInputElement).value; }">"#,
    ),
    (
        "native_block_arrow_with_ts_body_is_wrapped",
        r#"<button @pointerdown="(event) => { const target = event.target as HTMLElement; target.focus(); }"></button>"#,
    ),
    (
        "component_arrow_with_non_null_body_is_wrapped",
        r#"<VButton @click="() => runManualFlow(link.flow!)" />"#,
    ),
    (
        "typed_block_arrow_keeps_line_comment_spelling_when_wrapped",
        r#"<button @touchstart="(event: TouchEvent) => {
  // keep touch branch note
  start(event)
}"></button>"#,
    ),
    (
        "untyped_arrow_component_handler_stays_direct",
        r#"<FieldKeyValues @add="(key, value) => addKeyValue(headers, key, value)" />"#,
    ),
    (
        "typed_arrow_component_handler",
        r#"<FieldKeyValues @add="(key: string, value: string) => addKeyValue(headers, key, value)" />"#,
    ),
    (
        "typed_arrow_component_handler_with_static_props",
        r#"<FieldKeyValues label="Headers" @remove="(index: number) => removeKeyValue(index, headers)" />"#,
    ),
    (
        "block_arrow_handler_preserves_line_comment",
        r#"<button @click="() => {
if (active) {
  // keep active click note
  next();
}
else {
  // keep inactive click note
  select();
}
}"></button>"#,
    ),
];

#[test]
fn s2_real_project_handler_parity_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
