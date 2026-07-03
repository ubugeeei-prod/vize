import assert from "node:assert/strict";
import test from "node:test";

import { generatePreviewModule, generatePreviewModuleWithProps } from "./preview/index.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("generatePreviewModule emits Vue 2 preview runtime when requested", () => {
  const art = createBasicArtFile("/repo/components/MfCard.art.vue");

  const code = generatePreviewModule(art, "Default", "default", [], null, 2);

  assert.doesNotMatch(code, /\bcreateApp\b/);
  assert.match(code, /import Vue, \{ reactive \} from 'vue';/);
  assert.match(code, /new Vue\(\{ render: \(h\) => h\(VariantComponent\) \}\)/);
  assert.match(code, /currentApp\.\$destroy\(\);/);
});

void test("generatePreviewModuleWithProps emits Vue 2 prop binding when requested", () => {
  const art = createBasicArtFile("/repo/components/MfCard.art.vue");

  const code = generatePreviewModuleWithProps(
    art,
    "Default",
    "default",
    { label: "OK" },
    [],
    null,
    2,
  );

  assert.doesNotMatch(code, /\bcreateApp\b/);
  assert.match(code, /import Vue from 'vue';/);
  assert.match(code, /return h\(VariantComponent, \{ props: propsOverride \}\);/);
  assert.match(code, /app\.\$mount\(mountPoint\);/);
});

function createBasicArtFile(path: string): ArtFileInfo {
  return {
    path,
    metadata: {
      title: "Card",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "default",
        template: "<div />",
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}
