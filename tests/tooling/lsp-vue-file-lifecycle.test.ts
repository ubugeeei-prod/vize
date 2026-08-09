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

test("vize lsp follows directory lifecycle for closed on-disk Vue files", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-vue-file-lifecycle");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });

    const appSource = `<script setup lang="ts">
import DiskChild from "./components/DiskChild.vue"
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
    const componentsDir = path.join(sourceDir, "components");
    const movedComponentsDir = path.join(sourceDir, "moved-components");
    const childPath = path.join(componentsDir, "DiskChild.vue");
    const movedChildPath = path.join(movedComponentsDir, "DiskChild.vue");
    const childUri = pathToFileURL(childPath).href;
    const movedChildUri = pathToFileURL(movedChildPath).href;
    const componentsUri = pathToFileURL(componentsDir).href;
    const movedComponentsUri = pathToFileURL(movedComponentsDir).href;

    await t.test(
      "a directory create indexes nested closed files and resolves imported props",
      async () => {
        assert.equal(await workspaceSymbols(session, "diskLifecycleMarker"), null);
        assert.equal(await definitionAt(session, appUri, appSource, "message"), null);

        fs.mkdirSync(componentsDir);
        fs.writeFileSync(childPath, childSource, "utf8");
        session.notify("workspace/didCreateFiles", { files: [{ uri: componentsUri }] });

        const symbols = await workspaceSymbols(session, "diskLifecycleMarker");
        assert.equal(symbols?.length, 1, JSON.stringify(symbols));
        assert.equal(symbols?.[0]?.name, "diskLifecycleMarker");
        assert.equal(symbols?.[0]?.location.uri, childUri);

        const definition = await definitionAt(session, appUri, appSource, "message");
        assert.ok(definition != null && !Array.isArray(definition), JSON.stringify(definition));
        assert.equal(definition.uri, childUri);
      },
    );

    await t.test("directory renames relocate every nested closed-file symbol", async () => {
      fs.renameSync(componentsDir, movedComponentsDir);
      session.notify("workspace/didRenameFiles", {
        files: [{ oldUri: componentsUri, newUri: movedComponentsUri }],
      });

      const movedSymbols = await workspaceSymbols(session, "diskLifecycleMarker");
      assert.deepEqual(
        movedSymbols?.map((symbol) => symbol.location.uri),
        [movedChildUri],
      );
      assert.equal(await definitionAt(session, appUri, appSource, "message"), null);

      fs.renameSync(movedComponentsDir, componentsDir);
      session.notify("workspace/didRenameFiles", {
        files: [{ oldUri: movedComponentsUri, newUri: componentsUri }],
      });

      const restoredSymbols = await workspaceSymbols(session, "diskLifecycleMarker");
      assert.deepEqual(
        restoredSymbols?.map((symbol) => symbol.location.uri),
        [childUri],
      );
      const definition = await definitionAt(session, appUri, appSource, "message");
      assert.ok(definition != null && !Array.isArray(definition), JSON.stringify(definition));
      assert.equal(definition.uri, childUri);
    });

    await t.test("a directory delete removes nested closed files from both surfaces", async () => {
      fs.rmSync(componentsDir, { recursive: true });
      session.notify("workspace/didDeleteFiles", { files: [{ uri: componentsUri }] });

      assert.equal(await workspaceSymbols(session, "diskLifecycleMarker"), null);
      assert.equal(await definitionAt(session, appUri, appSource, "message"), null);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
