import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const sourcePath = "src/views/dashboard/admin/components/TransactionTable.vue";
const cleanBinding = ':data="list"';
const brokenBinding = ':data="missingList"';

const missingListDiagnostic = {
  range: {
    start: { line: 1, character: 19 },
    end: { line: 1, character: 30 },
  },
  severity: 1,
  code: 2304,
  source: "vize/types",
  message: "Cannot find name 'missingList'.",
};

test("vue-element-admin legacy slot scopes recover exact CLI diagnostics", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("src/api/remote-search.d.ts", remoteSearchDeclaration);
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write(
        "vize.config.json",
        json({
          compiler: { compatibility: { vueVersion: "2" } },
          globalTypes: { toThousandFilter: "any" },
          typeChecker: { corsaPath, legacyVue2: true },
        }),
      );

      const source = fixture.read(sourcePath);
      const sourceUri = pathToFileURL(fixture.resolve(sourcePath)).href;
      assertCleanCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

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
        const cleanPublish = await waitForDiagnostics(session, sourceUri, 1, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const brokenSource = fixture.applyExactPatch(sourcePath, cleanBinding, brokenBinding);
        assert.notEqual(source, brokenSource);
        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = await waitForDiagnostics(session, sourceUri, 2, true);
        assert.deepEqual(brokenPublish.diagnostics, [missingListDiagnostic]);
        assertBrokenCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

        const repairedSource = fixture.applyExactPatch(sourcePath, brokenBinding, cleanBinding);
        assert.equal(repairedSource, source);
        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = await waitForDiagnostics(session, sourceUri, 3, false);
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));
      } finally {
        await session.shutdown();
      }
    },
  );
});

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectMissingList: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      params.diagnostics.some(isMissingListDiagnostic) === expectMissingList,
    120_000,
  )) as PublishDiagnosticsParams;
}

function isMissingListDiagnostic(diagnostic: PublishDiagnosticsParams["diagnostics"][number]) {
  return (
    String(diagnostic.code).replace(/^TS/, "") === "2304" &&
    diagnostic.message === "Cannot find name 'missingList'."
  );
}

function assertCleanCheck(result: VizeCheckResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(result.report, {
    files: [{ file: sourcePath, diagnostics: [] }],
    errorCount: 0,
    warningCount: 0,
    fileCount: 1,
  });
}

function assertBrokenCheck(result: VizeCheckResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.deepEqual(result.report, {
    files: [
      {
        file: sourcePath,
        diagnostics: ["error:2:20 [TS2304] Cannot find name 'missingList'."],
      },
    ],
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
  });
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

const tsconfig = {
  compilerOptions: {
    allowJs: true,
    baseUrl: ".",
    lib: ["ES2022", "DOM"],
    paths: { "@/*": ["src/*"] },
    skipLibCheck: true,
    strict: false,
  },
  include: ["src"],
};

const remoteSearchDeclaration = `export function transactionList(): Promise<{
  data: {
    items: Array<{ order_no: string; price: number; status: "success" | "pending" }>;
  };
}>;
`;
