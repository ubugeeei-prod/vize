import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/selectionRange` parity suite for `vize lsp`.
 *
 * `@vue/language-server` 3.3.8 advertises `selectionRangeProvider: true`, so
 * "expand/shrink selection" (VS Code `Shift+Alt+Right`, Zed, Helix `Alt-o`,
 * Neovim/Emacs via their LSP clients) works inside `.vue` files there. Maestro
 * advertised nothing and answered `-32601 Method not found`, so the command
 * silently did nothing — a failure a user cannot see.
 *
 * `VUE_LANGUAGE_SERVER_CHAIN` below is not hand-written: it is the response
 * `@vue/language-server@3.3.8` returned for this exact fixture and position,
 * driven over stdio with the same request. Both sides of the oracle are
 * therefore recorded, per #2971. The suite asserts:
 *
 *   1. the FULL chain Vize returns, link by link (no substring matching), and
 *   2. that every link Vue LS returns is present in Vize's chain, in order,
 *      with byte-identical authored ranges.
 *
 * Every range addresses the authored `.vue` document, never virtual TypeScript.
 */

type Range = {
  start: { line: number; character: number };
  end: { line: number; character: number };
};

type SelectionRange = {
  range: Range;
  parent?: SelectionRange;
};

/** `[startLine, startCharacter, endLine, endCharacter]`. */
type FlatRange = [number, number, number, number];

const FIXTURE = `<script setup lang="ts">
const count = 1
</script>

<template>
  <div class="wrap">{{ count }}</div>
</template>
`;

/**
 * Recorded from `@vue/language-server@3.3.8` for FIXTURE at line 5 character 24
 * (inside `count` in `{{ count }}`), driven over stdio with a client-side
 * tsserver bridge answering `_vue:projectInfo`.
 */
const VUE_LANGUAGE_SERVER_CHAIN: FlatRange[] = [
  [5, 20, 5, 31], // `{{ count }}`
  [5, 2, 5, 37], // `<div class="wrap">{{ count }}</div>`
  [4, 10, 6, 0], // template block content
];

/**
 * Recorded from `@vue/language-server@3.3.8` for FIXTURE at line 5 character 15
 * (inside `wrap` in `class="wrap"`).
 */
const VUE_LANGUAGE_SERVER_ATTRIBUTE_CHAIN: FlatRange[] = [
  [5, 14, 5, 18], // `wrap`
  [5, 13, 5, 19], // `"wrap"` including the quotes
  [5, 7, 5, 19], // `class="wrap"`
  [5, 3, 5, 19], // `div class="wrap"` (start tag interior)
  [5, 2, 5, 37], // the whole element
  [4, 10, 6, 0], // template block content
];

function flatten(chain: SelectionRange): FlatRange[] {
  const flattened: FlatRange[] = [];
  let node: SelectionRange | undefined = chain;
  while (node) {
    flattened.push([
      node.range.start.line,
      node.range.start.character,
      node.range.end.line,
      node.range.end.character,
    ]);
    node = node.parent;
  }
  return flattened;
}

/**
 * Every entry of `expected` must appear in `actual` in the same relative order
 * with identical coordinates. Used to prove Vize's chain is a superset of the
 * reference server's, not merely a chain of the same length.
 */
function assertOrderedSuperset(actual: FlatRange[], expected: FlatRange[], label: string): void {
  const missing: FlatRange[] = [];
  let cursor = 0;
  for (const link of expected) {
    const index = actual.findIndex(
      (candidate, position) =>
        position >= cursor &&
        candidate[0] === link[0] &&
        candidate[1] === link[1] &&
        candidate[2] === link[2] &&
        candidate[3] === link[3],
    );
    if (index < 0) {
      missing.push(link);
      continue;
    }
    cursor = index + 1;
  }
  assert.deepEqual(
    missing,
    [],
    `${label}: these @vue/language-server links are missing from the Vize chain (actual: ${JSON.stringify(actual)})`,
  );
}

async function withSelectionRanges(
  label: string,
  positions: Array<{ line: number; character: number }>,
  run: (chains: SelectionRange[]) => void,
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

    const chains = (await session.request("textDocument/selectionRange", {
      textDocument: { uri },
      positions,
    })) as SelectionRange[] | null;
    assert.ok(chains, "vize lsp must answer textDocument/selectionRange");
    run(chains);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("selectionRange expands an interpolation identifier through authored .vue levels", async () => {
  await withSelectionRanges(
    "lsp-selrange-interpolation",
    [{ line: 5, character: 24 }],
    (chains) => {
      assert.equal(chains.length, 1, "one chain per requested position");
      const actual = flatten(chains[0]);

      assert.deepEqual(actual, [
        [5, 23, 5, 28], // `count`
        [5, 20, 5, 31], // `{{ count }}` (also the div's inner content)
        [5, 2, 5, 37], // `<div class="wrap">{{ count }}</div>`
        [4, 10, 6, 0], // template block content
        [4, 0, 6, 11], // `<template>` … `</template>`
        [0, 0, 7, 0], // whole document
      ]);

      assertOrderedSuperset(actual, VUE_LANGUAGE_SERVER_CHAIN, "interpolation identifier");
    },
  );
});

test("selectionRange expands an attribute value through value, quotes and attribute", async () => {
  await withSelectionRanges("lsp-selrange-attribute", [{ line: 5, character: 15 }], (chains) => {
    const actual = flatten(chains[0]);

    assert.deepEqual(actual, [
      [5, 14, 5, 18], // `wrap`
      [5, 13, 5, 19], // `"wrap"` including the quotes
      [5, 7, 5, 19], // `class="wrap"`
      [5, 2, 5, 20], // `<div class="wrap">`
      [5, 2, 5, 37], // the whole element
      [4, 10, 6, 0],
      [4, 0, 6, 11],
      [0, 0, 7, 0],
    ]);

    // Vue LS additionally offers `div class="wrap"` (the start tag *interior*)
    // where Vize offers `<div class="wrap">` (the start tag as a unit). Pin the
    // divergence explicitly so it cannot regress into an accident.
    assert.equal(
      actual.some((link) => link[0] === 5 && link[1] === 3 && link[2] === 5 && link[3] === 19),
      false,
      "documented difference: Vize does not offer the bare start-tag interior level",
    );
    assertOrderedSuperset(
      actual,
      VUE_LANGUAGE_SERVER_ATTRIBUTE_CHAIN.filter(
        (link) => !(link[0] === 5 && link[1] === 3 && link[2] === 5 && link[3] === 19),
      ),
      "attribute value",
    );
  });
});

test("selectionRange answers every requested position and keeps chains strictly nested", async () => {
  const positions = [
    { line: 1, character: 8 }, // `count` in `<script setup>`
    { line: 5, character: 4 }, // `div` tag name
    { line: 5, character: 24 }, // `count` in the interpolation
  ];

  await withSelectionRanges("lsp-selrange-multi", positions, (chains) => {
    assert.equal(chains.length, positions.length);

    assert.deepEqual(flatten(chains[0]), [
      [1, 6, 1, 11], // `count`
      [0, 24, 2, 0], // script block content
      [0, 0, 2, 9], // `<script setup lang="ts">` … `</script>`
      [0, 0, 7, 0],
    ]);
    assert.deepEqual(flatten(chains[1]), [
      [5, 3, 5, 6], // `div`
      [5, 2, 5, 20], // `<div class="wrap">`
      [5, 2, 5, 37],
      [4, 10, 6, 0],
      [4, 0, 6, 11],
      [0, 0, 7, 0],
    ]);
    assert.deepEqual(flatten(chains[2]), [
      [5, 23, 5, 28],
      [5, 20, 5, 31],
      [5, 2, 5, 37],
      [4, 10, 6, 0],
      [4, 0, 6, 11],
      [0, 0, 7, 0],
    ]);

    for (const [index, chain] of chains.entries()) {
      const links = flatten(chain);
      for (let position = 1; position < links.length; position += 1) {
        const child = links[position - 1];
        const parent = links[position];
        assert.ok(
          (parent[0] < child[0] || (parent[0] === child[0] && parent[1] <= child[1])) &&
            (parent[2] > child[2] || (parent[2] === child[2] && parent[3] >= child[3])),
          `chain ${index}: parent ${JSON.stringify(parent)} must enclose child ${JSON.stringify(child)}`,
        );
      }
    }
  });
});
