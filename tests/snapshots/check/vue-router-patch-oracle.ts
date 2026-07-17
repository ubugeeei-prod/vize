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

const appPath = "packages/playground/src/AppLink.vue";
const routerManifestPath = "packages/router/package.json";
const symbol = "packageBoundary";

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
