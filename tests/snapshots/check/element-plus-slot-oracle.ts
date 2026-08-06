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

const appPath = "docs/examples/badge/customize.vue";
const cleanExpression = "{{ value.toUpperCase() }}";
const brokenExpression = "{{ value.missing }}";
const repairedExpression = "{{ value.trim() }}";

test("Element Plus badge slot contracts stay exact across editor revisions", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "element-plus", includePaths: [appPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      symlinkDirectory(
        path.join(repoRoot, "node_modules/vite"),
        fixture.resolve("node_modules/vite"),
      );
      fixture.write("node_modules/element-plus/package.json", packageManifest("element-plus"));
      fixture.write("node_modules/element-plus/index.d.ts", elementPlusDeclaration);
      fixture.write(
        "node_modules/@element-plus/icons-vue/package.json",
        packageManifest("@element-plus/icons-vue"),
      );
      fixture.write("node_modules/@element-plus/icons-vue/index.d.ts", iconDeclaration);
      fixture.write("element-plus-global.d.ts", globalComponentsDeclaration);
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

      const cleanSource = fixture.applyExactPatch(appPath, "{{ value }}", cleanExpression);
      const appUri = pathToFileURL(fixture.resolve(appPath)).href;

      assertCleanParity(
        runVizeCheck(fixture.workspaceDir, corsaPath, []),
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
        const cleanPublish = await waitForDiagnostics(session, appUri, 1, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf(cleanExpression) + "{{ val".length,
          ),
        });
        const labels = completionLabels(completion);
        assert.deepEqual(labels, ["value", "Message"]);

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf(cleanExpression) + "{{ value".length,
          ),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        // Answered by the backend since #3321. The slot scope parameter still
        // widens to `any` in the generated virtual TS — a separate gap, but the
        // provenance is now the type backend rather than a generic fallback.
        // Backend-answered: the body opens with the signature fence (#3894).
        assert.equal(hoverText, "```typescript\n(parameter) value: any\n```");

        const brokenSource = fixture.applyExactPatch(appPath, cleanExpression, brokenExpression);
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = await waitForDiagnostics(session, appUri, 2, true);
        assertSingleMissingProperty(brokenPublish.diagnostics, brokenSource);
        assertBrokenParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, []),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );

        const repairedSource = fixture.applyExactPatch(
          appPath,
          brokenExpression,
          repairedExpression,
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
          runVizeCheck(fixture.workspaceDir, corsaPath, []),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );
      } finally {
        await session.shutdown();
      }
    },
  );
});

function assertCleanParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
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
        { diagnostics: [], file: "element-plus-global.d.ts" },
      ],
      warningCount: 0,
    },
  );
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.doesNotMatch(`${vueTsc.stdout}\n${vueTsc.stderr}`, /error TS\d+:/);
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 2, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.equal(vize.report.files.length, 2, JSON.stringify(vize.report));
  assert.equal(vize.report.files[0]?.file, appPath, JSON.stringify(vize.report));
  assert.deepEqual(
    vize.report.files[1],
    { diagnostics: [], file: "element-plus-global.d.ts" },
    JSON.stringify(vize.report),
  );
  assert.match(vize.report.files[0]?.diagnostics.join("\n") ?? "", /TS2339.*missing.*string/i);
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2339:/g)].length, 1, output);
  assert.match(output, /missing.*string/i);
  assert.doesNotMatch(output, /error TS(?!2339)\d+:/, output);
}

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectMissing: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      hasMissingProperty(params.diagnostics) === expectMissing,
    120_000,
  )) as PublishDiagnosticsParams;
}

function hasMissingProperty(diagnostics: LspDiagnostic[]): boolean {
  return diagnostics.some(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2339" &&
      /missing.*string/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMissingProperty(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMissingProperty([diagnostic]), JSON.stringify(diagnostic));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf(brokenExpression) + "{{ value.".length);
  assert.deepEqual(diagnostic.range, {
    start,
    end: { line: start.line, character: start.character + "missing".length },
  });
}

function packageManifest(name: string): string {
  return `${JSON.stringify({ name, type: "module", types: "./index.d.ts" }, null, 2)}\n`;
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
  include: [appPath, "element-plus-global.d.ts"],
};

const elementPlusDeclaration = `export type ElementComponent<
  Props extends Record<string, unknown> = Record<string, never>,
  Slots extends Record<string, unknown> = Record<string, never>,
> = new () => {
  $props: Props
  $slots: Slots
}

export const ElBadge: ElementComponent<
  { value?: string | number; class?: unknown },
  { default?: () => unknown; content?: (props: { value: string }) => unknown }
>
export const ElButton: ElementComponent<
  { class?: unknown },
  { default?: () => unknown }
>
export const ElIcon: ElementComponent<
  { class?: unknown },
  { default?: () => unknown }
>
`;

const globalComponentsDeclaration = `declare module 'vue' {
  export interface GlobalComponents {
    ElBadge: typeof import('element-plus')['ElBadge']
    ElButton: typeof import('element-plus')['ElButton']
    ElIcon: typeof import('element-plus')['ElIcon']
  }
}

export {}
`;

const iconDeclaration = `import type { Component } from 'vue'

export const Message: Component
`;
