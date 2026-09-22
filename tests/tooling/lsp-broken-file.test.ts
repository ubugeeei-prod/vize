import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * TS-47: hover and completion stay live in the well-formed region of a file
 * whose template has a parse hole elsewhere. `typecheck` and `lint` are off,
 * so the answer is Maestro's own template analysis.
 *
 * The stray `</stray>` is an S1 `Unexpected` hole. The unclosed `<div` in the
 * second file is an S1 `Missing` `>` (EOF in tag). Neither may blank the
 * other region's `v-for` alias or the script binding.
 */

const LABEL_HOVER = [
  "**label**",
  "",
  "_Template binding from script_",
  "",
  "```typescript",
  "label: literal const",
  "```",
  "",
  "Literal constant value, hoisted for optimization.",
  "",
  "**Source**",
  "",
  "`<script setup>`",
  "",
  "**Template behavior**",
  "- Ref values are automatically unwrapped in templates.",
  "- The binding is resolved from `<script setup>` analysis.",
].join("\n");

const V_FOR_HOVER = [
  "**item**",
  "",
  "_v-for scope binding_",
  "",
  "Loop value alias declared by the nearest `v-for` and available to this template scope.",
  "",
  "**Behavior**",
  "- Shadows outer template and script bindings with the same name.",
  "- Type-backed sessions should refine this with the iterable item type when the backend answers.",
].join("\n");

const BROKEN = `<script setup lang="ts">
const label = 'ok'
const items = [1]
</script>
<template>
  <p>{{ label }}</p>
  <li v-for="item in items">{{ item }}{{ }}</li>
  </stray>
</template>
`;

const MISSING_CLOSE = `<script setup lang="ts">
const label = 'ok'
const items = [1]
</script>
<template>
  <p>{{ label }}</p>
  <li v-for="item in items">{{ item }}{{ }}</li>
  <div
</template>
`;

const WELL_FORMED = `<script setup lang="ts">
const label = 'ok'
const items = [1]
</script>
<template>
  <p>{{ label }}</p>
  <li v-for="item in items">{{ item }}</li>
</template>
`;

type Diagnostic = { severity?: number; message?: string };

test("hover and completion stay live beside an unexpected end tag", async () => {
  await withSession(async (session, workspaceDir) => {
    const uri = await open(session, workspaceDir, "Broken.vue", BROKEN);
    const diagnostics = await published(session, uri);
    assert.deepEqual(
      diagnostics
        .filter((diagnostic) => diagnostic.severity === 1)
        .map((diagnostic) => diagnostic.message),
      ["Invalid end tag."],
    );

    const itemAt = BROKEN.lastIndexOf("{{ item }}") + "{{ ".length;
    assert.equal(hoverToText(await hoverAt(session, uri, BROKEN, itemAt)), V_FOR_HOVER);

    const labelAt = BROKEN.indexOf("{{ label }}") + "{{ ".length;
    assert.equal(hoverToText(await hoverAt(session, uri, BROKEN, labelAt)), LABEL_HOVER);

    const caret = BROKEN.lastIndexOf("{{ }}") + "{{ ".length;
    const labels = await labelsAt(session, uri, BROKEN, caret);
    assert.deepEqual(picked(labels), ["item", "items", "label"]);
  });
});

test("hover and completion stay live beside a missing end tag", async () => {
  await withSession(async (session, workspaceDir) => {
    const uri = await open(session, workspaceDir, "MissingClose.vue", MISSING_CLOSE);
    const diagnostics = await published(session, uri);
    assert.deepEqual(
      diagnostics
        .filter((diagnostic) => diagnostic.severity === 1)
        .map((diagnostic) => diagnostic.message)
        .sort(),
      [
        "Element is missing end tag.",
        "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue.",
      ],
    );

    const itemAt = MISSING_CLOSE.indexOf("{{ item }}") + "{{ ".length;
    assert.equal(hoverToText(await hoverAt(session, uri, MISSING_CLOSE, itemAt)), V_FOR_HOVER);

    const caret = MISSING_CLOSE.indexOf("{{ }}") + "{{ ".length;
    assert.deepEqual(picked(await labelsAt(session, uri, MISSING_CLOSE, caret)), [
      "item",
      "items",
      "label",
    ]);
  });
});

test("a well-formed file opened beside a broken one keeps an empty error list", async () => {
  await withSession(async (session, workspaceDir) => {
    const broken = await open(session, workspaceDir, "Broken.vue", BROKEN);
    await published(session, broken);
    const clean = await open(session, workspaceDir, "Clean.vue", WELL_FORMED);
    const diagnostics = await published(session, clean);
    assert.deepEqual(
      diagnostics.filter((diagnostic) => diagnostic.severity === 1),
      [],
    );

    const itemAt = WELL_FORMED.lastIndexOf("{{ item }}") + "{{ ".length;
    assert.equal(hoverToText(await hoverAt(session, clean, WELL_FORMED, itemAt)), V_FOR_HOVER);
  });
});

const PICKED = ["item", "items", "label", "stray"];

function picked(labels: readonly string[]): string[] {
  return PICKED.filter((label) => labels.includes(label));
}

async function withSession(
  body: (session: LspSession, workspaceDir: string) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, "lsp-broken-file");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });
    await body(session, workspaceDir);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
}

async function open(
  session: LspSession,
  workspaceDir: string,
  name: string,
  source: string,
): Promise<string> {
  const filePath = path.join(workspaceDir, name);
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, source, "utf8");
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
  });
  return uri;
}

async function published(session: LspSession, uri: string): Promise<Diagnostic[]> {
  const params = await session.waitForNotification("textDocument/publishDiagnostics", (value) =>
    isDiagnosticsForUri(value, uri),
  );
  return (params as { diagnostics: Diagnostic[] }).diagnostics;
}

async function hoverAt(
  session: LspSession,
  uri: string,
  source: string,
  offset: number,
): Promise<{ contents?: unknown } | null> {
  return (await session.request("textDocument/hover", {
    textDocument: { uri },
    position: offsetToPosition(source, offset),
  })) as { contents?: unknown } | null;
}

async function labelsAt(
  session: LspSession,
  uri: string,
  source: string,
  offset: number,
): Promise<string[]> {
  const result = (await session.request("textDocument/completion", {
    textDocument: { uri },
    position: offsetToPosition(source, offset),
  })) as Array<{ label: string }> | { items: Array<{ label: string }> } | null;
  const items = Array.isArray(result) ? result : (result?.items ?? []);
  return items.map((item) => item.label).sort();
}
