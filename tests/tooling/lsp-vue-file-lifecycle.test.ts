import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Location = {
  uri: string;
  range: { start: { line: number; character: number } };
};

type SymbolInformation = {
  name: string;
  location: Location;
};

async function workspaceSymbols(
  session: LspSession,
  query: string,
): Promise<SymbolInformation[] | null> {
  return (await session.request("workspace/symbol", { query })) as SymbolInformation[] | null;
}

async function definitionAt(
  session: LspSession,
  uri: string,
  source: string,
  marker: string,
): Promise<Location | Location[] | null> {
  const offset = source.indexOf(marker);
  assert.notEqual(offset, -1, `missing definition marker ${marker}`);
  return (await session.request("textDocument/definition", {
    textDocument: { uri },
    position: offsetToPosition(source, offset + 1),
  })) as Location | Location[] | null;
}

test("vize lsp follows created and deleted on-disk Vue files without opening them", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-vue-file-lifecycle");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });

    const appSource = `<script setup lang="ts">
import DiskChild from "./DiskChild.vue"
</script>

<template>
  <DiskChild :message="'hello'" />
</template>
`;
    const appPath = path.join(sourceDir, "App.vue");
    const appUri = pathToFileURL(appPath).href;
    fs.writeFileSync(appPath, appSource, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: appSource },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, appUri),
    );

    const childSource = `<script setup lang="ts">
defineProps<{ message: string }>()
const diskLifecycleMarker = 1
</script>

<template>
  <span>{{ message }} {{ diskLifecycleMarker }}</span>
</template>
`;
    const childPath = path.join(sourceDir, "DiskChild.vue");
    const childUri = pathToFileURL(childPath).href;

    await t.test(
      "a create event indexes the closed file and resolves its imported props",
      async () => {
        assert.equal(await workspaceSymbols(session, "diskLifecycleMarker"), null);
        assert.equal(await definitionAt(session, appUri, appSource, "message"), null);

        fs.writeFileSync(childPath, childSource, "utf8");
        session.notify("workspace/didCreateFiles", { files: [{ uri: childUri }] });

        const symbols = await workspaceSymbols(session, "diskLifecycleMarker");
        assert.equal(symbols?.length, 1, JSON.stringify(symbols));
        assert.equal(symbols?.[0]?.name, "diskLifecycleMarker");
        assert.equal(symbols?.[0]?.location.uri, childUri);

        const definition = await definitionAt(session, appUri, appSource, "message");
        assert.ok(definition != null && !Array.isArray(definition), JSON.stringify(definition));
        assert.equal(definition.uri, childUri);
      },
    );

    await t.test("a delete event removes the closed file from both surfaces", async () => {
      fs.rmSync(childPath);
      session.notify("workspace/didDeleteFiles", { files: [{ uri: childUri }] });

      assert.equal(await workspaceSymbols(session, "diskLifecycleMarker"), null);
      assert.equal(await definitionAt(session, appUri, appSource, "message"), null);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
