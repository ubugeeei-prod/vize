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
interface Product {
  name: string
}

const product = {} as Product
</script>

<template>
  <span>{{ product.name }}</span>
</template>
`;

test("vize lsp maps textDocument/typeDefinition from authored SFC template bindings", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-type-definition-capability");
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
  const filePath = path.join(workspaceDir, "Card.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: ServerCapabilities };
    assert.equal(init.capabilities?.typeDefinitionProvider, true);

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const position = offsetToPosition(source, source.lastIndexOf("product.name") + 3);
    const typeDefinition = await session.request("textDocument/typeDefinition", {
      textDocument: { uri },
      position,
    });
    const location = firstLocation(typeDefinition as never);
    assert.equal(location.uri, uri);
    assert.deepEqual(location.range.start, offsetToPosition(source, source.indexOf("Product {")));
    assert.ok(!location.uri.endsWith(".vue.ts"), JSON.stringify(typeDefinition));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
