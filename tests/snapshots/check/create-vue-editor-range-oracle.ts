import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import { resolveTsgoBinary, symlinkVueTypes } from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri, offsetToPosition } from "../../tooling/support/lsp/assertions.ts";
import {
  assertAuthoredRanges,
  assertTokenText,
  authoredPosition as position,
  authoredRange as range,
  decodeSemanticTokens,
  sameRange,
  semanticToken as token,
  semanticTokensLegend,
  type LspLocation,
} from "../../tooling/support/lsp/authored-ranges.ts";
import type { LspRange, PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

// #2971 audit item 8: editor features must answer in *authored* `.vue`
// coordinates, never the generated virtual-TS coordinates actually queried.
// Hover, semantic tokens, inlay-hint columns, folding lines, and reference
// exhaustiveness were only checked loosely elsewhere (`length > 0`,
// `.some(...)`, regex on markdown), so a systematic shift passed them.
const appPath = "template/bare/typescript/src/App.vue";
const upstreamSha256 = "bdafe70baf73a040d432b574108ce0e11823a3c1c1cc8bdfe01d118d6ff7d35a";

const upstreamScriptBlock = '<script setup lang="ts"></script>';
const authoredScriptBlock = `<script setup lang="ts">
import { computed, ref } from 'vue'

const visitCount = ref(0)
const doubled = computed(() => visitCount.value * 2)
</script>`;
const upstreamHeading = "  <h1>You did it!</h1>";
const authoredHeading = `  <h1>{{ doubled }}</h1>
  <button @click="visitCount++">bump</button>`;

// Authored 0-based line map after both patches, which every expectation below
// is stated against: 0 `<script setup>`, 3 `const visitCount`, 4 `const
// doubled`, 5 `</script>`, 7 `<template>`, 8 `<h1>{{ doubled }}`, 9 `<button
// @click="visitCount++">`, 14 `</template>`, 16 `<style scoped></style>`.
const VISIT_COUNT_DECL = range(3, 6, 16);
const VISIT_COUNT_SCRIPT_USE = range(4, 31, 41);
const VISIT_COUNT_TEMPLATE_USE = range(9, 18, 28);
const DOUBLED_DECL = range(4, 6, 13);
const DOUBLED_TEMPLATE_USE = range(8, 9, 16);

test("create-vue editor features answer in authored Vue ranges", async (t) => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      assert.equal(
        createHash("sha256").update(fixture.read(appPath)).digest("hex"),
        upstreamSha256,
        "pinned create-vue App.vue changed; re-derive every authored range below",
      );
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write("vize.config.json", json({ typeChecker: { corsaPath } }));

      fixture.applyExactPatch(appPath, upstreamScriptBlock, authoredScriptBlock);
      const source = fixture.applyExactPatch(appPath, upstreamHeading, authoredHeading);
      const uri = pathToFileURL(fixture.resolve(appPath)).href;
      const at = (needle: string, offset: number) =>
        offsetToPosition(source, source.indexOf(needle) + offset);
      const templateDoubled = at("{{ doubled }}", 4);
      const templateVisitCount = at('@click="visitCount', 12);
      const scriptVisitCount = at("const visitCount", 8);

      const session = new LspSession();
      try {
        const legend = semanticTokensLegend(
          await session.initialize(fixture.workspaceDir, initializationOptions),
        );
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        const publish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
          120_000,
        )) as PublishDiagnosticsParams;
        assert.deepEqual(publish.diagnostics, [], JSON.stringify(publish.diagnostics));

        const ask = (method: string, params: Record<string, unknown>) =>
          session.request(method, { textDocument: { uri }, ...params }, 60_000);

        await t.test("hover resolves a template binding through the authored script", async () => {
          const hover = (await ask("textDocument/hover", { position: templateDoubled })) as {
            contents: { kind: string; value: string };
          };
          assert.equal(hover.contents.kind, "markdown");
          // Template scope, so the backend reports the *unwrapped* computed
          // value rather than the script-side `ComputedRef` wrapper (#3321).
          assert.equal(fencedBlock(hover.contents.value), "var doubled: number");
          assert.match(hover.contents.value, /_Resolved through Vize virtual TypeScript_/);

          const scriptHover = (await ask("textDocument/hover", {
            position: scriptVisitCount,
          })) as { contents: { value: string } };
          assert.equal(
            fencedBlock(scriptHover.contents.value),
            "const visitCount: Ref<number, number>",
          );
          assert.match(scriptHover.contents.value, /_Resolved through Vize virtual TypeScript_/);
        });

        // Closed with #3321: backend-answered hovers carry the range tsgo
        // reports, mapped back into authored `.vue` coordinates.
        await t.test("hover reports the authored range of the hovered binding", async () => {
          const hover = (await ask("textDocument/hover", { position: templateDoubled })) as {
            range?: LspRange;
          };
          assert.deepEqual(hover.range, DOUBLED_TEMPLATE_USE);
        });

        await t.test("definition lands on the authored declaration span", async () => {
          const fromTemplate = (await ask("textDocument/definition", {
            position: templateDoubled,
          })) as LspLocation;
          assert.equal(fromTemplate.uri, uri);
          assert.deepEqual(fromTemplate.range, DOUBLED_DECL);

          const fromEventHandler = (await ask("textDocument/definition", {
            position: templateVisitCount,
          })) as LspLocation;
          assert.equal(fromEventHandler.uri, uri);
          assert.deepEqual(fromEventHandler.range, VISIT_COUNT_DECL);
          assertAuthoredRanges(source, [fromTemplate.range, fromEventHandler.range]);
        });

        await t.test("document highlight covers every authored occurrence", async () => {
          const highlights = (await ask("textDocument/documentHighlight", {
            position: scriptVisitCount,
          })) as Array<{ kind: number; range: LspRange }>;
          const hit = highlights.map((highlight) => highlight.range);
          assert.deepEqual(highlights, [
            { kind: 3, range: VISIT_COUNT_DECL },
            { kind: 2, range: VISIT_COUNT_SCRIPT_USE },
            { kind: 2, range: VISIT_COUNT_TEMPLATE_USE },
          ]);
          assertAuthoredRanges(source, hit);
        });

        await t.test("references stay inside the authored document", async () => {
          const references = (await ask("textDocument/references", {
            position: scriptVisitCount,
            context: { includeDeclaration: true },
          })) as LspLocation[];
          assert.deepEqual(
            [...new Set(references.map((reference) => reference.uri))],
            [uri],
            "references must never escape the authored SFC",
          );
          for (const expected of [VISIT_COUNT_DECL, VISIT_COUNT_TEMPLATE_USE]) {
            assert.ok(
              references.some((reference) => sameRange(reference.range, expected)),
              `missing ${JSON.stringify(expected)} in ${JSON.stringify(references)}`,
            );
          }
        });

        // Regression for #3325: script-side hits used to leak block-relative
        // coordinates, answering [3:6-16, 4:6-16, 5:31-41, 9:18-28] — the two
        // script occurrences one line below authored, with 5:31-41 landing past
        // the end of the 9-character `</script>` line and the authored use at
        // 4:31-41 missing entirely.
        await t.test("references map script-side hits to authored ranges", async () => {
          const references = (await ask("textDocument/references", {
            position: scriptVisitCount,
            context: { includeDeclaration: true },
          })) as LspLocation[];
          const hit = references.map((reference) => reference.range);
          assert.deepEqual(hit, [
            VISIT_COUNT_DECL,
            VISIT_COUNT_SCRIPT_USE,
            VISIT_COUNT_TEMPLATE_USE,
          ]);
          assertAuthoredRanges(source, hit);
        });

        await t.test("prepareRename and rename touch only authored spans", async () => {
          const prepared = (await ask("textDocument/prepareRename", {
            position: templateDoubled,
          })) as LspRange;
          assert.deepEqual(prepared, DOUBLED_TEMPLATE_USE);

          const fromScript = (await ask("textDocument/rename", {
            position: scriptVisitCount,
            newName: "visits",
          })) as { changes: Record<string, Array<{ range: LspRange; newText: string }>> };
          assert.deepEqual(Object.keys(fromScript.changes), [uri]);
          assert.deepEqual(fromScript.changes[uri], [
            { newText: "visits", range: VISIT_COUNT_DECL },
            { newText: "visits", range: VISIT_COUNT_SCRIPT_USE },
            { newText: "visits", range: VISIT_COUNT_TEMPLATE_USE },
          ]);

          const fromTemplate = (await ask("textDocument/rename", {
            position: templateDoubled,
            newName: "twice",
          })) as { changes: Record<string, Array<{ range: LspRange; newText: string }>> };
          assert.deepEqual(Object.keys(fromTemplate.changes), [uri]);
          assert.deepEqual(fromTemplate.changes[uri], [
            { newText: "twice", range: DOUBLED_DECL },
            { newText: "twice", range: DOUBLED_TEMPLATE_USE },
          ]);
          assertAuthoredRanges(source, [
            prepared,
            ...fromScript.changes[uri].map((edit) => edit.range),
            ...fromTemplate.changes[uri].map((edit) => edit.range),
          ]);
        });

        await t.test("semantic tokens decode to authored positions", async () => {
          const request = ask("textDocument/semanticTokens/full", {});
          const full = decodeSemanticTokens((await request) as { data: number[] }, legend);
          assert.deepEqual(full, [
            token(3, 19, 3, "function", ["defaultLibrary"]),
            token(4, 16, 8, "function", ["defaultLibrary"]),
            token(8, 9, 7, "variable"),
            token(9, 10, 6, "event"),
            token(9, 18, 10, "variable"),
            token(9, 28, 2, "operator"),
          ]);
          const covered = ["ref", "computed", "doubled", "@click", "visitCount", "++"];
          assertTokenText(source, full, covered);

          // The template-only window must re-base its deltas without shifting
          // the absolute authored positions of the tokens it keeps.
          const templateOnly = decodeSemanticTokens(
            (await ask("textDocument/semanticTokens/range", {
              range: { start: { line: 7, character: 0 }, end: { line: 14, character: 0 } },
            })) as { data: number[] },
            legend,
          );
          assert.deepEqual(templateOnly, full.slice(2));
        });

        await t.test("inlay hints anchor to the authored end of each declaration", async () => {
          const hints = await ask("textDocument/inlayHint", {
            range: { start: { line: 0, character: 0 }, end: { line: 17, character: 0 } },
          });
          assert.deepEqual(hints, [
            {
              kind: 1,
              label: ": Ref<number>",
              paddingLeft: true,
              position: VISIT_COUNT_DECL.end,
              tooltip: "Vue reactive binding (Ref)",
            },
            {
              kind: 1,
              label: ": ComputedRef<number>",
              paddingLeft: true,
              position: DOUBLED_DECL.end,
              tooltip: "Vue reactive binding (ComputedRef)",
            },
          ]);
        });

        await t.test("document symbols and folding ranges follow the authored blocks", async () => {
          const symbols = await ask("textDocument/documentSymbol", {});
          assert.deepEqual(symbols, [
            {
              kind: 2,
              name: "template",
              range: { start: position(7, 0), end: position(14, 0) },
              selectionRange: range(7, 0, 10),
            },
            {
              detail: "ts",
              kind: 2,
              name: "script setup",
              range: { start: position(0, 0), end: position(5, 0) },
              selectionRange: range(0, 0, 14),
            },
            {
              // Single-line blocks collapse: `<style scoped>` and `</style>`
              // share line 16, so the block range is zero-width.
              kind: 2,
              name: "style scoped",
              range: { start: position(16, 0), end: position(16, 0) },
              selectionRange: range(16, 0, 7),
            },
          ]);

          const folding = await ask("textDocument/foldingRange", {});
          assert.deepEqual(folding, [
            { collapsedText: "template", startLine: 7, endLine: 14, kind: "region" },
            { collapsedText: "script setup", startLine: 0, endLine: 5, kind: "region" },
          ]);
        });
      } finally {
        await session.shutdown();
      }
    },
  );
});

function fencedBlock(markdown: string): string {
  const match = /```typescript\n([\s\S]*?)\n```/.exec(markdown);
  assert.ok(match, `hover markdown has no typescript block: ${markdown}`);
  return match[1];
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

const initializationOptions = {
  definition: true,
  documentSymbols: true,
  editor: true,
  foldingRanges: true,
  hover: true,
  inlayHints: true,
  lint: false,
  references: true,
  rename: true,
  semanticTokens: true,
  typecheck: true,
};

const tsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
  include: ["template/bare/typescript/src/**/*.vue"],
};
