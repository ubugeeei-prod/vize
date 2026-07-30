import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/linkedEditingRange` parity suite for `vize lsp`.
 *
 * `@vue/language-server` 3.3.8 advertises `linkedEditingRangeProvider: true`, so
 * editing `<div>` in a `.vue` file rewrites the matching `</div>` as you type.
 * Maestro advertised nothing and answered `-32601 Method not found`: the close
 * tag was silently left behind, which is exactly the class of failure a user
 * cannot see until the template stops rendering.
 *
 * `VUE_LANGUAGE_SERVER_TAG_PAIR` is the response `@vue/language-server@3.3.8`
 * actually returned for this fixture and position, driven over stdio with the
 * same request, so both sides of the oracle are recorded (#2971).
 */

type Range = {
  start: { line: number; character: number };
  end: { line: number; character: number };
};

type LinkedEditingRanges = {
  ranges: Range[];
  wordPattern?: string;
};

type FlatRange = [number, number, number, number];

const FIXTURE = `<script setup lang="ts">
const count = 1
</script>

<template>
  <div class="wrap">{{ count }}</div>
</template>
`;

/** Recorded from `@vue/language-server@3.3.8` for FIXTURE at line 5, character 4. */
const VUE_LANGUAGE_SERVER_TAG_PAIR: FlatRange[] = [
  [5, 3, 5, 6], // `div` in the open tag
  [5, 33, 5, 36], // `div` in the close tag
];

function flatten(result: LinkedEditingRanges | null): FlatRange[] {
  if (result == null) return [];
  return result.ranges.map((range) => [
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  ]);
}

async function withSession(
  label: string,
  run: (
    ask: (position: { line: number; character: number }) => Promise<LinkedEditingRanges | null>,
  ) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, label);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

    const filePath = path.join(workspaceDir, "App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, FIXTURE, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: FIXTURE },
    });

    await run(
      (position) =>
        session.request("textDocument/linkedEditingRange", {
          textDocument: { uri },
          position,
        }) as Promise<LinkedEditingRanges | null>,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("linkedEditingRange links a tag-name pair byte-for-byte with the reference server", async () => {
  await withSession("lsp-linked-editing-pair", async (ask) => {
    const result = await ask({ line: 5, character: 4 });
    assert.deepEqual(flatten(result), VUE_LANGUAGE_SERVER_TAG_PAIR);
    // The reference server sends no wordPattern; the client uses its own.
    assert.equal(result?.wordPattern, undefined);

    // The close tag name links back to the same pair.
    assert.deepEqual(flatten(await ask({ line: 5, character: 34 })), VUE_LANGUAGE_SERVER_TAG_PAIR);
    // So does the caret position just after the name, where an editor leaves it
    // while the name is being typed.
    assert.deepEqual(flatten(await ask({ line: 5, character: 6 })), VUE_LANGUAGE_SERVER_TAG_PAIR);
  });
});

test("linkedEditingRange reports nothing where there is no tag-name pair", async () => {
  await withSession("lsp-linked-editing-negative", async (ask) => {
    // must-exclude set: every one of these must return null, not a one-element
    // list, because a single range makes the editor believe it linked something.
    for (const [line, character, what] of [
      [5, 2, "the `<` itself"],
      [5, 9, "an attribute name"],
      [5, 15, "an attribute value"],
      [5, 24, "an interpolation identifier"],
      [1, 8, "a script identifier"],
      [0, 4, "the `<script>` block tag name"],
    ] as const) {
      const result = await ask({ line, character });
      assert.equal(result, null, `${what} must not report linked ranges`);
    }
  });
});
