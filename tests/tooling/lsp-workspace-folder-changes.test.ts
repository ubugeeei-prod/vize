import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const V_FOR_KEY_RULE = "vue/require-v-for-key";
const SOURCE = `<script setup lang="ts">
const rows = ['alpha', 'beta']
</script>

<template>
  <ul>
    <li v-for="row in rows">{{ row }}</li>
  </ul>
</template>
`;

test("vize lsp revalidates open documents when workspace folders change", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-workspace-folder-changes");
  fs.mkdirSync(testRootDir, { recursive: true });
  const parentDir = fs.mkdtempSync(path.join(testRootDir, "roots-"));
  const rootA = path.join(parentDir, "root-a");
  const rootB = path.join(parentDir, "root-b");
  fs.mkdirSync(rootA);
  fs.mkdirSync(rootB);
  fs.writeFileSync(
    path.join(rootB, "vize.config.json"),
    JSON.stringify({ linter: { rules: { [V_FOR_KEY_RULE]: "off" } } }),
    "utf8",
  );

  const filePath = path.join(rootB, "List.vue");
  const uri = pathToFileURL(filePath).href;
  const rootBUri = pathToFileURL(rootB).href;
  fs.writeFileSync(filePath, SOURCE, "utf8");
  const session = new LspSession();
  let primaryError: unknown;

  try {
    await session.initialize(rootA, { editor: true, lint: true, typecheck: false });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });

    const initial = await waitForRuleState(session, uri, true);
    assert.equal(initial.version, 1);
    assert.equal(ruleDiagnostics(initial).length, 1);

    session.notify("workspace/didChangeWorkspaceFolders", {
      event: {
        added: [{ uri: rootBUri, name: "root-b" }],
        removed: [],
      },
    });
    const afterAdd = await waitForRuleState(session, uri, false);
    assert.equal(afterAdd.version, 1, "adding a folder must revalidate without a document edit");
    assert.deepEqual(ruleDiagnostics(afterAdd), []);

    session.notify("workspace/didChangeWorkspaceFolders", {
      event: {
        added: [],
        removed: [{ uri: rootBUri, name: "root-b" }],
      },
    });
    const afterRemove = await waitForRuleState(session, uri, true);
    assert.equal(
      afterRemove.version,
      1,
      "removing a folder must revalidate without a document edit",
    );
    assert.equal(ruleDiagnostics(afterRemove).length, 1);
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(parentDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

async function waitForRuleState(
  session: LspSession,
  uri: string,
  present: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
    if (!isDiagnosticsForUri(params, uri) || params.version !== 1) return false;
    return ruleDiagnostics(params).length > 0 === present;
  })) as PublishDiagnosticsParams;
}

function ruleDiagnostics(params: PublishDiagnosticsParams) {
  return params.diagnostics.filter(
    (diagnostic) => diagnostic.source === "vize/lint" && diagnostic.code === V_FOR_KEY_RULE,
  );
}
