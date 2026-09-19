import assert from "node:assert/strict";
import { test } from "node:test";
import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";
import { PatternSession, referenceLspSources } from "./support/upstream/pattern-lsp.ts";

const fixtures = referenceLspSources();
const source = fixtures.get("source")!;

test(
  "all upstream patterned-template editor cases use the real LSP",
  { timeout: 240_000 },
  async (t) => {
    const editor = new PatternSession();
    const hover = async (offset: number) =>
      hoverToText((await editor.request("hover", offset)) as Parameters<typeof hoverToText>[0]);
    const completion = async (offset: number) => {
      const result = (await editor.request("completion", offset)) as
        | { items: Array<{ label: string }> }
        | Array<{ label: string }>;
      return (Array.isArray(result) ? result : result.items).map((item) => item.label);
    };
    const definition = async (offset: number, declaration: number, name: string) => {
      const actual = firstLocation(
        (await editor.request("definition", offset)) as Parameters<typeof firstLocation>[0],
      );
      assert.deepEqual(actual, {
        uri: editor.uri,
        range: {
          start: offsetToPosition(editor.source, declaration),
          end: offsetToPosition(editor.source, declaration + name.length),
        },
      });
    };
    try {
      await editor.initialize();
      await t.test("binding hover, guard completion, definition and rename", async () => {
        assert.deepEqual(await editor.update(source), []);
        assert.match(
          await hover(source.indexOf("article.length")),
          /(?:const|let) article: string/,
        );
        const labels = await completion(source.indexOf("article.length") + "article.".length);
        assert.ok(labels.includes("toUpperCase"));
        assert.ok(!labels.includes("toFixed"));
        await definition(
          source.lastIndexOf("article"),
          source.indexOf("const article") + 6,
          "article",
        );
        assert.deepEqual(
          await editor.rename(source.lastIndexOf("article")),
          [...source.matchAll(/\barticle\b/g)].map((m) => m.index),
        );
      });
      await t.test("missing coverage diagnoses and clears after edits", async () => {
        const missing = source.replace(
          `<p v-when="{ kind: 'error', const error }">{{ error.message }}</p>`,
          "",
        );
        const diagnostics = await editor.update(missing);
        assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
        assert.match(diagnostics[0].message, /Non-exhaustive v-match/);
        assert.equal(String(diagnostics[0].code), "2322");
        assert.deepEqual(await editor.update(source), []);
      });
      await t.test("unreachable arms are warnings", async () => {
        const text = fixtures.get(
          "type-aware unreachable arms warn without becoming vue-tsc errors",
        )!;
        const diagnostics = await editor.update(text);
        assert.equal(diagnostics.length, 2, JSON.stringify(diagnostics));
        for (const diagnostic of diagnostics) {
          assert.equal(diagnostic.severity, 2);
          assert.match(diagnostic.message, /Unreachable v-when/);
        }
      });
      await t.test(
        "shadowed bindings retain their lexical declaration and references",
        async () => {
          const text = fixtures.get("definition and rename distinguish shadowed bindings")!;
          assert.deepEqual(await editor.update(text), []);
          const declaration = text.indexOf("{ const value") + "{ const ".length;
          const reference = text.indexOf("value.toUpperCase");
          await definition(reference, declaration, "value");
          assert.deepEqual(await editor.rename(reference), [
            declaration - "const ".length,
            text.indexOf("value.length"),
            text.indexOf(':title="value"') + ':title="'.length,
            reference,
            text.indexOf("in value") + "in ".length,
            text.indexOf("</b>{{ value") + "</b>{{ ".length,
          ]);
          assert.match(
            await hover(text.indexOf("value.toLowerCase")),
            /(?:const|let) value: string/,
          );
          // The upstream reference checks rename locations. Vize expands the
          // declaration edit to preserve the authored shorthand property key.
          const edits = editor.lastRenameEdits
            .map((edit) => {
              const offset = (position: typeof edit.range.start) =>
                text
                  .split("\n")
                  .slice(0, position.line)
                  .reduce((n, line) => n + line.length + 1, 0) + position.character;
              return { ...edit, start: offset(edit.range.start), end: offset(edit.range.end) };
            })
            .sort((a, b) => b.start - a.start);
          let renamed = text;
          for (const edit of edits)
            renamed = renamed.slice(0, edit.start) + edit.newText + renamed.slice(edit.end);
          assert.match(renamed, /\{ value: const renamed \}/);
          assert.ok(renamed.includes('v-for="value in renamed"'));
          assert.ok(renamed.includes("value.toLowerCase()"));
          assert.ok(renamed.includes("<footer>{{ value }}</footer>"));
          assert.deepEqual(await editor.update(renamed), []);
        },
      );
      await t.test("duplicate binding is a single authored diagnostic", async () => {
        const diagnostics = await editor.update(
          fixtures.get("editor rejects repeated names within one pattern")!,
        );
        assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
        assert.match(diagnostics[0].message, /Duplicate pattern binding value/);
      });
      for (const lang of ["", ' lang="html"'])
        await t.test(`top-level header navigation and completion (${lang})`, async () => {
          const text = source
            .replace("<template><template v-match", `<template${lang} v-match`)
            .replace("</template></template>", "</template>");
          assert.deepEqual(await editor.update(text), []);
          const offset = text.indexOf('v-match="result"') + 'v-match="'.length;
          assert.match(await hover(offset), /result:/);
          await definition(offset, text.indexOf("result:"), "result");
          assert.deepEqual(await editor.rename(offset), [text.indexOf("result:"), offset]);
          assert.ok((await completion(offset + 3)).includes("result"));
          assert.match(
            await hover(text.indexOf("article.length")),
            /(?:const|let) article: string/,
          );
        });
      await t.test(
        "header-only edits invalidate coverage and authored diagnostic ranges",
        async () => {
          const text = fixtures.get("header-only edits refresh coverage and diagnostic locations")!;
          assert.deepEqual(await editor.update(text), []);
          for (const header of ['v-match="b"', 'lang="html" v-match="b"']) {
            const edited = text.replace('v-match="a"', header);
            const diagnostics = await editor.update(edited);
            assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
            assert.match(diagnostics[0].message, /Non-exhaustive v-match/);
            const offset = edited.indexOf('v-match="b"') + 'v-match="'.length;
            assert.deepEqual(diagnostics[0].range, {
              start: offsetToPosition(edited, offset),
              end: offsetToPosition(edited, offset + 1),
            });
          }
          assert.deepEqual(await editor.update(text), []);
        },
      );
      await t.test("long-form directive names are completed", async () => {
        const text = "<template><div v- /></template>";
        await editor.update(text);
        const labels = await completion(text.indexOf("v-") + 2);
        assert.ok(labels.includes("v-match"));
        assert.ok(labels.includes("v-when"));
      });
    } finally {
      await editor.close();
    }
  },
);
