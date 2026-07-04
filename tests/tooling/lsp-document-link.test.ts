import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type DocumentLink = {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  target?: string;
};

test("vize lsp documentLink resolves relative imports and ranges", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-document-link");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    // Dependency component referenced by a relative import. The link target
    // resolves to this file's canonical URL once it exists on disk.
    const depPath = path.join(workspaceDir, "Dep.vue");
    fs.writeFileSync(
      depPath,
      `<script setup lang="ts"></script>
<template><span /></template>
`,
      "utf8",
    );
    const modulePath = path.join(workspaceDir, "useServer.mjs");
    fs.writeFileSync(modulePath, "export const useServer = () => 1\n", "utf8");

    const componentImportLine = `import Dep from './Dep.vue'`;
    const moduleImportLine = `import { useServer } from './useServer'`;
    const source = `<script setup lang="ts">
${componentImportLine}
${moduleImportLine}
import { ref } from 'vue'
const _x = ref(0)
</script>

<template>
  <Dep />
</template>
`;
    const filePath = path.join(workspaceDir, "Host.vue");
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

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    await t.test(
      "resolves a relative component import to an absolute file URL and skips bare imports",
      async () => {
        const links = (await session.request("textDocument/documentLink", {
          textDocument: { uri },
        })) as DocumentLink[] | null;

        assert.ok(Array.isArray(links), JSON.stringify(links));
        assert.equal(links.length, 2, JSON.stringify(links));

        const targets = links.map((link) => {
          assert.ok(link.target, JSON.stringify(link));
          return path.basename(decodeURIComponent(new URL(link.target).pathname));
        });
        assert.deepEqual(targets.sort(), ["Dep.vue", "useServer.mjs"]);
        assert.ok(
          links.every((link) => link.range.start.line !== 3),
          JSON.stringify(links),
        );
      },
    );

    await t.test("link range covers the quoted import string including both quotes", async () => {
      const links = (await session.request("textDocument/documentLink", {
        textDocument: { uri },
      })) as DocumentLink[] | null;

      assert.ok(Array.isArray(links), JSON.stringify(links));
      assert.equal(links.length, 2, JSON.stringify(links));

      // Compute expected columns from the source rather than hardcoding: the
      // range spans the opening quote through the closing quote inclusive.
      const lineStartOffset = source.indexOf(componentImportLine);
      const quoteStartOffset = source.indexOf("'", lineStartOffset);
      const quoteEndOffset = source.indexOf("'", quoteStartOffset + 1) + 1;

      const expectedStart = offsetToPosition(source, quoteStartOffset);
      const expectedEnd = offsetToPosition(source, quoteEndOffset);
      const componentLink = links.find((link) => link.target?.endsWith("/Dep.vue"));
      assert.ok(componentLink, JSON.stringify(links));

      assert.deepEqual(componentLink.range.start, expectedStart);
      assert.deepEqual(componentLink.range.end, expectedEnd);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
