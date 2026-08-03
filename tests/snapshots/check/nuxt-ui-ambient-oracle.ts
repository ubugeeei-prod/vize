import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
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
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const appPath = "docs/app/components/content/examples/tabs/TabsRouteQueryExample.vue";
const cleanExpression = "route.query.tab";
const brokenExpression = "route.missing";
const importAnchor = "import type { TabsItem } from '@nuxt/ui'";
const virtualImport = "import { computed, useRoute, useRouter } from '#imports'";

test("Nuxt UI generated ambient and #imports types stay exact across editor revisions", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "nuxt-ui", includePaths: [appPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("node_modules/@nuxt/ui/package.json", packageManifest);
      fixture.write("node_modules/@nuxt/ui/index.d.ts", nuxtUiDeclaration);
      fixture.write("node_modules/nuxt/package.json", nuxtPackageManifest);
      fixture.write("node_modules/nuxt/app.d.ts", nuxtAppDeclaration);
      fixture.write(".nuxt/imports.d.ts", generatedImportsDeclaration);
      fixture.write(".nuxt/virtual-imports.d.ts", generatedVirtualImportsDeclaration);
      fixture.write(".nuxt/components.d.cts", generatedComponentsDeclaration);
      fixture.write(".nuxt/tsconfig.json", json(generatedTsconfig));
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write("nuxt.config.ts", "export default defineNuxtConfig({})\n");
      fixture.write(
        "vize.config.json",
        json({
          lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
          typeChecker: { corsaPath },
        }),
      );

      const cleanSource = fixture.applyExactPatch(
        appPath,
        importAnchor,
        `${importAnchor}\n${virtualImport}`,
      );
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
            cleanSource.indexOf(':items="items"') + ':items="it'.length,
          ),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes("items"), labels.join(", "));
        assert.ok(labels.includes("active"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const activeUsage = cleanSource.indexOf('v-model="active"') + 'v-model="'.length;
        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(cleanSource, activeUsage + "active".length),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /active/);
        // Backend-answered since #3321. The `v-model` binding still widens to
        // `any` against the generated ambient component types; the point pinned
        // here is the provenance, not the precision.
        assert.match(hoverText, /_Resolved through Vize virtual TypeScript_/);
        assert.doesNotMatch(hoverText, /Template binding from script/);

        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri: appUri },
          position: offsetToPosition(cleanSource, activeUsage + "active".length),
        })) as
          | Array<{ uri: string; range: { start: { line: number; character: number } } }>
          | { uri: string; range: { start: { line: number; character: number } } };
        const definitionLocation = firstLocation(definition);
        assert.equal(definitionLocation.uri, appUri);
        assert.deepEqual(
          definitionLocation.range.start,
          offsetToPosition(cleanSource, cleanSource.indexOf("active = computed")),
        );

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

        const repairedSource = fixture.applyExactPatch(appPath, brokenExpression, cleanExpression);
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
  assert.equal(vize.report.errorCount, 0, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  assert.deepEqual(vize.report.files, [{ diagnostics: [], file: appPath }]);
  assert.equal(vize.report.fileCount, vize.report.files.length);
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.doesNotMatch(`${vueTsc.stdout}\n${vueTsc.stderr}`, /error TS\d+:/);
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  const diagnostic =
    vize.report.files.find((file) => file.file === appPath)?.diagnostics.join("\n") ?? "";
  assert.match(diagnostic, /TS2339.*missing.*NuxtRoute/i);
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2339:/g)].length, 1, output);
  assert.match(output, /missing.*NuxtRoute/i);
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
      /missing.*NuxtRoute/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMissingProperty(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMissingProperty([diagnostic]), JSON.stringify(diagnostic));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(source, source.indexOf(brokenExpression) + "route.".length);
  assert.deepEqual(diagnostic.range, {
    start,
    end: { line: start.line, character: start.character + "missing".length },
  });
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

const packageManifest = json({ name: "@nuxt/ui", type: "module", types: "./index.d.ts" });

const nuxtUiDeclaration = `export type TabsItem = {
  label: string
  icon: string
  value: string
}

export const UTabs: new () => {
  $props: {
    modelValue?: string
    'onUpdate:modelValue'?: (value: string) => void
    content?: boolean
    items?: TabsItem[]
    class?: unknown
  }
}
`;

const nuxtPackageManifest = json({
  name: "nuxt",
  type: "module",
  exports: { "./app": { types: "./app.d.ts", default: "./app.js" } },
});

const nuxtAppDeclaration = `export interface NuxtRoute {
  query: Record<string, unknown>
}

export interface NuxtRouter {
  push(to: { path: string; query: Record<string, unknown>; hash?: string }): Promise<void>
}

export function useRoute(): NuxtRoute
export function useRouter(): NuxtRouter
`;

const generatedImportsDeclaration = `
declare global {
  const defineNuxtConfig: <T extends Record<string, unknown>>(config: T) => T
}

export {}
`;

const generatedVirtualImportsDeclaration = `export { computed } from 'vue'
export { useRoute, useRouter } from 'nuxt/app'
`;

const generatedComponentsDeclaration = `declare module 'vue' {
  export interface GlobalComponents {
    UTabs: typeof import('@nuxt/ui')['UTabs']
  }
}

export {}
`;

const generatedTsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    paths: { "#imports": ["./virtual-imports"] },
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
  include: ["./imports.d.ts", "./virtual-imports.d.ts", "./components.d.cts", `../${appPath}`],
};

const tsconfig = {
  extends: "./.nuxt/tsconfig.json",
  include: [".nuxt/imports.d.ts", ".nuxt/virtual-imports.d.ts", ".nuxt/components.d.cts", appPath],
};
