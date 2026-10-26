//! Inline-template `v-on` handler regressions, pinned to `@vue/compiler-sfc`
//! 3.5.43 output.

use super::super::compile_sfc;
use crate::types::{ScriptCompileOptions, SfcCompileOptions};
use crate::{SfcParseOptions, parse_sfc};

fn compile_inline(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "App.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
                inline_template: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile SFC");
    result.code
}

fn assert_contains(code: &str, expected: &str) {
    assert!(
        code.contains(expected),
        "expected:\n{expected}\n\nin:\n{code}"
    );
}

/// #10777: a handler held in a ref is read through `unref` / `.value`.
#[test]
fn cached_reference_handlers_unwrap_refs() {
    let code = compile_inline(
        r#"<script setup>
import { ref } from 'vue'
import MyButton from './MyButton.vue'
import { useHandlers } from './useHandlers'
const { onPing: pingA } = useHandlers()
const { onPing: pingB } = useHandlers()
let swap = () => {}
const fnRef = ref(() => {})
</script>
<template>
  <MyButton id="a" @ping="pingA" />
  <button id="b" @click="pingB">b</button>
  <button @click="swap">c</button>
  <button @click="fnRef">d</button>
</template>"#,
    );
    assert_contains(
        &code,
        "      onPing: _unref(pingA)\n    }, null, 8 /* PROPS */, [\"onPing\"])",
    );
    assert_contains(
        &code,
        "onClick: _cache[0] || (_cache[0] = (...args) => (_unref(pingB) && _unref(pingB)(...args)))",
    );
    assert_contains(
        &code,
        "onClick: _cache[1] || (_cache[1] = (...args) => (_unref(swap) && _unref(swap)(...args)))",
    );
    assert_contains(
        &code,
        "onClick: _cache[2] || (_cache[2] = (...args) => (fnRef.value && fnRef.value(...args)))",
    );
}

/// #10778: surrounding whitespace keeps an identifier a method handler.
#[test]
fn padded_identifier_handlers_stay_references() {
    let code = compile_inline(
        r#"<script setup>
import MyButton from './MyButton.vue'
import { useCounter } from './useCounter'
const { increment: incrementA } = useCounter()
const { increment: incrementC } = useCounter()
</script>
<template>
  <MyButton
    id="a"
    @ping="
      incrementA
    "
  />
  <button
    id="c"
    @click="
      incrementC
    "
  >c</button>
  <button @click=" incrementC ">d</button>
</template>"#,
    );
    assert_contains(
        &code,
        "      onPing: \n      _unref(incrementA)\n    \n    }, null, 8 /* PROPS */, [\"onPing\"])",
    );
    assert_contains(
        &code,
        "onClick: _cache[0] || (_cache[0] = (...args) => (\n      _unref(incrementC)\n     && \n      _unref(incrementC)\n    (...args)))",
    );
    assert_contains(
        &code,
        "onClick: _cache[1] || (_cache[1] = (...args) => ( _unref(incrementC)  &&  _unref(incrementC) (...args)))",
    );
}

/// #10779: a component receives a method handler itself, keeping its arity
/// (vuejs/core#1541); inline statements are still cached.
#[test]
fn component_method_handlers_are_not_cached() {
    let code = compile_inline(
        r#"<script setup>
import { ref } from 'vue'
import MyButton from './MyButton.vue'
import { useExpand } from './useExpand'
const show = ref(false)
const { onEnter, onAfterEnter, hooks } = useExpand()
</script>
<template>
  <Transition :css="false" @enter="onEnter" @after-enter="onAfterEnter">
    <p v-if="show">box</p>
  </Transition>
  <MyButton @ping="hooks.ping" @pong.stop="hooks.pong" @done="hooks.done($event)" />
</template>"#,
    );
    assert_contains(
        &code,
        "      onEnter: _unref(onEnter),\n      onAfterEnter: _unref(onAfterEnter)\n    }",
    );
    assert_contains(&code, "}, 8 /* PROPS */, [\"onEnter\", \"onAfterEnter\"])");
    assert_contains(
        &code,
        "      onPing: _unref(hooks).ping,\n      onPong: _withModifiers(_unref(hooks).pong, [\"stop\"]),\n      onDone: _cache[0] || (_cache[0] = $event => (_unref(hooks).done($event)))\n    }, null, 8 /* PROPS */, [\"onPing\", \"onPong\"])",
    );
}
