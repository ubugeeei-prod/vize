import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  type CheckReport,
  omitProgramEvidence,
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
} from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const sourcePath = "src/views/dashboard/admin/components/TransactionTable.vue";

// The sibling legacy oracle renames the `data` key *and* its script usage, so
// the only stale reference left is the template binding. Renaming the key
// alone additionally leaves `this.list` stale in `fetchData`, which is the
// shape that used to publish a generated-coordinate diagnostic
// ("Property 'list' does not exist on type '__VizeThis'") positioned past the
// authored EOF — line 67 of a 56-line file — that `vize check` suppressed
// (#3299). That finding is real, and #3596 anchored it back onto the authored
// `this.list`, so it is published now. The rule this pins is unchanged: every
// published range must be addressable in the authored document, and the LSP
// must agree with the CLI.
const cleanDataKey = "list: null";
const brokenDataKey = "tableList: null";

const staleListBindingDiagnostic = {
  range: {
    start: { line: 1, character: 19 },
    end: { line: 1, character: 23 },
  },
  severity: 1,
  code: 2304,
  source: "vize/types",
  message: "Cannot find name 'list'.",
};

// `this.list` inside `fetchData`, reached through the Options API typed
// instance bridge. The LSP adds the Vue `.value` hint the CLI does not.
const staleListMemberDiagnostic = {
  range: {
    start: { line: 49, character: 13 },
    end: { line: 49, character: 17 },
  },
  severity: 1,
  code: 2339,
  source: "vize/types",
  message:
    "Property 'list' does not exist on type '__VizeThis'.\n\n" +
    "If you intended to read the reactive value, try `.value`. (vize/types)",
};

test("vue-element-admin data-key-only rename keeps every published range inside the document", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("src/api/remote-search.d.ts", remoteSearchDeclaration);
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write("vize.config.json", json(config(corsaPath)));

      const source = fixture.read(sourcePath);
      const sourceUri = pathToFileURL(fixture.resolve(sourcePath)).href;

      const session = new LspSession();
      try {
        await session.initialize(fixture.workspaceDir, {
          editor: true,
          legacyVue2: true,
          lint: false,
          typecheck: true,
        });

        session.notify("textDocument/didOpen", {
          textDocument: { uri: sourceUri, languageId: "vue", version: 1, text: source },
        });
        const cleanPublish = await waitForPublish(session, sourceUri, 1);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const brokenSource = fixture.applyExactPatch(sourcePath, cleanDataKey, brokenDataKey);
        assert.notEqual(source, brokenSource);
        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = await waitForPublish(session, sourceUri, 2);

        assertRangesWithinDocument(brokenPublish.diagnostics, brokenSource);
        assert.deepEqual(byPosition(brokenPublish.diagnostics), [
          staleListBindingDiagnostic,
          staleListMemberDiagnostic,
        ]);

        // The CLI's diagnostic set is the reference: the LSP must not add a
        // diagnostic the CLI drops as unmappable.
        const check = runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]);
        assert.equal(check.status, 1, check.stderr || check.stdout);
        assert.deepEqual(sortedReport(check.report), {
          files: [
            {
              file: sourcePath,
              diagnostics: [
                "error:2:20 [TS2304] Cannot find name 'list'.",
                "error:50:14 [TS2339] Property 'list' does not exist on type '__VizeThis'.",
              ],
            },
          ],
          errorCount: 2,
          warningCount: 0,
          fileCount: 1,
        });

        const repairedSource = fixture.applyExactPatch(sourcePath, brokenDataKey, cleanDataKey);
        assert.equal(repairedSource, source, "the reverse patch must restore the pinned source");
        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = await waitForPublish(session, sourceUri, 3);
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
      } finally {
        await session.shutdown();
      }
    },
  );
});

/// Publication order is not part of the contract; the set and the ranges are.
function byPosition(
  diagnostics: PublishDiagnosticsParams["diagnostics"],
): PublishDiagnosticsParams["diagnostics"] {
  return [...diagnostics].sort(
    (left, right) =>
      left.range.start.line - right.range.start.line ||
      left.range.start.character - right.range.start.character,
  );
}

function sortedReport(report: CheckReport): Omit<CheckReport, "programs"> {
  const diagnosticReport = omitProgramEvidence(report);
  return {
    ...diagnosticReport,
    files: diagnosticReport.files.map((file) => ({
      ...file,
      diagnostics: [...file.diagnostics].sort(),
    })),
  };
}

function assertRangesWithinDocument(
  diagnostics: PublishDiagnosticsParams["diagnostics"],
  source: string,
): void {
  const lines = source.split("\n");
  for (const diagnostic of diagnostics) {
    const label = `[${String(diagnostic.code)}] ${diagnostic.message}`;
    assert.ok(
      diagnostic.range.start.line < lines.length,
      `start line ${diagnostic.range.start.line} is past EOF (${lines.length} lines): ${label}`,
    );
    assert.ok(
      diagnostic.range.end.line < lines.length,
      `end line ${diagnostic.range.end.line} is past EOF (${lines.length} lines): ${label}`,
    );
    const endLine = lines[diagnostic.range.end.line] ?? "";
    assert.ok(
      diagnostic.range.end.character <= endLine.length,
      `end character ${diagnostic.range.end.character} is past the line end (${endLine.length}): ${label}`,
    );
  }
}

async function waitForPublish(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    120_000,
  )) as PublishDiagnosticsParams;
}

function config(corsaPath: string) {
  return {
    compiler: { compatibility: { vueVersion: "2" } },
    globalTypes: { toThousandFilter: "any" },
    typeChecker: { corsaPath, legacyVue2: true },
  };
}

const tsconfig = {
  compilerOptions: {
    target: "esnext",
    module: "esnext",
    moduleResolution: "bundler",
    // TransactionTable.vue is a plain-JavaScript SFC: type-checking it at all
    // is the `checkJs` opt-in TypeScript requires for a `lang="js"` block, and
    // this oracle exists to pin the checked surface (#3322).
    allowJs: true,
    checkJs: true,
    strict: false,
    skipLibCheck: true,
    noEmit: true,
    paths: { "@/*": ["./src/*"] },
  },
  include: ["src/**/*"],
};

const remoteSearchDeclaration =
  "export declare function searchUser(name: string): Promise<{ data: { items: unknown[] } }>;\n" +
  "export declare function transactionList(): Promise<{ data: { items: unknown[] } }>;\n";

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
