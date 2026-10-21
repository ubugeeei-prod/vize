import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const CHILD = `<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template>
  <section>
    <h2>{{ title }}</h2>
    <slot name="item" />
  </section>
</template>
`;

const BROKEN_CHILD = CHILD.replace(
  '    <slot name="item" />',
  '    <slot name="item" />\n    <template #',
);

const PARENT = `<script setup lang="ts">
import Child from "./Child.vue"

const label = "hello"
</script>

<template>
  <Child :title="label" />
</template>
`;

test("an imported SFC block parse error keeps importer diagnostics clean and the Corsa session warm", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7/Corsa runtime for imported SFC recovery",
    "TypeScript 7/Corsa runtime not found; skipping imported SFC recovery test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-imported-sfc-parse-recovery");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
    linkVuePackage(workspaceDir);
    writeVueConfigFixture(
      workspaceDir,
      JSON.stringify({
        lsp: { editor: true, lint: false, typecheck: true },
        typeChecker: { corsaPath },
      }),
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
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
      "utf8",
    );

    const childPath = path.join(sourceDir, "Child.vue");
    const parentPath = path.join(sourceDir, "Parent.vue");
    const childUri = pathToFileURL(childPath).href;
    const parentUri = pathToFileURL(parentPath).href;
    fs.writeFileSync(childPath, CHILD, "utf8");
    fs.writeFileSync(parentPath, PARENT, "utf8");

    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: childUri, languageId: "vue", version: 1, text: CHILD },
    });
    assert.deepEqual((await publication(session, childUri, 1)).diagnostics, []);
    session.notify("textDocument/didOpen", {
      textDocument: { uri: parentUri, languageId: "vue", version: 1, text: PARENT },
    });
    assert.deepEqual((await publication(session, parentUri, 1)).diagnostics, []);

    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 2 },
      contentChanges: [{ text: BROKEN_CHILD }],
    });
    const brokenChild = await publication(session, childUri, 2);
    assert.deepEqual(
      brokenChild.diagnostics.map(({ source, message }) => `${source}:${message}`),
      ["vize/sfc:Malformed <template> block: the closing tag is missing."],
    );

    const importerWhileBroken = await publication(session, parentUri, 1);
    assert.deepEqual(
      importerWhileBroken.diagnostics,
      [],
      `the importer must retain its last good type shape\n${session.stderrText}`,
    );
    assert.doesNotMatch(
      session.stderrText,
      /respawning and retrying|retired after a backend failure/u,
    );

    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 3 },
      contentChanges: [{ text: CHILD }],
    });
    assert.deepEqual((await publication(session, childUri, 3)).diagnostics, []);
    assert.deepEqual((await publication(session, parentUri, 1)).diagnostics, []);
    assert.doesNotMatch(
      session.stderrText,
      /respawning and retrying|retired after a backend failure/u,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

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

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for imported SFC recovery");

  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), "dir");
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), "dir");
  }
}
