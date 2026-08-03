import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspRequestError, LspSession } from "./support/lsp/session.ts";

const source = `<script setup lang="ts">
import { ref } from 'vue'

const message = ref('hello')
</script>

<template>
  <button>{{ message }}</button>
</template>
`;

const messageHover = `**message**

_Template binding from script_

\`\`\`typescript
message: Ref<string>
\`\`\`

Reactive reference created with \`ref()\`. Access \`.value\` in script, auto-unwrapped in template.

**Source**

\`<script setup>\`

**Template behavior**
- Ref values are automatically unwrapped in templates.
- The binding is resolved from \`<script setup>\` analysis.`;

test("vize lsp rejects malformed params without poisoning the session", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-invalid-params");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "InvalidParams.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: true, typecheck: false });
    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const invalidRequests = await Promise.all([
      captureRequestError(session.request("textDocument/hover", undefined)),
      captureRequestError(
        session.request("textDocument/hover", {
          textDocument: { uri },
        }),
      ),
      captureRequestError(
        session.request("textDocument/hover", {
          textDocument: { uri },
          position: { line: "zero", character: 0 },
        }),
      ),
    ]);
    assert.deepEqual(invalidRequests, [
      {
        code: -32602,
        message: "textDocument/hover: Missing params field",
        method: "textDocument/hover",
      },
      {
        code: -32602,
        message: "textDocument/hover: missing field `position`",
        method: "textDocument/hover",
      },
      {
        code: -32602,
        message: 'textDocument/hover: invalid type: string "zero", expected u32',
        method: "textDocument/hover",
      },
    ]);

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(source, source.lastIndexOf("message }}</button>") + 3),
    })) as { contents?: unknown } | null;
    assert.equal(hoverToText(hover), messageHover);

    const symbols = (await session.request("textDocument/documentSymbol", {
      textDocument: { uri },
    })) as Array<{ name: string }> | null;
    assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
    assert.deepEqual(
      symbols.map((symbol) => symbol.name),
      ["template", "script setup"],
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

async function captureRequestError(
  request: Promise<unknown>,
): Promise<{ code: number; message: string; method: string }> {
  try {
    await request;
  } catch (error) {
    assert.ok(error instanceof LspRequestError, String(error));
    return { code: error.code, message: error.message, method: error.method };
  }
  assert.fail("request unexpectedly succeeded");
}
