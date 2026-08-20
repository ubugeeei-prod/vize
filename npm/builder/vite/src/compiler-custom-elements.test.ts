/**
 * `compiler.customElements` tag patterns must compile matched renderer tags as
 * elements while explicit script setup imports still resolve as components.
 */
import assert from "node:assert/strict";

import { compileFile } from "./compiler.ts";

const customElementPatternSource = `<script setup lang="ts">
import { TresCanvas } from "@tresjs/core";
const visible = true;
</script>

<template>
  <TresCanvas>
    <TresMesh v-if="visible">
      <TresSpotLight />
    </TresMesh>
  </TresCanvas>
</template>`;

const customElementPatternCompiled = compileFile(
  "/src/TresCustomElements.vue",
  new Map(),
  {
    sourceMap: false,
    ssr: false,
    vapor: false,
    customRenderer: true,
    customElements: ["Tres*"],
  },
  customElementPatternSource,
);

assert.match(
  customElementPatternCompiled.code,
  /_createBlock\(\$setup\.TresCanvas/,
  "Explicit script setup imports should still win over custom-element patterns",
);
assert.match(customElementPatternCompiled.code, /import \{ TresCanvas \}/);
assert.match(
  customElementPatternCompiled.code,
  /_createElementBlock\("TresMesh"/,
  "Matched PascalCase renderer tags should compile as element blocks",
);
assert.match(
  customElementPatternCompiled.code,
  /_createElementVNode\("TresSpotLight"/,
  "Matched PascalCase renderer child tags should compile as element vnodes",
);
assert.doesNotMatch(
  customElementPatternCompiled.code,
  /_resolveComponent\("Tres(?:Canvas|Mesh|SpotLight)"\)/,
  "Matched PascalCase renderer tags must not use runtime component resolution",
);

// Control: without `customElements`, `customRenderer` alone keeps the same tags
// as components resolved at runtime.
const customRendererOnlyCompiled = compileFile(
  "/src/TresCustomElements.vue",
  new Map(),
  {
    sourceMap: false,
    ssr: false,
    vapor: false,
    customRenderer: true,
  },
  customElementPatternSource,
);

assert.match(
  customRendererOnlyCompiled.code,
  /_resolveComponent\("TresMesh"\)/,
  "Without customElements patterns, renderer tags stay runtime-resolved components",
);
