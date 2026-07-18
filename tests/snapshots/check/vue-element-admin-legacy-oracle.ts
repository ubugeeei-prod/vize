import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  resolveVizeCommand,
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

type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

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

test("vue-element-admin compiler lowers legacy slot scopes and filters deterministically", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      fixture.write("vize.config.json", json({ compiler: { compatibility: { vueVersion: "2" } } }));

      const source = fixture.read(sourcePath);
      const first = runVizeBuild(fixture.workspaceDir, sourcePath, ".vize-compiler-first");
      const second = runVizeBuild(fixture.workspaceDir, sourcePath, ".vize-compiler-second");

      assert.equal(first.status, 0, first.stderr || first.stdout);
      assert.equal(second.status, 0, second.stderr || second.stdout);
      assert.deepEqual(first.files, ["TransactionTable.json"]);
      assert.deepEqual(second.files, first.files);
      assert.equal(second.outputText, first.outputText, "compiler output must be byte-stable");
      assert.equal(fixture.read(sourcePath), source, "compiler must not mutate the Vue source");

      const output = first.output;
      assert.deepEqual(
        {
          css: output.css,
          errors: output.errors,
          filename: output.filename,
          macro_artifacts: output.macro_artifacts,
          script_lang: output.script_lang,
          warnings: output.warnings,
        },
        {
          css: null,
          errors: [],
          filename: "TransactionTable.vue",
          macro_artifacts: [],
          script_lang: "js",
          warnings: [],
        },
      );
      assertParsesAsModule(output.code, "TransactionTable.json#code");

      assert.equal(count(output.code, "resolveFilter as _resolveFilter"), 1, output.code);
      assert.equal(count(output.code, "_resolveFilter("), 3, output.code);
      for (const filter of ["orderNoFilter", "toThousandFilter", "statusFilter"]) {
        assert.equal(
          count(output.code, `const _filter_${filter} = _resolveFilter("${filter}")`),
          1,
          output.code,
        );
        assert.equal(count(output.code, `_filter_${filter}(`), 1, output.code);
      }

      assert.equal(count(output.code, "default: _withCtx((scope) => ["), 2, output.code);
      assert.equal(count(output.code, "default: _withCtx(({row}) => ["), 1, output.code);
      for (const expected of [
        "_toDisplayString(_filter_orderNoFilter(scope.row.order_no))",
        "_toDisplayString(_filter_toThousandFilter(scope.row.price))",
        "type: _filter_statusFilter(row.status)",
        "_toDisplayString(row.status)",
      ]) {
        assert.equal(count(output.code, expected), 1, `${expected}\n${output.code}`);
      }

      for (const forbidden of [
        "slot-scope",
        '_createElementVNode("template"',
        "scope.row.order_no | orderNoFilter",
        "scope.row.price | toThousandFilter",
        "row.status | statusFilter",
        "_ctx.scope",
        "_ctx.row",
      ]) {
        assert.equal(output.code.includes(forbidden), false, `${forbidden}\n${output.code}`);
      }
    },
  );
});

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

function runVizeBuild(
  workspaceDir: string,
  input: string,
  outputDirectory: string,
): {
  files: string[];
  output: CompilerOutput;
  outputText: string;
  status: number | null;
  stderr: string;
  stdout: string;
} {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "build", input, "--format", "json", "--output", outputDirectory],
    {
      cwd: workspaceDir,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 64 * 1024 * 1024,
      timeout: 120_000,
    },
  );
  if (result.error != null) throw result.error;

  const outputRoot = path.join(workspaceDir, outputDirectory);
  const files = fs.existsSync(outputRoot)
    ? fs
        .readdirSync(outputRoot, { recursive: true })
        .map(String)
        .filter((entry) => entry.endsWith(".json"))
        .sort()
    : [];
  const outputText =
    files.length === 1 ? fs.readFileSync(path.join(outputRoot, files[0]), "utf8") : "";
  return {
    files,
    output: outputText === "" ? ({} as CompilerOutput) : (JSON.parse(outputText) as CompilerOutput),
    outputText,
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

function count(source: string, needle: string): number {
  return source.split(needle).length - 1;
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
