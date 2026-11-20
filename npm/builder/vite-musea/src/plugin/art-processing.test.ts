import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { artScopeAttribute } from "../art-style.js";
import {
  extractDefineArtMeta,
  extractTargetInputs,
  extractVariantRuntimeOptions,
  processMuseaArtFile,
  reportArtStatusWarnings,
  reportDefineArtMetaWarnings,
} from "./art-processing.js";

void test("target inputs seed props and attrs from the authored component", () => {
  assert.deepEqual(
    extractTargetInputs(
      `<Counter :initial="0" step="2" disabled aria-label="Counter example" data-state="ready" />`,
      "Counter",
    ),
    {
      props: { initial: 0, step: "2", disabled: true },
      attrs: { "aria-label": "Counter example", "data-state": "ready" },
    },
  );
  assert.deepEqual(extractTargetInputs(`<Counter :initial="count" />`, "Counter"), {});
  assert.deepEqual(extractTargetInputs(`<Counter /><Counter />`, "Counter"), {});
});

void test("variant runtime attributes retain inline play, props, attrs and viewport", () => {
  assert.deepEqual(
    extractVariantRuntimeOptions(`<art title="Button">
      <variant name="Primary" :play="async ({ canvas }) => canvas.getByText('Save')" :props="{ label: 'Save', count: 2, enabled: true }" :attrs="{ 'aria-label': 'Save button', inert: false }" viewport="375x667@2"><button /></variant>
      <variant name="No test"><button /></variant>
    </art>`),
    [
      {
        name: "Primary",
        play: "async ({ canvas }) => canvas.getByText('Save')",
        props: { label: "Save", count: 2, enabled: true },
        attrs: { "aria-label": "Save button", inert: false },
        viewport: { width: 375, height: 667, deviceScaleFactor: 2 },
      },
      { name: "No test" },
    ],
  );
});

void test("variant props and attrs reject executable discovery expressions", () => {
  assert.throws(
    () =>
      extractVariantRuntimeOptions(
        `<art><variant name="Unsafe" :props="makeProps()"><button /></variant></art>`,
      ),
    /:props must be a statically serializable object literal/,
  );
  assert.throws(
    () =>
      extractVariantRuntimeOptions(
        `<art><variant name="Unsafe" attrs="[]"><button /></variant></art>`,
      ),
    /attrs must be a JSON object/,
  );
});

void test("defineArt metadata preserves nested parameters from both public macros", () => {
  const parameters = {
    design: { type: "figma", url: "https://www.figma.com/design/example" },
  };
  assert.deepEqual(
    extractDefineArtMeta(
      `defineArtMeta({ title: "Button", parameters: ${JSON.stringify(parameters)} })`,
    ),
    { title: "Button", parameters },
  );
  assert.deepEqual(
    extractDefineArtMeta(`defineArt(Button, { parameters: ${JSON.stringify(parameters)} })`),
    { parameters },
  );
  assert.deepEqual(
    extractDefineArtMeta(
      `defineArtMeta({ parameters: ((({ design: (({ type: ("figma"), url: ("https://www.figma.com/design/example") })) }))) })`,
    ),
    { parameters },
  );
  assert.deepEqual(extractDefineArtMeta(`defineArtMeta({ parameters: createParameters() })`), {});
});

void test("defineArt metadata warns about unknown keys and statuses", () => {
  const messages: string[] = [];
  reportDefineArtMetaWarnings(
    { status: "wip", figma: "1234:5678" } as never,
    "Button.art.vue",
    (message) => messages.push(message),
  );
  assert.deepEqual(messages, [
    '[musea] Button.art.vue: unknown defineArt metadata key "figma"; ignoring it',
    '[musea] Button.art.vue: unknown status "wip"; ignoring it (expected "draft" | "ready" | "deprecated")',
  ]);
});

void test("processMuseaArtFile infers a missing title from the art filename", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-bad-art-"));
  const artPath = path.join(tempDir, "stories", "Broken.art.vue");

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(
      artPath,
      '<!-- <art status="deprecated"><variant name="Old"><div /></variant></art> -->\n<art status="ready"><variant name="Default"><div /></variant></art>',
      "utf8",
    );

    const art = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.equal(art?.metadata.title, "Broken");
    assert.equal(art?.metadata.status, "ready");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile ignores comment markers in scripts and reads unquoted status", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-active-art-"));
  const artPath = path.join(tempDir, "stories", "Active.art.vue");

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(
      artPath,
      '<!-- <script> is text -->\n<script setup>const marker = "<!--"</script>\n<art status=ready><variant name="Default"><div /></variant></art>\n<!-- inactive -->',
      "utf8",
    );

    const art = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.equal(art?.metadata.title, "Active");
    assert.equal(art?.metadata.status, "ready");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile infers the target and reads optional defineArtMeta", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-inferred-art-"));
  const artPath = path.join(tempDir, "Counter.art.vue");
  const componentPath = path.join(tempDir, "Counter.vue");
  try {
    await fs.promises.writeFile(componentPath, "<template><button /></template>", "utf8");
    await fs.promises.writeFile(
      artPath,
      `<script setup lang="ts">
import Counter from "./Counter.vue"
defineArtMeta({ title: "Interactive Counter", category: "Input" })
</script>
<art>
  <variant name="Default" default>
    <Counter :initial="0" step="2" aria-label="Counter example" />
  </variant>
</art>`,
      "utf8",
    );
    const art = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.ok(art);
    assert.equal(art.metadata.title, "Interactive Counter");
    assert.equal(art.metadata.category, "Input");
    assert.equal(art.metadata.component, "./Counter.vue");
    assert.deepEqual(art.variants[0].props, { initial: 0, step: "2" });
    assert.deepEqual(art.variants[0].attrs, { "aria-label": "Counter example" });
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile scopes art style blocks onto variant elements", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scoped-art-"));
  const artPath = path.join(tempDir, "my-card.art.vue");

  try {
    await fs.promises.writeFile(
      artPath,
      `<art title="MyCard">
  <variant name="Default" default>
    <div class="wrap"><span>content</span></div>
  </variant>
</art>
<style scoped>
.wrap { padding: 16px; }
</style>
`,
      "utf8",
    );

    const info = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.ok(info);
    const scopeAttr = artScopeAttribute(artPath);
    assert.match(info.variants[0]?.template ?? "", new RegExp(`<div class="wrap" ${scopeAttr}>`));
    assert.match(info.variants[0]?.template ?? "", new RegExp(`<span ${scopeAttr}>`));
    assert.equal(info.styleBlocks?.[0], ".wrap { padding: 16px; }");
    assert.deepEqual(info.styleBlockDetails?.[0], {
      content: ".wrap { padding: 16px; }",
      lang: "css",
      scoped: true,
    });
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile keeps arrow functions intact under a scoped style (#9752)", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scoped-arrow-"));
  const artPath = path.join(tempDir, "WithScopedStyle.art.vue");

  try {
    await fs.promises.writeFile(
      artPath,
      `<script setup lang="ts">
import PressButton from "./PressButton.vue";
defineArt(PressButton, { title: "With scoped style" });
</script>

<style scoped>
button {
  padding: 8px;
}
</style>

<art title="A > B" action-events="press">
  <variant default name="Arrow">
    <PressButton @press="() => {}" />
  </variant>
</art>
`,
      "utf8",
    );

    const info = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.ok(info);
    const scopeAttr = artScopeAttribute(artPath);
    assert.equal(
      info.variants[0]?.template.trim(),
      `<PressButton @press="() => {}" ${scopeAttr} />`,
    );
    assert.deepEqual(info.metadata.actionEvents, ["press"]);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile keeps dev processing non-fatal", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-dev-bad-art-"));
  const artPath = path.join(tempDir, "stories", "Broken.art.vue");
  const messages: string[] = [];

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(artPath, "<art>", "utf8");

    const info = await processMuseaArtFile(artPath, {
      root: tempDir,
      command: "serve",
      onError: (message) => messages.push(message),
    });

    assert.equal(info, null);
    assert.equal(messages.length, 1);
    assert.match(messages[0], /\[musea\] Failed to process stories\/Broken\.art\.vue/);
    assert.match(messages[0], /No <art> block found in file/);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("reportArtStatusWarnings prints native unknown-status diagnostics", () => {
  const messages: string[] = [];
  reportArtStatusWarnings(
    {
      parseArtStatusWarnings: (source, options) => {
        assert.equal(source, '<art title="Button" status="wip"></art>');
        assert.equal(options?.filename, "button.art.vue");
        return [
          'button.art.vue: unknown status "wip"; falling back to "draft" (expected "draft" | "ready" | "deprecated")',
        ];
      },
    },
    '<art title="Button" status="wip"></art>',
    "button.art.vue",
    (message) => messages.push(message),
  );
  assert.deepEqual(messages, [
    '[musea] button.art.vue: unknown status "wip"; falling back to "draft" (expected "draft" | "ready" | "deprecated")',
  ]);
});

void test("reportArtStatusWarnings is a no-op when the native export is missing", () => {
  const messages: string[] = [];
  reportArtStatusWarnings({}, "<art></art>", "button.art.vue", (message) => messages.push(message));
  assert.deepEqual(messages, []);
});

void test("inline Art retains the native custom-block index after another block", async () => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-inline-index-"));
  const file = path.join(root, "Card.vue");
  try {
    await fs.promises.writeFile(
      file,
      `<script setup>const label = '<art title="script text" />';</script>
<template><div>{{ label }}</div></template>
<docs>Owned by another plugin.</docs>
<art title="Registered" status="ready"><variant name="Default"><Card /></variant></art>`,
    );
    const art = await processMuseaArtFile(file, { root, command: "build" });
    assert.ok(art);
    assert.deepEqual(
      {
        isInline: art.isInline,
        componentPath: art.componentPath,
        inlineArtBlockIndex: art.inlineArtBlockIndex,
        metadata: art.metadata,
        variants: art.variants,
      },
      {
        isInline: true,
        componentPath: file,
        inlineArtBlockIndex: 1,
        metadata: {
          title: "Registered",
          description: undefined,
          component: undefined,
          category: undefined,
          tags: [],
          status: "ready",
          order: undefined,
          parameters: undefined,
          actionEvents: undefined,
        },
        variants: [{ name: "Default", template: "<Card />", isDefault: false, skipVrt: false }],
      },
    );
  } finally {
    await fs.promises.rm(root, { recursive: true, force: true });
  }
});
