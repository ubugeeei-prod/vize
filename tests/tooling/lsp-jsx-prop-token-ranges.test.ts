import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

test("JSX and TSX prop hover, references and rename preserve complete authored tokens", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript runtime for JSX prop token ranges",
    "TypeScript runtime unavailable",
  );
  if (!corsaPath) return;
  const vue = requireTypecheckDependency(
    t,
    [
      process.env.VIZE_TEST_VUE_PACKAGE,
      path.join(root, "node_modules/vue"),
      path.join(root, "tests/node_modules/vue"),
    ].find((candidate) => candidate && fs.existsSync(candidate)),
    "real Vue declarations for JSX prop token ranges",
    "Vue declarations unavailable",
  );
  if (!vue) return;
  for (const [extension, languageId] of [
    ["tsx", "typescriptreact"],
    ["jsx", "javascriptreact"],
  ]) {
    for (const newline of ["\n", "\r\n"]) {
      await t.test(`${extension} ${newline === "\n" ? "LF" : "CRLF"}`, async () => {
        fs.mkdirSync(testOutputRoot, { recursive: true });
        const workspace = fs.mkdtempSync(path.join(testOutputRoot, "jsx-prop-token-ranges-"));
        const session = new LspSession();
        try {
          const modules = path.join(workspace, "node_modules");
          fs.mkdirSync(modules);
          const realVue = fs.realpathSync(vue);
          const kind = process.platform === "win32" ? "junction" : "dir";
          fs.symlinkSync(realVue, path.join(modules, "vue"), kind);
          fs.symlinkSync(
            path.join(path.dirname(realVue), "@vue"),
            path.join(modules, "@vue"),
            kind,
          );
          fs.writeFileSync(
            path.join(workspace, "package.json"),
            '{"private":true,"type":"module"}\n',
          );
          fs.writeFileSync(
            path.join(workspace, "tsconfig.json"),
            JSON.stringify({
              compilerOptions: {
                allowJs: true,
                checkJs: true,
                jsx: "preserve",
                jsxImportSource: "vue",
                module: "ESNext",
                moduleResolution: "bundler",
                noEmit: true,
                strict: true,
                target: "ES2022",
              },
              include: ["*.vue", "*.tsx", "*.jsx"],
            }),
          );
          writeVueConfigFixture(workspace, {
            lsp: { lint: false, typecheck: true },
            typeChecker: { corsaPath, jsxTypecheck: true },
          });
          const child =
            '<script setup lang="ts">\n// 🧭 physical declarations\ndefineProps<{ count: number; fooBar: string; active: boolean; 名称: string }>();\n</script>\n<template><span /></template>\n';
          const childFile = path.join(workspace, "Child.vue");
          const childUri = pathToFileURL(childFile).href;
          fs.writeFileSync(childFile, child);
          await session.initialize(workspace, { editor: true, lint: false, typecheck: true });

          const source =
            '// 🧭 source position\nimport Child from "./Child.vue";\nconst count = 1; const label = "hello";\nexport const view = <div data-note="🧭"><Child count={count} foo-bar={label} active 名称={label} /></div>;\n'.replaceAll(
              "\n",
              newline,
            );
          const file = path.join(workspace, `Consumer-${extension}-${newline.length}.${extension}`);
          const uri = pathToFileURL(file).href;
          fs.writeFileSync(file, source);
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId, version: 1, text: source },
          });
          const publication = await session.waitForNotification(
            "textDocument/publishDiagnostics",
            (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
          );
          assert.deepEqual(
            publication,
            { uri, version: 1, diagnostics: [] },
            "complete diagnostics packet",
          );
          for (const [authored, canonical, type, renamed] of [
            ["count", "count", "number", "renamedCount"],
            ["foo-bar", "fooBar", "string", "renamedLabel"],
            ["active", "active", "boolean", "enabled"],
            ["名称", "名称", "string", "renamedName"],
          ]) {
            const at = source.indexOf(authored === "active" ? "active 名称" : `${authored}=`);
            assert.notEqual(at, -1, `authored ${authored} token exists`);
            const range = rangeAt(source, at, authored.length);
            const params = { textDocument: { uri }, position: range.start };
            const hover = {
              contents: {
                kind: "markdown",
                value: `\`\`\`typescript\n(property) ${canonical}: ${type}\n\`\`\``,
              },
              range,
            };
            assert.deepEqual(
              await session.request("textDocument/hover", params),
              hover,
              `${authored} hover at first character`,
            );
            assert.deepEqual(
              await session.request("textDocument/hover", {
                ...params,
                position: offsetToPosition(source, at + authored.length - 1),
              }),
              hover,
              `${authored} hover at last character`,
            );
            const declaration = {
              uri: childUri,
              range: rangeAt(child, child.indexOf(`${canonical}:`), canonical.length),
            };
            const usage = { uri, range };
            assert.deepEqual(
              await session.request("textDocument/references", {
                ...params,
                context: { includeDeclaration: true },
              }),
              [declaration, usage],
              `${authored} references including declaration`,
            );
            assert.deepEqual(
              await session.request("textDocument/references", {
                ...params,
                context: { includeDeclaration: false },
              }),
              [usage],
              `${authored} references excluding declaration`,
            );
            assert.deepEqual(
              await session.request("textDocument/prepareRename", params),
              {
                range,
                placeholder: authored,
              },
              `${authored} prepare rename`,
            );
            const edit = (await session.request("textDocument/rename", {
              ...params,
              newName: renamed,
            })) as { changes: Record<string, Edit[]> };
            assert.deepEqual(
              edit,
              {
                changes: {
                  [childUri]: [{ range: declaration.range, newText: renamed }],
                  [uri]: [{ range, newText: renamed }],
                },
              },
              `${authored} complete workspace edit`,
            );
            // Apply the exact edit to physical source without changing the files.
            // A dropped first character would leave `crenamedCount` here.
            assert.equal(
              apply(source, edit.changes[uri]),
              source.slice(0, at) + renamed + source.slice(at + authored.length),
              `${authored} edited consumer bytes`,
            );
            assert.equal(
              apply(child, edit.changes[childUri]),
              child.replace(`${canonical}:`, `${renamed}:`),
              `${authored} edited declaration bytes`,
            );
          }
          const binding = source.indexOf("count}");
          const params = { textDocument: { uri }, position: offsetToPosition(source, binding) };
          assert.deepEqual(await session.request("textDocument/hover", params), {
            contents: { kind: "markdown", value: "```typescript\nconst count: 1\n```" },
            range: rangeAt(source, binding, 5),
          });
          assert.deepEqual(await session.request("textDocument/definition", params), {
            uri,
            range: rangeAt(source, source.indexOf("count ="), 5),
          });
          session.notify("textDocument/didClose", { textDocument: { uri } });
        } finally {
          await session.shutdown();
          fs.rmSync(workspace, { recursive: true, force: true });
        }
      });
    }
  }
});

type Position = { line: number; character: number };
type Edit = { range: { start: Position; end: Position }; newText: string };
function rangeAt(source: string, start: number, length: number) {
  return { start: offsetToPosition(source, start), end: offsetToPosition(source, start + length) };
}
function apply(source: string, edits: Edit[]) {
  const offset = (position: Position) => {
    let start = 0;
    for (let line = 0; line < position.line; line++) start = source.indexOf("\n", start) + 1;
    return start + position.character;
  };
  for (const edit of [...edits].sort((a, b) => offset(b.range.start) - offset(a.range.start)))
    source =
      source.slice(0, offset(edit.range.start)) +
      edit.newText +
      source.slice(offset(edit.range.end));
  return source;
}
