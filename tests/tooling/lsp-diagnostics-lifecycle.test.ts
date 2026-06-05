import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  assertNoDiagnosticSource,
  hasDiagnosticSource,
  isDiagnosticsForUri,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

test("vize lsp incremental range didChange publishes diagnostics carrying the new version", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-lifecycle-incremental");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const message = 'hello'
</script>

<template>
  <p>{{ message }}</p>
</template>
`;
    const filePath = path.join(workspaceDir, "Incremental.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    const openPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
    )) as PublishDiagnosticsParams;
    assert.equal(openPublish.version, 1);

    // Replace the contents of the string literal 'hello' -> 'world' using a
    // range-scoped contentChange and bump straight to version 5.
    const helloStart = source.indexOf("hello");
    const helloEnd = helloStart + "hello".length;
    const helloLine = source.slice(0, helloStart).split("\n").length - 1;
    const lineStart = source.lastIndexOf("\n", helloStart - 1) + 1;
    const startCharacter = helloStart - lineStart;
    const endCharacter = helloEnd - lineStart;

    session.notify("textDocument/didChange", {
      textDocument: {
        uri,
        version: 5,
      },
      contentChanges: [
        {
          range: {
            start: { line: helloLine, character: startCharacter },
            end: { line: helloLine, character: endCharacter },
          },
          text: "world",
        },
      ],
    });

    const changePublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 5,
    )) as PublishDiagnosticsParams;
    assert.equal(changePublish.version, 5);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp didClose clears diagnostics and a subsequent reopen re-parses cleanly", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-lifecycle-close-reopen");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const malformedSource = "<template><div></div>";
    const filePath = path.join(workspaceDir, "CloseReopen.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, malformedSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: malformedSource,
      },
    });

    const openPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => hasDiagnosticSource(params, uri, "vize/sfc"),
    )) as PublishDiagnosticsParams;
    assert.equal(openPublish.version, 1);
    assert.ok(
      openPublish.diagnostics.some((diagnostic) => diagnostic.source === "vize/sfc"),
      JSON.stringify(openPublish.diagnostics),
    );

    session.notify("textDocument/didClose", {
      textDocument: { uri },
    });

    const closePublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === undefined,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(closePublish.diagnostics, []);
    assert.equal(closePublish.version, undefined);

    const wellFormedSource = `<template>
  <div>fixed</div>
</template>
`;
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 2,
        text: wellFormedSource,
      },
    });

    const reopenPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
    )) as PublishDiagnosticsParams;
    assert.equal(reopenPublish.version, 2);
    assertNoDiagnosticSource(reopenPublish.diagnostics, "vize/sfc");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp lint-disabled init keeps SFC parse diagnostics but suppresses lint-rule diagnostics", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-lifecycle-lint-gating");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const malformedSource = "<template><div></div>";
    const malformedPath = path.join(workspaceDir, "Malformed.vue");
    const malformedUri = pathToFileURL(malformedPath).href;
    fs.writeFileSync(malformedPath, malformedSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: malformedUri,
        languageId: "vue",
        version: 1,
        text: malformedSource,
      },
    });

    const malformedPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, malformedUri),
    )) as PublishDiagnosticsParams;
    assert.ok(
      malformedPublish.diagnostics.some((diagnostic) => diagnostic.source === "vize/sfc"),
      JSON.stringify(malformedPublish.diagnostics),
    );

    const lintOnlySource = `<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`;
    const lintOnlyPath = path.join(workspaceDir, "LintOnly.vue");
    const lintOnlyUri = pathToFileURL(lintOnlyPath).href;
    fs.writeFileSync(lintOnlyPath, lintOnlySource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: lintOnlyUri,
        languageId: "vue",
        version: 1,
        text: lintOnlySource,
      },
    });

    const lintOnlyPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, lintOnlyUri),
    )) as PublishDiagnosticsParams;
    assertNoDiagnosticSource(lintOnlyPublish.diagnostics, "vize/lint");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp art-vue didOpen publishes diagnostics for a valid art file with version tracking", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-lifecycle-art-vue");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts"></script>
<template><button /></template>
`,
      "utf8",
    );

    const artSource = `<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
</art>
`;
    const artPath = path.join(workspaceDir, "Button.art.vue");
    const artUri = pathToFileURL(artPath).href;
    fs.writeFileSync(artPath, artSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: artUri,
        languageId: "art-vue",
        version: 1,
        text: artSource,
      },
    });

    const artPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, artUri),
    )) as PublishDiagnosticsParams;
    assert.equal(artPublish.version, 1);
    assert.deepEqual(artPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
