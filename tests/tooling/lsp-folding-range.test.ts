import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/foldingRange` parity suite for `vize lsp`.
 *
 * `endLine` is the last line the client *hides*, not the closing line of the
 * construct: a region that reaches the closing tag folds the terminator away
 * and leaves a collapsed block with nothing to show where it ends. Maestro
 * reported `4..9` for a `<template>` closing on line 9 and folded nothing
 * inside a block, so a large template was one all-or-nothing region (#3455).
 *
 * `VUE_LANGUAGE_SERVER_FOUR_DIVS` is the response `@vue/language-server@3.3.8`
 * actually returned for FOUR_DIVS, driven over stdio with the same request, so
 * both sides of the oracle are recorded.
 */

type FoldingRange = {
  startLine: number;
  endLine: number;
  startCharacter?: number;
  endCharacter?: number;
  kind?: string;
  collapsedText?: string;
};

/** `[startLine, endLine, kind, collapsedText]` — the FULL serialized region. */
type FlatRange = [number, number, string | undefined, string | undefined];

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

/**
 * Recorded from `@vue/language-server@3.3.8`: the multi-line `<div class="a">`,
 * the script-setup block and the template block. Vize reports the same three
 * regions; it emits blocks before elements and labels blocks with
 * `collapsedText`, neither of which the reference server does.
 */
const VUE_LANGUAGE_SERVER_FOUR_DIVS = [
  { startLine: 5, endLine: 6 },
  { startLine: 0, endLine: 1 },
  { startLine: 4, endLine: 8 },
];

const MULTI_BLOCK = `<script setup lang="ts">
const x = 1
</script>

<template>
  <div>{{ x }}</div>
</template>

<style scoped>
.a {
  color: red;
}
</style>

<style module="m">
.b {
  color: blue;
}
</style>
`;

async function withSession(
  name: string,
  run: (ask: (source: string, fileName: string) => Promise<FlatRange[]>) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, name);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

    await run(async (source, fileName) => {
      const filePath = path.join(workspaceDir, fileName);
      const uri = pathToFileURL(filePath).href;
      fs.writeFileSync(filePath, source, "utf8");
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
        isDiagnosticsForUri(params, uri),
      );

      const ranges = (await session.request("textDocument/foldingRange", {
        textDocument: { uri },
      })) as FoldingRange[] | null;
      if (ranges == null) return [];
      return ranges.map((range) => {
        // Line folds only: the character fields are never serialized.
        assert.equal(range.startCharacter, undefined, JSON.stringify(range));
        assert.equal(range.endCharacter, undefined, JSON.stringify(range));
        return [range.startLine, range.endLine, range.kind, range.collapsedText] as FlatRange;
      });
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("foldingRange folds elements and stops one line before every closing tag", async () => {
  await withSession("lsp-folding-four-divs", async (ask) => {
    // Before #3455 this was [[4,9,"region","template"], [0,2,"region","script setup"]]:
    // no element region at all, and both block regions folded their own
    // closing tag away.
    assert.deepEqual(await ask(FOUR_DIVS, "FourDivs.vue"), [
      [4, 8, "region", "template"],
      [0, 1, "region", "script setup"],
      [5, 6, undefined, undefined],
    ]);

    // Same three regions as the reference server, ignoring order and the
    // labels Vize adds.
    assert.deepEqual(
      [...VUE_LANGUAGE_SERVER_FOUR_DIVS]
        .map((range) => [range.startLine, range.endLine])
        .sort((a, b) => a[0] - b[0] || a[1] - b[1]),
      [
        [0, 1],
        [4, 8],
        [5, 6],
      ],
    );
  });
});

test("foldingRange emits one region per multi-line block with block-named collapsedText", async () => {
  await withSession("lsp-folding-multi-block", async (ask) => {
    // Styles always collapse to "style"; the single-line `<div>{{ x }}</div>`
    // hides nothing and contributes no element region.
    assert.deepEqual(await ask(MULTI_BLOCK, "MultiBlock.vue"), [
      [4, 5, "region", "template"],
      [0, 1, "region", "script setup"],
      [8, 11, "region", "style"],
      [14, 17, "region", "style"],
    ]);
  });
});

test("foldingRange folds a multi-line comment as a comment region", async () => {
  await withSession("lsp-folding-comment", async (ask) => {
    const source = `<template>
  <!--
    note
  -->
  <div>
    x
  </div>
</template>
`;
    assert.deepEqual(await ask(source, "Comment.vue"), [
      [0, 6, "region", "template"],
      [1, 2, "comment", undefined],
      [4, 5, undefined, undefined],
    ]);
  });
});

test("foldingRange returns null when nothing hides a line", async () => {
  await withSession("lsp-folding-nothing", async (ask) => {
    // Everything on one line, so there is nothing to fold.
    assert.deepEqual(await ask("<template><div/></template>\n", "OneLine.vue"), []);
    // Blocks whose tags are on consecutive lines hide nothing either: a region
    // would have to swallow the closing tag to be non-empty.
    assert.deepEqual(await ask("<template>\n</template>\n<style>\n</style>\n", "Empty.vue"), []);
  });
});
