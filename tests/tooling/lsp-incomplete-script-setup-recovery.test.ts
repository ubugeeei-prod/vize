import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const APP = `<script setup lang="ts">
import { ref } from "vue";
import MyButton from "./MyButton.vue";
import type { Item } from "./types";

const items = ref<Item[]>([]);
const selected = ref<Item | null>(null);

function pick(item: Item): void {
  selected.value = item;
}
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.id">
      <MyButton :label="item.label" @click="pick(item)" />
    </li>
  </ul>
</template>
`;

const BUTTON = `<script setup lang="ts">
defineProps<{ label: string }>();
</script>

<template><button type="button">{{ label }}</button></template>
`;

test("incomplete script setup statements keep relative imports and native syntax diagnostics", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7/Corsa runtime for incomplete script setup recovery",
    "TypeScript 7/Corsa runtime not found; skipping incomplete script setup recovery test",
  );
  if (corsaPath == null) return;

  const output = path.join(testOutputRoot, "lsp-incomplete-script-setup-recovery");
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(output, "workspace-"));
  const session = new LspSession();

  try {
    const source = path.join(workspace, "src");
    fs.mkdirSync(source, { recursive: true });
    linkVuePackage(workspace);
    writeVueConfigFixture(workspace, {
      lsp: { editor: true, lint: false, typecheck: true },
      typeChecker: { corsaPath },
    });
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      }),
    );
    fs.writeFileSync(path.join(source, "MyButton.vue"), BUTTON);
    fs.writeFileSync(
      path.join(source, "types.ts"),
      "export interface Item { id: number; label: string }\n",
    );
    const appPath = path.join(source, "App.vue");
    const appUri = pathToFileURL(appPath).href;
    fs.writeFileSync(appPath, APP);

    await session.initialize(workspace, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: APP },
    });
    assert.deepEqual((await publication(session, appUri, 1)).diagnostics, []);

    const cases = [
      ["items.value.\n", "const selected", [1003]],
      ["selected.value.\n", "function pick", [1003, 18047]],
      ["pick(\n", "function pick", [1005, 2304, 2339]],
      ["const next =\n", "function pick", [2339]],
      ["void selected.value;\n", "function pick", []],
    ] as const;
    let version = 1;
    for (const [inserted, before, expectedCodes] of cases) {
      const changed = APP.replace(before, `${inserted}${before}`);
      version++;
      session.notify("textDocument/didChange", {
        textDocument: { uri: appUri, version },
        contentChanges: [{ text: changed }],
      });
      const diagnostics = (await publication(session, appUri, version)).diagnostics;
      assert.deepEqual(
        codes(diagnostics),
        expectedCodes,
        `${inserted}\n${JSON.stringify(diagnostics, null, 2)}\n${session.stderrText}`,
      );
      assert.ok(
        diagnostics.every((diagnostic) => Number(diagnostic.code) !== 2307),
        `${inserted} must keep both relative imports resolved`,
      );

      version++;
      session.notify("textDocument/didChange", {
        textDocument: { uri: appUri, version },
        contentChanges: [{ text: APP }],
      });
      assert.deepEqual((await publication(session, appUri, version)).diagnostics, []);
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

function codes(diagnostics: LspDiagnostic[]): number[] {
  return diagnostics.map((diagnostic) => Number(diagnostic.code)).sort((a, b) => a - b);
}

async function publication(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    45_000,
  )) as PublishDiagnosticsParams;
}

function linkVuePackage(workspace: string): void {
  const vue = [path.join(root, "node_modules/vue"), path.join(root, "tests/node_modules/vue")].find(
    (candidate) => fs.existsSync(candidate),
  );
  assert.ok(vue, "Vue package is required for incomplete script setup recovery");
  const modules = path.join(workspace, "node_modules");
  fs.mkdirSync(modules, { recursive: true });
  fs.symlinkSync(vue, path.join(modules, "vue"), "dir");
  const namespace = path.join(path.dirname(vue), "@vue");
  if (fs.existsSync(namespace)) fs.symlinkSync(namespace, path.join(modules, "@vue"), "dir");
}
