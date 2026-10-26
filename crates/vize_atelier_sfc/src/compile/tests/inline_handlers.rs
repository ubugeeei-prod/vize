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

/// The inline render function, from its `return` to its closing brace.
fn render(code: &str) -> &str {
    let start = code
        .find("return (_ctx, _cache) => {")
        .expect("inline render");
    let rest = code.get(start..).expect("render start");
    rest.get(..rest.find("\n}\n").expect("render end") + 2)
        .expect("render body")
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
    assert_eq!(
        render(&code),
        concat!(
            "return (_ctx, _cache) => {\n",
            "  return (_openBlock(), _createElementBlock(_Fragment, null, [\n",
            "    _createVNode(MyButton, {\n",
            "      id: \"a\",\n",
            "      onPing: _unref(pingA)\n",
            "    }, null, 8 /* PROPS */, [\"onPing\"]),\n",
            "    _createElementVNode(\"button\", {\n",
            "      id: \"b\",\n",
            "      onClick: _cache[0] || (_cache[0] = (...args) => (_unref(pingB) && _unref(pingB)(...args)))\n",
            "    }, \"b\"),\n",
            "    _createElementVNode(\"button\", {\n",
            "      onClick: _cache[1] || (_cache[1] = (...args) => (_unref(swap) && _unref(swap)(...args)))\n",
            "    }, \"c\"),\n",
            "    _createElementVNode(\"button\", {\n",
            "      onClick: _cache[2] || (_cache[2] = (...args) => (fnRef.value && fnRef.value(...args)))\n",
            "    }, \"d\")\n",
            "  ], 64 /* STABLE_FRAGMENT */))\n",
            "}",
        )
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
    assert_eq!(
        render(&code),
        concat!(
            "return (_ctx, _cache) => {\n",
            "  return (_openBlock(), _createElementBlock(_Fragment, null, [\n",
            "    _createVNode(MyButton, {\n",
            "      id: \"a\",\n",
            "      onPing: \n",
            "      _unref(incrementA)\n",
            "    \n",
            "    }, null, 8 /* PROPS */, [\"onPing\"]),\n",
            "    _createElementVNode(\"button\", {\n",
            "      id: \"c\",\n",
            "      onClick: _cache[0] || (_cache[0] = (...args) => (\n",
            "      _unref(incrementC)\n",
            "     && \n",
            "      _unref(incrementC)\n",
            "    (...args)))\n",
            "    }, \"c\"),\n",
            "    _createElementVNode(\"button\", {\n",
            "      onClick: _cache[1] || (_cache[1] = (...args) => ( _unref(incrementC)  &&  _unref(incrementC) (...args)))\n",
            "    }, \"d\")\n",
            "  ], 64 /* STABLE_FRAGMENT */))\n",
            "}",
        )
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
    assert_eq!(
        render(&code),
        concat!(
            "return (_ctx, _cache) => {\n",
            "  return (_openBlock(), _createElementBlock(_Fragment, null, [\n",
            "    _createVNode(_Transition, {\n",
            "      css: false,\n",
            "      onEnter: _unref(onEnter),\n",
            "      onAfterEnter: _unref(onAfterEnter)\n",
            "    }, {\n",
            "      default: _withCtx(() => [\n",
            "        (show.value)\n",
            "          ? (_openBlock(), _createElementBlock(\"p\", { key: 0 }, \"box\"))\n",
            "          : _createCommentVNode(\"v-if\", true)\n",
            "      ]),\n",
            "      _: 1 /* STABLE */\n",
            "    }, 8 /* PROPS */, [\"onEnter\", \"onAfterEnter\"]),\n",
            "    _createVNode(MyButton, {\n",
            "      onPing: _unref(hooks).ping,\n",
            "      onPong: _withModifiers(_unref(hooks).pong, [\"stop\"]),\n",
            "      onDone: _cache[0] || (_cache[0] = $event => (_unref(hooks).done($event)))\n",
            "    }, null, 8 /* PROPS */, [\"onPing\", \"onPong\"])\n",
            "  ], 64 /* STABLE_FRAGMENT */))\n",
            "}",
        )
    );
}

/// A padded identifier is a compound expression in Vue, never a constant, so
/// a component that receives it uncached patches it.
#[test]
fn padded_const_handlers_on_components_stay_dynamic() {
    let code = compile_inline(
        r#"<script setup>
import { ref } from 'vue'
import MyButton from './MyButton.vue'
const show = ref(true)
function save() {}
</script>
<template>
  <MyButton
    @ping="
      save
    "
  />
  <MyButton
    v-if="show"
    @ping="
      save
    "
  />
  <MyButton @ping="save" />
</template>"#,
    );
    // Vue indents a conditional branch two columns deeper; the rest is
    // byte-identical.
    assert_eq!(
        render(&code),
        concat!(
            "return (_ctx, _cache) => {\n",
            "  return (_openBlock(), _createElementBlock(_Fragment, null, [\n",
            "    _createVNode(MyButton, {\n",
            "      onPing: \n",
            "      save\n",
            "    \n",
            "    }, null, 8 /* PROPS */, [\"onPing\"]),\n",
            "    (show.value)\n",
            "      ? (_openBlock(), _createBlock(MyButton, {\n",
            "        key: 0,\n",
            "        onPing: \n",
            "      save\n",
            "    \n",
            "      }, null, 8 /* PROPS */, [\"onPing\"]))\n",
            "      : _createCommentVNode(\"v-if\", true),\n",
            "    _createVNode(MyButton, { onPing: save })\n",
            "  ], 64 /* STABLE_FRAGMENT */))\n",
            "}",
        )
    );
}
