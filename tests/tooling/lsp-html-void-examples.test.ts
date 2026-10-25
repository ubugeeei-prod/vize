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
  const referenceContents = new Map<string, unknown>();
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
    for (const templateLang of ["html", "pug", "jade"]) {
      for (const tag of [...voidTags, "button", "div", "textarea", "svg", "math"]) {
        for (const ending of ["\n", "\r\n"]) {
          const isVoid = voidTags.includes(tag);
          const body =
            templateLang === "html"
              ? `<${tag} id="sample">${isVoid ? "" : `</${tag}>`}`
              : `${tag}(id="sample")`;
          const source =
            `<!-- 日本 🧭 -->\n<template lang="${templateLang}">\n${body}\n</template>\n`.replaceAll(
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
            ["tag", source.indexOf(body) + (templateLang === "html" ? 1 : 0), ""],
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
            })) as { contents: { kind: string; value: string }; range?: unknown };
            assert.equal(result.contents.kind, "markdown");
            const key = `${tag}/${kind}`;
            if (referenceContents.has(key))
              assert.deepEqual(
                result.contents,
                referenceContents.get(key),
                `${templateLang}: complete Markdown parity`,
              );
            else referenceContents.set(key, result.contents);
            assert.deepEqual(
              result.range,
              {
                start: position,
                end: {
                  line: position.line,
                  character: position.character + (kind === "tag" ? tag.length : 2),
                },
              },
              `${templateLang}: authored range`,
            );
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
    }
    assert.equal(requests, 220);
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
