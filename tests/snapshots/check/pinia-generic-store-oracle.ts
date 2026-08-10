import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { symlinkDirectory, withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  type CommandResult,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import {
  completionLabels,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const appPath = "packages/playground/src/views/CounterSetupStore.vue";
const storePath = "packages/playground/src/stores/counterSetup.ts";
const piniaTypesPath = "packages/pinia/src/types.ts";
const cleanIncrement = `  function increment(amount = 1) {
    if (typeof amount !== 'number') {
      amount = 1
    }
    state.incrementedTimes++
    state.n += amount
  }`;
const brokenIncrement = `  function increment(amount = '1') {
    const numericAmount = Number(amount)
    state.incrementedTimes++
    state.n += numericAmount
  }`;

test("Pinia generic stores stay exact across dependency edits", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    {
      fixtureId: "pinia",
      includePaths: [appPath, storePath, piniaTypesPath, "packages/playground/src/vite-env.d.ts"],
    },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("packages/pinia/src/rootStore.d.ts", "export interface Pinia {}\n");
      fixture.write("packages/pinia/src/index.d.ts", piniaDeclaration);
      fixture.write(
        "packages/pinia/package.json",
        `${JSON.stringify({ name: "pinia", type: "module", types: "./src/index.d.ts" }, null, 2)}\n`,
      );
      symlinkDirectory(fixture.resolve("packages/pinia"), fixture.resolve("node_modules/pinia"));
      fixture.write("tsconfig.json", `${JSON.stringify(tsconfig, null, 2)}\n`);
      fixture.write(
        "vize.config.json",
        `${JSON.stringify(
          {
            lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
            typeChecker: { corsaPath },
          },
          null,
          2,
        )}\n`,
      );

      const cleanSource = fixture.applyExactPatch(
        appPath,
        `    <button @click="counter.increment(10)">+10</button>`,
        `    <button @click="counter.increment()">+10</button>`,
      );
      const storeSource = fixture.read(storePath);
      const appFile = fixture.resolve(appPath);
      const storeFile = fixture.resolve(storePath);
      const appUri = pathToFileURL(appFile).href;
      const storeUri = pathToFileURL(storeFile).href;

      assertCleanParity(
        runVizeCheck(fixture.workspaceDir, corsaPath, [appPath]),
        runVueTsc(fixture.workspaceDir, vueTscPath),
      );

      const session = new LspSession();
      try {
        await session.initialize(fixture.workspaceDir, {
          completion: true,
          editor: true,
          hover: true,
          lint: false,
          typecheck: true,
        });
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri: storeUri,
            languageId: "typescript",
            version: 1,
            text: storeSource,
          },
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
        });
        const cleanPublish = await waitForAppDiagnostics(session, appUri, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const memberOffset = cleanSource.indexOf("counter.n") + "coun".length;
        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(cleanSource, memberOffset),
        });
        const labels = completionLabels(completion);
        for (const label of ["counter", "n", "useCounter"]) {
          assert.ok(labels.includes(label), `${label}: ${labels.join(", ")}`);
        }
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const brokenStore = fixture.applyExactPatch(storePath, cleanIncrement, brokenIncrement);
        session.notify("textDocument/didChange", {
          textDocument: { uri: storeUri, version: 2 },
          contentChanges: [{ text: brokenStore }],
        });
        const brokenPublish = await waitForAppDiagnostics(session, appUri, true);
        assertSingleMismatch(brokenPublish.diagnostics, cleanSource);
        assertBrokenParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [appPath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );

        const repairedStore = fixture.applyExactPatch(storePath, brokenIncrement, cleanIncrement);
        session.notify("textDocument/didChange", {
          textDocument: { uri: storeUri, version: 3 },
          contentChanges: [{ text: repairedStore }],
        });
        const repairedPublish = await waitForAppDiagnostics(session, appUri, false);
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [appPath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );
      } finally {
        await session.shutdown();
      }
    },
  );
});

function assertCleanParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.doesNotMatch(`${vueTsc.stdout}\n${vueTsc.stderr}`, /error TS\d+:/);
  assert.equal(vize.status, 0, vize.stderr || vize.stdout);
  assert.deepEqual(
    {
      errorCount: vize.report.errorCount,
      fileCount: vize.report.fileCount,
      files: vize.report.files,
      warningCount: vize.report.warningCount,
    },
    {
      errorCount: 0,
      fileCount: 2,
      // The authored store the app imports is reported alongside it, so a
      // regression that only surfaces in the dependency cannot hide (#3996).
      files: [
        { diagnostics: [], file: storePath },
        { diagnostics: [], file: appPath },
      ],
      warningCount: 0,
    },
  );
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 2, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.deepEqual(
    vize.report.files.map((file) => file.file),
    [storePath, appPath],
    JSON.stringify(vize.report),
  );
  assert.deepEqual(vize.report.files[0]?.diagnostics, [], JSON.stringify(vize.report));
  assert.match(
    vize.report.files[1]?.diagnostics.join("\n") ?? "",
    /TS2345.*number.*not assignable.*string/i,
  );
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2345:/g)].length, 1, output);
  assert.doesNotMatch(output, /error TS(?!2345)\d+:/, output);
}

async function waitForAppDiagnostics(
  session: LspSession,
  uri: string,
  expectMismatch: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === 1 &&
      hasMismatch(params.diagnostics) === expectMismatch,
    120_000,
  )) as PublishDiagnosticsParams;
}

function hasMismatch(diagnostics: LspDiagnostic[]): boolean {
  return diagnostics.some(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2345" &&
      /number.*not assignable.*string/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMismatch(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMismatch([diagnostic]), JSON.stringify(diagnostic));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf("counter.increment(100)") + 18);
  assert.deepEqual(diagnostic.range, {
    start,
    end: { line: start.line, character: start.character + 3 },
  });
}

const tsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    paths: { pinia: ["./packages/pinia/src/index.d.ts"] },
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
    types: ["vite/client"],
  },
  include: [appPath, storePath, piniaTypesPath, "packages/**/*.d.ts"],
};

const piniaDeclaration = `import type {
  DefineSetupStoreOptions,
  StoreDefinition,
  _ExtractActionsFromSetupStore,
  _ExtractGettersFromSetupStore,
  _ExtractStateFromSetupStore,
} from './types'

export type * from './types'

export interface SetupStoreHelpers {
  action: <Fn extends (...args: any[]) => any>(fn: Fn, name?: string) => Fn
}

export interface SetupStoreDefinition<Id extends string, SS>
  extends StoreDefinition<
    Id,
    _ExtractStateFromSetupStore<SS>,
    _ExtractGettersFromSetupStore<SS>,
    _ExtractActionsFromSetupStore<SS>
  > {}

export declare function defineStore<
  Id extends string,
  SS,
>(
  id: Id,
  storeSetup: (helpers: SetupStoreHelpers) => SS,
  options?: DefineSetupStoreOptions<
    Id,
    _ExtractStateFromSetupStore<SS>,
    _ExtractGettersFromSetupStore<SS>,
    _ExtractActionsFromSetupStore<SS>
  >
): SetupStoreDefinition<Id, SS>

export declare function acceptHMRUpdate<T extends StoreDefinition>(
  initialUseStore: T,
  hot: unknown
): (newModule: unknown) => void
`;
