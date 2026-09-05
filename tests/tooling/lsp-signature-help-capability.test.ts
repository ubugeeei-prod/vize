import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const source = `<script setup lang="ts">
function format(value: string, precision: number): string {
  return value.repeat(precision)
}
</script>

<template>
  <span>{{ format('hello', ) }}</span>
</template>
`;

type SignatureHelp = {
  activeParameter?: number;
  signatures?: Array<{
    label?: string;
    parameters?: unknown[];
  }>;
};

test("vize lsp maps textDocument/signatureHelp from authored SFC template calls", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-signature-help-capability");
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
  const filePath = path.join(workspaceDir, "Formatter.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: ServerCapabilities };
    assert.deepEqual(init.capabilities?.signatureHelpProvider, {
      triggerCharacters: ["(", ",", "<"],
      retriggerCharacters: [")"],
    });

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const position = offsetToPosition(
      source,
      source.indexOf("format('hello', ") + "format('hello', ".length,
    );
    const help = (await session.request("textDocument/signatureHelp", {
      textDocument: { uri },
      position,
      context: { triggerKind: 1, isRetrigger: false },
    })) as SignatureHelp | null;

    assert.ok(help, "signature help should answer the authored template call");
    assert.equal(help.activeParameter, 1);
    assert.equal(help.signatures?.length, 1, JSON.stringify(help));
    assert.match(help.signatures[0].label ?? "", /format/);
    assert.match(help.signatures[0].label ?? "", /value: string/);
    assert.match(help.signatures[0].label ?? "", /precision: number/);
    assert.equal(help.signatures[0].parameters?.length, 2, JSON.stringify(help));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
