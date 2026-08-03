import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { repoRoot, withPinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import type { LspDiagnostic, LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const CREATE_VUE_APP = "template/code/typescript-default/src/App.vue";
const CLEAN_ATTRIBUTE = 'alt="Vue logo" class="logo"';
const BROKEN_ATTRIBUTE = 'alt="Vue logo"  class="logo"';

test("release-script CI hydrates the pinned create-vue code-action oracle", () => {
  const workflow = fs.readFileSync(`${repoRoot}/.github/workflows/check.yml`, "utf8");
  assert.match(
    workflow,
    /git submodule update --init --force --depth 1 --[^\n]*tests\/_fixtures\/_git\/create-vue/,
  );
});

type CodeAction = {
  title: string;
  kind?: string;
  isPreferred?: boolean;
  edit?: {
    changes?: Record<string, Array<{ range: LspRange; newText: string }>>;
  };
};

type ExpectedAction = {
  title: string;
  kind?: string;
  isPreferred?: boolean;
  edits: Array<{ range: LspRange; newText: string }>;
};

function openDocument(
  session: LspSession,
  filePath: string,
  languageId: string,
  source: string,
): { uri: string; diagnostics: Promise<PublishDiagnosticsParams> } {
  const uri = pathToFileURL(filePath).href;
  const diagnostics = session.waitForNotification("textDocument/publishDiagnostics", (params) =>
    isDiagnosticsForUri(params, uri),
  ) as Promise<PublishDiagnosticsParams>;
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId, version: 1, text: source },
  });
  return { uri, diagnostics };
}

async function requestActions(
  session: LspSession,
  uri: string,
  range: LspRange,
  diagnostic: LspDiagnostic,
  only?: string[],
): Promise<CodeAction[] | null> {
  return (await session.request("textDocument/codeAction", {
    textDocument: { uri },
    range,
    context: { diagnostics: [diagnostic], ...(only == null ? {} : { only }) },
  })) as CodeAction[] | null;
}

function normalize(actions: CodeAction[] | null, uri: string): ExpectedAction[] | null {
  return (
    actions?.map((action) => ({
      title: action.title,
      kind: action.kind,
      isPreferred: action.isPreferred,
      edits: action.edit?.changes?.[uri] ?? [],
    })) ?? null
  );
}

function lintDiagnostic(publish: PublishDiagnosticsParams): LspDiagnostic {
  const diagnostic = publish.diagnostics.find(
    (item) => item.source === "vize/lint" && item.code === "vue/no-multi-spaces",
  );
  assert.ok(diagnostic, JSON.stringify(publish.diagnostics));
  return diagnostic;
}

test("vize lsp honors requested quickfix kinds for pinned create-vue SFC and TSX", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [CREATE_VUE_APP] },
    async (fixture) => {
      const source = fixture.applyExactPatch(CREATE_VUE_APP, CLEAN_ATTRIBUTE, BROKEN_ATTRIBUTE);
      const session = new LspSession();

      try {
        await session.initialize(fixture.workspaceDir, {
          lint: true,
          codeActions: true,
          typecheck: false,
        });

        const sfc = openDocument(session, fixture.resolve(CREATE_VUE_APP), "vue", source);
        const sfcDiagnostic = lintDiagnostic(await sfc.diagnostics);
        const gapStart = source.indexOf("  class=");
        assert.notEqual(gapStart, -1);
        const sfcRange = {
          start: offsetToPosition(source, gapStart),
          end: offsetToPosition(source, gapStart + 2),
        };
        assert.deepEqual(sfcDiagnostic.range, sfcRange);

        const expectedSfcActions: ExpectedAction[] = [
          {
            title: "Fix: Replace multiple spaces with single space",
            kind: "quickfix",
            isPreferred: true,
            edits: [{ range: sfcRange, newText: " " }],
          },
          {
            title: "Suppress with @vize:forget (vue/no-multi-spaces)",
            kind: "quickfix",
            isPreferred: false,
            edits: [
              {
                range: {
                  start: { line: sfcRange.start.line, character: 0 },
                  end: { line: sfcRange.start.line, character: 0 },
                },
                newText: "    <!-- @vize:forget vue/no-multi-spaces -->\n",
              },
            ],
          },
        ];
        assert.deepEqual(
          normalize(
            await requestActions(session, sfc.uri, sfcRange, sfcDiagnostic, ["quickfix"]),
            sfc.uri,
          ),
          expectedSfcActions,
        );
        assert.deepEqual(
          normalize(await requestActions(session, sfc.uri, sfcRange, sfcDiagnostic, [""]), sfc.uri),
          expectedSfcActions,
          "the empty root kind contains every concrete code action kind",
        );
        assert.equal(
          await requestActions(session, sfc.uri, sfcRange, sfcDiagnostic, ["refactor", "source"]),
          null,
        );
        assert.equal(
          await requestActions(session, sfc.uri, sfcRange, sfcDiagnostic, ["quickfix.fixAll"]),
          null,
          "a child-only request must not include its quickfix parent",
        );

        const jsxSource = 'const C = () => <div    class="a">x</div>;\n';
        const jsxPath = fixture.resolve("src/CodeAction.tsx");
        fs.mkdirSync(new URL(".", pathToFileURL(jsxPath)), { recursive: true });
        fs.writeFileSync(jsxPath, jsxSource, "utf8");
        const jsx = openDocument(session, jsxPath, "typescriptreact", jsxSource);
        await jsx.diagnostics;
        const jsxGapStart = jsxSource.indexOf("    class=");
        assert.notEqual(jsxGapStart, -1);
        const jsxRange = {
          start: offsetToPosition(jsxSource, jsxGapStart),
          end: offsetToPosition(jsxSource, jsxGapStart + 4),
        };
        const jsxDiagnostic: LspDiagnostic = {
          source: "vize/lint",
          code: "vue/no-multi-spaces",
          message: "Replace multiple spaces with single space",
          range: jsxRange,
        };
        assert.deepEqual(
          normalize(
            await requestActions(session, jsx.uri, jsxRange, jsxDiagnostic, ["quickfix"]),
            jsx.uri,
          ),
          [
            {
              title: "Fix: Replace multiple spaces with single space",
              kind: "quickfix",
              isPreferred: true,
              edits: [{ range: jsxRange, newText: " " }],
            },
          ],
        );
        assert.equal(
          await requestActions(session, jsx.uri, jsxRange, jsxDiagnostic, ["source"]),
          null,
          "JSX must apply the same requested-kind filter as SFC",
        );
      } finally {
        await session.shutdown();
      }
    },
  );
});
