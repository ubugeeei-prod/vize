import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  withPinnedFixtureWorkspace,
  type PinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type {
  LspInitializationOptions,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const sourcePath = "src/views/dashboard/admin/components/TransactionTable.vue";

// Options API script-side edit: rename the `data` key `list` together with its
// script usage in `fetchData`, leaving the template binding `:data="list"`
// stale. The only expected diagnostic is the template binding at its authored
// range; the reverse patches restore the pinned source byte-for-byte.
const cleanDataKey = "list: null";
const brokenDataKey = "tableList: null";
const cleanMethodAssignment = "this.list = ";
const brokenMethodAssignment = "this.tableList = ";

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

type DialectVariant = {
  name: string;
  config: (corsaPath: string) => unknown;
  initializationOptions: LspInitializationOptions;
};

// The same broken/repaired cycle must hold for both legacy dialect spellings:
// the historical `compiler.compatibility.vueVersion: "2"` used by the batch
// oracle and the unified `vue.version: "2.7"` selector (#2971 audit item 7).
const dialectVariants: DialectVariant[] = [
  {
    name: 'legacy dialect (compiler.compatibility.vueVersion: "2")',
    config: (corsaPath) => ({
      compiler: { compatibility: { vueVersion: "2" } },
      globalTypes: { toThousandFilter: "any" },
      typeChecker: { corsaPath, legacyVue2: true },
    }),
    initializationOptions: { editor: true, legacyVue2: true, lint: false, typecheck: true },
  },
  {
    name: 'Vue 2.7 dialect (vue.version: "2.7")',
    config: (corsaPath) => ({
      vue: { version: "2.7" },
      globalTypes: { toThousandFilter: "any" },
      typeChecker: { corsaPath, legacyVue2: true },
    }),
    initializationOptions: { editor: true, legacyVue2: true, lint: false, typecheck: true },
  },
];

for (const variant of dialectVariants) {
  test(`vue-element-admin ${variant.name} repairs a data-key rename over didChange`, async () => {
    const corsaPath = resolveTsgoBinary();

    await withPinnedFixtureWorkspace(
      { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
      async (fixture) => {
        symlinkVueTypes(fixture.workspaceDir);
        fixture.write("src/api/remote-search.d.ts", remoteSearchDeclaration);
        fixture.write("tsconfig.json", json(tsconfig));
        fixture.write("vize.config.json", json(variant.config(corsaPath)));

        const source = fixture.read(sourcePath);
        const sourceUri = pathToFileURL(fixture.resolve(sourcePath)).href;
        assertCleanCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

        const session = new LspSession();
        try {
          await session.initialize(fixture.workspaceDir, variant.initializationOptions);
          const serverPid = session.processId;
          session.notify("textDocument/didOpen", {
            textDocument: { uri: sourceUri, languageId: "vue", version: 1, text: source },
          });
          const cleanPublish = await waitForDiagnostics(session, sourceUri, 1, false);
          assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

          const brokenSource = applyDataKeyRename(fixture);
          assert.notEqual(source, brokenSource);
          session.notify("textDocument/didChange", {
            textDocument: { uri: sourceUri, version: 2 },
            contentChanges: [{ text: brokenSource }],
          });
          const brokenPublish = await waitForDiagnostics(session, sourceUri, 2, true);
          assert.deepEqual(brokenPublish.diagnostics, [staleListBindingDiagnostic]);
          assertBrokenCheck(runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]));

          const repairedSource = repairDataKeyRename(fixture);
          assert.equal(repairedSource, source, "reverse patches must restore the pinned source");
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
          assert.equal(session.processId, serverPid, "one server must serve the whole cycle");
        } finally {
          await session.shutdown();
        }
      },
    );
  });
}

// Known gap (found while adding this oracle): with only `vue.version: "2.7"`
// in vize.config.json (no `typeChecker.legacyVue2`, no `legacyVue2`
// initialization option), `vize check` accepts the pristine TransactionTable
// (`slot-scope` scopes and filters resolve), but `vize lsp` publishes TS2304 /
// TS2552 for `scope`, `row`, and the filter names on the same clean document.
// The LSP must derive legacy template lowering from `vue.version` exactly like
// the CLI before this can be asserted.
test("vue-element-admin vue.version 2.7 alone drives legacy lowering in the LSP", {
  skip:
    "vize lsp ignores the vue.version 2.7 dialect for slot-scope/filter lowering " +
    "unless legacyVue2 is also set, diverging from vize check on the clean file",
});

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectStaleListBinding: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      params.diagnostics.some(isStaleListDiagnostic) === expectStaleListBinding,
    120_000,
  )) as PublishDiagnosticsParams;
}

function isStaleListDiagnostic(diagnostic: PublishDiagnosticsParams["diagnostics"][number]) {
  return (
    String(diagnostic.code).replace(/^TS/, "") === "2304" &&
    diagnostic.message === "Cannot find name 'list'."
  );
}

function applyDataKeyRename(fixture: PinnedFixtureWorkspace): string {
  fixture.applyExactPatch(sourcePath, cleanDataKey, brokenDataKey);
  return fixture.applyExactPatch(sourcePath, cleanMethodAssignment, brokenMethodAssignment);
}

function repairDataKeyRename(fixture: PinnedFixtureWorkspace): string {
  fixture.applyExactPatch(sourcePath, brokenDataKey, cleanDataKey);
  return fixture.applyExactPatch(sourcePath, brokenMethodAssignment, cleanMethodAssignment);
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
        diagnostics: ["error:2:20 [TS2304] Cannot find name 'list'."],
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
