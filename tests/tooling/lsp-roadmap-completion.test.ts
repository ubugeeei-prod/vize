import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { completionLabels, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// Behavioral suite for the Tier-1 roadmap items (#677, #678, #679, #680).
// Each fixture below targets one acceptance criterion. New tests live here
// rather than in lsp-smoke.test.ts so the smoke suite stays compact and the
// roadmap criteria are clearly separated for review.
test("vize lsp surfaces roadmap Tier-1 completions", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-roadmap-completion");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    // #679 — scope-aware completion (script side, nested closure).
    const closureSource = `<script setup lang="ts">
import { ref } from 'vue'

const outer = ref(0)

function increment() {
  const localStep = 1
  loc
}
</script>
`;
    const closurePath = path.join(workspaceDir, "Closure.vue");
    fs.writeFileSync(closurePath, closureSource, "utf8");

    // #679 — scope-aware completion (template side, v-for body).
    const vForSource = `<script setup lang="ts">
const items = [1, 2, 3]
</script>
<template>
  <div v-for="item in items">{{ it }}</div>
</template>
`;
    const vForPath = path.join(workspaceDir, "VFor.vue");
    fs.writeFileSync(vForPath, vForSource, "utf8");

    await session.initialize(workspaceDir);

    const closureUri = pathToFileURL(closurePath).href;
    const vForUri = pathToFileURL(vForPath).href;

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: closureUri,
        languageId: "vue",
        version: 1,
        text: closureSource,
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: vForUri,
        languageId: "vue",
        version: 1,
        text: vForSource,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics");

    await t.test("closure-local bindings surface in script completion", async () => {
      const offset = closureSource.indexOf("loc\n") + "loc".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: closureUri },
        position: offsetToPosition(closureSource, offset),
      });
      const labels = completionLabels(response);
      assert.ok(
        labels.includes("localStep"),
        `closure local should be visible; got ${labels.join(", ")}`,
      );
      assert.ok(
        labels.includes("outer"),
        `setup-scope binding should also be visible; got ${labels.join(", ")}`,
      );
    });

    await t.test("v-for iteration variable surfaces inside its subtree", async () => {
      const offset = vForSource.indexOf("{{ it ") + "{{ it".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: vForUri },
        position: offsetToPosition(vForSource, offset),
      });
      const labels = completionLabels(response);
      assert.ok(
        labels.includes("item"),
        `v-for variable should be visible inside v-for; got ${labels.join(", ")}`,
      );
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});
