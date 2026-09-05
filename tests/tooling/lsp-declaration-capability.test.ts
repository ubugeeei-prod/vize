import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { firstLocation, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const source = `<script setup lang="ts">
const message = "hello"
</script>

<template>
  <span>{{ message }}</span>
</template>
`;

type DeclarationCapabilities = ServerCapabilities & {
  declarationProvider?: unknown;
};

test("vize lsp maps textDocument/declaration from authored SFC template bindings", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-declaration-capability");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
        },
        include: ["**/*"],
      },
      null,
      2,
    ),
  );
  const filePath = path.join(workspaceDir, "Widget.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: DeclarationCapabilities };
    assert.equal(init.capabilities?.declarationProvider, true);

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const position = offsetToPosition(source, source.lastIndexOf("message }}</span>") + 3);
    const declaration = (await session.request("textDocument/declaration", {
      textDocument: { uri },
      position,
    })) as Array<{ uri: string; range: { start: { line: number; character: number } } }>;
    const location = firstLocation(declaration);
    assert.equal(location.uri, uri);
    assert.deepEqual(location.range.start, offsetToPosition(source, source.indexOf("message =")));
    assert.ok(!location.uri.endsWith(".vue.ts"), JSON.stringify(declaration));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
