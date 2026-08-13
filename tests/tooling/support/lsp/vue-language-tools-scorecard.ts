import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri, offsetToPosition } from "./assertions.ts";
import { testOutputRoot } from "./paths.ts";
import type { LspRange, PublishDiagnosticsParams } from "./protocol.ts";
import { LspSession } from "./session.ts";

export type ScorecardContext = {
  publish: PublishDiagnosticsParams;
  session: LspSession;
  source: string;
  uri: string;
  workspaceDir: string;
};

export const scorecardSource = `<script setup lang="ts">
import Child from './Child.vue'
import { computed, ref } from 'vue'
import { useThing } from './useThing'

const count = ref(0)
const doubled = computed(() => count.value * 2)
const message = ref('hello')
const items = [1, 2]

function submitMessage() {
  return useThing() + count.value
}
</script>

<template>
  <Child  :label="message"
    @click="
      () => {
        cou
      }
    "
  />
  <button :class="$style.primary">{{ message }}</button>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>

<style module>
.primary {}
</style>
`;

function hasLintCode(params: PublishDiagnosticsParams, code: string): boolean {
  return params.diagnostics.some(
    (diagnostic) => diagnostic.source === "vize/lint" && diagnostic.code === code,
  );
}

export async function withScorecardDocument(
  run: (ctx: ScorecardContext) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, "lsp-vue-language-tools-scorecard");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let failure: unknown;

  try {
    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><button>{{ label }}</button></template>
`,
      "utf8",
    );
    fs.writeFileSync(path.join(workspaceDir, "useThing.mjs"), "export const useThing = () => 1\n");

    const filePath = path.join(workspaceDir, "Scorecard.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, scorecardSource, "utf8");

    await session.initialize(workspaceDir, {
      codeActions: true,
      editor: true,
      fileRename: true,
      formatting: true,
      lint: true,
      typecheck: false,
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: scorecardSource },
    });
    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        hasLintCode(params, "vue/require-v-for-key") &&
        hasLintCode(params, "vue/no-multi-spaces"),
    )) as PublishDiagnosticsParams;

    await run({ publish, session, source: scorecardSource, uri, workspaceDir });
  } catch (error) {
    failure = error;
  } finally {
    try {
      await session.shutdown();
    } catch (error) {
      if (failure == null) {
        failure = error;
      }
      await session.kill().catch(() => undefined);
    }
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
  if (failure != null) throw failure;
}

export function rangeFor(source: string, needle: string): LspRange {
  const start = source.indexOf(needle);
  assert.notEqual(start, -1, `missing source anchor ${JSON.stringify(needle)}`);
  return {
    start: offsetToPosition(source, start),
    end: offsetToPosition(source, start + needle.length),
  };
}

export function normalizeItems(
  response: Array<Record<string, unknown>> | { items?: Array<Record<string, unknown>> } | null,
): Array<Record<string, unknown>> {
  if (response == null) return [];
  return Array.isArray(response) ? response : (response.items ?? []);
}

export function startsForEdits(
  edit: {
    changes?: Record<string, Array<{ range: LspRange; newText: string }>>;
  } | null,
  uri: string,
): Array<{ line: number; character: number }> {
  return (edit?.changes?.[uri] ?? [])
    .map((textEdit) => textEdit.range.start)
    .sort((left, right) => left.line - right.line || left.character - right.character);
}
