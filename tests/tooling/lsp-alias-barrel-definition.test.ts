import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

const BARREL = `export { WidgetA } from "./WidgetA.vue";
export { WidgetB } from "./WidgetB";
export { WidgetC as RenamedWidget } from "./WidgetC";
`;

const WIDGET_A = `<script setup lang="ts">
defineProps<{ label: string }>();
</script>
<template><b>{{ label }}</b></template>
`;

const WIDGET_B = `import { defineComponent } from "vue";

export const WidgetB = defineComponent({});
`;

const WIDGET_C = `import { defineComponent } from "vue";

export const WidgetC = defineComponent({});
`;

const APP = `<script setup lang="ts">
import { WidgetA, WidgetB, RenamedWidget as LocalWidget } from "@/comps";
</script>
<template>
  <WidgetA label="x" />
  <WidgetB />
  <LocalWidget />
</template>
`;

// Definition on a component tag imported through a tsconfig `paths` alias and
// a directory barrel (#3932): the manual finder resolved only relative
// specifiers, so these tags answered null while hover was typed. reka-ui's
// `import { Primitive } from '@/Primitive'` is this shape. No type checker is
// required — the import-follow path is manual.
test("definition on an alias-barrel component tag reaches the source", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-alias-barrel-definition");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  try {
    const compsDir = path.join(workspaceDir, "src/comps");
    fs.mkdirSync(compsDir, { recursive: true });
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        include: ["src/**/*"],
        compilerOptions: {
          strict: true,
          paths: { "@/*": ["./src/*"] },
        },
      }),
      "utf8",
    );
    fs.writeFileSync(path.join(compsDir, "index.ts"), BARREL, "utf8");
    fs.writeFileSync(path.join(compsDir, "WidgetA.vue"), WIDGET_A, "utf8");
    fs.writeFileSync(path.join(compsDir, "WidgetB.ts"), WIDGET_B, "utf8");
    fs.writeFileSync(path.join(compsDir, "WidgetC.ts"), WIDGET_C, "utf8");
    const appPath = path.join(workspaceDir, "src/App.vue");
    fs.writeFileSync(appPath, APP, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      typecheck: false,
      lint: false,
      autoInsert: false,
    });
    const appUri = pathToFileURL(appPath).href;
    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: APP },
    });

    const definitionAt = async (line: number, character: number) =>
      (await session.request("textDocument/definition", {
        textDocument: { uri: appUri },
        position: { line, character },
      })) as { uri: string; range: { start: { line: number } } } | null;

    // `<WidgetA` on line 4 resolves through the barrel to the .vue source.
    const vueTarget = await definitionAt(4, 4);
    assert.ok(vueTarget, "the .vue barrel tag must answer a definition");
    assert.ok(
      vueTarget.uri.endsWith("src/comps/WidgetA.vue"),
      `expected WidgetA.vue, got ${vueTarget.uri}`,
    );

    // `<WidgetB` on line 5 follows the extensionless re-export hop to the
    // defineComponent declaration inside the .ts module (the reka-ui shape).
    const tsTarget = await definitionAt(5, 4);
    assert.ok(tsTarget, "the .ts barrel tag must answer a definition");
    assert.ok(
      tsTarget.uri.endsWith("src/comps/WidgetB.ts"),
      `expected WidgetB.ts, got ${tsTarget.uri}`,
    );
    assert.equal(
      tsTarget.range.start.line,
      2,
      "the jump lands on the exported declaration, not the file head",
    );

    // `<LocalWidget` on line 6 is aliased twice — `RenamedWidget as
    // LocalWidget` at the import site, `WidgetC as RenamedWidget` in the
    // barrel — so both hops must carry the source name, not the local alias.
    const aliasTarget = await definitionAt(6, 4);
    assert.ok(aliasTarget, "the aliased barrel tag must answer a definition");
    assert.ok(
      aliasTarget.uri.endsWith("src/comps/WidgetC.ts"),
      `expected WidgetC.ts, got ${aliasTarget.uri}`,
    );
    assert.equal(
      aliasTarget.range.start.line,
      2,
      "the aliased jump lands on the exported declaration",
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
