import assert from "node:assert/strict";

import { compileFile } from "./compiler.ts";

const compiled = compileFile(
  "/src/OptionsApiMethodHandler.vue",
  new Map(),
  { sourceMap: false, ssr: false, vapor: false },
  `<script>
export default {
  methods: {
    onFocus(event) {
      this.$emit("focus", event);
    }
  }
}
</script>

<template>
  <input id="test" @focus="onFocus" />
</template>`,
);

assert.match(
  compiled.code,
  /onFocus:\s*\(\.\.\.args\) => \(_ctx\.onFocus && _ctx\.onFocus\(\.\.\.args\)\)/,
  "Options API v-on method refs must perform a live public instance lookup",
);
assert.doesNotMatch(
  compiled.code,
  /\$options\.onFocus/,
  "Options API v-on method refs must not capture the original $options method",
);

console.log("vite-plugin-vize Options API event tests passed!");
