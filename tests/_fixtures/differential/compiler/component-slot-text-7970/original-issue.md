### Area

Compiler (atelier, VDOM): text and interpolations inside component slot content

### Version

`vize` / `@vizejs/native` / `@vizejs/vite-plugin` 0.432.0 (compared with `@vue/compiler-dom` / `@vitejs/plugin-vue` on Vue 3.5.43), Node v26.8.1, macOS arm64

### Summary

Inside the content of a component (its default slot), Vize emits each piece of adjacent text and each interpolation as its own `createTextVNode`. Vue merges them into one compound text node, as it does everywhere else. The same markup as the child of a plain element is merged by Vize too, so this only happens in slot content.

The rendered DOM differs: `<MyTitle>Hello {{ name }}!</MyTitle>` produces three text nodes instead of one. That is visible in tests that look at `childNodes`, and in rendering: in a screenshot test, a title that starts with `" "` before an interpolation, with `letter-spacing` and a CJK font, moved by a fraction of a pixel (text shaping runs per text node). It also adds vnodes to every slot with text.

The `whitespace` option does not matter (`condense` and `preserve` behave the same).

### Reproduction (template compiler)

```js
// cmp.mjs — npm i @vizejs/native@0.432.0 @vue/compiler-dom@3.5.43
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const vize = require("@vizejs/native");
const { compile } = require("@vue/compiler-dom");

const cases = {
  "element child": '<div>\n  <span class="icon"></span>\n  {{ title }}\n</div>',
  "component slot": '<MyTitle>\n  <span class="icon"></span>\n  {{ title }}\n</MyTitle>',
  "component slot, text only": "<MyTitle>Hello {{ name }}!</MyTitle>"
};
for (const [name, template] of Object.entries(cases)) {
  const texts = code => code.match(/_createTextVNode\([^\n]*\)/g);
  console.log(name);
  console.log("  vize:", JSON.stringify(texts(vize.compile(template, {}).code)));
  console.log("  vue: ", JSON.stringify(texts(compile(template, {}).code)));
}
```

```console
$ node cmp.mjs
element child
  vize: ["_createTextVNode(\" \" + _toDisplayString(title), 1 /* TEXT */)"]
  vue:  ["_createTextVNode(\" \" + _toDisplayString(title), 1 /* TEXT */)"]
component slot
  vize: ["_createTextVNode(\" \")","_createTextVNode(_toDisplayString(title), 1 /* TEXT */)"]
  vue:  ["_createTextVNode(\" \" + _toDisplayString(title), 1 /* TEXT */)"]
component slot, text only
  vize: ["_createTextVNode(\"Hello \")","_createTextVNode(_toDisplayString(name), 1 /* TEXT */)","_createTextVNode(\"!\")"]
  vue:  ["_createTextVNode(\"Hello \" + _toDisplayString(name) + \"!\", 1 /* TEXT */)"]
```

### Reproduction (SFC through Vite, rendered DOM)

`MyTitle.vue`

```vue
<template>
  <h2 class="my-title"><slot></slot></h2>
</template>
```

`App.vue`

```vue
<script setup lang="ts">
import MyTitle from "./MyTitle.vue";

const name = "world";
const title = "Spring course";
</script>

<template>
  <MyTitle>Hello {{ name }}!</MyTitle>
  <MyTitle>
    <span class="icon"></span>
    {{ title }}
  </MyTitle>
</template>
```

Built once with `@vitejs/plugin-vue` and once with `@vizejs/vite-plugin` (Vite 8.2.2), then in the browser:

```js
[...document.querySelectorAll(".my-title")].map(el => [...el.childNodes].map(n => n.nodeType === 3 ? n.data : `<${n.nodeName.toLowerCase()}>`))
```

```
plugin-vue: [["", "Hello world!", ""], ["", "<span>", " Spring course", ""]]
vize:       [["", "Hello ", "world", "!", ""], ["", "<span>", " ", "Spring course", ""]]
```

(The empty strings are the slot fragment anchors, the same in both.)

### Expected

Adjacent text and interpolations in slot content are merged into one text vnode, as in element children and as `@vue/compiler-dom` does, so both compilers render the same DOM.
