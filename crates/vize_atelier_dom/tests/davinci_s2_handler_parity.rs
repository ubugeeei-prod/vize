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
        "native_param_arrow_handler_stays_direct",
        r#"<button @contextmenu="event => openContextMenu(event, image)">open</button>"#,
    ),
    (
        "native_no_param_arrow_handler_stays_direct",
        r#"<button @click="() => toggleDark()">toggle</button>"#,
    ),
    (
        "native_null_handler_stays_direct",
        r#"<Select @change="null" />"#,
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
