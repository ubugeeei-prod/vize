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

const sourcePath = "src/client/theme-default/NotFound.vue";
const dataPath = "src/client/theme-default/composables/data.ts";
const langsPath = "src/client/theme-default/composables/langs.ts";
const themeDeclarationPath = "node_modules/vitepress/theme.d.ts";
const cleanCodeDeclaration = "    code?: string";
const brokenCodeDeclaration = "    knownCode?: string";
// `theme.notFound?.co|` is a member position, so the answer is the declared
// surface of `DefaultTheme.NotFoundOptions` in declaration order (what Volar
// answers there) rather than the setup-binding list this used to pin (#3911).
const expectedCompletionLabels = ["title?", "quote?", "link?", "linkLabel?", "linkText?", "code?"];

test("VitePress theme exports refresh exact template diagnostics", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vitepress", includePaths: [sourcePath, dataPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write(langsPath, langsModule);
      fixture.write("node_modules/vitepress/package.json", packageManifest);
      fixture.write("node_modules/vitepress/index.d.ts", vitepressDeclaration);
      fixture.write(themeDeclarationPath, themeDeclaration);
      fixture.write("tsconfig.json", json(tsconfig));
      fixture.write(
        "vize.config.json",
        json({
          lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
          typeChecker: { corsaPath },
        }),
      );

      const source = fixture.read(sourcePath);
      const cleanThemeDeclaration = fixture.read(themeDeclarationPath);
      const sourceUri = pathToFileURL(fixture.resolve(sourcePath)).href;
      const themeDeclarationUri = pathToFileURL(fixture.resolve(themeDeclarationPath)).href;

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
            uri: themeDeclarationUri,
            languageId: "typescript",
            version: 1,
            text: cleanThemeDeclaration,
          },
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri: sourceUri, languageId: "vue", version: 1, text: source },
        });
        const cleanPublish = await waitForDiagnostics(session, sourceUri, 1, false);
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const completionSource = replaceExactlyOnce(
          source,
          "theme.notFound?.code",
          "theme.notFound?.co",
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 2 },
          contentChanges: [{ text: completionSource }],
        });
        const completionUsage =
          completionSource.indexOf("theme.notFound?.co") + "theme.notFound?.co".length;
        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(completionSource, completionUsage),
        });
        const labels = completionLabels(completion);
        assert.deepEqual(labels, expectedCompletionLabels);
        assert.ok(!labels.includes("knownCode"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        session.notify("textDocument/didChange", {
          textDocument: { uri: sourceUri, version: 3 },
          contentChanges: [{ text: source }],
        });
        const restoredPublish = await waitForDiagnostics(session, sourceUri, 3, false);
        assert.deepEqual(restoredPublish.diagnostics, [], JSON.stringify(restoredPublish));

        const codeUsage = source.indexOf("theme.notFound?.code") + "theme.notFound?.".length;
        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(source, codeUsage + "co".length),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        // Since #3321 the backend resolves the optional theme property through
        // the refreshed declaration instead of falling back to a generic
        // template-expression hover.
        assert.equal(
          hoverText,
          "```typescript\n(property) DefaultTheme.NotFoundOptions.code?: string | undefined\n```",
        );

        const themeUsage = source.indexOf("theme.notFound");
        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(source, themeUsage + "the".length),
        })) as
          | Array<{ uri: string; range: { start: { line: number; character: number } } }>
          | { uri: string; range: { start: { line: number; character: number } } };
        const definitionLocation = firstLocation(definition);
        assert.equal(definitionLocation.uri, sourceUri);
        assert.deepEqual(
          definitionLocation.range.start,
          offsetToPosition(source, source.indexOf("const { theme }") + "const { ".length),
        );

        const brokenThemeDeclaration = fixture.applyExactPatch(
          themeDeclarationPath,
          cleanCodeDeclaration,
          brokenCodeDeclaration,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: themeDeclarationUri, version: 2 },
          contentChanges: [{ text: brokenThemeDeclaration }],
        });
        const brokenPublish = await waitForDiagnostics(session, sourceUri, 3, true);
        assertSingleMissingProperty(brokenPublish.diagnostics, source);
        assertBrokenParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );

        const repairedThemeDeclaration = fixture.applyExactPatch(
          themeDeclarationPath,
          brokenCodeDeclaration,
          cleanCodeDeclaration,
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: themeDeclarationUri, version: 3 },
          contentChanges: [{ text: repairedThemeDeclaration }],
        });
        const repairedPublish = await waitForDiagnostics(session, sourceUri, 3, false);
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanParity(
          runVizeCheck(fixture.workspaceDir, corsaPath, [sourcePath]),
          runVueTsc(fixture.workspaceDir, vueTscPath),
        );
        const repairedCompletion = await session.request("textDocument/completion", {
          textDocument: { uri: sourceUri },
          position: offsetToPosition(source, codeUsage + "co".length),
        });
        assert.deepEqual(completionLabels(repairedCompletion), expectedCompletionLabels);
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
      fileCount: 3,
      // The authored composables the theme imports are reported alongside it,
      // so a regression that only surfaces in a dependency cannot hide (#3996).
      files: [
        { diagnostics: [], file: sourcePath },
        { diagnostics: [], file: dataPath },
        { diagnostics: [], file: langsPath },
      ],
      warningCount: 0,
    },
  );
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.doesNotMatch(`${vueTsc.stdout}\n${vueTsc.stderr}`, /error TS\d+:/);
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.fileCount, 3, JSON.stringify(vize.report));
  assert.equal(vize.report.errorCount, 1, JSON.stringify(vize.report));
  assert.equal(vize.report.warningCount, 0, JSON.stringify(vize.report));
  // The authored composables the theme imports are reported alongside it, so the
  // single error must stay pinned to the SFC and the dependencies stay clean (#3996).
  assert.deepEqual(
    vize.report.files.map((file) => file.file),
    [sourcePath, dataPath, langsPath],
    JSON.stringify(vize.report),
  );
  assert.deepEqual(vize.report.files[1]?.diagnostics, [], JSON.stringify(vize.report));
  assert.deepEqual(vize.report.files[2]?.diagnostics, [], JSON.stringify(vize.report));
  assert.match(
    vize.report.files[0]?.diagnostics.join("\n") ?? "",
    /TS2339.*code.*NotFoundOptions/i,
  );
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  const output = `${vueTsc.stdout}\n${vueTsc.stderr}`;
  assert.equal([...output.matchAll(/error TS2339:/g)].length, 1, output);
  assert.match(output, /code.*NotFoundOptions/i);
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
      /code.*NotFoundOptions/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleMissingProperty(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = diagnostics;
  assert.ok(hasMissingProperty([diagnostic]), JSON.stringify(diagnostic));
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  const start = offsetToPosition(
    source,
    source.indexOf("theme.notFound?.code") + "theme.notFound?.".length,
  );
  assert.deepEqual(diagnostic.range, {
    start,
    end: { line: start.line, character: start.character + "code".length },
  });
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function replaceExactlyOnce(source: string, expected: string, replacement: string): string {
  const first = source.indexOf(expected);
  assert.notEqual(first, -1, `missing patch anchor: ${expected}`);
  assert.equal(source.indexOf(expected, first + expected.length), -1, `duplicate patch anchor`);
  return `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
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
    types: [],
  },
  include: [sourcePath, dataPath, langsPath],
};

const packageManifest = json({
  name: "vitepress",
  type: "module",
  exports: {
    ".": { types: "./index.d.ts", default: "./index.js" },
    "./theme": { types: "./theme.d.ts", default: "./theme.js" },
  },
});

const vitepressDeclaration = `import type { Ref } from 'vue'

export interface VitePressData<ThemeConfig = unknown> {
  theme: Ref<ThemeConfig>
}

export declare function useData<ThemeConfig = unknown>(): VitePressData<ThemeConfig>
export declare function withBase(path: string): string
`;

const themeDeclaration = `export namespace DefaultTheme {
  export interface Config {
    notFound?: NotFoundOptions
  }

  export interface NotFoundOptions {
    title?: string
    quote?: string
    link?: string
    linkLabel?: string
    linkText?: string
    code?: string
  }
}
`;

const langsModule = `import { computed } from 'vue'

export function useLangs() {
  const currentLang = computed(() => ({ link: '/' }))
  return { currentLang }
}
`;
