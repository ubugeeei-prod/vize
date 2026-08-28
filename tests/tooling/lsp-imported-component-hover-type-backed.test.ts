import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const source = `<script setup lang="ts">
import Child from './Child.vue'

Child
</script>

<template>
  <Child label="ready" />
</template>
`;

const childSource = `<script setup lang="ts">
defineProps<{ label: string; count?: number }>()
defineEmits<{ save: [value: string] }>()
defineSlots<{ default(props: { value: string }): unknown }>()
defineModel<boolean>()
</script>

<template><slot value="ready" /></template>
`;

test("script hover describes imported SFC contracts instead of generated markers", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for imported SFC hover",
    "TypeScript 7/Corsa runtime not found; skipping imported SFC hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-imported-component-hover");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let initialized = false;

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
    linkVuePackage(workspaceDir);
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({
        lsp: { hover: true, lint: false, typecheck: true },
        typeChecker: { corsaPath },
      }),
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*.vue"],
      }),
    );

    fs.writeFileSync(path.join(workspaceDir, "src/Child.vue"), childSource, "utf8");
    const filePath = path.join(workspaceDir, "src/App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");
    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    initialized = true;
    session.notify("textDocument/didOpen", {
      textDocument: { languageId: "vue", text: source, uri, version: 1 },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );

    const importBinding = rangeFor("Child", source.indexOf("import Child"));
    const scriptUsage = rangeFor("Child", source.indexOf("\nChild\n"));

    const importHover = await hoverAt(session, uri, importBinding.start);
    assert.deepEqual(
      importHover?.range,
      importBinding,
      "imported component hover must select the authored import binding",
    );
    assertImportedComponentHover(hoverToText(importHover));

    const usageHover = await hoverAt(session, uri, scriptUsage.start);
    assert.deepEqual(
      usageHover?.range,
      scriptUsage,
      "imported component usage hover must select the authored script token",
    );
    assertImportedComponentHover(hoverToText(usageHover));
  } finally {
    if (initialized) {
      await session.shutdown();
    } else {
      await session.kill().catch(() => undefined);
    }
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

type Position = { line: number; character: number };

type Range = { start: Position; end: Position };

type Hover = {
  contents?: unknown;
  range?: Range;
} | null;

async function hoverAt(session: LspSession, uri: string, position: Position): Promise<Hover> {
  return (await session.request(
    "textDocument/hover",
    { position, textDocument: { uri } },
    120_000,
  )) as Hover;
}

function rangeFor(symbol: string, nearOffset: number): Range {
  const startOffset = source.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    end: offsetToPosition(source, startOffset + symbol.length),
    start: offsetToPosition(source, startOffset),
  };
}

function assertImportedComponentHover(hoverText: string): void {
  assert.match(hoverText, /^```typescript\n/);
  assert.match(hoverText, /const Child: VueComponent/);
  assert.match(hoverText, /props: \{ label: string; count\?: number \};/);
  assert.match(hoverText, /emits: \{ save: \[value: string\] \};/);
  assert.match(hoverText, /slots: \{ default\(props: \{ value: string \}\): unknown \};/);
  assert.match(hoverText, /model: "modelValue": boolean;/);
  assert.doesNotMatch(hoverText, /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/);
}

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for imported component hover test");

  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  symlink(vuePackage, path.join(nodeModules, "vue"));

  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlink(vueNamespace, path.join(nodeModules, "@vue"));
  }
}

function symlink(sourcePath: string, targetPath: string): void {
  fs.rmSync(targetPath, { force: true, recursive: true });
  fs.symlinkSync(sourcePath, targetPath, process.platform === "win32" ? "junction" : "dir");
}
