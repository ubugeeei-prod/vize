// The single-file-component contract of the patterned-template reference
// implementation (vuejs/core#15531, `packages/compiler-sfc/__tests__/vMatch.spec.ts`),
// run through the real CLI: a `v-match` on the SFC's own `<template>` block is
// the same match as one nested inside it, its header is part of the template
// for import usage, and an invalid header is one error at the directive.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  buildPatternComponents,
  installDom,
  type PatternBackend,
} from "./support/upstream/pattern-runtime.ts";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";

await installDom();
const { createApp, nextTick } = await import("vue");

const arms = `<p v-when="{ kind: 'ok', const data }">{{ data }}</p><i v-when="{ kind: 'err' }">error</i>`;

for (const backend of ["dom", "ssr", "vapor"] as PatternBackend[]) {
  test(`a root match compiles like the same match in an inner block (${backend})`, () => {
    const build = buildPatternComponents(
      {
        "Root.vue": `<template v-match="result">${arms}</template>\n`,
        "Nested.vue": `<template><template v-match="result">${arms}</template></template>\n`,
      },
      {},
      { backends: [backend] },
    );
    try {
      const compiled = (name: string) =>
        fs.readFileSync(path.join(build.directory, backend, `${name}.js`), "utf8");
      assert.equal(compiled("Root"), compiled("Nested"));
    } finally {
      build.dispose();
    }
  });
}

test("the header expression keeps a TypeScript import alive", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": `<script setup lang="ts">import { result } from "./state.mjs"</script><template v-match="result">${arms}</template>\n`,
    },
    {
      "state.mjs": `import { ref } from "vue";\nexport const result = ref({ kind: "ok", data: "payload" });\n`,
    },
    { backends: ["dom"] },
  );
  try {
    const { default: App } = await build.load<{ default: object }>("dom", "App.js");
    const { result } = await build.load<{ result: { value: object } }>("dom", "state.mjs");
    const root = document.createElement("div");
    const app = createApp(App);
    try {
      app.mount(root);
      assert.equal(root.innerHTML, "<p>payload</p>");
      result.value = { kind: "err" };
      await nextTick();
      assert.equal(root.innerHTML, "<i>error</i>");
    } finally {
      app.unmount();
    }
  } finally {
    build.dispose();
  }
});

test("an invalid root header is one error at the directive", async () => {
  const headers = ["v-match", 'v-match=""', 'v-match:arg="x"', 'v-match.foo="x"'];
  const directory = workspace("pattern-sfc-");
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(
      path.join(directory, "vize.config.json"),
      JSON.stringify({ experimentals: { patternedTemplate: true } }),
    );
    // The checker refuses to borrow the repository's own project for this directory.
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          skipLibCheck: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
          types: [],
        },
        include: ["src/**/*"],
      }),
    );
    for (const [index, header] of headers.entries()) {
      fs.writeFileSync(
        path.join(directory, "src", `Header${index}.vue`),
        `<template ${header}><p v-when="_"/></template>\n`,
      );
    }
    const diagnostics = await check(directory);
    assert.deepEqual(
      diagnostics.sort((left, right) => left.file.localeCompare(right.file)),
      headers.map((_, index) => ({
        file: `src/Header${index}.vue`,
        severity: "error",
        line: 1,
        column: "<template ".length + 1,
        message: "v-match requires one subject expression without arguments or modifiers.",
      })),
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
