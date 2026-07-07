import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Position = { line: number; character: number };
type Range = { start: Position; end: Position };
type TextEdit = { range: Range; newText: string };
type WorkspaceEdit = {
  changes?: Record<string, TextEdit[]>;
  documentChanges?: Array<{
    textDocument?: { uri?: string };
    edits?: TextEdit[];
  }>;
} | null;
type SymbolInformation = {
  name: string;
  location: { uri: string };
};

function editsForUri(edit: WorkspaceEdit, uri: string): TextEdit[] {
  if (!edit) {
    return [];
  }

  const edits = edit.changes?.[uri] ?? [];
  for (const documentChange of edit.documentChanges ?? []) {
    if (documentChange.textDocument?.uri === uri) {
      edits.push(...(documentChange.edits ?? []));
    }
  }

  return edits.sort(
    (left, right) =>
      left.range.start.line - right.range.start.line ||
      left.range.start.character - right.range.start.character,
  );
}

function offsetForPosition(source: string, position: Position): number {
  let offset = 0;
  for (let line = 0; line < position.line; line += 1) {
    const nextLine = source.indexOf("\n", offset);
    assert.notEqual(nextLine, -1, `line ${position.line} is outside source`);
    offset = nextLine + 1;
  }
  return offset + position.character;
}

function textAtRange(source: string, range: Range): string {
  return source.slice(offsetForPosition(source, range.start), offsetForPosition(source, range.end));
}

async function workspaceSymbolUris(session: LspSession, query: string): Promise<string[]> {
  const symbols = (await session.request("workspace/symbol", { query })) as
    | SymbolInformation[]
    | null;
  return (symbols ?? []).map((symbol) => symbol.location.uri).sort();
}

test("vize lsp willRenameFiles rewrites Vue importer specifiers and preserves query suffixes", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-file-rename-workspace");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      fileRename: true,
      lint: false,
      typecheck: false,
    });

    const componentsDir = path.join(workspaceDir, "src", "components");
    fs.mkdirSync(componentsDir, { recursive: true });

    const oldComponentPath = path.join(componentsDir, "Foo.vue");
    const newComponentPath = path.join(componentsDir, "Bar.vue");
    fs.writeFileSync(oldComponentPath, "<template><div>foo</div></template>\n", "utf8");

    const source = `<script setup lang="ts">
import Foo from "./components/Foo.vue";
const Lazy = () => import("./components/Foo.vue?raw");
type FooModule = typeof import("./components/Foo.vue");
</script>

<template>
  <Foo />
</template>
`;
    const appPath = path.join(workspaceDir, "src", "App.vue");
    const appUri = pathToFileURL(appPath).href;
    fs.writeFileSync(appPath, source, "utf8");

    const edit = (await session.request("workspace/willRenameFiles", {
      files: [
        {
          oldUri: pathToFileURL(oldComponentPath).href,
          newUri: pathToFileURL(newComponentPath).href,
        },
      ],
    })) as WorkspaceEdit;

    const appEdits = editsForUri(edit, appUri);
    assert.equal(appEdits.length, 3, JSON.stringify(edit));
    assert.deepEqual(
      appEdits.map((textEdit) => textEdit.newText),
      ["./components/Bar.vue", "./components/Bar.vue?raw", "./components/Bar.vue"],
    );
    assert.deepEqual(
      appEdits.map((textEdit) => textAtRange(source, textEdit.range)),
      ["./components/Foo.vue", "./components/Foo.vue?raw", "./components/Foo.vue"],
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp didRenameFiles moves open document state to the new URI", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-did-rename-workspace");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      fileRename: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const movedLabel = "ready"
</script>

<template>
  <span>{{ movedLabel }}</span>
</template>
`;
    const oldPath = path.join(workspaceDir, "OldCard.vue");
    const newPath = path.join(workspaceDir, "NewCard.vue");
    const oldUri = pathToFileURL(oldPath).href;
    const newUri = pathToFileURL(newPath).href;
    fs.writeFileSync(oldPath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: oldUri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, oldUri),
    );

    await t.test("workspace symbols initially point at the old URI", async () => {
      assert.deepEqual(await workspaceSymbolUris(session, "movedLabel"), [oldUri]);
    });

    session.notify("workspace/didRenameFiles", {
      files: [
        {
          oldUri,
          newUri,
        },
      ],
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, newUri),
    );

    await t.test("workspace symbols move to the new URI", async () => {
      assert.deepEqual(await workspaceSymbolUris(session, "movedLabel"), [newUri]);
    });

    await t.test("the stale old URI no longer serves document features", async () => {
      const oldSymbols = await session.request("textDocument/documentSymbol", {
        textDocument: { uri: oldUri },
      });
      assert.equal(oldSymbols, null);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
