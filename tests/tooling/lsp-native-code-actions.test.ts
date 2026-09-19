import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import type { LspRange } from "./support/lsp/protocol.ts";

type Action = {
  title: string;
  kind: string;
  edit: {
    documentChanges: Array<{
      textDocument: { uri: string; version: number };
      edits: Array<{ range: LspRange; newText: string }>;
    }>;
  };
};
const declaration =
  "interface Greeting { greet(name: string): string; }\nclass Welcome implements Greeting {}";
const cases = {
  setup: `<script setup lang="ts">\n${declaration}\n</script>`,
  normal: `<script lang="ts">\n${declaration}\nexport { Welcome };\n</script>`,
  inline: `<script setup lang="ts">const marker = "😀"; ${declaration.replaceAll("\n", " ")}</script>`,
  crlf: `<!-- 😀 -->\r\n<script setup lang="ts">\r\n${declaration.replaceAll("\n", "\r\n")}\r\n</script>`,
  tsx: `<script setup lang="tsx">\n${declaration}\nconst node = <div />;\n</script>`,
};

function offset(source: string, position: LspRange["start"]): number {
  const lines = source.split("\n");
  return (
    lines.slice(0, position.line).reduce((length, line) => length + line.length + 1, 0) +
    position.character
  );
}

test("checker quick fixes apply to the current authored Vue buffer without lint", async (t) => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "native-code-actions-"));
  const vue = path.dirname(createRequire(import.meta.url).resolve("vue/package.json"));
  fs.mkdirSync(path.join(workspace, "node_modules"));
  const linkType = process.platform === "win32" ? "junction" : "dir";
  fs.symlinkSync(vue, path.join(workspace, "node_modules/vue"), linkType);
  fs.symlinkSync(
    path.join(path.dirname(vue), "@vue"),
    path.join(workspace, "node_modules/@vue"),
    linkType,
  );
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      include: ["**/*.vue"],
    }),
  );
  const session = new LspSession();
  const file = path.join(workspace, "App.vue");
  const uri = pathToFileURL(file).href;
  fs.writeFileSync(file, cases.setup);
  const request = (range: LspRange, only = ["quickfix"]) =>
    session.request("textDocument/codeAction", {
      textDocument: { uri },
      range,
      context: { diagnostics: [], only },
    }) as Promise<Action[] | null>;
  try {
    await session.initialize(workspace, {
      editor: true,
      lint: false,
      typecheck: true,
      codeActions: true,
    });
    for (const [name, source] of Object.entries(cases)) {
      await t.test(name, async () => {
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (p) =>
            isDiagnosticsForUri(p, uri) &&
            p.version === 1 &&
            p.diagnostics.some((d) => String(d.code) === "2420"),
        );
        const start = offsetToPosition(source, source.indexOf("Welcome implements"));
        const range = { start, end: { ...start, character: start.character + "Welcome".length } };
        const actions = await request(range);
        assert.deepEqual(
          actions?.map(({ title, kind }) => ({ title, kind })),
          [{ title: "Implement interface 'Greeting'", kind: "quickfix" }],
        );
        assert.equal(await request(range, ["source"]), null);
        assert.equal(await request(range, []), null);
        assert.equal(
          await request({ start: { line: 0, character: 1 }, end: { line: 0, character: 1 } }),
          null,
        );
        const changes = actions![0].edit.documentChanges;
        assert.equal(changes.length, 1);
        assert.deepEqual(changes[0].textDocument, { uri, version: 1 });
        assert.equal(changes[0].edits.length, 1, "same-position backend insertions are coalesced");
        const edit = changes[0].edits[0];
        const insertion = source.indexOf("{}", source.indexOf("class Welcome")) + 1;
        const expected = offsetToPosition(source, insertion);
        assert.deepEqual(edit.range, { start: expected, end: expected });
        assert.match(edit.newText, /greet\(name: string\): string/);
        assert.match(edit.newText, /throw new Error\(["']Method not implemented\.["']\)/);
        assert.doesNotMatch(JSON.stringify(actions), /__Vize|__vize|\.vue\.ts|backend\.apply/);
        const repaired =
          source.slice(0, offset(source, edit.range.start)) +
          edit.newText +
          source.slice(offset(source, edit.range.end));
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: repaired }],
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (p) =>
            isDiagnosticsForUri(p, uri) &&
            p.version === 2 &&
            p.diagnostics.every((d) => d.severity !== 1),
        );
        assert.equal(
          await request(range),
          null,
          "a stale diagnostic cannot recreate the applied fix",
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 3 },
          contentChanges: [{ text: source }],
        });
        const fresh = await request(range);
        assert.equal(fresh?.[0].edit.documentChanges[0].textDocument.version, 3);
        session.notify("textDocument/didClose", { textDocument: { uri } });
        assert.equal(await request(range), null);
      });
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
