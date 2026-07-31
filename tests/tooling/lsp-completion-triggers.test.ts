import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * Behavioural cover for the completion trigger characters `vize lsp` advertises
 * (#3458).
 *
 * `lsp-capabilities.test.ts` pins the exact ordered list. This suite pins that
 * each character actually *does* something: a trigger whose position answers
 * with an empty list is worse than no trigger at all, because the editor reacts
 * and offers nothing. Every case asserts a must-include AND a must-exclude set
 * with `assert.deepEqual` over the filtered labels — never `.includes` on its
 * own, which would pass on a list of the wrong things.
 *
 * The session runs with `typecheck: false`, so every label below comes from
 * Maestro's own template analysis rather than a tsserver round trip.
 */

type CompletionItem = { label: string };

const HEAD = `<script setup lang="ts">
const count = 1
const items = [1, 2]
function pick(n: number) { return n }
</script>

<template>
`;
const TAIL = `
</template>
`;

/** Every binding the fixture's `<script setup>` puts in template scope. */
const BINDINGS = ["count", "items", "pick"];
/** Directive names, which only belong to an attribute-*name* position. */
const DIRECTIVES = ["v-if", "v-for"];

type Case = {
  /** The trigger character this position is reached by typing. */
  trigger: string;
  /** A single template line; `|` marks the caret and is stripped. */
  line: string;
  include: string[];
  exclude: string[];
};

const CASES: Case[] = [
  // `=` had no handler at all before #3458: the attribute-name list rejects
  // every candidate once the prefix contains `=`, so this position answered
  // with nothing.
  { trigger: "=", line: `  <div :title=|></div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "=", line: `  <div @click=|></div>`, include: BINDINGS, exclude: DIRECTIVES },
  // End of a start tag: the text node that follows takes elements and
  // directives, never bare bindings.
  { trigger: ">", line: `  <div>|</div>`, include: DIRECTIVES, exclude: BINDINGS },
  // kebab-case attribute names are one hyphenated word, so the list has to
  // refresh mid-word.
  {
    trigger: "-",
    line: `  <div :aria-|></div>`,
    include: ["aria-label"],
    exclude: [...DIRECTIVES, ...BINDINGS],
  },
  { trigger: "{", line: `  <div>{{|}}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "}", line: `  <div>{{ count }|}</div>`, include: DIRECTIVES, exclude: BINDINGS },
  { trigger: "(", line: `  <div>{{ pick(| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: ")", line: `  <div>{{ pick(count)| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "[", line: `  <div>{{ items[| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "]", line: `  <div>{{ items[0]| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "$", line: `  <div>{{ $| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "+", line: `  <div>{{ count +| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "^", line: `  <div>{{ count ^| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "*", line: `  <div>{{ count *| }}</div>`, include: BINDINGS, exclude: DIRECTIVES },
  // Already advertised before #3458; kept so the whole advertised set is
  // covered by one suite.
  { trigger: '"', line: `  <div :title="|"></div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: "'", line: `  <div :title='|'></div>`, include: BINDINGS, exclude: DIRECTIVES },
  { trigger: ":", line: `  <div :|></div>`, include: ["class"], exclude: BINDINGS },
  { trigger: "@", line: `  <div @|></div>`, include: ["@click"], exclude: BINDINGS },
  { trigger: "<", line: `  <|`, include: ["Transition"], exclude: BINDINGS },
  { trigger: ".", line: `  <div>{{ count.|}}</div>`, include: [], exclude: DIRECTIVES },
  { trigger: "#", line: `  <Child #|></Child>`, include: ["#"], exclude: BINDINGS },
  { trigger: "/", line: `  <div></|`, include: DIRECTIVES, exclude: BINDINGS },
];

function positionAt(text: string, offset: number): { line: number; character: number } {
  let line = 0;
  let character = 0;
  for (let index = 0; index < offset; index += 1) {
    if (text[index] === "\n") {
      line += 1;
      character = 0;
    } else {
      character += 1;
    }
  }
  return { line, character };
}

test("every advertised completion trigger answers with a useful list", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-completion-triggers");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

    for (const [index, testCase] of CASES.entries()) {
      const marked = HEAD + testCase.line + TAIL;
      const caret = marked.indexOf("|");
      const source = marked.replace("|", "");
      const filePath = path.join(workspaceDir, `Case${index}.vue`);
      const uri = pathToFileURL(filePath).href;
      fs.writeFileSync(filePath, source, "utf8");
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });

      const result = (await session.request("textDocument/completion", {
        textDocument: { uri },
        position: positionAt(source, caret),
      })) as CompletionItem[] | { items: CompletionItem[] } | null;
      const items = Array.isArray(result) ? result : (result?.items ?? []);
      const labels = new Set(items.map((item) => item.label));
      const where = `${testCase.trigger} in ${JSON.stringify(testCase.line)}`;

      // Non-empty is the floor: the trigger fired, so something must come back.
      assert.ok(labels.size > 0, `${where} opened an empty completion list`);
      assert.deepEqual(
        testCase.include.filter((label) => labels.has(label)),
        testCase.include,
        `${where} is missing expected labels`,
      );
      assert.deepEqual(
        testCase.exclude.filter((label) => labels.has(label)),
        [],
        `${where} offered labels that do not belong at this position`,
      );
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
