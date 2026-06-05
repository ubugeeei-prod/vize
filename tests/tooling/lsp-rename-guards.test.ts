import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type PrepareRenameResult = {
  start?: { line: number; character: number };
  end?: { line: number; character: number };
} | null;

type WorkspaceEdit = {
  changes?: Record<
    string,
    Array<{
      range: {
        start: { line: number; character: number };
        end: { line: number; character: number };
      };
      newText: string;
    }>
  >;
} | null;

test("vize lsp prepareRename guards reject non-renamable positions", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-rename-guards");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const items = [1, 2, 3]
</script>

<template>
  <ul>
    <li v-for="i in items">{{ i }}</li>
  </ul>
</template>
`;
    const filePath = path.join(workspaceDir, "ForList.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    await t.test("prepareRename returns null on a Vue directive name (v-for)", async () => {
      const directiveOffset = source.indexOf("v-for") + 2;
      const response = (await session.request("textDocument/prepareRename", {
        textDocument: { uri },
        position: offsetToPosition(source, directiveOffset),
      })) as PrepareRenameResult;

      assert.equal(response, null);
    });

    await t.test("prepareRename returns null on whitespace / non-identifier position", async () => {
      const whitespaceOffset = source.indexOf("<template>") - 1;
      const response = (await session.request("textDocument/prepareRename", {
        textDocument: { uri },
        position: offsetToPosition(source, whitespaceOffset),
      })) as PrepareRenameResult;

      assert.equal(response, null);
    });

    await t.test(
      "prepareRename succeeds on a template-used binding and reports its identifier range",
      async () => {
        const itemsUsageOffset = source.lastIndexOf("items") + 2;
        const response = (await session.request("textDocument/prepareRename", {
          textDocument: { uri },
          position: offsetToPosition(source, itemsUsageOffset),
        })) as PrepareRenameResult;

        const start = offsetToPosition(source, source.lastIndexOf("items"));
        assert.deepEqual(response?.start, start);
        assert.deepEqual(response?.end, {
          line: start.line,
          character: start.character + "items".length,
        });
      },
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp rename of a template-used binding edits declaration and template usage", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-rename-guards-edit");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const total = 5
</script>

<template>
  <span>{{ total }}</span>
</template>
`;
    const filePath = path.join(workspaceDir, "Total.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const templateUsageOffset = source.lastIndexOf("total") + 2;
    const edit = (await session.request("textDocument/rename", {
      textDocument: { uri },
      position: offsetToPosition(source, templateUsageOffset),
      newName: "sum",
    })) as WorkspaceEdit;

    const edits = edit?.changes?.[uri] ?? [];
    assert.equal(edits.length, 2, JSON.stringify(edit));
    assert.ok(
      edits.every((textEdit) => textEdit.newText === "sum"),
      JSON.stringify(edits),
    );

    const declarationStart = offsetToPosition(
      source,
      source.indexOf("const total") + "const ".length,
    );
    const templateStart = offsetToPosition(source, source.lastIndexOf("total"));

    const starts = edits
      .map((textEdit) => textEdit.range.start)
      .sort((a, b) => a.line - b.line || a.character - b.character);
    const expected = [declarationStart, templateStart].sort(
      (a, b) => a.line - b.line || a.character - b.character,
    );
    assert.deepEqual(starts, expected, JSON.stringify(edits));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
