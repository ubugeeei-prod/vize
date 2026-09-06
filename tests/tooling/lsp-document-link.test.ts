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

function offsetForPosition(source: string, position: { line: number; character: number }): number {
  let offset = 0;
  for (let line = 0; line < position.line; line += 1) {
    const nextLine = source.indexOf("\n", offset);
    assert.notEqual(nextLine, -1, `line ${position.line} is outside source`);
    offset = nextLine + 1;
  }
  return offset + position.character;
}

function textAtLinkRange(source: string, link: DocumentLink): string {
  return source.slice(
    offsetForPosition(source, link.range.start),
    offsetForPosition(source, link.range.end),
  );
}

function basenameForTarget(link: DocumentLink): string {
  assert.ok(link.target, JSON.stringify(link));
  return path.basename(decodeURIComponent(new URL(link.target).pathname));
}

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

test("vize lsp documentLink handles multiline imports and ignores inactive specifiers", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-document-link-script-comments");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    for (const fileName of ["Multi.vue", "Real.vue", "Exported.vue"]) {
      fs.writeFileSync(
        path.join(workspaceDir, fileName),
        `<script setup lang="ts"></script>
<template><span /></template>
`,
        "utf8",
      );
    }

    const source = `<script setup lang="ts">
/* import Block from './Block.vue' */
// import Ghost from './Ghost.vue'
const note = "import Hidden from './Hidden.vue'"
import {
  real /* from './CommentOnly.vue' */,
} from './Multi.vue'
import Real from './Real.vue'
export {
  default as Exported /* from './FakeExport.vue' */,
} from './Exported.vue'
</script>

<template>
  <Real />
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

    const links = (await session.request("textDocument/documentLink", {
      textDocument: { uri },
    })) as DocumentLink[] | null;

    assert.ok(Array.isArray(links), JSON.stringify(links));
    assert.deepEqual(links.map(basenameForTarget), ["Multi.vue", "Real.vue", "Exported.vue"]);
    assert.deepEqual(links.map((link) => textAtLinkRange(source, link)), [
      "'./Multi.vue'",
      "'./Real.vue'",
      "'./Exported.vue'",
    ]);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp documentLink resolves SFC block src attributes and CSS imports", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-document-link-src-css");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    fs.mkdirSync(path.join(workspaceDir, "partials"), { recursive: true });
    fs.mkdirSync(path.join(workspaceDir, "styles"), { recursive: true });
    fs.writeFileSync(path.join(workspaceDir, "partials", "card.html"), "<section />\n", "utf8");
    fs.writeFileSync(path.join(workspaceDir, "entry.ts"), "export const value = 1\n", "utf8");
    fs.writeFileSync(path.join(workspaceDir, "styles", "card.css"), ".card {}\n", "utf8");
    fs.writeFileSync(
      path.join(workspaceDir, "styles", "reset.css"),
      "* { box-sizing: border-box }\n",
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "styles", "theme.css"),
      ":root { color: black }\n",
      "utf8",
    );

    const source = `<template src="./partials/card.html"></template>
<script src="./entry.ts"></script>
<style src="./styles/card.css"></style>
<style>
@import "./styles/reset.css";
@import url("./styles/theme.css");
@import "https://cdn.example.test/remote.css";
</style>
`;
    const filePath = path.join(workspaceDir, "LinkedBlocks.vue");
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

    const links = (await session.request("textDocument/documentLink", {
      textDocument: { uri },
    })) as DocumentLink[] | null;
    assert.ok(Array.isArray(links), JSON.stringify(links));

    await t.test("links only local block and CSS targets", async () => {
      assert.deepEqual(links.map(basenameForTarget).sort(), [
        "card.css",
        "card.html",
        "entry.ts",
        "reset.css",
        "theme.css",
      ]);
      assert.equal(
        links.some((link) => link.target?.includes("cdn.example.test")),
        false,
        JSON.stringify(links),
      );
    });

    await t.test("block src ranges cover only the attribute value", async () => {
      const templateLink = links.find((link) => basenameForTarget(link) === "card.html");
      const scriptLink = links.find((link) => basenameForTarget(link) === "entry.ts");
      const styleLink = links.find((link) => basenameForTarget(link) === "card.css");

      assert.ok(templateLink, JSON.stringify(links));
      assert.ok(scriptLink, JSON.stringify(links));
      assert.ok(styleLink, JSON.stringify(links));
      assert.equal(textAtLinkRange(source, templateLink), "./partials/card.html");
      assert.equal(textAtLinkRange(source, scriptLink), "./entry.ts");
      assert.equal(textAtLinkRange(source, styleLink), "./styles/card.css");
    });

    await t.test("CSS import ranges keep the quoted specifier selected", async () => {
      const resetLink = links.find((link) => basenameForTarget(link) === "reset.css");
      const themeLink = links.find((link) => basenameForTarget(link) === "theme.css");

      assert.ok(resetLink, JSON.stringify(links));
      assert.ok(themeLink, JSON.stringify(links));
      assert.equal(textAtLinkRange(source, resetLink), '"./styles/reset.css"');
      assert.equal(textAtLinkRange(source, themeLink), '"./styles/theme.css"');
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
