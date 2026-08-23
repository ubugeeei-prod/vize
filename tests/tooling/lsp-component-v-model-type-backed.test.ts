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

const source = `<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'

const model = ref('hello')
</script>

<template>
  <Child v-model="model" v-model:title="model" />
</template>
`;

const childSource = `<script setup lang="ts">
const value = defineModel<string>({ required: true })
const title = defineModel<string>('title', { required: true })
void value
void title
</script>

<template>{{ value }} {{ title }}</template>
`;

test("component v-model hover and definition use the child model contract", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for component v-model hover",
    "tsgo binary not found; skipping component v-model hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-component-v-model-type-backed");
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

    const childPath = path.join(workspaceDir, "src/Child.vue");
    const childUri = pathToFileURL(childPath).href;
    fs.writeFileSync(childPath, childSource, "utf8");
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
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );

    const defaultModel = rangeFor("v-model", source.indexOf("v-model="));
    const namedModel = rangeFor("title", source.indexOf("v-model:title"));
    const childDefaultModel = rangeForIn(
      childSource,
      "defineModel",
      childSource.indexOf("defineModel<string>({"),
    );
    const childNamedModel = rangeForIn(childSource, "title", childSource.indexOf("'title'"));

    const defaultHover = await hoverAt(session, uri, defaultModel.start);
    assert.deepEqual(
      defaultHover?.range,
      defaultModel,
      "argument-less component v-model hover must select the authored directive",
    );
    const defaultHoverText = hoverToText(defaultHover);
    assert.match(defaultHoverText, /modelValue/);
    assert.match(defaultHoverText, /string/);
    assertNoDirectiveFallback(defaultHoverText);
    await assertDefinitionUriAndRange(
      session,
      uri,
      defaultModel.start,
      childUri,
      childDefaultModel,
      "argument-less component v-model",
    );

    const namedHover = await hoverAt(session, uri, namedModel.start);
    assert.deepEqual(
      namedHover?.range,
      namedModel,
      "named component v-model hover must select the authored model argument",
    );
    const namedHoverText = hoverToText(namedHover);
    assert.match(namedHoverText, /title/);
    assert.match(namedHoverText, /string/);
    assertNoDirectiveFallback(namedHoverText);
    await assertDefinitionUriAndRange(
      session,
      uri,
      namedModel.start,
      childUri,
      childNamedModel,
      "named component v-model",
    );
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
    {
      textDocument: { uri },
      position,
    },
    120_000,
  )) as Hover;
}

async function assertDefinitionUriAndRange(
  session: LspSession,
  uri: string,
  position: Position,
  expectedUri: string,
  expectedRange: Range,
  label: string,
): Promise<void> {
  const response = await session.request(
    "textDocument/definition",
    {
      textDocument: { uri },
      position,
    },
    120_000,
  );
  assert.deepEqual(
    firstLocation(response as Parameters<typeof firstLocation>[0]),
    {
      range: expectedRange,
      uri: expectedUri,
    },
    `${label} definition must jump to the child defineModel declaration`,
  );
}

function rangeFor(symbol: string, nearOffset: number): Range {
  return rangeForIn(source, symbol, nearOffset);
}

function rangeForIn(document: string, symbol: string, nearOffset: number): Range {
  const startOffset = document.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    start: offsetToPosition(document, startOffset),
    end: offsetToPosition(document, startOffset + symbol.length),
  };
}

function assertNoDirectiveFallback(hoverText: string): void {
  assert.notEqual(hoverText.trim(), "");
  assert.doesNotMatch(hoverText, /Vue directive/);
  assert.doesNotMatch(hoverText, /Template expression/);
  assert.doesNotMatch(hoverText, /MaybeRef<unknown>/);
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
  assert.ok(vuePackage, "Vue package is required for type-backed hover test");

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
