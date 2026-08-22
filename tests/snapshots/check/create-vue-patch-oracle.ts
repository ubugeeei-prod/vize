import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import {
  createVueBrokenCount,
  createVueCleanCount,
  createVueRepairedCount,
  createVueTypecheckAppPath,
  materializeCreateVueTypecheckSource,
} from "../../_helpers/create-vue-typecheck-patch.ts";
import {
  repoRoot,
  symlinkDirectory,
  withPinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  completionLabels,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const appPath = createVueTypecheckAppPath;
const sourceSha256 = "bdafe70baf73a040d432b574108ce0e11823a3c1c1cc8bdfe01d118d6ff7d35a";
const compiledCodeSha256 = "1216a548943fe8a09ab349a913c627d4115f93935b75ec89922415511475eff7";
const formattedSourceSha256 = "70a313db2f293cad24d1ea32a0a24538d81f78df3a73712172968624b092bd42";
const formattedCodeSha256 = "17e394dd258667ffaae5ae7bc92357ff703fcb15f26854244404c8587f47d07b";
const cleanHeading = "  <h1>You did it!</h1>";
const brokenHeading = "  <h1>You did it!</h2>";
const cleanHref = 'href="https://vuejs.org/"';
const brokenHref = "href='https://vuejs.org/'";
const formattedAnchor = '<a href="https://vuejs.org/" rel="noopener" target="_blank">';
const brokenFormattedAnchor = '<a href="https://vuejs.org/" rel="noopener"  target="_blank">';

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

test("create-vue compiler is deterministic and recovers from an exact template parse error", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      const source = fixture.read(appPath);
      const sourceMode = fs.statSync(fixture.resolve(appPath)).mode & 0o777;
      assert.equal(sha256(source), sourceSha256, "pinned create-vue source changed");

      const cleanFirst = runBuild(fixture.workspaceDir, ".vize-compiler-clean-first");
      const cleanSecond = runBuild(fixture.workspaceDir, ".vize-compiler-clean-second");
      assertSuccessfulBuild(cleanFirst);
      assertSuccessfulBuild(cleanSecond);
      assert.equal(
        cleanSecond.outputText,
        cleanFirst.outputText,
        "clean output must be byte-stable",
      );
      assert.equal(fixture.read(appPath), source, "compiler must not mutate the source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const output = cleanFirst.output;
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
          filename: "App.vue",
          macro_artifacts: [],
          script_lang: "ts",
          warnings: [],
        },
      );
      assert.equal(sha256(output.code), compiledCodeSha256, output.code);
      assertParsesAsModule(output.code, "create-vue App.json#code");
      for (const expected of [
        "import { createElementVNode as _createElementVNode",
        '__name: "App"',
        '_createElementVNode("h1", null, "You did it!")',
        'href: "https://vuejs.org/"',
        'rel: "noopener"',
        "/* STABLE_FRAGMENT */",
      ]) {
        assert.equal(count(output.code, expected), 1, `${expected}\n${output.code}`);
      }
      assert.equal(count(output.code, "_cache[0] || (_cache[0] = _createElementVNode("), 1);

      const brokenSource = fixture.applyExactPatch(appPath, cleanHeading, brokenHeading);
      const brokenFirst = runBuild(fixture.workspaceDir, ".vize-compiler-broken-first");
      const brokenSecond = runBuild(fixture.workspaceDir, ".vize-compiler-broken-second");
      assertBrokenBuild(brokenFirst);
      assertBrokenBuild(brokenSecond);
      assert.equal(
        brokenSecond.outputText,
        brokenFirst.outputText,
        "broken output must be byte-stable",
      );
      assert.equal(fixture.read(appPath), brokenSource, "compiler must preserve the broken source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const repairedSource = fixture.applyExactPatch(appPath, brokenHeading, cleanHeading);
      assert.equal(repairedSource, source);
      const repaired = runBuild(fixture.workspaceDir, ".vize-compiler-repaired");
      assertSuccessfulBuild(repaired);
      assert.equal(repaired.outputText, cleanFirst.outputText, "repair must restore exact output");
      assert.equal(fixture.read(appPath), source);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
    },
  );
});

test("create-vue linter reports and repairs one exact HTML quote edit", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      fixture.write(
        "vize.config.json",
        `${JSON.stringify(
          {
            linter: {
              preset: "incremental",
              rules: { "vue/html-quotes": "error" },
            },
          },
          null,
          2,
        )}\n`,
      );

      const source = fixture.read(appPath);
      const sourceMode = fs.statSync(fixture.resolve(appPath)).mode & 0o777;
      assert.equal(sha256(source), sourceSha256, "pinned create-vue source changed");

      const cleanFirst = runLint(fixture.workspaceDir);
      const cleanSecond = runLint(fixture.workspaceDir);
      assertCleanLint(cleanFirst);
      assertCleanLint(cleanSecond);
      assert.equal(cleanSecond.stdout, cleanFirst.stdout, "clean lint JSON must be byte-stable");
      assert.equal(fixture.read(appPath), source, "read-only lint must preserve the source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const brokenSource = fixture.applyExactPatch(appPath, cleanHref, brokenHref);
      const brokenFirst = runLint(fixture.workspaceDir);
      const brokenSecond = runLint(fixture.workspaceDir);
      assertBrokenLint(brokenFirst);
      assertBrokenLint(brokenSecond);
      assert.equal(brokenSecond.stdout, brokenFirst.stdout, "broken lint JSON must be byte-stable");
      assert.equal(fixture.read(appPath), brokenSource, "read-only lint must preserve the edit");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const fixed = runLint(fixture.workspaceDir, true);
      assertCleanLint(fixed);
      assert.equal(fixed.stdout, cleanFirst.stdout, "fixed lint JSON must match clean JSON");
      assert.equal(fixture.read(appPath), source, "--fix must restore the exact pinned source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const repaired = runLint(fixture.workspaceDir);
      const idempotentFix = runLint(fixture.workspaceDir, true);
      assertCleanLint(repaired);
      assertCleanLint(idempotentFix);
      assert.equal(repaired.stdout, cleanFirst.stdout);
      assert.equal(idempotentFix.stdout, cleanFirst.stdout);
      assert.equal(fixture.read(appPath), source);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
    },
  );
});

test("create-vue formatter converges and repairs one exact attribute-spacing edit", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      const pinnedSource = fixture.read(appPath);
      const sourceMode = fs.statSync(fixture.resolve(appPath)).mode & 0o777;
      assert.equal(sha256(pinnedSource), sourceSha256, "pinned create-vue source changed");

      const initialCheck = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(initialCheck, 1, wouldReformatOutput);
      assert.equal(fixture.read(appPath), pinnedSource, "--check must preserve pinned source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const initialWrite = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(initialWrite, 0, reformattedOutput);
      const formattedSource = fixture.read(appPath);
      assert.notEqual(formattedSource, pinnedSource);
      assert.equal(sha256(formattedSource), formattedSourceSha256);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
      assert.equal(formattedSource.startsWith('<script setup lang="ts">\n\n</script>\n'), true);
      assert.equal(formattedSource.endsWith("<style scoped>\n\n</style>\n"), true);
      assert.equal(formattedSource.includes("\r"), false);
      assert.equal(formattedSource.includes("  \n"), false, "canonical source has trailing spaces");
      assert.equal(count(formattedSource, formattedAnchor), 1);
      assert.equal(count(formattedSource, "You did it!"), 1);
      assert.equal(count(formattedSource, "vuejs.org"), 2);

      const compiled = runBuild(fixture.workspaceDir, ".vize-formatter-compile");
      assertSuccessfulBuild(compiled);
      assert.equal(sha256(compiled.output.code), formattedCodeSha256, compiled.output.code);
      assert.equal(count(compiled.output.code, '_createTextVNode(" Visit ")'), 1);
      assert.equal(
        count(compiled.output.code, '_createTextVNode(" to read the documentation ")'),
        1,
      );
      assert.deepEqual(
        {
          css: compiled.output.css,
          errors: compiled.output.errors,
          filename: compiled.output.filename,
          macro_artifacts: compiled.output.macro_artifacts,
          script_lang: compiled.output.script_lang,
          warnings: compiled.output.warnings,
        },
        {
          css: "\n\n",
          errors: [],
          filename: "App.vue",
          macro_artifacts: [],
          script_lang: "ts",
          warnings: [],
        },
      );
      assertParsesAsModule(compiled.output.code, "formatted create-vue App.json#code");

      const cleanFirst = runFmt(fixture.workspaceDir, "--check");
      const cleanSecond = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(cleanFirst, 0, alreadyFormattedOutput);
      assertFmtResult(cleanSecond, 0, alreadyFormattedOutput);
      assert.deepEqual(cleanSecond, cleanFirst, "clean formatter output must be deterministic");

      const brokenSource = fixture.applyExactPatch(appPath, formattedAnchor, brokenFormattedAnchor);
      const brokenFirst = runFmt(fixture.workspaceDir, "--check");
      const brokenSecond = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(brokenFirst, 1, wouldReformatOutput);
      assertFmtResult(brokenSecond, 1, wouldReformatOutput);
      assert.deepEqual(brokenSecond, brokenFirst, "broken formatter output must be deterministic");
      assert.equal(fixture.read(appPath), brokenSource, "--check must preserve broken source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const repaired = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(repaired, 0, reformattedOutput);
      assert.equal(fixture.read(appPath), formattedSource, "--write must restore canonical source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const repairedCheck = runFmt(fixture.workspaceDir, "--check");
      const idempotentWrite = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(repairedCheck, 0, alreadyFormattedOutput);
      assertFmtResult(idempotentWrite, 0, unchangedOutput);
      assert.equal(fixture.read(appPath), formattedSource);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
    },
  );
});

test("create-vue clean, broken, and repaired patches agree across check and LSP", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write(
        "tsconfig.json",
        `${JSON.stringify(
          {
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
          },
          null,
          2,
        )}\n`,
      );
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

      materializeCreateVueTypecheckSource(fixture);
      const cleanSource = fixture.applyExactPatch(
        appPath,
        `${createVueCleanCount}\nconst label = 'ready'`,
        `${createVueCleanCount}\nconst countActive = 2\nconst countArchived = 3\nconst label = 'ready'`,
      );
      const appFile = fixture.resolve(appPath);
      const appUri = pathToFileURL(appFile).href;

      assertCleanCheck(runCheck(fixture.workspaceDir, corsaPath));

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
          textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
        });
        const cleanPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, appUri) && params.version === 1,
        )) as PublishDiagnosticsParams;
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const requestCompletionRanking = async () => {
          const completion = (await session.request("textDocument/completion", {
            textDocument: { uri: appUri },
            position: offsetToPosition(
              cleanSource,
              cleanSource.indexOf("{{ count }}") + "{{ cou".length,
            ),
          })) as
            | Array<{ label: string; sortText?: string }>
            | { items?: Array<{ label: string; sortText?: string }> }
            | null;
          const items = Array.isArray(completion) ? completion : (completion?.items ?? []);
          return items
            .filter((item) => item.label.startsWith("count"))
            .map((item) => ({ label: item.label, sortText: item.sortText ?? null }));
        };

        // #3224 completion-ranking oracle: these candidates share the exact
        // prefix at the request position. Pin both response order and the LSP
        // sortText contract, then repeat the request so map/set iteration or a
        // warm Corsa session cannot silently reshuffle the production answer.
        const expectedCompletionRanking = [
          { label: "count", sortText: "0count" },
          { label: "countActive", sortText: "0countActive" },
          { label: "countArchived", sortText: "0countArchived" },
        ];
        assert.deepEqual(await requestCompletionRanking(), expectedCompletionRanking);
        assert.deepEqual(await requestCompletionRanking(), expectedCompletionRanking);

        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf("{{ count }}") + "{{ cou".length,
          ),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes("count"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf("{{ count }}") + "{{ count".length,
          ),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /count/);
        assert.match(hoverText, /number/i);

        const brokenSource = fixture.applyExactPatch(
          appPath,
          createVueCleanCount,
          createVueBrokenCount,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, appUri) &&
            params.diagnostics.some((diagnostic) =>
              diagnostic.message?.includes("not assignable to type 'number'"),
            ),
        )) as PublishDiagnosticsParams;
        assert.equal(brokenPublish.version, 2, JSON.stringify(brokenPublish));
        const typeError = brokenPublish.diagnostics.find((diagnostic) =>
          diagnostic.message?.includes("not assignable to type 'number'"),
        );
        assert.ok(typeError, JSON.stringify(brokenPublish.diagnostics));
        assert.equal(typeError.source, "vize/types");
        assert.equal(typeError.severity, 1);
        assert.equal(String(typeError.code).replace(/^TS/, ""), "2322");
        assert.match(typeError.message ?? "", /string.*not assignable.*number/i);
        assert.deepEqual(typeError.range?.start, { line: 1, character: 6 });
        assert.deepEqual(typeError.range?.end, { line: 1, character: 11 });

        const brokenCheck = runCheck(fixture.workspaceDir, corsaPath);
        assert.equal(brokenCheck.status, 1, brokenCheck.stderr);
        assert.equal(brokenCheck.report.errorCount, 1, JSON.stringify(brokenCheck.report));
        assert.match(
          brokenCheck.report.files[0]?.diagnostics.join("\n") ?? "",
          /TS2322.*string.*not assignable.*number/i,
        );

        const repairedSource = fixture.applyExactPatch(
          appPath,
          createVueBrokenCount,
          createVueRepairedCount,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, appUri) &&
            params.version === 3 &&
            params.diagnostics.every(
              (diagnostic) => !diagnostic.message?.includes("not assignable to type 'number'"),
            ),
        )) as PublishDiagnosticsParams;
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanCheck(runCheck(fixture.workspaceDir, corsaPath));
      } finally {
        await session.shutdown();
      }
    },
  );
});

type CheckReport = {
  errorCount: number;
  fileCount: number;
  files: Array<{ file: string; diagnostics: string[] }>;
};

function assertCleanCheck(result: ReturnType<typeof runCheck>): void {
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.report.fileCount, 1, JSON.stringify(result.report));
  assert.equal(result.report.errorCount, 0, JSON.stringify(result.report));
  assert.deepEqual(result.report.files[0]?.diagnostics, []);
}

function runCheck(
  workspaceDir: string,
  corsaPath: string,
): { status: number | null; stderr: string; report: CheckReport } {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "check", appPath, "--format", "json", "--quiet", "--corsa-path", corsaPath],
    { cwd: workspaceDir, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, report: JSON.parse(result.stdout) };
}

function resolveVizeCommand(): string[] {
  const candidates = [
    process.env.VIZE_TEST_BIN,
    path.join(repoRoot, "target/debug/vize"),
    path.join(repoRoot, "target/ci/vize"),
    path.join(repoRoot, "target/release/vize"),
    "vize",
  ].filter((candidate): candidate is string => Boolean(candidate));
  for (const candidate of candidates) {
    if (spawnSync(candidate, ["--version"], { cwd: repoRoot }).status === 0) return [candidate];
  }
  return ["cargo", "run", "-q", "-p", "vize", "--"];
}

function resolveTsgoBinary(): string {
  const candidates = [
    process.env.VIZE_TEST_TSGO,
    path.join(repoRoot, "../corsa-bind/.cache/tsgo"),
    path.join(repoRoot, "node_modules/.bin/tsgo"),
    path.join(repoRoot, "tests/node_modules/.bin/tsgo"),
  ].filter((candidate): candidate is string => Boolean(candidate));
  const binary = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(binary, "tsgo binary is required for real-world patch oracles");
  return binary;
}

function symlinkVueTypes(workspaceDir: string): void {
  const candidates = [
    path.join(repoRoot, "node_modules/vue"),
    path.join(repoRoot, "tests/node_modules/vue"),
  ];
  const vuePackage = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for real-world patch oracles");
  symlinkDirectory(vuePackage, path.join(workspaceDir, "node_modules/vue"));
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlinkDirectory(vueNamespace, path.join(workspaceDir, "node_modules/@vue"));
  }
}

type BuildResult = {
  files: string[];
  output: CompilerOutput;
  outputText: string;
  status: number | null;
  stderr: string;
  stdout: string;
};

function runBuild(workspaceDir: string, outputDirectory: string): BuildResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "build", appPath, "--format", "json", "--output", outputDirectory],
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

function assertSuccessfulBuild(result: BuildResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(result.files, ["App.json"]);
  assert.deepEqual(result.output.errors, []);
  assert.deepEqual(result.output.warnings, []);
}

function assertBrokenBuild(result: BuildResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(result.files, ["App.json"]);
  assert.deepEqual(result.output, {
    filename: "App.vue",
    code: 'export default {\n  __name: "App",\n  setup(__props) {}\n};',
    css: null,
    errors: [
      'Template compilation errors: [CompilerError { code: InvalidEndTag, message: "Invalid end tag.", loc: Some(SourceLocation { start: Position { offset: 18, line: 2, column: 18 }, end: Position { offset: 23, line: 2, column: 23 }, source: "</h2>" }) }, CompilerError { code: MissingEndTag, message: "Element is missing end tag.", loc: Some(SourceLocation { start: Position { offset: 3, line: 2, column: 3 }, end: Position { offset: 7, line: 2, column: 7 }, source: "<h1>" }) }]',
    ],
    warnings: [],
    script_lang: "ts",
    macro_artifacts: [],
  });
}

type LintResult = {
  report: LintReport;
  status: number | null;
  stderr: string;
  stdout: string;
};

function runLint(workspaceDir: string, fix = false): LintResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "lint", appPath, "--format", "json", ...(fix ? ["--fix"] : [])],
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

function assertCleanLint(result: LintResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.stderr, "");
  assert.deepEqual(result.report, [
    {
      file: appPath,
      messages: [],
      errorCount: 0,
      warningCount: 0,
    },
  ]);
}

function assertBrokenLint(result: LintResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.equal(result.stderr, "");
  assert.deepEqual(result.report, [
    {
      file: appPath,
      messages: [
        {
          ruleId: "vue/html-quotes",
          ruleDocsPath: "docs/content/rules/vue.md",
          severity: 2,
          message: "[vize:vue/html-quotes] Expected double quotes but found single quotes",
          line: 6,
          // The reported range is the attribute value node, delimiters
          // included, which is the span `eslint-plugin-vue` reports: line 6
          // opens the value at column 19 and closes it at column 38.
          column: 19,
          endLine: 6,
          endColumn: 39,
          help: "Use consistent quote style for HTML attribute values.",
        },
      ],
      errorCount: 1,
      warningCount: 0,
    },
  ]);
}

type CommandResult = {
  status: number | null;
  stderr: string;
  stdout: string;
};

const wouldReformatOutput = `Found 1 file(s)
Would reformat: ${appPath}

Checked 1 file(s)
  1 file(s) would be reformatted
`;

const reformattedOutput = `Found 1 file(s)
Reformatted: ${appPath}

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

function runFmt(workspaceDir: string, mode: "--check" | "--write"): CommandResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(command, [...prefixArgs, "fmt", mode, appPath], {
    cwd: workspaceDir,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, stdout: result.stdout };
}

function assertFmtResult(result: CommandResult, status: number, stderr: string): void {
  assert.deepEqual(result, { status, stdout: "", stderr });
}

function sha256(source: string | Buffer): string {
  return createHash("sha256").update(source).digest("hex");
}

function count(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
