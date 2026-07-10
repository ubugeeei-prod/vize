import test from "node:test";
import assert from "node:assert/strict";

import { generateArtModule } from "./art-module.ts";
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

  assert.match(code, /import \{ defineComponent as __museaDefineComponent \} from 'vue';/);
  assert.match(code, /import \{ computed, h, isVNode, defineComponent \} from "vue";/);
  assert.doesNotMatch(code, /import \{ defineComponent, h \} from 'vue';/);
  assert.match(code, /export const Default = __museaDefineComponent\(\{/);
});
