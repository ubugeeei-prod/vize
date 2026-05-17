import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  assertDiagnosticRange,
  assertEmptyEditorRequests,
  assertNoDiagnosticSource,
  completionLabels,
  firstLocation,
  hasDiagnosticSource,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

test("vize lsp smoke-tests production editor flows", async (t) => {
  const agentOnlyDir = path.join(root, "__agent_only", "lsp-smoke");
  fs.mkdirSync(agentOnlyDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(agentOnlyDir, "workspace-"));
  const session = new LspSession();

  try {
    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts"></script>
<template><button /></template>
`,
      "utf8",
    );

    const parentSource = `<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
</template>
`;
    const parentPath = path.join(workspaceDir, "Parent.vue");
    fs.writeFileSync(parentPath, parentSource, "utf8");

    const artSource = `<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
  <variant name="Secondary">
    <Child>{{ secondaryLabel }}</Child>
  </variant>
</art>
`;
    const artPath = path.join(workspaceDir, "Button.art.vue");
    fs.writeFileSync(artPath, artSource, "utf8");

    const init = (await session.initialize(workspaceDir)) as {
      capabilities?: {
        completionProvider?: {
          triggerCharacters?: string[];
        };
        hoverProvider?: boolean;
        definitionProvider?: boolean;
        referencesProvider?: boolean;
        semanticTokensProvider?: {
          range?: boolean;
          full?: boolean | unknown;
        };
      };
    };

    assert.equal(init.capabilities?.hoverProvider, true);
    assert.equal(init.capabilities?.definitionProvider, true);
    assert.equal(init.capabilities?.referencesProvider, true);
    assert.equal(init.capabilities?.semanticTokensProvider?.range, true);
    assert.ok(init.capabilities?.completionProvider?.triggerCharacters?.includes("."));

    const parentUri = pathToFileURL(parentPath).href;
    const artUri = pathToFileURL(artPath).href;

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: parentUri,
        languageId: "vue",
        version: 1,
        text: parentSource,
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: artUri,
        languageId: "art-vue",
        version: 1,
        text: artSource,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics");

    await t.test("go-to-definition resolves component tags in Vue templates", async () => {
      const childUsageOffset = parentSource.indexOf("Child />") + "Child".length;
      const definition = (await session.request("textDocument/definition", {
        textDocument: { uri: parentUri },
        position: offsetToPosition(parentSource, childUsageOffset),
      })) as
        | Array<{
            uri: string;
            range: { start: { line: number; character: number } };
          }>
        | {
            uri: string;
            range: { start: { line: number; character: number } };
          };

      const location = firstLocation(definition);
      assert.equal(location.uri, pathToFileURL(path.join(workspaceDir, "Child.vue")).href);
      assert.deepEqual(location.range.start, { line: 0, character: 0 });
    });

    await t.test("hover and definition stay correct in non-default art variants", async () => {
      const secondaryLabelOffset =
        artSource.lastIndexOf("secondaryLabel") + "secondaryLabel".length;

      const hover = (await session.request("textDocument/hover", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, secondaryLabelOffset),
      })) as { contents?: unknown } | null;

      const hoverText = hoverToText(hover);
      assert.match(hoverText, /secondaryLabel/);
      assert.match(hoverText, /(Ref<string>|string)/);

      const definition = (await session.request("textDocument/definition", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, secondaryLabelOffset),
      })) as
        | Array<{
            uri: string;
            range: { start: { line: number; character: number } };
          }>
        | {
            uri: string;
            range: { start: { line: number; character: number } };
          };

      const location = firstLocation(definition);
      assert.equal(location.uri, artUri);
      assert.deepEqual(location.range.start, { line: 4, character: 6 });
    });

    await t.test("completion surfaces bindings and directives inside art variants", async () => {
      const completionOffset = artSource.lastIndexOf("secondaryLabel") + "secondaryLabel".length;

      const response = (await session.request("textDocument/completion", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, completionOffset),
      })) as Array<{ label: string }> | { items?: Array<{ label: string }> } | null;

      const labels = completionLabels(response);
      assert.ok(labels.includes("secondaryLabel"), labels.join(", "));
      assert.ok(labels.includes("primaryLabel"), labels.join(", "));
      assert.ok(labels.includes("v-if"), labels.join(", "));
    });

    await t.test("definition and references use UTF-16 LSP coordinates", async () => {
      const utf16Source = `<script setup lang="ts">
const emoji = "😀"; const message = ref(emoji)
</script>

<template>
  <p>{{ message }}</p>
</template>
`;
      const utf16Path = path.join(workspaceDir, "Utf16.vue");
      fs.writeFileSync(utf16Path, utf16Source, "utf8");
      const utf16Uri = pathToFileURL(utf16Path).href;

      session.notify("textDocument/didOpen", {
        textDocument: {
          uri: utf16Uri,
          languageId: "vue",
          version: 1,
          text: utf16Source,
        },
      });

      await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
        isDiagnosticsForUri(params, utf16Uri),
      );

      const usageOffset = utf16Source.lastIndexOf("message") + "message".length;
      const declarationOffset = utf16Source.indexOf("message =");
      const declarationPosition = offsetToPosition(utf16Source, declarationOffset);

      const definition = (await session.request("textDocument/definition", {
        textDocument: { uri: utf16Uri },
        position: offsetToPosition(utf16Source, usageOffset),
      })) as
        | Array<{
            uri: string;
            range: { start: { line: number; character: number } };
          }>
        | {
            uri: string;
            range: { start: { line: number; character: number } };
          };

      const location = firstLocation(definition);
      assert.equal(location.uri, utf16Uri);
      assert.deepEqual(location.range.start, declarationPosition);

      const references = (await session.request("textDocument/references", {
        textDocument: { uri: utf16Uri },
        position: offsetToPosition(utf16Source, usageOffset),
        context: {
          includeDeclaration: true,
        },
      })) as Array<{ uri: string; range: { start: { line: number; character: number } } }>;

      assert.ok(
        references.some(
          (reference) =>
            reference.uri === utf16Uri &&
            reference.range.start.line === declarationPosition.line &&
            reference.range.start.character === declarationPosition.character,
        ),
        JSON.stringify(references),
      );
    });

    await t.test("semantic token range requests are implemented", async () => {
      const response = (await session.request("textDocument/semanticTokens/range", {
        textDocument: { uri: artUri },
        range: {
          start: { line: 8, character: 0 },
          end: { line: 11, character: 0 },
        },
      })) as { data?: unknown[] } | null;

      assert.ok(response);
      assert.ok(Array.isArray(response.data));
      assert.equal(response.data.length % 5, 0);
      assert.ok(response.data.length > 0);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(agentOnlyDir, { recursive: true, force: true });
  }
});

test("vize lsp returns empty results for unopened and closed editor documents", async () => {
  const agentOnlyDir = path.join(root, "__agent_only", "lsp-empty-editor-docs");
  fs.mkdirSync(agentOnlyDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(agentOnlyDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: true,
    });

    const unopenedUri = pathToFileURL(path.join(workspaceDir, "NeverOpened.vue")).href;
    await assertEmptyEditorRequests(session, unopenedUri, "an unopened document");

    const closedSource = `<script setup lang="ts">
const message = 'hello'
</script>

<template>
  {{ message }}
</template>
`;
    const closedPath = path.join(workspaceDir, "Closed.vue");
    const closedUri = pathToFileURL(closedPath).href;
    fs.writeFileSync(closedPath, closedSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: closedUri,
        languageId: "vue",
        version: 1,
        text: closedSource,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, closedUri),
    );

    session.notify("textDocument/didClose", {
      textDocument: {
        uri: closedUri,
      },
    });

    const closePublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, closedUri),
    )) as PublishDiagnosticsParams;
    assert.deepEqual(closePublish.diagnostics, []);

    await assertEmptyEditorRequests(session, closedUri, "a closed document");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(agentOnlyDir, { recursive: true, force: true });
  }
});

test("vize lsp publishes and clears malformed SFC diagnostics", async () => {
  const agentOnlyDir = path.join(root, "__agent_only", "lsp-malformed");
  fs.mkdirSync(agentOnlyDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(agentOnlyDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      lint: true,
      typecheck: false,
    });

    const brokenPath = path.join(workspaceDir, "Broken.vue");
    const brokenUri = pathToFileURL(brokenPath).href;
    const brokenSource = "<template><div></div>";
    fs.writeFileSync(brokenPath, brokenSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: brokenUri,
        languageId: "vue",
        version: 1,
        text: brokenSource,
      },
    });

    const brokenPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => hasDiagnosticSource(params, brokenUri, "vize/sfc"),
    )) as PublishDiagnosticsParams;
    const parserDiagnostic = brokenPublish.diagnostics.find(
      (diagnostic) => diagnostic.source === "vize/sfc",
    );

    assert.ok(parserDiagnostic, JSON.stringify(brokenPublish.diagnostics));
    assert.equal(parserDiagnostic.severity, 1);
    assert.match(parserDiagnostic.message ?? "", /template/i);
    assertDiagnosticRange(parserDiagnostic);

    const fixedSource = `<template>
  <div>fixed</div>
</template>
`;
    session.notify("textDocument/didChange", {
      textDocument: {
        uri: brokenUri,
        version: 2,
      },
      contentChanges: [
        {
          text: fixedSource,
        },
      ],
    });

    const fixedPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, brokenUri),
    )) as PublishDiagnosticsParams;

    assert.deepEqual(fixedPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(agentOnlyDir, { recursive: true, force: true });
  }
});

test("vize lsp keeps type diagnostics disabled by initialization options", async () => {
  const agentOnlyDir = path.join(root, "__agent_only", "lsp-typecheck-disabled");
  fs.mkdirSync(agentOnlyDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(agentOnlyDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      lint: true,
      typecheck: false,
    });

    const typeErrorPath = path.join(workspaceDir, "TypeError.vue");
    const typeErrorUri = pathToFileURL(typeErrorPath).href;
    const typeErrorSource = `<script setup lang="ts">
const label: string = 1
const items = [1, 2]
</script>

<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`;
    fs.writeFileSync(typeErrorPath, typeErrorSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: typeErrorUri,
        languageId: "vue",
        version: 1,
        text: typeErrorSource,
      },
    });

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, typeErrorUri),
    )) as PublishDiagnosticsParams;

    assertNoDiagnosticSource(publish.diagnostics, "vize/types");
    assertNoDiagnosticSource(publish.diagnostics, "vize/corsa");
    assert.ok(
      publish.diagnostics.some(
        (diagnostic) =>
          diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
      ),
      JSON.stringify(publish.diagnostics),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(agentOnlyDir, { recursive: true, force: true });
  }
});
