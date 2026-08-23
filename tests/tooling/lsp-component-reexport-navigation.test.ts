import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

const appSource = `<script setup lang="ts">
import { BarrelChild, StarChild } from './components'
import { PackageChild as UiChild } from '@fixture/ui'
</script>

<template>
  <BarrelChild :label="'barrel'" />
  <StarChild mode="wide" />
  <UiChild tone="info" />
</template>
`;

const barrelChildSource = `<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><span>{{ label }}</span></template>
`;

const packageChildSource = `<script setup lang="ts">
defineProps<{ tone: 'info' | 'warn' }>()
</script>
<template><span>{{ tone }}</span></template>
`;

const starChildSource = `<script setup lang="ts">
defineProps<{ mode: 'wide' | 'narrow' }>()
</script>
<template><span>{{ mode }}</span></template>
`;

test("component definition and prop hovers survive re-exported barrel and package boundaries", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-component-reexport-navigation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let initialized = false;

  try {
    const srcDir = path.join(workspaceDir, "src");
    const componentsDir = path.join(srcDir, "components");
    const packageDir = path.join(workspaceDir, "node_modules/@fixture/ui");
    fs.mkdirSync(componentsDir, { recursive: true });
    fs.mkdirSync(packageDir, { recursive: true });
    fs.writeFileSync(path.join(workspaceDir, "tsconfig.json"), JSON.stringify({}));
    fs.writeFileSync(path.join(workspaceDir, "vize.config.json"), JSON.stringify({}));

    const appPath = path.join(srcDir, "App.vue");
    const barrelChildPath = path.join(componentsDir, "BarrelChild.vue");
    const starChildPath = path.join(componentsDir, "StarChild.vue");
    const packageChildPath = path.join(packageDir, "PackageChild.vue");
    fs.writeFileSync(appPath, appSource, "utf8");
    fs.writeFileSync(barrelChildPath, barrelChildSource, "utf8");
    fs.writeFileSync(starChildPath, starChildSource, "utf8");
    fs.writeFileSync(packageChildPath, packageChildSource, "utf8");
    fs.writeFileSync(
      path.join(componentsDir, "index.ts"),
      [
        "export { default as BarrelChild } from './BarrelChild.vue'",
        "export * from './nested'",
      ].join("\n") + "\n",
      "utf8",
    );
    fs.writeFileSync(
      path.join(componentsDir, "nested.ts"),
      "export { default as StarChild } from './StarChild.vue'\n",
      "utf8",
    );
    fs.writeFileSync(
      path.join(packageDir, "package.json"),
      JSON.stringify({ name: "@fixture/ui", exports: { ".": "./index.ts" } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(packageDir, "index.ts"),
      "export { default as PackageChild } from './PackageChild.vue'\n",
      "utf8",
    );

    const uri = pathToFileURL(appPath).href;
    const barrelChildUri = pathToFileURL(barrelChildPath).href;
    const starChildUri = pathToFileURL(starChildPath).href;
    const packageChildUri = pathToFileURL(packageChildPath).href;
    await session.initialize(workspaceDir, { editor: true, typecheck: false, lint: false });
    initialized = true;
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: appSource },
    });

    await assertDefinitionUri(session, uri, positionOf("<BarrelChild") + 1, barrelChildUri);
    await assertDefinitionRange(
      session,
      uri,
      positionOf(":label") + 1,
      barrelChildUri,
      rangeForIn(barrelChildSource, "label", barrelChildSource.indexOf("defineProps")),
    );
    await assertDefinitionUri(session, uri, positionOf("<StarChild") + 1, starChildUri);
    await assertDefinitionRange(
      session,
      uri,
      positionOf("mode="),
      starChildUri,
      rangeForIn(starChildSource, "mode", starChildSource.indexOf("defineProps")),
    );
    await assertDefinitionUri(session, uri, positionOf("<UiChild") + 1, packageChildUri);
    await assertDefinitionRange(
      session,
      uri,
      positionOf("tone="),
      packageChildUri,
      rangeForIn(packageChildSource, "tone", packageChildSource.indexOf("defineProps")),
    );

    await assertComponentPropHover(
      session,
      uri,
      positionOf(":label") + 1,
      rangeForIn(appSource, ":label", positionOf(":label")),
      [/Component prop/, /label: string/],
      "barrel dynamic prop",
    );
    await assertComponentPropHover(
      session,
      uri,
      positionOf("mode="),
      rangeForIn(appSource, "mode", positionOf("mode=")),
      [/Component prop/, /mode: 'wide' \| 'narrow'/],
      "star re-exported static prop",
    );
    await assertComponentPropHover(
      session,
      uri,
      positionOf("tone="),
      rangeForIn(appSource, "tone", positionOf("tone=")),
      [/Component prop/, /tone: 'info' \| 'warn'/],
      "package static prop",
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

async function assertComponentPropHover(
  session: LspSession,
  uri: string,
  offset: number,
  expectedRange: Range,
  expectedText: RegExp[],
  label: string,
): Promise<void> {
  const hover = (await session.request(
    "textDocument/hover",
    {
      textDocument: { uri },
      position: offsetToPosition(appSource, offset),
    },
    120_000,
  )) as Parameters<typeof hoverToText>[0];
  assert.deepEqual(hover?.range, expectedRange, `${label} hover range`);
  const text = hoverToText(hover);
  for (const expected of expectedText) {
    assert.match(text, expected, `${label} hover text`);
  }
  assert.doesNotMatch(
    text,
    /Vue attribute \/ prop binding/,
    `${label} must not use v-bind fallback`,
  );
  assert.notEqual(text.trim(), "", `${label} must not be empty`);
}

async function assertDefinitionUri(
  session: LspSession,
  uri: string,
  offset: number,
  expectedUri: string,
): Promise<void> {
  const response = await requestDefinition(session, uri, offset);
  assert.equal(firstLocation(response).uri, expectedUri);
}

async function assertDefinitionRange(
  session: LspSession,
  uri: string,
  offset: number,
  expectedUri: string,
  expectedRange: Range,
): Promise<void> {
  const response = await requestDefinition(session, uri, offset);
  assert.deepEqual(firstLocation(response), { range: expectedRange, uri: expectedUri });
}

async function requestDefinition(
  session: LspSession,
  uri: string,
  offset: number,
): Promise<Parameters<typeof firstLocation>[0]> {
  return (await session.request(
    "textDocument/definition",
    {
      textDocument: { uri },
      position: offsetToPosition(appSource, offset),
    },
    120_000,
  )) as Parameters<typeof firstLocation>[0];
}

type Position = { line: number; character: number };
type Range = { start: Position; end: Position };

function positionOf(marker: string): number {
  const offset = appSource.indexOf(marker);
  assert.ok(offset >= 0, `missing marker ${marker}`);
  return offset;
}

function rangeForIn(document: string, symbol: string, nearOffset: number): Range {
  const startOffset = document.indexOf(symbol, nearOffset);
  assert.ok(startOffset >= 0, `missing ${symbol} near ${nearOffset}`);
  return {
    start: offsetToPosition(document, startOffset),
    end: offsetToPosition(document, startOffset + symbol.length),
  };
}
