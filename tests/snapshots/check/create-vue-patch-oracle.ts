import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";
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

const appPath = "template/bare/typescript/src/App.vue";
const sourceSha256 = "bdafe70baf73a040d432b574108ce0e11823a3c1c1cc8bdfe01d118d6ff7d35a";
const compiledCodeSha256 = "1216a548943fe8a09ab349a913c627d4115f93935b75ec89922415511475eff7";
const cleanHeading = "  <h1>You did it!</h1>";
const brokenHeading = "  <h1>You did it!</h2>";

type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

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

      fixture.applyExactPatch(
        appPath,
        '<script setup lang="ts"></script>',
        `<script setup lang="ts">\nconst count: number = 1\nconst label = 'ready'\n</script>`,
      );
      const cleanSource = fixture.applyExactPatch(
        appPath,
        "  <h1>You did it!</h1>",
        "  <h1>{{ label }}</h1>\n  <p>{{ count }}</p>",
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
          "const count: number = 1",
          "const count: number = 'broken'",
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
          "const count: number = 'broken'",
          "const count: number = 2",
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
      'Template compilation errors: [CompilerError { code: InvalidEndTag, message: "Invalid end tag.", loc: Some(SourceLocation { start: Position { offset: 18, line: 1, column: 19 }, end: Position { offset: 23, line: 1, column: 24 }, source: "</h2>" }) }, CompilerError { code: MissingEndTag, message: "Element is missing end tag.", loc: Some(SourceLocation { start: Position { offset: 3, line: 1, column: 4 }, end: Position { offset: 7, line: 1, column: 8 }, source: "<h1>" }) }]',
    ],
    warnings: [],
    script_lang: "ts",
    macro_artifacts: [],
  });
}

function sha256(source: string | Buffer): string {
  return createHash("sha256").update(source).digest("hex");
}

function count(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
