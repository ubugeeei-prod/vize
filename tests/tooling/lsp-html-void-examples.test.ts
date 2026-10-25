import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// Independent authoring grammar: https://html.spec.whatwg.org/multipage/syntax.html#void-elements
const voidTags = [
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "source",
  "track",
  "wbr",
];

test("native HTML hover examples preserve complete void and paired forms", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-html-void-examples-"));
  const session = new LspSession();
  let requests = 0;
  try {
    await session.initialize(workspace, { editor: true, typecheck: false, lint: false });
    const fixture = loadLspManifest(
      path.join(root, "tests/_fixtures/differential/lsp/manifest.json"),
    ).cases.find((row) => row.id === "lsp/regression/html-void-hover-examples");
    assert.ok(fixture);
    for (const file of fixture.files) {
      const target = path.join(workspace, file.runtimePath);
      const uri = pathToFileURL(target).href;
      const text = file.bytes.toString("utf8");
      fs.writeFileSync(target, text);
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text },
      });
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri),
        120_000,
      );
      for (const request of fixture.requests) {
        const result = await session.request("textDocument/hover", {
          ...request.params,
          textDocument: { uri },
        });
        assert.deepEqual(result, request.result, `${file.runtimePath}: complete Hover`);
        requests += 1;
      }
      session.notify("textDocument/didClose", { textDocument: { uri } });
    }
    for (const tag of [...voidTags, "button", "div", "textarea", "svg", "math"]) {
      for (const ending of ["\n", "\r\n"]) {
        const isVoid = voidTags.includes(tag);
        const source =
          `<!-- 日本 🧭 -->\n<template>\n<${tag} id="sample">${isVoid ? "" : `</${tag}>`}\n</template>\n`.replaceAll(
            "\n",
            ending,
          );
        const target = path.join(workspace, `Case${requests}.vue`);
        const uri = pathToFileURL(target).href;
        fs.writeFileSync(target, source);
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri),
          120_000,
        );
        const selections: [string, number, string][] = [
          ["tag", source.indexOf(`<${tag} `) + 1, ""],
          ["attribute", source.indexOf('id="sample"'), ' id="..."'],
        ];
        for (const [kind, at, attribute] of selections) {
          const preceding = source.slice(0, at);
          const position = {
            line: preceding.split("\n").length - 1,
            character: preceding.length - preceding.lastIndexOf("\n") - 1,
          };
          const result = (await session.request("textDocument/hover", {
            textDocument: { uri },
            position,
          })) as { contents: { kind: string; value: string } };
          assert.equal(result.contents.kind, "markdown");
          const examples = [
            ...result.contents.value.matchAll(/```vue\r?\n([\s\S]*?)\r?\n```/g),
          ].map((match) => match[1]);
          const expected = `<template>\n  <${tag}${attribute}>${isVoid ? "" : `...</${tag}>`}\n</template>`;
          assert.deepEqual(examples, [expected], `${tag}/${kind}/${JSON.stringify(ending)}`);
          const syntax = result.contents.value
            .split("\n\n**HTML syntax**\n\n")
            .slice(1)
            .map((part) => part.split("\n\n")[0]);
          assert.deepEqual(
            syntax,
            isVoid && kind === "tag"
              ? ["Void elements do not have child content or an end tag."]
              : [],
          );
          requests += 1;
        }
        session.notify("textDocument/didClose", { textDocument: { uri } });
      }
    }
    assert.equal(requests, 76);
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
