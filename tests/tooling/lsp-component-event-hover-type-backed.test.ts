import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

const appSource = `<script setup lang="ts">
import Child from './Child.vue'

function save(value: string) {
  return value
}
</script>

<template>
  <Child @save="save" />
</template>
`;

const childSource = `<script setup lang="ts">
defineEmits<{ save: [value: string] }>()
</script>

<template><button /></template>
`;

test("component event hovers and definitions use child emit contracts", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for component event hover",
    "tsgo binary not found; skipping component event hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-component-event-hover");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let initialized = false;

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
    linkVuePackage(workspaceDir);
    writeProjectConfig(workspaceDir, corsaPath);

    const childPath = path.join(workspaceDir, "src/Child.vue");
    const childUri = pathToFileURL(childPath).href;
    fs.writeFileSync(childPath, childSource, "utf8");
    const appPath = path.join(workspaceDir, "src/App.vue");
    const appUri = pathToFileURL(appPath).href;
    fs.writeFileSync(appPath, appSource, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    initialized = true;
    session.notify("textDocument/didOpen", {
      textDocument: { languageId: "vue", text: appSource, uri: appUri, version: 1 },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, appUri),
      120_000,
    );

    const saveEvent = rangeFor(appSource, "save", appSource.indexOf("@save"));
    const childSaveEvent = rangeFor(childSource, "save", childSource.indexOf("defineEmits"));
    const hover = await hoverAt(session, appUri, saveEvent.start);
    assert.deepEqual(hover?.range, saveEvent);
    const hoverText = hoverToText(hover);
    assert.match(hoverText, /Component event/);
    assert.match(hoverText, /@save/);
    assert.match(hoverText, /\[value: string\]/);
    assert.doesNotMatch(hoverText, /_Script binding_|_Template binding|MaybeRef<unknown>/);
    await assertDefinition(session, appUri, saveEvent.start, childUri, childSaveEvent);
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

function writeProjectConfig(workspaceDir: string, corsaPath: string): void {
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    JSON.stringify({
      lsp: { hover: true, lint: false, typecheck: true },
      typeChecker: { corsaPath },
    }),
    "utf8",
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
    "utf8",
  );
}

async function hoverAt(session: LspSession, uri: string, position: Position): Promise<Hover> {
  return (await session.request(
    "textDocument/hover",
    { position, textDocument: { uri } },
    120_000,
  )) as Hover;
}

async function assertDefinition(
  session: LspSession,
  uri: string,
  position: Position,
  expectedUri: string,
  expectedRange: Range,
): Promise<void> {
  const response = await session.request(
    "textDocument/definition",
    { position, textDocument: { uri } },
    120_000,
  );
  assert.deepEqual(firstLocation(response as Parameters<typeof firstLocation>[0]), {
    range: expectedRange,
    uri: expectedUri,
  });
}

function rangeFor(document: string, symbol: string, nearOffset: number): Range {
  const startOffset = document.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    end: offsetToPosition(document, startOffset + symbol.length),
    start: offsetToPosition(document, startOffset),
  };
}

function resolveTsgoBinary(): string | undefined {
  const candidates = [
    process.env.CORSA_BIN,
    path.join(root, "../corsa-bind/.cache/tsgo"),
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ].filter((candidate): candidate is string => candidate != null && candidate.length > 0);
  return candidates.find((candidate) => fs.existsSync(candidate));
}

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for component event hover test");

  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  symlink(vuePackage, path.join(nodeModules, "vue"));

  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlink(vueNamespace, path.join(nodeModules, "@vue"));
  }
}

function symlink(source: string, target: string): void {
  fs.rmSync(target, { force: true, recursive: true });
  fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
}
