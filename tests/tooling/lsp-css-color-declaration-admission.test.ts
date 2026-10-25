import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Positive = {
  language: string;
  newline: string;
  source: string;
  sourceSha256: string;
  diagnostics: Record<string, unknown>;
  documentColors: Array<{ color: unknown; range: unknown }>;
  presentations: unknown;
  hover: Array<{ kind: string; params: { position: unknown }; result: unknown }>;
};
const frozenRoot = path.join(
  root,
  "tests/_fixtures/differential/lsp/css-color-declaration-admission-10562",
);
const baseline = JSON.parse(
  readPinnedArtifact(frozenRoot, {
    path: "positive.public-0.565.0.json",
    sha256: "abe5753a99c3356c9625ec466a4a71ef438d72b23328e10f7707535a5892b25a",
  }).toString("utf8"),
) as { rows: Positive[] };
const currentHovers = JSON.parse(
  readPinnedArtifact(frozenRoot, {
    path: "hover.expected-stylesheet-examples-10572.json",
    sha256: "f6d95ee0c3fce94d54b6790d7ce6a942f35e496f6ff4796abcdd4c121cadeb11",
  }).toString("utf8"),
) as {
  rows: Array<{ language: string; newline: string; sourceSha256: string; results: unknown[] }>;
};
assert.deepEqual(
  currentHovers.rows.map(({ language, newline, sourceSha256 }) => ({
    language,
    newline,
    sourceSha256,
  })),
  baseline.rows.map(({ language, newline, sourceSha256 }) => ({ language, newline, sourceSha256 })),
  "current hover expectations must cover every frozen source row in order",
);
const negative = readPinnedArtifact(frozenRoot, {
  path: "negative.public-0.565.0.json",
  sha256: "2d7b4e8643d18e8719dc0f2c5fa8a0a7074945780fbe0228f81f8c4d36e24194",
});
assert.equal(JSON.parse(negative.toString("utf8")).provider.version, "vize 0.565.0");

async function withSession(
  label: string,
  run: (session: LspSession, workspace: string, observations: unknown[]) => Promise<void>,
) {
  const output = path.join(testOutputRoot, "css-color-declaration-admission");
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(output, `${label}-`));
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    "export default {vue:{languageServer:{hover:true,lint:false,typecheck:false}}};\n",
  );
  const session = new LspSession({
    repoRoot: root,
    binary: process.env.VIZE_LSP_BIN ?? path.join(root, "target/ci/vize"),
  });
  const observations: unknown[] = [];
  try {
    observations.push({
      initialize: await session.initialize(workspace, {
        editor: true,
        hover: true,
        lint: false,
        typecheck: false,
      }),
    });
    await run(session, workspace, observations);
  } finally {
    await session.shutdown();
    await session.waitForClosed();
    fs.writeFileSync(
      path.join(output, `${label}.json`),
      JSON.stringify(observations, null, 2) + "\n",
    );
    fs.writeFileSync(path.join(output, `${label}.stderr.txt`), session.stderrText);
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}

async function open(session: LspSession, uri: string, source: string, version = 1) {
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version, text: source },
  });
  const diagnostics = await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
  );
  assert.deepEqual(diagnostics, { uri, version, diagnostics: [] });
}

await test("CSS declaration admission preserves all eighteen complete public color and hover controls", async () => {
  assert.equal(baseline.rows.length, 18);
  await withSession("positive", async (session, workspace, observations) => {
    for (const [rowIndex, row] of baseline.rows.entries()) {
      const uri = pathToFileURL(
        path.join(workspace, `日本語-${row.language}-${row.newline}.vue`),
      ).href;
      await open(session, uri, row.source);
      const colors = await session.request("textDocument/documentColor", { textDocument: { uri } });
      observations.push({ source: row.source, colors });
      assert.deepEqual(colors, row.documentColors, `${row.language}/${row.newline} whole colors`);
      const expectedHovers = currentHovers.rows[rowIndex]!.results;
      assert.equal(expectedHovers.length, row.hover.length);
      for (const [requestIndex, request] of row.hover.entries()) {
        const params = { textDocument: { uri }, position: request.params.position };
        const result = await session.request("textDocument/hover", params);
        observations.push({ params, result });
        assert.deepEqual(
          result,
          expectedHovers[requestIndex],
          `${row.language}/${row.newline} ${request.kind} whole hover`,
        );
      }
      const first = row.documentColors[0];
      const params = { textDocument: { uri }, color: first.color, range: first.range };
      const presentations = await session.request("textDocument/colorPresentation", params);
      observations.push({ params, presentations });
      assert.deepEqual(
        presentations,
        row.presentations,
        `${row.language}/${row.newline} whole presentations`,
      );
      session.notify("textDocument/didClose", { textDocument: { uri } });
    }
  });
});

await test("selectors are refused and actual Stylus colonless values retain complete authored vectors", async () => {
  const loaded = loadLspManifest(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"));
  const cases = loaded.cases.filter((fixture) =>
    fixture.id.startsWith("lsp/regression/css-color-declaration-admission-10562-"),
  );
  assert.equal(cases.length, 2);
  await withSession("negative-and-lifecycle", async (session, workspace, observations) => {
    for (const fixture of cases) {
      const source = fixture.files[0].bytes.toString("utf8");
      const uri = pathToFileURL(
        path.join(workspace, `${path.basename(fixture.inputs.root)}.vue`),
      ).href;
      await open(session, uri, source);
      const expected = fixture.requests[0].result;
      const result = await session.request("textDocument/documentColor", { textDocument: { uri } });
      observations.push({ source, result });
      assert.deepEqual(result, expected, `${fixture.id} whole authored color vector`);
      const edited =
        "<template><div /></template>\r\n<style>#abc:hover { content: '#fff'; }</style>";
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text: edited }],
      });
      const diagnostics = await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
      );
      assert.deepEqual(diagnostics, { uri, version: 2, diagnostics: [] });
      assert.deepEqual(
        await session.request("textDocument/documentColor", { textDocument: { uri } }),
        [],
      );
      session.notify("textDocument/didClose", { textDocument: { uri } });
      assert.deepEqual(
        await session.request("textDocument/documentColor", { textDocument: { uri } }),
        [],
      );
      await open(session, uri, source);
      assert.deepEqual(
        await session.request("textDocument/documentColor", { textDocument: { uri } }),
        expected,
      );
      session.notify("textDocument/didClose", { textDocument: { uri } });
    }
  });
});
