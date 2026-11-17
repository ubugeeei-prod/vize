//! A generated `$name` never captures an authored binding of the same name.
//!
//! The generator spells its listener arguments `$args`, its handler wrapper
//! parameter `$cb` and its `v-model` write-back parameter `$value`. An SFC
//! that authors those names (and reads Vue's own `$attrs`) must check exactly
//! like the same SFC with other names of the same length.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::{String, cstr};

const CHILD: &str = r#"<script setup lang="ts">
defineEmits<{ pick: [value: number] }>();
</script>
<template><span /></template>
"#;

fn parent(args: &str, cb: &str, value: &str) -> String {
    cstr!(
        r#"<script setup lang="ts">
import {{ ref }} from 'vue'
import Child from './Child.vue'
const {args} = 'authored'
const {cb} = (text: string) => text.length
const {value} = ref('text')
function takesString(text: string) {{ return text }}
function takesNumber(count: number) {{ return count }}
</script>
<template>
  <Child @pick="takesString({args})" />
  <Child @pick="takesNumber({args})" />
  <button @click="{cb}({args})">{{{{ {args}.toUpperCase() }}}} {{{{ $attrs.id }}}}</button>
  <input v-model="{value}" />
</template>
"#
    )
}

fn diagnostics(
    name: &str,
    args: &str,
    cb: &str,
    value: &str,
) -> Option<Vec<(String, Option<u32>, String)>> {
    let source = parent(args, cb, value);
    let root = create_project_case(
        name,
        &[
            ("src/Child.vue", CHILD),
            ("src/Parent.vue", source.as_str()),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot
}

#[test]
fn authored_dollar_bindings_check_like_any_other_name() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let Some(dollar) = diagnostics("generated-name-hygiene-dollar", "$args", "$cb", "$value")
    else {
        return;
    };
    let Some(plain) = diagnostics("generated-name-hygiene-plain", "xargs", "xcb", "xvalue") else {
        return;
    };
    assert_eq!(dollar.len(), 1, "{dollar:?}");
    assert_eq!(dollar.first().and_then(|(_, code, _)| *code), Some(2345));
    let rewrite = |(file, code, message): &(String, Option<u32>, String)| {
        (
            file.clone(),
            *code,
            String::from(message.replace("xargs", "$args").as_str()),
        )
    };
    assert_eq!(dollar, plain.iter().map(rewrite).collect::<Vec<_>>());
}
