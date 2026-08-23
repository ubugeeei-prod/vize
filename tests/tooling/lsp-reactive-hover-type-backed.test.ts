import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

const source = `<script setup lang="ts">
import { computed, ref, useTemplateRef } from 'vue'

const count = ref(1)
const doubled = computed(() => count.value * 2)
const button = useTemplateRef<HTMLButtonElement>('button')
</script>

<template>
  <button ref="button">{{ count }} {{ doubled }} {{ button }}</button>
</template>
`;

test("type-backed hover keeps reactive script and template surfaces precise", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for type-backed reactive hover",
    "tsgo binary not found; skipping type-backed reactive hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-reactive-hover-type-backed");
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

    await assertHover(
      session,
      uri,
      rangeFor("count", source.indexOf("count = ref")),
      /const count: Ref<number(?:, number)?>/,
    );
    await assertHover(
      session,
      uri,
      rangeFor("doubled", source.indexOf("doubled = computed")),
      /const doubled: ComputedRef<number>/,
    );
    await assertHover(
      session,
      uri,
      rangeFor("button", source.indexOf("button = useTemplateRef")),
      /const button: .*HTMLButtonElement.*null/,
    );
    await assertHover(
      session,
      uri,
      rangeFor("count", source.indexOf("{{ count")),
      /const count: number/,
    );
    await assertHover(
      session,
      uri,
      rangeFor("doubled", source.indexOf("{{ doubled")),
      /const doubled: number/,
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

async function assertHover(
  session: LspSession,
  uri: string,
  expectedRange: Range,
  expectedText: RegExp,
): Promise<void> {
  const hover = await session.request(
    "textDocument/hover",
    {
      position: expectedRange.start,
      textDocument: { uri },
    },
    120_000,
  );
  assert.deepEqual(
    (hover as { range?: Range } | null)?.range,
    expectedRange,
    "hover must select the authored token, not generated TypeScript",
  );
  const hoverText = hoverToText(hover as Parameters<typeof hoverToText>[0]);
  assert.match(hoverText, /^```typescript\n/);
  assert.match(hoverText, expectedText);
  assert.doesNotMatch(hoverText, /Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>/);
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
  assert.ok(vuePackage, "Vue package is required for type-backed reactive hover test");

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
