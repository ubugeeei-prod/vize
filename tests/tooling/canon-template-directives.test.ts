import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { LspSession } from "./support/lsp/session.ts";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";

test(
  "CLI and live editor use the same authored-node diagnostic directives",
  { timeout: 120_000 },
  async () => {
    const editor = new PatternSession();
    const errors = async (source: string) => {
      fs.writeFileSync(editor.file, source);
      const cli = (await check(editor.directory))
        .filter((d) => d.severity === "error")
        .map((d) => ({
          code: d.code,
          line: d.line - 1,
          character: d.column - 1,
        }));
      const lsp = (await editor.update(source))
        .filter((d) => d.severity === 1)
        .map((d) => ({
          code: Number(d.code),
          ...d.range.start,
        }));
      const sort = (a: (typeof cli)[number], b: (typeof cli)[number]) =>
        a.line - b.line || a.character - b.character || a.code - b.code;
      assert.deepEqual(
        cli.sort(sort),
        lsp.sort(sort),
        "CLI and editor must agree after every edit",
      );
      return cli;
    };
    const expected = (source: string, token: string, code = 2339) => ({
      code,
      ...offsetToPosition(source, source.indexOf(token)),
    });
    try {
      await editor.initialize();
      const source = `<script setup lang="ts">const valid = 1;</script>\r\n<template>😀\r\n<!-- @vue-ignore --><div\r\n :id="ignored">{{ child }}</div>\r\n<!-- @vue-expect-error --><div :id="expectedA" :title="expectedB">\r\n<!-- @vue-expect-error -->{{ nested }}\r\n</div>\r\n<!-- @vue-skip --><div :id="skipped">{{ skippedChild }}<!-- @vue-expect-error -->{{ valid }}</div>\r\n<!-- prose @vue-ignore -->{{ visible }}\r\n<div title="<!-- @vue-ignore -->">{{ another }}</div>\r\n</template>`;
      assert.deepEqual(
        await errors(source),
        ["child", "visible", "another"].map((token) => expected(source, token)),
      );

      const expectation = `<script setup lang="ts">const valid = 1;</script>\r\n<template>😀<!-- @vue-expect-error -->\r\n{{ missing }}\r\n</template>`;
      assert.deepEqual(await errors(expectation), []);
      const repaired = expectation.replace("missing", "valid");
      assert.deepEqual(await errors(repaired), [
        expected(repaired, "<!-- @vue-expect-error", 2578),
      ]);
      assert.deepEqual(await errors(repaired.replace("<!-- @vue-expect-error -->", "")), []);
      const unguarded = expectation.replace("<!-- @vue-expect-error -->", "");
      assert.deepEqual(await errors(unguarded), [expected(unguarded, "missing")]);

      const strict = `<!-- @strictTemplates true -->
<script setup lang="ts">
import { defineComponent } from 'vue';
const Comp = defineComponent({ props: { title: String } });
</script>
<template>
<Comp unknownComponent="value" />
<div unknownNative="value" data-custom="value" />
</template>`;
      assert.equal((await errors(strict)).length, 2);
      const relaxed = strict.replace("@strictTemplates true", "@strictTemplates false");
      assert.deepEqual(await errors(relaxed), []);
      assert.deepEqual(
        await errors(relaxed.replace("<template>", "<template><!-- @strictTemplates true -->")),
        [],
      );
      assert.deepEqual(
        await errors(strict.replace("<template>", "<!-- @checkUnknownProps false --><template>")),
        [],
      );
      const unknown = `<!-- @strictTemplates true -->
<script setup lang="ts"></script>
<template><MissingComponent /><missing-component /></template>`;
      assert.deepEqual(await errors(unknown), [
        expected(unknown, "MissingComponent"),
        expected(unknown, "missing-component"),
      ]);
      const unchecked = unknown.replace("@strictTemplates true", "@strictTemplates false");
      assert.deepEqual(await errors(unchecked), []);
      assert.deepEqual(
        await errors(
          unknown.replace("<template>", "<!-- @checkUnknownComponents false --><template>"),
        ),
        [],
      );
      assert.equal(
        (
          await errors(
            unchecked.replace("<template>", "<!-- @checkUnknownComponents true --><template>"),
          )
        ).length,
        2,
      );
      assert.deepEqual(
        await errors(
          unchecked.replace("<template>", "<template><!-- @checkUnknownComponents true -->"),
        ),
        [],
      );
      const event = `<!-- @strictTemplates true -->
<script setup lang="ts"></script>
<template><Teleport to="body" @unknown="() => {}" /></template>`;
      assert.deepEqual(await errors(event), [expected(event, "unknown", 2353)]);
      const looseEvent = event.replace("@strictTemplates true", "@strictTemplates false");
      assert.deepEqual(await errors(looseEvent), []);
      assert.deepEqual(
        await errors(event.replace("<template>", "<!-- @checkUnknownEvents false --><template>")),
        [],
      );
      const strictEvent = looseEvent.replace(
        "<template>",
        "<!-- @checkUnknownEvents true --><template>",
      );
      assert.deepEqual(await errors(strictEvent), [expected(strictEvent, "unknown", 2353)]);
      assert.deepEqual(
        await errors(
          looseEvent.replace("<template>", "<template><!-- @checkUnknownEvents true -->"),
        ),
        [],
      );
    } finally {
      await editor.close();
    }
  },
);

test(
  "#8003 dynamic :is never becomes an unknown registered component",
  { timeout: 120_000 },
  async () => {
    const fixture = new URL("../fixtures/typechecker/unknown-dynamic-component/", import.meta.url);
    const original = fs.readFileSync(new URL("src/App.vue", fixture), "utf8");
    const config = JSON.parse(fs.readFileSync(new URL("tsconfig.json", fixture), "utf8"));
    const observations: object[] = [];
    const capture = path.resolve(
      "target/differential/typechecker-dynamic-component-8003/receipt.json",
    );
    fs.mkdirSync(path.dirname(capture), { recursive: true });
    for (const option of [true, false, undefined]) {
      const directory = workspace("dynamic-component-8003-");
      const file = path.join(directory, "src/App.vue");
      const uri = pathToFileURL(file).href;
      const session = new LspSession();
      let version = 0;
      fs.mkdirSync(path.dirname(file));
      fs.writeFileSync(file, original);
      fs.copyFileSync(
        new URL("vize.config.json", fixture),
        path.join(directory, "vize.config.json"),
      );
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          ...config,
          ...(option === undefined
            ? { vueCompilerOptions: {} }
            : {
                vueCompilerOptions: { checkUnknownComponents: option },
              }),
        }),
      );
      const compare = async (source: string, expected: Array<{ code: number; token: string }>) => {
        fs.writeFileSync(file, source);
        const cli = await check(directory, ["src/App.vue"]);
        version++;
        session.notify(
          version === 1 ? "textDocument/didOpen" : "textDocument/didChange",
          version === 1
            ? {
                textDocument: { uri, languageId: "vue", version, text: source },
              }
            : {
                textDocument: { uri, version },
                contentChanges: [{ text: source }],
              },
        );
        const publication = await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (value) => isDiagnosticsForUri(value, uri) && value.version === version,
        );
        assert.ok(isDiagnosticsForUri(publication, uri));
        observations.push({ option: option ?? "absent", source, cli, publication });
        fs.writeFileSync(capture, JSON.stringify({ issue: 8003, observations }, null, 2));
        const identities = expected.map(({ code, token }) => ({
          code,
          ...offsetToPosition(source, source.indexOf(token)),
        }));
        assert.deepEqual(
          cli.map((diagnostic) => ({
            code: diagnostic.code,
            line: diagnostic.line - 1,
            character: diagnostic.column - 1,
          })),
          identities,
          JSON.stringify(cli),
        );
        assert.deepEqual(
          publication.diagnostics.map((diagnostic) => ({
            code: Number(diagnostic.code),
            ...diagnostic.range?.start,
          })),
          identities,
          JSON.stringify(publication),
        );
        for (const diagnostic of cli) assert.equal(diagnostic.severity, "error");
        for (const diagnostic of publication.diagnostics) {
          assert.equal(diagnostic.severity, 1);
          assert.equal(diagnostic.source, "vize/types");
          assert.ok(diagnostic.range?.end);
          assert.ok(diagnostic.message);
        }
      };
      try {
        await session.initialize(directory, { editor: true, typecheck: true, lint: false });
        await compare(original, []);
        const dynamic = `<script setup lang="ts">
const open = true;
const selected = 'button';
const attrs = { id: 'dynamic' };
function choose() { return selected; }
</script>
<template>😀

<component :is="selected" />
<component :is="open ? 'div' : 'button'" ref="target" v-bind="attrs" />
<component :is="choose()" />
</template>`;
        await compare(dynamic, []);
        const unknown = original.replace("</template>", "  <MissingWidget />\n</template>");
        await compare(unknown, option === true ? [{ code: 2339, token: "MissingWidget" }] : []);
        const reserved = original.replace(
          "</template>",
          "  <Unknown__vize_dynamic_is_777 />\n</template>",
        );
        await compare(
          reserved,
          option === true ? [{ code: 2339, token: "Unknown__vize_dynamic_is_777" }] : [],
        );
        const missing = original.replace("'button'", "missing ? 'div' : 'button'");
        await compare(missing, [{ code: 2339, token: "missing" }]);
        const wrongProp = `<script setup lang="ts">
import { defineComponent } from 'vue';
const Choice = defineComponent({ props: { count: { type: Number, required: true } } });
const open = true;
</script>
<template><component :is="open ? Choice : Choice" count="wrong" /></template>`;
        await compare(wrongProp, [{ code: 2322, token: 'count="wrong"' }]);
        await compare(original, []);
        const named = original
          .replace("'button'", "registry.button")
          .replace("</script>", "const registry = { button: 'button' };\n</script>");
        await compare(named, []);
        await compare(named.replace("registry.button", "registry.missing"), [
          { code: 2339, token: "missing" },
        ]);
        const badValue = named
          .replace("button: 'button'", "button: 1")
          .replace("registry.button", "registry.button.toUpperCase()");
        await compare(badValue, [{ code: 2339, token: "toUpperCase" }]);
        const namedProp = wrongProp
          .replace("</script>", "const registry = { Choice };\n</script>")
          .replace("open ? Choice : Choice", "registry.Choice");
        await compare(namedProp, [{ code: 2322, token: 'count="wrong"' }]);
      } finally {
        await session.shutdown();
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  },
);
