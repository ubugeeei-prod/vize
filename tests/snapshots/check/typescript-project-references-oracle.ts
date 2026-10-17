import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  repoRoot,
  symlinkDirectory,
  withPinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  type CommandResult,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTscBuild,
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
import { vueConfigFixtureSource } from "../../tooling/support/vue-config-fixture.ts";

const projectPath = "packages/experiments-playground";
const appPath = "src/pages/about.vue";
const fixtureAppPath = `${projectPath}/${appPath}`;
const appTsconfigPath = `${projectPath}/tsconfig.app.json`;
const symbol = "projectReferenceValue";

test("Vue Router solution tsconfigs preserve exact diagnostics across edits", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    {
      fixtureId: "vue-router",
      includePaths: [
        fixtureAppPath,
        `${projectPath}/tsconfig.json`,
        appTsconfigPath,
        `${projectPath}/tsconfig.node.json`,
      ],
    },
    async (fixture) => {
      const projectDir = fixture.resolve(projectPath);
      symlinkVueTypes(projectDir);
      fixture.applyExactPatch(
        appTsconfigPath,
        '"extends": "@vue/tsconfig/tsconfig.dom.json"',
        '"extends": "./tsconfig.vue-base.json"',
      );
      fixture.write(
        `${projectPath}/tsconfig.vue-base.json`,
        `${JSON.stringify(baseTsconfig, null, 2)}\n`,
      );
      fixture.write(
        `${projectPath}/node_modules/@tsconfig/node22/tsconfig.json`,
        `${JSON.stringify(nodeBaseTsconfig, null, 2)}\n`,
      );
      symlinkDirectory(
        path.join(repoRoot, "node_modules/@types/node"),
        fixture.resolve(`${projectPath}/node_modules/@types/node`),
      );
      // The referenced Node project includes TypeScript Vite configs, not .mjs.
      fixture.write(
        `${projectPath}/vite.config.ts`,
        vueConfigFixtureSource({
          lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
          typeChecker: { corsaPath },
        }),
      );

      fixture.applyExactPatch(
        fixtureAppPath,
        '<template>\n  <div class="about">',
        `<script setup lang="ts">\nconst ${symbol}: number = 1\n</script>\n\n<template>\n  <div class="about">`,
      );
      const cleanSource = fixture.applyExactPatch(
        fixtureAppPath,
        "    <h1>About</h1>",
        `    <h1>About</h1>\n    <p>{{ ${symbol} }}</p>`,
      );
      const appFile = fixture.resolve(fixtureAppPath);
      const appUri = pathToFileURL(appFile).href;

      assertCleanParity(
        runVizeCheck(projectDir, corsaPath, []),
        runVueTscBuild(projectDir, vueTscPath),
      );

      const session = new LspSession();
      try {
        await session.initialize(projectDir, {
          completion: true,
          editor: true,
          hover: true,
          lint: false,
          typecheck: true,
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
        });
        const cleanPublish = await waitForDiagnostics(session, appUri, 1, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const templateOffset = cleanSource.lastIndexOf(symbol) + "projectRef".length;
        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(cleanSource, templateOffset),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes(symbol), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(cleanSource, cleanSource.lastIndexOf(symbol) + 2),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, new RegExp(symbol));
        assert.match(hoverText, /number/i);

        const brokenSource = fixture.applyExactPatch(
          fixtureAppPath,
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
          runVizeCheck(projectDir, corsaPath, []),
          runVueTscBuild(projectDir, vueTscPath),
        );

        const repairedSource = fixture.applyExactPatch(
          fixtureAppPath,
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
          runVizeCheck(projectDir, corsaPath, []),
          runVueTscBuild(projectDir, vueTscPath),
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
      files: [
        { diagnostics: [], file: appPath },
        { diagnostics: [], file: "vite.config.ts" },
      ],
      warningCount: 0,
    },
  );
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 2, JSON.stringify(vize.report));
  assert.equal(vize.report.files.length, 2, JSON.stringify(vize.report));
  assert.equal(vize.report.files[0]?.file, appPath, JSON.stringify(vize.report));
  assert.equal(vize.report.files[1]?.file, "vite.config.ts", JSON.stringify(vize.report));
  assert.deepEqual(vize.report.files[1]?.diagnostics, []);
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.match(
    vize.report.files[0]?.diagnostics.join("\n") ?? "",
    /TS2322.*string.*not assignable.*number/i,
  );
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2322:/g)].length, 1, output);
  assert.doesNotMatch(output, /error TS(?!2322)\d+:/, output);
}

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectMismatch: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      hasMismatch(params.diagnostics) === expectMismatch,
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
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMismatch([diagnostic]), JSON.stringify(diagnostics));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf(`const ${symbol}`) + "const ".length);
  assert.deepEqual(diagnostic.range?.start, start);
  assert.deepEqual(diagnostic.range?.end, {
    line: start.line,
    character: start.character + symbol.length,
  });
}

const baseTsconfig = {
  compilerOptions: {
    composite: true,
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleDetection: "force",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
};

const nodeBaseTsconfig = {
  compilerOptions: {
    composite: true,
    lib: ["ES2022"],
    module: "NodeNext",
    moduleResolution: "NodeNext",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
};
