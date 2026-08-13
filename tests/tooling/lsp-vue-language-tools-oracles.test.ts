import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  completionLabels,
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import type { LspDiagnostic, LspRange } from "./support/lsp/protocol.ts";
import {
  normalizeItems,
  rangeFor,
  startsForEdits,
  withScorecardDocument,
} from "./support/lsp/vue-language-tools-scorecard.ts";

test("Maestro scorecard executes representative must-include and must-exclude LSP oracles", async (t) => {
  await withScorecardDocument(async ({ publish, session, source, uri, workspaceDir }) => {
    await t.test("diagnostics publish lint findings on authored template ranges", () => {
      const keyDiagnostic = publish.diagnostics.find(
        (diagnostic) =>
          diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
      );
      assert.ok(keyDiagnostic, JSON.stringify(publish.diagnostics));
      assert.deepEqual(keyDiagnostic.range, rangeFor(source, 'v-for="item in items"'));
    });

    await t.test("completion includes ranked bindings and excludes context leakage", async () => {
      const response = (await session.request("textDocument/completion", {
        textDocument: { uri },
        position: offsetToPosition(source, source.indexOf("        cou") + "        cou".length),
      })) as Array<Record<string, unknown>> | { items?: Array<Record<string, unknown>> } | null;
      const labels = completionLabels(response as never);
      assert.ok(labels.includes("count"), labels.join(", "));
      assert.ok(labels.includes("message"), labels.join(", "));
      for (const forbidden of ["v-if", "class", "@click"]) {
        assert.equal(
          labels.includes(forbidden),
          false,
          `${forbidden} leaked into ${labels.join(", ")}`,
        );
      }
      const count = normalizeItems(response).find((item) => item.label === "count");
      assert.equal(count?.sortText, "0count");
    });

    await t.test(
      "hover, definition, references, and rename stay on authored bindings",
      async () => {
        const messageUsageOffset = source.lastIndexOf("message }}</button>") + "message".length;
        const messageUsagePosition = offsetToPosition(source, messageUsageOffset);
        const declarationPosition = offsetToPosition(source, source.indexOf("message = ref"));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri },
          position: messageUsagePosition,
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /message/);
        assert.match(hoverText, /Ref<string>|Template binding from script/);
        assert.doesNotMatch(hoverText, /Vue event listener/);

        const definition = await session.request("textDocument/definition", {
          textDocument: { uri },
          position: messageUsagePosition,
        });
        const location = firstLocation(definition as never);
        assert.equal(location.uri, uri);
        assert.deepEqual(location.range.start, declarationPosition);

        const references = (await session.request("textDocument/references", {
          textDocument: { uri },
          position: messageUsagePosition,
          context: { includeDeclaration: true },
        })) as Array<{ uri: string; range: LspRange }>;
        const referenceStarts = references.map((reference) => reference.range.start);
        assert.ok(
          referenceStarts.some(
            (start) =>
              start.line === declarationPosition.line &&
              start.character === declarationPosition.character,
          ),
          JSON.stringify(references),
        );
        assert.ok(
          referenceStarts.some(
            (start) =>
              start.line === messageUsagePosition.line &&
              start.character === messageUsagePosition.character - "message".length,
          ),
          JSON.stringify(references),
        );

        const directiveRename = await session.request("textDocument/prepareRename", {
          textDocument: { uri },
          position: offsetToPosition(source, source.indexOf("v-for") + 2),
        });
        assert.equal(directiveRename, null);

        const edit = (await session.request("textDocument/rename", {
          textDocument: { uri },
          position: messageUsagePosition,
          newName: "title",
        })) as { changes?: Record<string, Array<{ range: LspRange; newText: string }>> } | null;
        assert.deepEqual(startsForEdits(edit, uri), [
          declarationPosition,
          offsetToPosition(source, source.indexOf(':label="message"') + ':label="'.length),
          offsetToPosition(source, source.lastIndexOf("message }}</button>")),
        ]);
        assert.ok((edit?.changes?.[uri] ?? []).every((item) => item.newText === "title"));
      },
    );

    await t.test(
      "code actions include quick fixes and exclude unrelated requested kinds",
      async () => {
        const diagnostic = publish.diagnostics.find(
          (item): item is LspDiagnostic =>
            item.source === "vize/lint" && item.code === "vue/no-multi-spaces",
        );
        assert.ok(diagnostic, JSON.stringify(publish.diagnostics));
        const actions = (await session.request("textDocument/codeAction", {
          textDocument: { uri },
          range: diagnostic.range,
          context: { diagnostics: [diagnostic], only: ["quickfix"] },
        })) as Array<{ title?: string; kind?: string; isPreferred?: boolean }> | null;
        const titles = (actions ?? []).map((action) => action.title);
        assert.deepEqual(titles, [
          "Fix: Replace multiple spaces with single space",
          "Suppress with @vize:forget (vue/no-multi-spaces)",
        ]);
        assert.equal(actions?.[0]?.kind, "quickfix");
        assert.equal(actions?.[0]?.isPreferred, true);

        const refactors = await session.request("textDocument/codeAction", {
          textDocument: { uri },
          range: diagnostic.range,
          context: { diagnostics: [diagnostic], only: ["refactor", "source"] },
        });
        assert.equal(refactors, null);
      },
    );

    await t.test(
      "semantic tokens and inlay hints include signal without plain-text leakage",
      async () => {
        const semanticTokens = (await session.request("textDocument/semanticTokens/full", {
          textDocument: { uri },
        })) as { data?: number[] } | null;
        assert.ok(Array.isArray(semanticTokens?.data), JSON.stringify(semanticTokens));
        assert.equal(semanticTokens.data.length % 5, 0);
        assert.ok(semanticTokens.data.length > 0);

        const hints = (await session.request("textDocument/inlayHint", {
          textDocument: { uri },
          range: { start: { line: 0, character: 0 }, end: { line: 1000, character: 0 } },
        })) as Array<{ label: string | Array<{ value: string }> }> | null;
        const hintLabels = (hints ?? []).map((hint) =>
          typeof hint.label === "string"
            ? hint.label
            : hint.label.map((part) => part.value).join(""),
        );
        assert.ok(hintLabels.includes(": Ref<number>"), hintLabels.join(", "));
        assert.ok(hintLabels.includes(": ComputedRef<number>"), hintLabels.join(", "));
        assert.equal(hintLabels.includes(": Ref<boolean>"), false, hintLabels.join(", "));

        const noiseSource = `<template>
  <p>email dev@example.com and plain text v-if @click :class</p>
</template>
`;
        const noisePath = path.join(workspaceDir, "Noise.vue");
        const noiseUri = pathToFileURL(noisePath).href;
        fs.writeFileSync(noisePath, noiseSource, "utf8");
        session.notify("textDocument/didOpen", {
          textDocument: { uri: noiseUri, languageId: "vue", version: 1, text: noiseSource },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, noiseUri),
        );
        const noiseTokens = (await session.request("textDocument/semanticTokens/range", {
          textDocument: { uri: noiseUri },
          range: { start: { line: 1, character: 0 }, end: { line: 2, character: 0 } },
        })) as { data?: number[] } | null;
        assert.deepEqual(noiseTokens?.data, []);
      },
    );

    await t.test(
      "document features, file rename, and workspace symbols include and exclude targets",
      async () => {
        const symbols = (await session.request("textDocument/documentSymbol", {
          textDocument: { uri },
        })) as Array<{ name: string }> | null;
        const symbolNames = (symbols ?? []).map((symbol) => symbol.name);
        assert.deepEqual(symbolNames, ["template", "script setup", "style module=$style"]);

        const folding = await session.request("textDocument/foldingRange", {
          textDocument: { uri },
        });
        assert.ok(Array.isArray(folding), JSON.stringify(folding));

        const links = (await session.request("textDocument/documentLink", {
          textDocument: { uri },
        })) as Array<{ target?: string }> | null;
        const linkTargets = (links ?? []).map((link) =>
          path.basename(decodeURIComponent(new URL(link.target ?? "").pathname)),
        );
        assert.ok(linkTargets.includes("Child.vue"), linkTargets.join(", "));
        assert.ok(linkTargets.includes("useThing.mjs"), linkTargets.join(", "));
        assert.equal(linkTargets.includes("Missing.vue"), false, linkTargets.join(", "));

        const renamedChild = pathToFileURL(path.join(workspaceDir, "RenamedChild.vue")).href;
        const renameEdit = (await session.request("workspace/willRenameFiles", {
          files: [
            {
              oldUri: pathToFileURL(path.join(workspaceDir, "Child.vue")).href,
              newUri: renamedChild,
            },
          ],
        })) as { changes?: Record<string, Array<{ newText: string }>> } | null;
        const fileRenameTexts = (renameEdit?.changes?.[uri] ?? []).map((edit) => edit.newText);
        assert.ok(fileRenameTexts.includes("./RenamedChild.vue"), JSON.stringify(renameEdit));
        assert.equal(fileRenameTexts.includes("./useThing"), false, JSON.stringify(renameEdit));

        const workspaceSymbols = (await session.request("workspace/symbol", {
          query: "submitMessage",
        })) as Array<{ name: string; location: { uri: string } }> | null;
        assert.ok(
          workspaceSymbols?.some(
            (symbol) => symbol.name === "submitMessage" && symbol.location.uri === uri,
          ),
          JSON.stringify(workspaceSymbols),
        );
        assert.equal(
          await session.request("workspace/symbol", { query: "missingScorecardSymbol" }),
          null,
        );
      },
    );
  });
});
