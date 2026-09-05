import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { firstLocation, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspRequestError, LspSession } from "./support/lsp/session.ts";

const source = `<script setup lang="ts">
const message = "hello"
</script>

<template>
  <span>{{ message }}</span>
</template>
`;

type ImplementationCapabilities = ServerCapabilities & {
  implementationProvider?: unknown;
};

test("vize lsp keeps textDocument/implementation fail-closed until #3953 implements it", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-implementation-capability");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "Widget.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    })) as { capabilities?: ImplementationCapabilities };
    assert.equal(init.capabilities?.implementationProvider, undefined);

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    const position = offsetToPosition(source, source.lastIndexOf("message }}</span>") + 3);
    await assert.rejects(
      session.request("textDocument/implementation", {
        textDocument: { uri },
        position,
      }),
      (error) =>
        error instanceof LspRequestError &&
        error.method === "textDocument/implementation" &&
        error.code === -32601,
    );

    const definition = (await session.request("textDocument/definition", {
      textDocument: { uri },
      position,
    })) as Array<{ uri: string; range: { start: { line: number; character: number } } }>;
    const location = firstLocation(definition);
    assert.equal(location.uri, uri);
    assert.deepEqual(location.range.start, offsetToPosition(source, source.indexOf("message =")));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
