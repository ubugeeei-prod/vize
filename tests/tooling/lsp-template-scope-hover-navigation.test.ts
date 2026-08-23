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
const items: Array<{ id: string; label: string }> = [{ id: "a", label: "Alpha" }]
const visible = true
</script>

<template>
  <ul v-if="visible">
    <li v-for="(item, index) in items" :key="item.id">
      {{ item.label }} {{ index.toFixed(0) }}
    </li>
  </ul>
</template>
`;

test("template scoped aliases hover and jump to authored anchors", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for template scoped hover/navigation",
    "tsgo binary not found; skipping scoped template hover/navigation test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-template-scope-hover-navigation");
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
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*.vue"],
      }),
      "utf8",
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
    const publish = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );
    assert.deepEqual(publish.diagnostics, []);

    await assertHoverAndDefinition(
      session,
      uri,
      rangeFor("visible", source.indexOf('v-if="visible"')),
      /const visible: true/,
      rangeFor("visible", source.indexOf("visible = true")),
      "v-if condition",
    );
    await assertHoverAndDefinition(
      session,
      uri,
      rangeFor("item", source.indexOf("{{ item.label")),
      /const item: \{\s+id: string;\s+label: string;\s+\}/,
      rangeFor("item", source.indexOf('v-for="(item')),
      "v-for value alias",
    );
    await assertHoverAndDefinition(
      session,
      uri,
      rangeFor("index", source.indexOf("index.toFixed")),
      /const index: number/,
      rangeFor("index", source.indexOf("item, index")),
      "v-for index alias",
    );
    await assertHoverAndDefinition(
      session,
      uri,
      rangeFor("item", source.indexOf(':key="item.id"')),
      /const item: \{\s+id: string;\s+label: string;\s+\}/,
      rangeFor("item", source.indexOf('v-for="(item')),
      "v-for alias in directive expression",
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

async function assertHoverAndDefinition(
  session: LspSession,
  uri: string,
  usage: Range,
  expectedText: RegExp,
  expectedTarget: Range,
  label: string,
): Promise<void> {
  const hover = await session.request(
    "textDocument/hover",
    {
      position: usage.start,
      textDocument: { uri },
    },
    120_000,
  );
  assert.deepEqual((hover as { range?: Range } | null)?.range, usage, `${label} hover range`);
  const hoverText = hoverToText(hover as Parameters<typeof hoverToText>[0]);
  assert.match(hoverText, /^```typescript\n/);
  assert.match(hoverText, expectedText);
  assert.doesNotMatch(hoverText, /Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>/);
  assert.doesNotMatch(hoverText, /_Template binding_|_Template expression_/);

  const definition = await session.request(
    "textDocument/definition",
    {
      position: usage.start,
      textDocument: { uri },
    },
    120_000,
  );
  assert.deepEqual(firstLocation(definition as Parameters<typeof firstLocation>[0]), {
    range: expectedTarget,
    uri,
  });
}

function rangeFor(symbol: string, nearOffset: number): Range {
  const startOffset = source.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    end: offsetToPosition(source, startOffset + symbol.length),
    start: offsetToPosition(source, startOffset),
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
  assert.ok(vuePackage, "Vue package is required for scoped template hover/navigation test");
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), dirLinkType());
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), dirLinkType());
  }
}

function dirLinkType(): fs.symlink.Type {
  return process.platform === "win32" ? "junction" : "dir";
}
