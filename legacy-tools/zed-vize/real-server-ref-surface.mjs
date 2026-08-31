import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const refSurfaceSource = `<script setup lang="ts">
import { computed, ref } from "vue";

const count = ref(1);
const doubled = computed(() => count.value * 2);
</script>

<template>
  <p>{{ count }} {{ doubled }}</p>
</template>
`;

const expectedRefSurfaceHovers = {
  scriptCount: {
    contents: {
      kind: "markdown",
      value: "```typescript\nconst count: Ref<number, number>\n```",
    },
    range: {
      end: { character: 11, line: 3 },
      start: { character: 6, line: 3 },
    },
  },
  scriptDoubled: {
    contents: {
      kind: "markdown",
      value: "```typescript\nconst doubled: ComputedRef<number>\n```",
    },
    range: {
      end: { character: 13, line: 4 },
      start: { character: 6, line: 4 },
    },
  },
  templateCount: {
    contents: {
      kind: "markdown",
      value: "```typescript\nconst count: number\n```",
    },
    range: {
      end: { character: 13, line: 8 },
      start: { character: 8, line: 8 },
    },
  },
  templateDoubled: {
    contents: {
      kind: "markdown",
      value: "```typescript\nconst doubled: number\n```",
    },
    range: {
      end: { character: 27, line: 8 },
      start: { character: 20, line: 8 },
    },
  },
};

export async function assertRefSurfaceHovers({ isDiagnosticsForUri, session, workspacePath }) {
  const documentPath = path.join(workspacePath, "src", "RefSurface.vue");
  fs.writeFileSync(documentPath, refSurfaceSource, "utf8");
  const uri = pathToFileURL(documentPath).href;
  session.notify("textDocument/didOpen", {
    textDocument: {
      uri,
      languageId: "vue",
      version: 1,
      text: refSurfaceSource,
    },
  });
  const diagnostics = await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri),
    120_000,
  );
  assert.deepEqual(diagnostics.diagnostics, []);

  const hovers = {
    scriptCount: await hoverAt(session, uri, 3, 8),
    scriptDoubled: await hoverAt(session, uri, 4, 8),
    templateCount: await hoverAt(session, uri, 8, 10),
    templateDoubled: await hoverAt(session, uri, 8, 22),
  };
  assert.deepEqual(hovers, expectedRefSurfaceHovers);
  for (const value of Object.values(hovers).map((hover) => hover.contents.value)) {
    assert.doesNotMatch(value, /Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>/);
  }
}

function hoverAt(session, uri, line, character) {
  return session.request(
    "textDocument/hover",
    { textDocument: { uri }, position: { character, line } },
    120_000,
  );
}
