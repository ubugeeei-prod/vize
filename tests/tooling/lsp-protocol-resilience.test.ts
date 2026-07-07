import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  completionLabels,
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspRequestError, LspSession } from "./support/lsp/session.ts";

type ProtocolContext = {
  session: LspSession;
  source: string;
  uri: string;
  workspaceDir: string;
};

async function withProtocolDocument(
  label: string,
  run: (ctx: ProtocolContext) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, `lsp-protocol-resilience-${label}`);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: false,
    });

    fs.writeFileSync(
      path.join(workspaceDir, "Dep.vue"),
      `<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><button /></template>
`,
      "utf8",
    );

    const source = `<script setup lang="ts">
import Dep from './Dep.vue'
import { ref } from 'vue'

const message = ref('hello')
const enabled = ref(false)
</script>

<template>
  <Dep :label="message" />
  <button :disabled="enabled" @click="message = 'clicked'">{{ message }}</button>
</template>
`;
    const filePath = path.join(workspaceDir, "Protocol.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    await run({ session, source, uri, workspaceDir });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("vize lsp reports unsupported requests without poisoning the session", async () => {
  await withProtocolDocument("unknown-request", async ({ session, source, uri }) => {
    await assert.rejects(
      session.request("vize/definitelyMissing", { reason: "coverage" }),
      (error) =>
        error instanceof LspRequestError &&
        error.method === "vize/definitelyMissing" &&
        error.code === -32601,
    );

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(source, source.lastIndexOf("message }}</button>") + 3),
    })) as { contents?: unknown } | null;
    assert.match(hoverToText(hover), /message/);
  });
});

test("vize lsp ignores cancellation and workspace notifications it does not consume", async () => {
  await withProtocolDocument(
    "no-op-notifications",
    async ({ session, source, uri, workspaceDir }) => {
      session.notify("$/cancelRequest", { id: 999_999 });
      session.notify("workspace/didChangeConfiguration", {
        settings: { vize: { trace: "messages" } },
      });
      session.notify("workspace/didChangeWatchedFiles", {
        changes: [
          {
            uri: pathToFileURL(path.join(workspaceDir, "Ghost.vue")).href,
            type: 1,
          },
        ],
      });
      session.notify("textDocument/didSave", {
        textDocument: { uri },
        text: source,
      });

      const symbols = (await session.request("textDocument/documentSymbol", {
        textDocument: { uri },
      })) as Array<{ name: string }> | null;
      assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
      assert.ok(
        symbols.some((symbol) => symbol.name === "template"),
        JSON.stringify(symbols),
      );

      const tokens = (await session.request("textDocument/semanticTokens/full", {
        textDocument: { uri },
      })) as { data?: number[] } | null;
      assert.ok((tokens?.data?.length ?? 0) > 0, JSON.stringify(tokens));
    },
  );
});

test("vize lsp resolves concurrent editor requests to their matching responses", async () => {
  await withProtocolDocument("concurrent-requests", async ({ session, source, uri }) => {
    const templateMessage = offsetToPosition(
      source,
      source.lastIndexOf("message }}</button>") + "message".length,
    );
    const enabledValue = offsetToPosition(source, source.indexOf('"enabled"') + '"en'.length);

    const [
      hover,
      definition,
      references,
      completion,
      documentSymbols,
      foldingRanges,
      documentLinks,
      semanticTokens,
      workspaceSymbols,
    ] = await Promise.all([
      session.request("textDocument/hover", {
        textDocument: { uri },
        position: templateMessage,
      }),
      session.request("textDocument/definition", {
        textDocument: { uri },
        position: templateMessage,
      }),
      session.request("textDocument/references", {
        textDocument: { uri },
        position: templateMessage,
        context: { includeDeclaration: true },
      }),
      session.request("textDocument/completion", {
        textDocument: { uri },
        position: enabledValue,
      }),
      session.request("textDocument/documentSymbol", {
        textDocument: { uri },
      }),
      session.request("textDocument/foldingRange", {
        textDocument: { uri },
      }),
      session.request("textDocument/documentLink", {
        textDocument: { uri },
      }),
      session.request("textDocument/semanticTokens/full", {
        textDocument: { uri },
      }),
      session.request("workspace/symbol", {
        query: "message",
      }),
    ]);

    assert.match(hoverToText(hover as { contents?: unknown } | null), /message/);

    const location = firstLocation(
      definition as
        | Array<{ uri: string; range: { start: { line: number; character: number } } }>
        | { uri: string; range: { start: { line: number; character: number } } },
    );
    assert.equal(location.uri, uri);
    assert.deepEqual(location.range.start, offsetToPosition(source, source.indexOf("message =")));

    assert.ok(
      Array.isArray(references) &&
        references.some(
          (reference) =>
            reference.uri === uri &&
            reference.range.start.line === templateMessage.line &&
            reference.range.start.character === templateMessage.character - "message".length,
        ),
      JSON.stringify(references),
    );

    assert.ok(completionLabels(completion).includes("enabled"), JSON.stringify(completion));
    assert.ok(
      Array.isArray(documentSymbols) &&
        documentSymbols.some((symbol: { name?: string }) => symbol.name === "script setup"),
      JSON.stringify(documentSymbols),
    );
    assert.ok(
      Array.isArray(foldingRanges) && foldingRanges.length > 0,
      JSON.stringify(foldingRanges),
    );
    assert.ok(
      Array.isArray(documentLinks) &&
        documentLinks.some((link: { target?: string }) => link.target?.endsWith("/Dep.vue")),
      JSON.stringify(documentLinks),
    );
    assert.ok(
      Array.isArray((semanticTokens as { data?: unknown[] } | null)?.data) &&
        ((semanticTokens as { data?: unknown[] }).data?.length ?? 0) > 0,
      JSON.stringify(semanticTokens),
    );
    assert.ok(
      Array.isArray(workspaceSymbols) &&
        workspaceSymbols.some((symbol: { name?: string }) => symbol.name === "message"),
      JSON.stringify(workspaceSymbols),
    );
  });
});

test("vize lsp applies multiple range changes in one didChange notification", async () => {
  await withProtocolDocument("multi-range-change", async ({ session, source, uri }) => {
    const declarationStart = source.indexOf("message =");
    const templateStart = source.lastIndexOf("{{ message }}") + "{{ ".length;

    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [
        {
          range: {
            start: offsetToPosition(source, declarationStart),
            end: offsetToPosition(source, declarationStart + "message".length),
          },
          text: "counter",
        },
        {
          range: {
            start: offsetToPosition(source, templateStart),
            end: offsetToPosition(source, templateStart + "message".length),
          },
          text: "counter",
        },
      ],
    });

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
    )) as PublishDiagnosticsParams;
    assert.equal(publish.version, 2);

    const updatedSource =
      source.slice(0, declarationStart) +
      "counter" +
      source.slice(declarationStart + "message".length, templateStart) +
      "counter" +
      source.slice(templateStart + "message".length);
    const counterUsage = offsetToPosition(
      updatedSource,
      updatedSource.lastIndexOf("counter }}") + 3,
    );

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: counterUsage,
    })) as { contents?: unknown } | null;
    assert.match(hoverToText(hover), /counter/);

    const definition = (await session.request("textDocument/definition", {
      textDocument: { uri },
      position: counterUsage,
    })) as
      | Array<{ uri: string; range: { start: { line: number; character: number } } }>
      | { uri: string; range: { start: { line: number; character: number } } };
    assert.deepEqual(
      firstLocation(definition).range.start,
      offsetToPosition(updatedSource, declarationStart),
    );
  });
});

test("vize lsp full-text changes replace stale virtual document contents", async () => {
  await withProtocolDocument("full-text-change", async ({ session, uri }) => {
    const nextSource = `<script setup lang="ts">
import { ref } from 'vue'

const status = ref('ready')
</script>

<template>
  <p>{{ status }}</p>
</template>
`;

    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 3 },
      contentChanges: [{ text: nextSource }],
    });

    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 3,
    );

    const statusUsage = offsetToPosition(nextSource, nextSource.lastIndexOf("status }}</p>") + 3);
    const definition = (await session.request("textDocument/definition", {
      textDocument: { uri },
      position: statusUsage,
    })) as
      | Array<{ uri: string; range: { start: { line: number; character: number } } }>
      | { uri: string; range: { start: { line: number; character: number } } };
    assert.deepEqual(
      firstLocation(definition).range.start,
      offsetToPosition(nextSource, nextSource.indexOf("status =")),
    );

    const staleSymbols = (await session.request("workspace/symbol", {
      query: "message",
    })) as Array<{ name?: string; location?: { uri?: string } }> | null;
    assert.equal(
      (staleSymbols ?? []).some(
        (symbol) => symbol.name === "message" && symbol.location?.uri === uri,
      ),
      false,
      JSON.stringify(staleSymbols),
    );
  });
});
