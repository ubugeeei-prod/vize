import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/documentHighlight` is a vize-native navigation feature
 * (enabled by the editor bundle, independent of corsa/typecheck). These tests
 * characterize the current highlight kinds (DocumentHighlightKind: TEXT=1,
 * READ=2, WRITE=3), UTF-16 ranges, CRLF handling, and the empty-document
 * behavior by talking to the real `vize lsp` binary over stdio.
 */

type DocumentHighlight = {
  kind?: number;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
};

async function withSession(
  name: string,
  run: (ctx: { session: LspSession; workspaceDir: string }) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, name);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });
    await run({ session, workspaceDir });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

function openVue(
  session: LspSession,
  workspaceDir: string,
  fileName: string,
  source: string,
): { uri: string; ready: Promise<unknown> } {
  const filePath = path.join(workspaceDir, fileName);
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
  const ready = session.waitForNotification("textDocument/publishDiagnostics", (params) =>
    isDiagnosticsForUri(params, uri),
  );
  return { uri, ready };
}

const DH_SOURCE = `<script setup lang="ts">
const count = ref(0)
function inc() {
  count.value++
}
</script>

<template>
  <button @click="inc">{{ count }}</button>
</template>
`;

test("documentHighlight marks the script declaration WRITE and other uses READ", async () => {
  await withSession("lsp-document-highlight-rw", async ({ session, workspaceDir }) => {
    const { uri, ready } = openVue(session, workspaceDir, "Dh.vue", DH_SOURCE);
    await ready;

    const declarationOffset = DH_SOURCE.indexOf("const count") + "const ".length;
    const highlights = (await session.request("textDocument/documentHighlight", {
      textDocument: { uri },
      position: offsetToPosition(DH_SOURCE, declarationOffset),
    })) as DocumentHighlight[] | null;

    assert.ok(Array.isArray(highlights), JSON.stringify(highlights));
    assert.equal(highlights.length, 3, JSON.stringify(highlights));

    // Exactly one WRITE at the declaration, the rest READ, never TEXT.
    const writes = highlights.filter((highlight) => highlight.kind === 3);
    const reads = highlights.filter((highlight) => highlight.kind === 2);
    assert.equal(writes.length, 1, JSON.stringify(highlights));
    assert.equal(reads.length, 2, JSON.stringify(highlights));
    assert.equal(
      highlights.some((highlight) => highlight.kind === 1),
      false,
      JSON.stringify(highlights),
    );

    // The single WRITE marks the `count` binding in `const count`.
    const declarationStart = DH_SOURCE.indexOf("const count") + "const ".length;
    const declarationEnd = declarationStart + "count".length;
    assert.deepEqual(writes[0].range, {
      start: offsetToPosition(DH_SOURCE, declarationStart),
      end: offsetToPosition(DH_SOURCE, declarationEnd),
    });

    // The two READ ranges cover `count` in `count.value++` and in `{{ count }}`.
    const useInScriptStart = DH_SOURCE.indexOf("count.value");
    const useInTemplateStart = DH_SOURCE.indexOf("{{ count }}") + "{{ ".length;
    const readRanges = reads
      .map((highlight) => highlight.range)
      .sort((a, b) => a.start.line - b.start.line || a.start.character - b.start.character);
    assert.deepEqual(readRanges, [
      {
        start: offsetToPosition(DH_SOURCE, useInScriptStart),
        end: offsetToPosition(DH_SOURCE, useInScriptStart + "count".length),
      },
      {
        start: offsetToPosition(DH_SOURCE, useInTemplateStart),
        end: offsetToPosition(DH_SOURCE, useInTemplateStart + "count".length),
      },
    ]);
  });
});

/**
 * Fixture from #3454: four `<div>`s, so a document-wide name scan and a
 * stack-based pair resolution give visibly different answers.
 */
const FOUR_DIVS = `<script setup lang="ts">
const count = 1
</script>

<template>
  <div class="a">
    <div class="b">{{ count }}</div>
  </div>
  <div class="c" />
</template>
`;

type FlatHighlight = [number, number, number, number, number | undefined];

function flatten(highlights: DocumentHighlight[] | null): FlatHighlight[] {
  if (highlights == null) return [];
  return highlights.map((highlight) => [
    highlight.range.start.line,
    highlight.range.start.character,
    highlight.range.end.line,
    highlight.range.end.character,
    highlight.kind,
  ]);
}

test("documentHighlight on a tag name reports the matching pair, not every same-named tag", async () => {
  await withSession("lsp-document-highlight-pair", async ({ session, workspaceDir }) => {
    const { uri, ready } = openVue(session, workspaceDir, "FourDivs.vue", FOUR_DIVS);
    await ready;

    const ask = (position: { line: number; character: number }) =>
      session.request("textDocument/documentHighlight", {
        textDocument: { uri },
        position,
      }) as Promise<DocumentHighlight[] | null>;

    // Recorded from `@vue/language-server@3.3.8` for this fixture at line 6,
    // character 6: [[6,5,6,8], [6,32,6,35]] — the matching pair only. Before
    // #3454 `vize lsp` answered five ranges here, adding [5,3,5,6], [7,4,7,7]
    // and [8,3,8,6] (the outer element's pair and the unrelated self-closing
    // `<div class="c" />`), which is exactly the must-exclude set below.
    assert.deepEqual(flatten(await ask({ line: 6, character: 6 })), [
      [6, 5, 6, 8, 1],
      [6, 32, 6, 35, 1],
    ]);

    // The close tag name resolves back to the same pair.
    assert.deepEqual(flatten(await ask({ line: 6, character: 33 })), [
      [6, 5, 6, 8, 1],
      [6, 32, 6, 35, 1],
    ]);

    // The outer element pairs with the `</div>` on line 7, never with the
    // inner element's close tag on line 6.
    assert.deepEqual(flatten(await ask({ line: 5, character: 4 })), [
      [5, 3, 5, 6, 1],
      [7, 4, 7, 7, 1],
    ]);

    // A self-closing element has no counterpart: exactly its own name.
    assert.deepEqual(flatten(await ask({ line: 8, character: 4 })), [[8, 3, 8, 6, 1]]);
  });
});

test("documentHighlight on a tag name highlights open and close tags as TEXT", async () => {
  await withSession("lsp-document-highlight-tag", async ({ session, workspaceDir }) => {
    const { uri, ready } = openVue(session, workspaceDir, "Dh.vue", DH_SOURCE);
    await ready;

    // Anchor inside `button` of the opening tag.
    const tagAnchor = DH_SOURCE.indexOf("<button") + "<bu".length;
    const highlights = (await session.request("textDocument/documentHighlight", {
      textDocument: { uri },
      position: offsetToPosition(DH_SOURCE, tagAnchor),
    })) as DocumentHighlight[] | null;

    assert.ok(Array.isArray(highlights), JSON.stringify(highlights));
    assert.equal(highlights.length, 2, JSON.stringify(highlights));

    // Both spans are TEXT and exactly cover the 6-character `button` name.
    for (const highlight of highlights) {
      assert.equal(highlight.kind, 1, JSON.stringify(highlight));
      assert.equal(highlight.range.start.line, highlight.range.end.line);
      assert.equal(
        highlight.range.end.character - highlight.range.start.character,
        "button".length,
      );
    }

    const openNameStart = DH_SOURCE.indexOf("<button") + "<".length;
    const closeNameStart = DH_SOURCE.indexOf("</button") + "</".length;
    const expected = [
      {
        start: offsetToPosition(DH_SOURCE, openNameStart),
        end: offsetToPosition(DH_SOURCE, openNameStart + "button".length),
      },
      {
        start: offsetToPosition(DH_SOURCE, closeNameStart),
        end: offsetToPosition(DH_SOURCE, closeNameStart + "button".length),
      },
    ];
    const actual = highlights
      .map((highlight) => highlight.range)
      .sort((a, b) => a.start.line - b.start.line || a.start.character - b.start.character);
    assert.deepEqual(actual, expected);
  });
});

test("documentHighlight handles CRLF line endings without column drift", async () => {
  await withSession("lsp-document-highlight-crlf", async ({ session, workspaceDir }) => {
    const crlfSource = `<script setup lang="ts">\r\nconst tag = 1\r\nconst use = tag + 1\r\n</script>\r\n`;
    const { uri, ready } = openVue(session, workspaceDir, "Crlf.vue", crlfSource);
    await ready;

    // Anchor on the `tag` declaration (line 1, character 6).
    const highlights = (await session.request("textDocument/documentHighlight", {
      textDocument: { uri },
      position: { line: 1, character: 6 },
    })) as DocumentHighlight[] | null;

    assert.ok(Array.isArray(highlights), JSON.stringify(highlights));
    assert.equal(highlights.length, 2, JSON.stringify(highlights));

    const declaration = highlights.find((highlight) => highlight.kind === 3);
    const use = highlights.find((highlight) => highlight.kind === 2);
    assert.ok(declaration, JSON.stringify(highlights));
    assert.ok(use, JSON.stringify(highlights));

    // `const tag` declaration WRITE on line 1, columns 6-9 (CRLF must not shift columns).
    assert.deepEqual(declaration.range, {
      start: { line: 1, character: 6 },
      end: { line: 1, character: 9 },
    });
    // `tag` use READ on line 2, columns 12-15.
    assert.deepEqual(use.range, {
      start: { line: 2, character: 12 },
      end: { line: 2, character: 15 },
    });
  });
});

test("documentHighlight returns null for an empty document", async () => {
  await withSession("lsp-document-highlight-empty", async ({ session, workspaceDir }) => {
    const { uri, ready } = openVue(session, workspaceDir, "Empty.vue", "");
    await ready;

    const highlights = await session.request("textDocument/documentHighlight", {
      textDocument: { uri },
      position: { line: 0, character: 0 },
    });
    assert.equal(highlights, null);

    // Hover on the same empty document is also inert.
    const hover = await session.request("textDocument/hover", {
      textDocument: { uri },
      position: { line: 0, character: 0 },
    });
    assert.equal(hover, null);
  });
});
