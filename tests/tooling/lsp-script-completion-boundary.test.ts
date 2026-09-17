import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { completionLabels, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Item = { label: string; kind?: number; data?: unknown; detail?: string; insertText?: string };
const rootLabels = ["script", "script setup", "style", "style scoped", "template"];
const declaration = "function greetVisitor(name: string): string { return name; }";
const script = `${declaration} const greeting = greetVis`;
const layouts = {
  inline: script,
  unicode: `const marker = "\u96ea\u{1f600}"; ${script}`,
  crlf: `\r\n${declaration}\r\nconst marker = "\u96ea\u{1f600}";\r\nconst greeting = greetVis`,
};

for (const typecheck of [true, false]) {
  test(`script completion insertion boundaries (typecheck=${typecheck})`, async (t) => {
    fs.mkdirSync(testOutputRoot, { recursive: true });
    const workspace = fs.mkdtempSync(path.join(testOutputRoot, "script-boundary-"));
    const vue = path.dirname(createRequire(import.meta.url).resolve("vue/package.json"));
    fs.mkdirSync(path.join(workspace, "node_modules"));
    for (const [name, target] of [
      ["vue", vue],
      ["@vue", path.join(path.dirname(vue), "@vue")],
    ]) {
      fs.symlinkSync(
        target,
        path.join(workspace, "node_modules", name),
        process.platform === "win32" ? "junction" : "dir",
      );
    }
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
    const file = path.join(workspace, "App.vue");
    fs.writeFileSync(file, '<script setup lang="ts"></script>');
    const uri = pathToFileURL(file).href;
    const session = new LspSession();
    const open = (text: string) =>
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text },
      });
    const close = () => session.notify("textDocument/didClose", { textDocument: { uri } });
    const complete = async (text: string, offset: number): Promise<Item[]> => {
      const response = (await session.request("textDocument/completion", {
        textDocument: { uri },
        position: offsetToPosition(text, offset),
      })) as Item[] | { items: Item[] } | null;
      return response === null ? [] : Array.isArray(response) ? response : response.items;
    };
    const assertScript = (items: Item[], setup: boolean) => {
      const labels = completionLabels(items);
      assert.deepEqual(
        items.filter((item) => item.kind === 15 && rootLabels.includes(item.label)),
        [],
        "Vue's template function is not a root-block snippet",
      );
      assert.ok(labels.includes("ref"), labels.join(", "));
      assert.equal(
        items.some((item) => item.label === "defineProps" && item.insertText?.includes("$1")),
        setup,
      );
    };
    try {
      await session.initialize(workspace, { editor: true, typecheck, lint: false });
      for (const setup of [false, true]) {
        const opening = `<script${setup ? " setup" : ""} lang="ts">`;
        for (const [layout, body] of Object.entries(layouts)) {
          await t.test(`${setup ? "setup" : "normal"} ${layout}`, async () => {
            const source = `${opening}${body}</script>`;
            const end = source.indexOf("</script>");
            open(source);
            for (const offset of [end - 1, end]) {
              const items = await complete(source, offset);
              assertScript(items, setup);
              const candidate = items.find((item) => item.label === "greetVisitor");
              assert.ok(candidate, JSON.stringify(items));
              if (typecheck)
                assert.ok(candidate.data, "native checker resolution data is retained");
            }
            for (const offset of [opening.length - 1, end + 1, source.length]) {
              assert.deepEqual(completionLabels(await complete(source, offset)).sort(), rootLabels);
            }
            close();
          });
        }
        for (const body of ["", "\r\n"]) {
          await t.test(`${setup ? "setup" : "normal"} empty ${JSON.stringify(body)}`, async () => {
            const source = `${opening}${body}</script>`;
            open(source);
            assertScript(await complete(source, source.indexOf("</script>")), setup);
            const edited = `${opening}${script}</script>`;
            session.notify("textDocument/didChange", {
              textDocument: { uri, version: 2 },
              contentChanges: [{ text: edited }],
            });
            const items = await complete(edited, edited.indexOf("</script>"));
            assert.ok(
              items.some((item) => item.label === "greetVisitor"),
              JSON.stringify(items),
            );
            session.notify("textDocument/didChange", {
              textDocument: { uri, version: 3 },
              contentChanges: [{ text: source }],
            });
            const repaired = await complete(source, source.indexOf("</script>"));
            assertScript(repaired, setup);
            assert.equal(
              repaired.some((item) => item.label === "greetVisitor"),
              false,
            );
            close();
          });
        }
        await t.test(`${setup ? "setup" : "normal"} hover and definition boundaries`, async () => {
          const source = `${opening}${declaration} const greeting = greetVisitor</script>`;
          const end = source.indexOf("</script>");
          open(source);
          const request = (method: string, offset: number) =>
            session.request(method, {
              textDocument: { uri },
              position: offsetToPosition(source, offset),
            });
          const hover = (await request("textDocument/hover", end - 1)) as {
            contents: unknown;
            range: unknown;
          };
          assert.match(hoverToText(hover), /\bgreetVisitor\b/);
          assert.deepEqual(
            hover.range,
            typecheck
              ? {
                  start: offsetToPosition(source, source.lastIndexOf("greetVisitor")),
                  end: offsetToPosition(source, end),
                }
              : undefined,
          );
          const definition = await request("textDocument/definition", end - 1);
          assert.deepEqual(Array.isArray(definition) ? definition : [definition], [
            {
              uri,
              range: {
                start: offsetToPosition(source, source.indexOf("greetVisitor")),
                end: offsetToPosition(
                  source,
                  source.indexOf("greetVisitor") + "greetVisitor".length,
                ),
              },
            },
          ]);
          for (const method of ["textDocument/hover", "textDocument/definition"]) {
            assert.equal(
              await request(method, end),
              null,
              `${method} keeps its existing half-open block range`,
            );
            assert.equal(
              await request(method, end + 1),
              null,
              `${method} does not enter the closing tag`,
            );
          }
          close();
        });
      }
    } finally {
      await session.shutdown();
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  });
}
