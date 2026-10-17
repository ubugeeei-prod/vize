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
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const sourcePath = "packages/experiments-playground/src/pages/about.vue";
const routerManifestPath = "packages/router/package.json";
const routesDeclarationPath = "packages/router/vue-router-auto-routes.d.mts";
const cleanRoutesDeclaration = "export declare const routes: RouteRecordRaw[]";
const brokenRoutesDeclaration = "export declare const routes: string[]";

test("Vue Router .d.mts exports refresh exact dependent diagnostics", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    {
      fixtureId: "vue-router",
      includePaths: [sourcePath, routerManifestPath, routesDeclarationPath],
    },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("packages/router/dist/vue-router.d.ts", routerDeclaration);
      fixture.write("packages/router/dist/experimental/index.d.ts", experimentalDeclaration);
      symlinkDirectory(
        fixture.resolve("packages/router"),
        fixture.resolve("node_modules/vue-router"),
      );
      fixture.write("tsconfig.json", `${JSON.stringify(tsconfig, null, 2)}\n`);
      fixture.writeVueConfig(
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
        sourcePath,
        "<template>",
        `<script setup lang="ts">\nimport { routes } from 'vue-router/auto-routes'\nimport type { RouteRecordRaw } from 'vue-router'\n\nconst selectedRoute: RouteRecordRaw = routes[0]!\n</script>\n\n<template>`,
      );
      const cleanSource = fixture.applyExactPatch(
        sourcePath,
        "    <h1>About</h1>",
        "    <h1>About</h1>\n    <p>{{ selectedRoute.path }}</p>",
      );
      const cleanDeclaration = fixture.read(routesDeclarationPath);
      const sourceFile = fixture.resolve(sourcePath);
      const declarationFile = fixture.resolve(routesDeclarationPath);
      const sourceUri = pathToFileURL(sourceFile).href;
      const declarationUri = pathToFileURL(declarationFile).href;

      assertCleanParity(
        runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]),
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
            uri: declarationUri,
            languageId: "typescript",
            version: 1,
            text: cleanDeclaration,
          },
        });
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri: sourceUri,
            languageId: "vue",
            version: 1,
            text: cleanSource,
          },
        });
        const cleanPublish = await waitForDiagnostics(session, sourceUri, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const moduleDefinition = await definitionLocations(
          session,
          sourceUri,
          cleanSource,
          cleanSource.indexOf("'vue-router/auto-routes'") + 2,
        );
        assert.deepEqual(moduleDefinition, [
          {
            range: {
              start: { line: 0, character: 0 },
              end: { line: 0, character: 0 },
            },
            uri: declarationUri,
          },
        ]);

        const selectedRouteUse = cleanSource.lastIndexOf("selectedRoute.path");
        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(cleanSource, selectedRouteUse + "selectedR".length),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /selectedRoute/);
        assert.match(hoverText, /RouteRecordRaw/);

        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.lastIndexOf("selectedRoute.path") + "selectedR".length,
          ),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes("selectedRoute"), labels.join(", "));
        assert.ok(labels.includes("routes"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const brokenDeclaration = fixture.applyExactPatch(
          routesDeclarationPath,
          cleanRoutesDeclaration,
          brokenRoutesDeclaration,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: declarationUri, version: 2 },
          contentChanges: [{ text: brokenDeclaration }],
        });
        const brokenPublish = await waitForDiagnostics(session, sourceUri, true);
        assertSingleMismatch(brokenPublish.diagnostics, cleanSource);
        assertBrokenParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );

        const repairedDeclaration = fixture.applyExactPatch(
          routesDeclarationPath,
          brokenRoutesDeclaration,
          cleanRoutesDeclaration,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: declarationUri, version: 3 },
          contentChanges: [{ text: repairedDeclaration }],
        });
        const repairedPublish = await waitForDiagnostics(session, sourceUri, false);
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );
      } finally {
        await session.shutdown();
      }
    },
  );
});

async function definitionLocations(
  session: LspSession,
  uri: string,
  source: string,
  offset: number,
): Promise<Array<{ range?: unknown; uri?: string }>> {
  const definition = (await session.request("textDocument/definition", {
    textDocument: { uri },
    position: offsetToPosition(source, offset),
  })) as Array<{ range?: unknown; uri?: string }> | { range?: unknown; uri?: string } | null;
  return Array.isArray(definition) ? definition : definition == null ? [] : [definition];
}

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
      fileCount: 1,
      files: [{ diagnostics: [], file: sourcePath }],
      warningCount: 0,
    },
  );
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files.length, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.files[0]?.file, sourcePath, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.match(
    vize.report.files[0]?.diagnostics.join("\n") ?? "",
    /TS2322.*string.*not assignable.*RouteRecordRaw/i,
  );
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2322:/g)].length, 1, output);
  assert.doesNotMatch(output, /error TS(?!2322)\d+:/, output);
}

async function waitForDiagnostics(
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
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*RouteRecordRaw/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMismatch(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMismatch([diagnostic]), JSON.stringify(diagnostics));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf("const selectedRoute") + "const ".length);
  assert.deepEqual(diagnostic.range?.start, start);
  assert.deepEqual(diagnostic.range?.end, {
    line: start.line,
    character: start.character + "selectedRoute".length,
  });
}

const tsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
    types: [],
  },
  include: [sourcePath],
};

const routerDeclaration = `export interface RouteRecordRaw {
  path: string
}

export interface Router {}
`;

const experimentalDeclaration = `export function definePage(page: { name: string }): void
`;
