import test from "node:test";
import assert from "node:assert/strict";

import { generateArtModule } from "./art-module.ts";
import { buildVariantSfcSource } from "./art-variant-sfc.ts";
import { generateSharedSetupModule } from "./art-shared-setup.ts";
import type { ArtFileInfo } from "./types/art.ts";

// Behaviour that moved from the art module into the per-variant SFCs (#3857):
// component resolution, setup isolation, and the shared-setup opt-out.

void test("generateArtModule treats defineArt as a compiler macro and isolates setup by variant", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Button.art.vue",
    metadata: {
      title: "Button",
      component: "./Button.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      { name: "Primary", template: `<Button :count="count" />`, isDefault: true, skipVrt: false },
      {
        name: "Secondary",
        template: `<Button :count="count" />`,
        isDefault: false,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { ref } from "vue"

defineArt("./Button.vue", {
  title: "Button",
})

const count = ref(0)
`.trim(),
    scriptSetupIsolated: true,
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  assert.match(code, /import Button from "\/repo\/components\/Button\.vue";/);
  assert.doesNotMatch(code, /\bdefineArt\s*\(/);
  // Each variant is its own module, so isolation is structural rather than a
  // per-variant copy of the setup body.
  assert.match(code, /import Primary from "virtual:musea-variant:[^"]*:Primary"/);
  assert.match(code, /import Secondary from "virtual:musea-variant:[^"]*:Secondary"/);

  for (const variantName of ["Primary", "Secondary"]) {
    const variant = buildVariantSfcSource(art, `<Button :count="count" />`, variantName, {
      artFilePath: art.path,
      componentImportPath: "/repo/components/Button.vue",
      componentBindingName: "Button",
    });
    assert.match(variant, /const count = ref\(0\)/, variantName);
    assert.doesNotMatch(variant, /\bdefineArt\s*\(/, `${variantName} keeps the macro out`);
    assert.match(variant, /import Button from "\/repo\/components\/Button\.vue"/, variantName);
  }
});

void test("generateArtModule does not register local PascalCase constants as components", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Foo.art.vue",
    metadata: {
      title: "Foo",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "Default",
        template: `<Foo />`,
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
defineArt("./Foo.vue", { title: "Foo" });
const SAMPLE = "hello";
`.trim(),
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  // `<script setup>` resolves `<Foo />` from its imported binding, so a local
  // PascalCase constant can no longer be mistaken for a component at all.
  const variant = buildVariantSfcSource(art, `<Foo />`, "Default", {
    artFilePath: art.path,
    componentImportPath: "/repo/components/Foo.vue",
    componentBindingName: "Foo",
  });
  assert.match(variant, /import Foo from "\/repo\/components\/Foo\.vue"/);
  assert.match(variant, /const SAMPLE = "hello"/);
  assert.doesNotMatch(variant, /"SAMPLE"\s*:/);
  assert.doesNotMatch(code, /\bdefineArt\s*\(/);
});

void test("generateArtModule shares setup when script setup isolate is false", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Button.art.vue",
    metadata: {
      title: "Button",
      component: "./Button.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      { name: "Primary", template: `<Button :count="count" />`, isDefault: true, skipVrt: false },
      {
        name: "Secondary",
        template: `<Button :count="count" />`,
        isDefault: false,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { ref } from "vue"
import Button from "./Button.vue"
const count = ref(0)
`.trim(),
    scriptSetupIsolated: false,
    hasScript: false,
    styleCount: 0,
  };

  // Opting out of isolation must still mean one setup instance. Variants import
  // the shared module's bindings instead of each re-declaring them, which would
  // give every variant its own `count`.
  const shared = generateSharedSetupModule(art, art.path);
  assert.match(shared, /const count = ref\(0\)/);
  assert.match(shared, /export \{[^}]*\bcount\b/);
  assert.equal((shared.match(/const count = ref\(0\)/g) ?? []).length, 1);

  const variant = buildVariantSfcSource(art, `<Button :count="count" />`, "Primary", {
    artFilePath: art.path,
    sharedBindings: {
      moduleId: `virtual:musea-shared:${art.path}`,
      names: ["Button", "count"],
    },
  });
  assert.match(variant, /import \{ Button, count \} from "virtual:musea-shared:/);
  assert.doesNotMatch(variant, /const count = ref\(0\)/, "the variant must not re-declare it");
});

void test("a shared variant imports the demonstrated component when setup does not", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Button.art.vue",
    metadata: {
      title: "Button",
      // Named through metadata rather than imported in `<script setup>`, so the
      // component is not one of the shared bindings.
      component: "./Button.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      { name: "Primary", template: `<Button :count="count" />`, isDefault: true, skipVrt: false },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { ref } from "vue"
const count = ref(0)
`.trim(),
    scriptSetupIsolated: false,
    hasScript: false,
    styleCount: 0,
  };

  const sharedBindings = {
    moduleId: `virtual:musea-shared:${art.path}`,
    names: ["count"],
  };

  // `<Self>` expands to `<Button>`, so the variant needs that binding even
  // though the shared module does not export it.
  const variant = buildVariantSfcSource(art, `<Button :count="count" />`, "Primary", {
    artFilePath: art.path,
    componentImportPath: "/repo/components/Button.vue",
    componentBindingName: "Button",
    sharedBindings,
  });
  assert.match(variant, /import Button from "\/repo\/components\/Button\.vue"/);
  assert.match(variant, /import \{ count \} from "virtual:musea-shared:/);

  // When the shared module does export it, importing it again would redeclare
  // the binding.
  const shared = buildVariantSfcSource(art, `<Button :count="count" />`, "Primary", {
    artFilePath: art.path,
    componentImportPath: "/repo/components/Button.vue",
    componentBindingName: "Button",
    sharedBindings: { ...sharedBindings, names: ["Button", "count"] },
  });
  assert.doesNotMatch(shared, /import Button from "\/repo\/components\/Button\.vue"/);
  assert.match(shared, /import \{ Button, count \} from "virtual:musea-shared:/);
});
