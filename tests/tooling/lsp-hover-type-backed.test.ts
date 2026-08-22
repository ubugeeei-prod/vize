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
import { computed, ref } from 'vue'

const count = ref(1)
const doubled = computed(() => count.value * 2)
const label: string = 'hello'
</script>

<template>
  <button :title="label">{{ count }} {{ doubled }}</button>
</template>
`;

test("hover and definition answer authored template anchors with backend type text", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for type-backed hover",
    "tsgo binary not found; skipping type-backed hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-hover-type-backed");
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

    const labelDeclaration = rangeFor("label", source.indexOf("label: string"));
    const countDeclaration = rangeFor("count", source.indexOf("count = ref"));
    const labelUsage = rangeFor("label", source.indexOf(':title="label"'));
    const countUsage = rangeFor("count", source.indexOf("{{ count"));

    const scriptHover = await hoverAt(session, uri, labelDeclaration.start);
    const scriptHoverText = hoverToText(scriptHover);
    assert.match(scriptHoverText, /^```typescript\n/);
    assert.match(scriptHoverText, /const label: string/);
    assertNoHeuristic(scriptHoverText);

    const templateLabelHover = await hoverAt(session, uri, labelUsage.start);
    assert.deepEqual(
      templateLabelHover?.range,
      labelUsage,
      "template attribute hover must select the authored token, not generated TS",
    );
    const templateLabelHoverText = hoverToText(templateLabelHover);
    assert.match(templateLabelHoverText, /^```typescript\n/);
    assert.match(templateLabelHoverText, /const label: string/);
    assertNoHeuristic(templateLabelHoverText);
    await assertDefinition(session, uri, labelUsage.start, labelDeclaration, "template label");

    const templateCountHover = await hoverAt(session, uri, countUsage.start);
    assert.deepEqual(
      templateCountHover?.range,
      countUsage,
      "template interpolation hover must select the authored token, not generated TS",
    );
    const templateCountHoverText = hoverToText(templateCountHover);
    assert.match(templateCountHoverText, /^```typescript\n/);
    assert.match(templateCountHoverText, /const count: number/);
    assert.doesNotMatch(templateCountHoverText, /Ref</);
    assertNoHeuristic(templateCountHoverText);
    await assertDefinition(session, uri, countUsage.start, countDeclaration, "template count");
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

async function assertDefinition(
  session: LspSession,
  uri: string,
  position: Position,
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
      uri,
    },
    `${label} definition must jump to the authored script declaration`,
  );
}

function rangeFor(symbol: string, nearOffset: number): Range {
  const startOffset = source.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    start: offsetToPosition(source, startOffset),
    end: offsetToPosition(source, startOffset + symbol.length),
  };
}

function assertNoHeuristic(hoverText: string): void {
  assert.notEqual(hoverText.trim(), "");
  assert.doesNotMatch(hoverText, /_Script binding_|_Template binding/);
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
