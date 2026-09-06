import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveVizeCommand,
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type CommandResult,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const sourcePath = "src/views/dashboard/admin/components/TransactionTable.vue";
const cleanBinding = ':data="list"';
const brokenBinding = ':data="missingList"';
const brokenLintBinding = 'v-bind:data="list"';
const formattedTable = '  <el-table :data="list" style="width: 100%;padding-top: 15px;">';
const brokenFormattedTable = '  <el-table :data="list"  style="width: 100%;padding-top: 15px;">';
const formattedSourceSha256 = "bb911ad5e002bee6c7c7800d92b787c486cdb6e56fade344575bcc8f464e5b3c";

type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

type LintReport = Array<{
  errorCount: number;
  file: string;
  messages: Array<{
    column: number;
    endColumn: number;
    endLine: number;
    help: string;
    line: number;
    message: string;
    ruleDocsPath: string;
    ruleId: string;
    severity: number;
  }>;
  warningCount: number;
}>;

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

test("vue-element-admin linter reports and repairs one exact legacy SFC edit", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      fixture.write(
        "vize.config.json",
        json({
          compiler: { compatibility: { vueVersion: "2" } },
          linter: {
            preset: "incremental",
            rules: { "vue/v-bind-style": "error" },
          },
        }),
      );

      const source = fixture.read(sourcePath);
      const sourceMode = fs.statSync(fixture.resolve(sourcePath)).mode & 0o777;
      const cleanFirst = runVizeLint(fixture.workspaceDir, sourcePath);
      const cleanSecond = runVizeLint(fixture.workspaceDir, sourcePath);

      assertCleanLint(cleanFirst);
      assertCleanLint(cleanSecond);
      assert.equal(cleanSecond.stdout, cleanFirst.stdout, "clean lint JSON must be byte-stable");
      assert.equal(
        fixture.read(sourcePath),
        source,
        "read-only lint must not mutate the Vue source",
      );

      const brokenSource = fixture.applyExactPatch(sourcePath, cleanBinding, brokenLintBinding);
      const brokenFirst = runVizeLint(fixture.workspaceDir, sourcePath);
      const brokenSecond = runVizeLint(fixture.workspaceDir, sourcePath);

      assertBrokenLint(brokenFirst);
      assertBrokenLint(brokenSecond);
      assert.equal(brokenSecond.stdout, brokenFirst.stdout, "broken lint JSON must be byte-stable");
      assert.equal(
        fixture.read(sourcePath),
        brokenSource,
        "read-only lint must preserve the broken source",
      );

      const fixed = runVizeLint(fixture.workspaceDir, sourcePath, true);
      assertCleanLint(fixed);
      assert.equal(fixture.read(sourcePath), source, "--fix must restore the exact pinned source");
      assert.equal(fs.statSync(fixture.resolve(sourcePath)).mode & 0o777, sourceMode);

      const repaired = runVizeLint(fixture.workspaceDir, sourcePath);
      assertCleanLint(repaired);
      assert.equal(repaired.stdout, cleanFirst.stdout, "repaired lint JSON must match clean JSON");
    },
  );
});

test("vue-element-admin formatter converges and repairs one exact legacy SFC edit", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-element-admin", includePaths: [sourcePath] },
    async (fixture) => {
      fixture.write("vize.config.json", json({ compiler: { compatibility: { vueVersion: "2" } } }));

      const pinnedSource = fixture.read(sourcePath);
      const sourceMode = fs.statSync(fixture.resolve(sourcePath)).mode & 0o777;
      const initialCheck = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      assertFmtResult(initialCheck, 1, wouldReformatOutput);
      assert.equal(
        fixture.read(sourcePath),
        pinnedSource,
        "--check must preserve the pinned source",
      );

      const initialWrite = runVizeFmt(fixture.workspaceDir, sourcePath, "--write");
      assertFmtResult(initialWrite, 0, reformattedOutput);
      const formattedSource = fixture.read(sourcePath);
      assert.notEqual(formattedSource, pinnedSource);
      assert.equal(sha256(formattedSource), formattedSourceSha256);
      assert.equal(fs.statSync(fixture.resolve(sourcePath)).mode & 0o777, sourceMode);
      assert.equal(formattedSource.startsWith("<script>\n"), true);
      assert.equal(formattedSource.endsWith("</template>\n"), true);
      assert.equal(formattedSource.includes("\r"), false);
      assert.equal(count(formattedSource, 'slot-scope="scope"'), 2);
      assert.equal(count(formattedSource, 'slot-scope="{row}"'), 1);
      assert.equal(count(formattedSource, "scope.row.order_no | orderNoFilter"), 1);
      assert.equal(count(formattedSource, "scope.row.price | toThousandFilter"), 1);
      assert.equal(count(formattedSource, "row.status | statusFilter"), 1);
      assert.equal(formattedSource.includes("v-slot"), false);

      const compiled = runVizeBuild(fixture.workspaceDir, sourcePath, ".vize-formatter-compile");
      assert.equal(compiled.status, 0, compiled.stderr || compiled.stdout);
      assert.deepEqual(compiled.output.errors, []);
      assert.deepEqual(compiled.output.warnings, []);
      assertParsesAsModule(compiled.output.code, "formatted TransactionTable.json#code");
      assert.ok(
        compiled.output.code.indexOf("data: $data.list") <
          compiled.output.code.indexOf('style: "width: 100%;padding-top: 15px;"'),
        compiled.output.code,
      );

      const cleanFirst = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      const cleanSecond = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      assertFmtResult(cleanFirst, 0, alreadyFormattedOutput);
      assertFmtResult(cleanSecond, 0, alreadyFormattedOutput);
      assert.deepEqual(cleanSecond, cleanFirst, "clean formatter output must be deterministic");

      const brokenSource = fixture.applyExactPatch(
        sourcePath,
        formattedTable,
        brokenFormattedTable,
      );
      const brokenFirst = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      const brokenSecond = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      assertFmtResult(brokenFirst, 1, wouldReformatOutput);
      assertFmtResult(brokenSecond, 1, wouldReformatOutput);
      assert.deepEqual(brokenSecond, brokenFirst, "broken formatter output must be deterministic");
      assert.equal(fixture.read(sourcePath), brokenSource, "--check must preserve broken source");

      const repaired = runVizeFmt(fixture.workspaceDir, sourcePath, "--write");
      assertFmtResult(repaired, 0, reformattedOutput);
      assert.equal(
        fixture.read(sourcePath),
        formattedSource,
        "--write must exactly repair the source",
      );
      assert.equal(fs.statSync(fixture.resolve(sourcePath)).mode & 0o777, sourceMode);

      const repairedCheck = runVizeFmt(fixture.workspaceDir, sourcePath, "--check");
      const idempotentWrite = runVizeFmt(fixture.workspaceDir, sourcePath, "--write");
      assertFmtResult(repairedCheck, 0, alreadyFormattedOutput);
      assertFmtResult(idempotentWrite, 0, unchangedOutput);
      assert.equal(fixture.read(sourcePath), formattedSource);
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
  assert.deepEqual(omitProgramEvidence(result.report), {
    files: [{ file: sourcePath, diagnostics: [] }],
    errorCount: 0,
    warningCount: 0,
    fileCount: 1,
  });
}

function assertBrokenCheck(result: VizeCheckResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.deepEqual(omitProgramEvidence(result.report), {
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

function assertCleanLint(result: VizeLintResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.stderr, "");
  assert.deepEqual(result.report, [
    {
      file: sourcePath,
      messages: [],
      errorCount: 0,
      warningCount: 0,
    },
  ]);
}

function assertBrokenLint(result: VizeLintResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.equal(result.stderr, "");
  assert.deepEqual(result.report, [
    {
      file: sourcePath,
      messages: [
        {
          ruleId: "vue/v-bind-style",
          ruleDocsPath: "docs/content/rules/vue.md",
          severity: 2,
          message: "[vize:vue/v-bind-style] Prefer shorthand `:` over `v-bind:`",
          line: 2,
          column: 13,
          endLine: 2,
          endColumn: 31,
          help: 'Use :attr="value" instead of v-bind:attr="value"',
        },
      ],
      errorCount: 1,
      warningCount: 0,
    },
  ]);
}

type VizeLintResult = {
  report: LintReport;
  status: number | null;
  stderr: string;
  stdout: string;
};

function runVizeLint(workspaceDir: string, input: string, fix = false): VizeLintResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "lint", input, "--format", "json", ...(fix ? ["--fix"] : [])],
    {
      cwd: workspaceDir,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 64 * 1024 * 1024,
      timeout: 120_000,
    },
  );
  if (result.error != null) throw result.error;
  return {
    report: JSON.parse(result.stdout) as LintReport,
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

const wouldReformatOutput = `Found 1 file(s)
Would reformat: ${sourcePath}

Checked 1 file(s)
  1 file(s) would be reformatted
`;

const reformattedOutput = `Found 1 file(s)
Reformatted: ${sourcePath}

Formatted 1 file(s)
  1 file(s) reformatted
`;

const alreadyFormattedOutput = `Found 1 file(s)

Checked 1 file(s)
  1 file(s) already formatted
`;

const unchangedOutput = `Found 1 file(s)

Formatted 1 file(s)
  1 file(s) unchanged
`;

function assertFmtResult(result: CommandResult, status: number, stderr: string): void {
  assert.deepEqual(result, { status, stdout: "", stderr });
}

function runVizeFmt(
  workspaceDir: string,
  input: string,
  mode: "--check" | "--write",
): CommandResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(command, [...prefixArgs, "fmt", mode, input], {
    cwd: workspaceDir,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, stdout: result.stdout };
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
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
    // Legacy vue-element-admin is plain JavaScript; checking it at all is the
    // `checkJs` opt-in TypeScript requires for a `lang="js"` block (#3322).
    checkJs: true,
    lib: ["ES2022", "DOM"],
    paths: { "@/*": ["./src/*"] },
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
