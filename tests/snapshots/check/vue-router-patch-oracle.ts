import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { symlinkDirectory, withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  type CommandResult,
  resolveVizeCommand,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import {
  completionLabels,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const appPath = "packages/playground/src/AppLink.vue";
const routerManifestPath = "packages/router/package.json";
const symbol = "packageBoundary";
const sourceSha256 = "f5c888da7bae9c61151c7fbfde578ccdb768e75fbe2e69637d8298ca4f702c96";
const cleanOutputSha256 = "baa6667f47df3663c55e938e7f93f0e12eb2f0bd931cc1b3de43b0dbbc110883";
const cleanCodeSha256 = "d305d39f63d298418bb52dc4bd1d6c02d73a3f5a52205c6166a8811dbec7c3b9";
const brokenOutputSha256 = "15e571e43307b3ac025db3929f4f11029f1b7f5991d11b405433b6a31c5bc90e";
const brokenCodeSha256 = "eed24c03d97029fca5d3d13f81ac416099d3876e7f36572d113acc0768872679";
const cleanHref = ':href="String(to)"';
const brokenHref = ':href="String(to"';

type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

type BuildResult = {
  files: string[];
  output: CompilerOutput;
  outputText: string;
  status: number | null;
  stderr: string;
  stdout: string;
};

test("Vue Router compiler is deterministic and recovers from an exact expression error", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-router", includePaths: [appPath] },
    async (fixture) => {
      const source = fixture.read(appPath);
      const sourceMode = fs.statSync(fixture.resolve(appPath)).mode & 0o777;
      assert.equal(sha256(source), sourceSha256, "pinned Vue Router source changed");
      assert.equal(count(source, cleanHref), 1, "compiler patch anchor must stay unique");

      const cleanFirst = runBuild(fixture.workspaceDir, ".vize-compiler-clean-first");
      const cleanSecond = runBuild(fixture.workspaceDir, ".vize-compiler-clean-second");
      assertSuccessfulBuild(cleanFirst);
      assertSuccessfulBuild(cleanSecond);
      assert.equal(sha256(cleanFirst.outputText), cleanOutputSha256);
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
          filename: "AppLink.vue",
          macro_artifacts: [],
          script_lang: "ts",
          warnings: [],
        },
      );
      assert.equal(sha256(output.code), cleanCodeSha256, output.code);
      assertParsesAsModule(output.code, "vue-router AppLink.json#code");
      for (const expected of [
        "renderSlot as _renderSlot, mergeProps as _mergeProps",
        '__name: "AppLink"',
        "props: { disabled: {",
        "href: String(__props.to)",
        "href: _unref(href)",
        "onClick: _cache[0] || (_cache[0] = (...args) => navigate && navigate(...args))",
      ]) {
        assert.equal(count(output.code, expected), 1, `${expected}\n${output.code}`);
      }
      assert.equal(count(output.code, "key: 0"), 1, output.code);
      assert.equal(count(output.code, "key: 1"), 1, output.code);
      assert.equal(
        count(output.code, '_createElementBlock("a", _mergeProps(_unref(attrs), {'),
        2,
        output.code,
      );
      assert.equal(
        count(output.code, 'class: _normalizeClass(["router-link", classes.value])'),
        2,
        output.code,
      );
      assert.equal(count(output.code, '_renderSlot(_ctx.$slots, "default")'), 2, output.code);
      assert.equal(output.code.includes("RouterLinkProps"), false, output.code);

      const brokenSource = fixture.applyExactPatch(appPath, cleanHref, brokenHref);
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

      const repairedSource = fixture.applyExactPatch(appPath, brokenHref, cleanHref);
      assert.equal(repairedSource, source);
      const repaired = runBuild(fixture.workspaceDir, ".vize-compiler-repaired");
      assertSuccessfulBuild(repaired);
      assert.equal(repaired.outputText, cleanFirst.outputText, "repair must restore exact output");
      assert.equal(fixture.read(appPath), source);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
    },
  );
});

test("Vue Router package exports stay exact across clean, broken, and repaired edits", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-router", includePaths: [appPath, routerManifestPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("packages/router/dist/vue-router.d.ts", routerDeclaration);
      symlinkDirectory(
        fixture.resolve("packages/router"),
        fixture.resolve("node_modules/vue-router"),
      );
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

      fixture.applyExactPatch(
        appPath,
        "const attrs = useAttrs()",
        `const attrs = useAttrs()\nconst ${symbol}: number = 1`,
      );
      const cleanSource = fixture.applyExactPatch(
        appPath,
        "<template>\n",
        `<template>\n  <span>{{ ${symbol} }}</span>\n`,
      );
      const appFile = fixture.resolve(appPath);
      const appUri = pathToFileURL(appFile).href;

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
          textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
        });
        const cleanPublish = await waitForDiagnostics(session, appUri, 1);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf(`{{ ${symbol} }}`) + `{{ packageB`.length,
          ),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes(symbol), labels.join(", "));
        assert.ok(labels.includes("to"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf(`{{ ${symbol} }}`) + `{{ ${symbol}`.length,
          ),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, new RegExp(symbol));
        assert.match(hoverText, /number/i);

        const usageDefinition = await definitionLocations(session, appUri, cleanSource, {
          offset: cleanSource.lastIndexOf("RouterLinkProps") + 1,
        });
        const importStart = offsetToPosition(cleanSource, cleanSource.indexOf("RouterLinkProps"));
        assert.deepEqual(usageDefinition, [
          {
            range: {
              start: importStart,
              end: {
                line: importStart.line,
                character: importStart.character + "RouterLinkProps".length,
              },
            },
            uri: appUri,
          },
        ]);

        const packageDefinition = await definitionLocations(session, appUri, cleanSource, {
          offset: cleanSource.indexOf("'vue-router'") + 2,
        });
        assert.deepEqual(packageDefinition, [
          {
            range: {
              start: { line: 0, character: 0 },
              end: { line: 0, character: 0 },
            },
            uri: pathToFileURL(fixture.resolve("packages/router/dist/vue-router.d.ts")).href,
          },
        ]);

        const brokenSource = fixture.applyExactPatch(
          appPath,
          `const ${symbol}: number = 1`,
          `const ${symbol}: number = 'broken'`,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = await waitForDiagnostics(session, appUri, 2, true);
        assertSingleMismatch(brokenPublish.diagnostics, brokenSource);
        assertBrokenParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [appPath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );

        const repairedSource = fixture.applyExactPatch(
          appPath,
          `const ${symbol}: number = 'broken'`,
          `const ${symbol}: number = 2`,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = await waitForDiagnostics(session, appUri, 3, false);
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
  assert.deepEqual(result.files, ["AppLink.json"]);
  assert.deepEqual(result.output.errors, []);
  assert.deepEqual(result.output.warnings, []);
}

function assertBrokenBuild(result: BuildResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(result.files, ["AppLink.json"]);
  assert.equal(sha256(result.outputText), brokenOutputSha256);
  assert.deepEqual(
    {
      css: result.output.css,
      errors: result.output.errors,
      filename: result.output.filename,
      macro_artifacts: result.output.macro_artifacts,
      script_lang: result.output.script_lang,
      warnings: result.output.warnings,
    },
    {
      css: null,
      errors: [
        'Template compilation errors: [CompilerError { code: InvalidExpression, message: "Error parsing JavaScript expression: Expected `)` but found `EOF`", loc: Some(SourceLocation { start: Position { offset: 157, line: 1, column: 158 }, end: Position { offset: 166, line: 1, column: 167 }, source: "String(to" }) }]',
      ],
      filename: "AppLink.vue",
      macro_artifacts: [],
      script_lang: "ts",
      warnings: [],
    },
  );
  assert.equal(sha256(result.output.code), brokenCodeSha256, result.output.code);
  assertParsesAsModule(result.output.code, "broken vue-router AppLink.json#code");
  assert.equal(result.output.code.includes("_createElementBlock"), false, result.output.code);
  assert.equal(count(result.output.code, "return {"), 1, result.output.code);
}

function sha256(source: string | Buffer): string {
  return createHash("sha256").update(source).digest("hex");
}

function count(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

async function definitionLocations(
  session: LspSession,
  uri: string,
  source: string,
  target: { offset: number },
): Promise<Array<{ range?: unknown; uri?: string }>> {
  const definition = (await session.request("textDocument/definition", {
    textDocument: { uri },
    position: offsetToPosition(source, target.offset),
  })) as Array<{ range?: unknown; uri?: string }> | { range?: unknown; uri?: string } | null;
  return Array.isArray(definition) ? definition : definition == null ? [] : [definition];
}

function assertCleanParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 0, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files.length, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files[0]?.file, appPath, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 0, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.deepEqual(vize.report.files[0]?.diagnostics, []);
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.doesNotMatch(`${vueTsc.stdout}\n${vueTsc.stderr}`, /error TS\d+:/);
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files.length, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files[0]?.file, appPath, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  const diagnostics = vize.report.files[0]?.diagnostics.join("\n") ?? "";
  assert.match(diagnostics, /TS2322.*string.*not assignable.*number/i);
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2322:/g)].length, 1, output);
  assert.doesNotMatch(output, /error TS(?!2322)\d+:/, output);
}

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectMismatch?: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => {
      if (!isDiagnosticsForUri(params, uri) || params.version !== version) return false;
      return expectMismatch == null || hasMismatch(params.diagnostics) === expectMismatch;
    },
    120_000,
  )) as PublishDiagnosticsParams;
}

function hasMismatch(diagnostics: LspDiagnostic[]): boolean {
  return diagnostics.some(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMismatch(diagnostics: LspDiagnostic[], source: string): void {
  const mismatches = diagnostics.filter((diagnostic) => hasMismatch([diagnostic]));
  assert.equal(mismatches.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = mismatches;
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf(`const ${symbol}`) + "const ".length);
  assert.deepEqual(diagnostic.range?.start, start);
  assert.deepEqual(diagnostic.range?.end, {
    line: start.line,
    character: start.character + symbol.length,
  });
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
}

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
  include: [appPath],
};

const routerDeclaration = `import type { ComputedRef, Ref } from 'vue'

export interface RouterLinkProps {
  to: string | { path: string }
  replace?: boolean
}

export interface RouteLocation {
  path: string
}

export const START_LOCATION: RouteLocation
export function useRoute(): RouteLocation
export function useLink(options: {
  to: ComputedRef<unknown>
  replace?: boolean
}): {
  route: Ref<RouteLocation>
  href: Ref<string>
  isActive: Ref<boolean>
  isExactActive: Ref<boolean>
  navigate: (event?: MouseEvent) => Promise<void>
}
`;
