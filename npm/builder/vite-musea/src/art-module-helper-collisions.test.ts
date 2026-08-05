import test from "node:test";
import assert from "node:assert/strict";

import { generateArtModule } from "./art-module.ts";
import { compileVariantSfc } from "./art-variant-sfc.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("generateArtModule avoids Vue helper collisions with script setup imports", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/StoryResult.art.vue",
    metadata: {
      title: "Story Result",
      component: "./StoryResult.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "Default",
        template: `<StoryResult :result="renderStoryResult(result)" />`,
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { computed, h, isVNode, defineComponent } from "vue";

const result = computed(() => StoryResult);

function renderStoryResult(value) {
  return isVNode(value) ? value : h(value);
}
`.trim(),
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  assert.match(code, /import \{ computed, h, isVNode, defineComponent \} from "vue";/);
  assert.match(code, /import Default from "virtual:musea-variant:[^"]*:Default"/);

  // The art file imports `defineComponent` and `h` itself. The variant compiles
  // through the SFC pipeline, which aliases its own runtime helpers, so the two
  // sets cannot collide into a redeclaration.
  const variant = compileVariantSfc(art, art.variants[0].template, "Default", art.path);
  assert.deepEqual(variant.errors, []);
  // The art file's own bindings survive, and the compiler's runtime helpers are
  // aliased (`createVNode as _createVNode`), so the two sets share no name.
  assert.match(variant.code, /import \{ computed, h, isVNode \} from "vue"/);
  assert.match(variant.code, /createVNode as _createVNode/);
  assert.match(variant.code, /isVNode\(value\) \? value : h\(value\)/);
});
