import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const types = "export interface Item { id: number; label: string }\n";
const source = `<script setup lang="ts" generic="T extends Item, K extends keyof T">
import type { Item } from "./types";
const props = defineProps<{ item: T; key: K }>();
const local: T = props.item;
</script>
<template><p>T {{ (props.item as T).id }} {{ local }}</p></template>
`;

type Range = {
  start: { line: number; character: number };
  end: { line: number; character: number };
};
type Location = { uri: string; range: Range };
type TextEdit = { range: Range; newText: string };

/** The `length` bytes at `skip` past the first occurrence of `needle`. */
function at(text: string, needle: string, skip: number, length: number): Range {
  const start = text.indexOf(needle);
  assert.notEqual(start, -1, needle);
  return {
    start: offsetToPosition(text, start + skip),
    end: offsetToPosition(text, start + skip + length),
  };
}

function applyEdits(text: string, edits: TextEdit[]): string {
  const offset = ({ line, character }: Range["start"]) =>
    text
      .split("\n")
      .slice(0, line)
      .reduce((sum, row) => sum + row.length + 1, 0) + character;
  return [...edits]
    .sort((left, right) => offset(right.range.start) - offset(left.range.start))
    .reduce(
      (current, edit) =>
        current.slice(0, offset(edit.range.start)) +
        edit.newText +
        current.slice(offset(edit.range.end)),
      text,
    );
}

/** Every authored `T`: the attribute declaration first, then its references. */
const T = {
  declaration: at(source, 'generic="T', 9, 1),
  constraint: at(source, "keyof T", 6, 1),
  script: at(source, "item: T", 6, 1),
  annotation: at(source, "local: T", 7, 1),
  template: at(source, "as T", 3, 1),
};
const typeParameter = Object.values(T);
const local = [at(source, "local:", 0, 5), at(source, "{{ local }}", 3, 5)];

async function withDocument(
  typecheck: boolean,
  run: (context: {
    session: LspSession;
    uri: string;
    typesUri: string;
    request: (method: string, range: Range, extra?: object) => Promise<unknown>;
    change: (text: string) => Promise<Array<{ message?: string; range: Range }>>;
  }) => Promise<void>,
) {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "generic-attribute-navigation-"));
  const session = new LspSession();
  try {
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
        },
        include: ["./**/*"],
      }),
    );
    const typesFile = path.join(workspace, "types.ts");
    fs.writeFileSync(typesFile, types);
    const file = path.join(workspace, "App.vue");
    fs.writeFileSync(file, source);
    const uri = pathToFileURL(file).href;
    await session.initialize(workspace, { editor: true, lint: false, typecheck });

    let version = 1;
    const published = async () => {
      const expected = version;
      const params = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === expected,
        60_000,
      )) as PublishDiagnosticsParams;
      return params.diagnostics.map(({ message, range }) => ({ message, range: range as Range }));
    };
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version, text: source },
    });
    assert.deepEqual(await published(), []);
    await run({
      session,
      uri,
      typesUri: pathToFileURL(typesFile).href,
      request: (method, range, extra = {}) =>
        session.request(method, { textDocument: { uri }, position: range.start, ...extra }),
      change: async (text) => {
        version += 1;
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        });
        return published();
      },
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}

const locations = (response: unknown): Location[] =>
  response == null ? [] : Array.isArray(response) ? response : [response as Location];

test("generic attribute type parameters navigate like script setup code", async () => {
  await withDocument(true, async ({ uri, typesUri, request, change }) => {
    const scriptHover = (await request("textDocument/hover", T.script)) as {
      contents: unknown;
    } | null;
    assert.ok(scriptHover, "a body usage of T has a hover");
    assert.deepEqual(scriptHover.contents, {
      kind: "markdown",
      value: "```typescript\n(type parameter) T extends Item\n```",
    });
    for (const [label, query] of Object.entries(T)) {
      assert.deepEqual(
        await request("textDocument/hover", query),
        { contents: scriptHover.contents, range: query },
        `hover ${label}`,
      );
      assert.deepEqual(
        locations(await request("textDocument/definition", query)),
        [{ uri, range: T.declaration }],
        `definition ${label}`,
      );
      assert.deepEqual(
        await request("textDocument/references", query, { context: { includeDeclaration: true } }),
        typeParameter.map((range) => ({ uri, range })),
        `references ${label}`,
      );
      assert.deepEqual(
        await request("textDocument/prepareRename", query),
        query,
        `prepareRename ${label}`,
      );
      assert.deepEqual(
        await request("textDocument/rename", query, { newName: "Row" }),
        { changes: { [uri]: typeParameter.map((range) => ({ range, newText: "Row" })) } },
        `rename ${label}`,
      );
    }

    const K = { declaration: at(source, "K extends", 0, 1), script: at(source, "key: K", 5, 1) };
    for (const [label, query] of Object.entries(K)) {
      assert.deepEqual(
        locations(await request("textDocument/definition", query)),
        [{ uri, range: K.declaration }],
        `definition K ${label}`,
      );
    }

    // A constraint reference resolves like its script import.
    const item = at(source, "T extends Item", 10, 4);
    const imported = at(source, "{ Item }", 2, 4);
    const importHover = (await request("textDocument/hover", imported)) as {
      contents: unknown;
    } | null;
    assert.ok(importHover);
    assert.deepEqual(await request("textDocument/hover", item), {
      contents: importHover.contents,
      range: item,
    });
    assert.deepEqual(locations(await request("textDocument/definition", item)), [
      { uri: typesUri, range: at(types, "Item", 0, 4) },
    ]);
    assert.deepEqual(
      locations(
        await request("textDocument/references", item, { context: { includeDeclaration: true } }),
      ),
      locations(
        await request("textDocument/references", imported, {
          context: { includeDeclaration: true },
        }),
      ),
    );

    // Negative controls: text that is not a type parameter stays unmapped,
    // and a non-generic local keeps its own identity.
    for (const control of [
      at(source, "<p>T", 3, 1),
      at(source, 'generic="', 1, 1),
      at(source, 'lang="ts"', 6, 1),
    ]) {
      assert.equal(await request("textDocument/hover", control), null);
      assert.deepEqual(locations(await request("textDocument/definition", control)), []);
      assert.equal(await request("textDocument/prepareRename", control), null);
    }
    assert.deepEqual(await request("textDocument/rename", local[0], { newName: "row" }), {
      changes: { [uri]: local.map((range) => ({ range, newText: "row" })) },
    });

    // The rename result still type-checks; renaming only the usages does not.
    const renamed = applyEdits(
      source,
      typeParameter.map((range) => ({ range, newText: "Row" })),
    );
    assert.match(renamed, /generic="Row extends Item, K extends keyof Row"/);
    assert.deepEqual(await change(renamed), []);
    const usagesOnly = applyEdits(
      source,
      [T.script, T.annotation, T.template].map((range) => ({ range, newText: "Row" })),
    );
    const broken = await change(usagesOnly);
    assert.ok(
      broken.some((diagnostic) => diagnostic.message?.includes("Cannot find name 'Row'")),
      JSON.stringify(broken),
    );
  });
});

test("checker-free references and rename include the generic attribute", async () => {
  await withDocument(false, async ({ uri, request }) => {
    for (const [label, query] of Object.entries(T)) {
      assert.deepEqual(
        await request("textDocument/references", query, { context: { includeDeclaration: true } }),
        typeParameter.map((range) => ({ uri, range })),
        `references ${label}`,
      );
      assert.deepEqual(
        await request("textDocument/rename", query, { newName: "Row" }),
        { changes: { [uri]: typeParameter.map((range) => ({ range, newText: "Row" })) } },
        `rename ${label}`,
      );
    }
    for (const control of [at(source, "<p>T", 3, 1), at(source, 'generic="', 1, 1)]) {
      assert.equal(await request("textDocument/prepareRename", control), null);
    }
    assert.deepEqual(await request("textDocument/rename", local[0], { newName: "row" }), {
      changes: { [uri]: local.map((range) => ({ range, newText: "row" })) },
    });
  });
});
