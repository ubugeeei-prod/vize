import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

// #11519: LSP push replaces a URI's whole set, so a lint/parser publication
// for a new version must keep the latest type diagnostics, moved to its text.
const SOURCE = fs.readFileSync(
  path.join(root, "tests/_fixtures/differential/lsp/type-diagnostic-retention-11519/App.vue.txt"),
  "utf8",
);

type Position = { line: number; character: number };
type Edit = { start: Position; end: Position; text: string };
type Step = { version: number; edit: Edit; type: Position | null; lint: boolean };

const at = (line: number, character: number): Position => ({ line, character });
const insert = (position: Position, text: string): Edit => ({
  start: position,
  end: position,
  text,
});
const BROKEN = at(2, SOURCE.split("\n")[2].indexOf("broken"));

const STEPS: Step[] = [
  // A new line above the error moves it down.
  { version: 2, edit: insert(at(1, 0), "// note\n"), type: at(3, BROKEN.character), lint: true },
  // An indent before it on its own line moves it right.
  { version: 3, edit: insert(at(3, 0), "  "), type: at(3, BROKEN.character + 2), lint: true },
  // A lint repair elsewhere clears the lint promptly and keeps the type error.
  {
    version: 4,
    edit: insert(at(8, '    <li v-for="item in items"'.length), ' :key="item"'),
    type: at(3, BROKEN.character + 2),
    lint: false,
  },
  // Negative control: repairing the value clears the type error.
  {
    version: 5,
    edit: { start: at(3, 25), end: at(3, 32), text: "2" },
    type: null,
    lint: false,
  },
];

const typeErrors = (params: PublishDiagnosticsParams) =>
  params.diagnostics.filter(
    (diagnostic) => diagnostic.source === "vize/types" && Number(diagnostic.code) === 2322,
  );
const vForKeyWarnings = (params: PublishDiagnosticsParams) =>
  params.diagnostics.filter(
    (diagnostic) =>
      diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
  );
const summary = (publications: PublishDiagnosticsParams[]) =>
  publications
    .map(
      ({ version, diagnostics }) =>
        `v${version ?? "-"} [${diagnostics.map((d) => `${d.source}:${String(d.code)}@${d.range?.start?.line}:${d.range?.start?.character}`).join(", ")}]`,
    )
    .join("\n");

test("lint publications after an edit keep the moved type diagnostics", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7 runtime for retained type diagnostics",
    "TypeScript 7 runtime not found; skipping retained type diagnostics",
  );
  if (!runtime) return;
  const directory = workspace("lsp-type-retention-");
  const filePath = path.join(directory, "src/App.vue");
  const uri = pathToFileURL(filePath).href;
  const publications: PublishDiagnosticsParams[] = [];
  const session = new LspSession();
  let stopped = false;
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(filePath, SOURCE);
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({ compilerOptions: { strict: true, noEmit: true }, include: ["src"] }),
    );
    writeVueConfigFixture(directory, {
      lsp: { lint: true, typecheck: true },
      typeChecker: { corsaPath: runtime },
    });
    session.notificationObservers.push((method, params) => {
      if (method === "textDocument/publishDiagnostics" && isDiagnosticsForUri(params, uri)) {
        publications.push(params as PublishDiagnosticsParams);
      }
    });
    await session.initialize(directory, { editor: true, lint: true, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });
    const opened = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).version === 1 &&
        typeErrors(params as PublishDiagnosticsParams).length > 0,
      60_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(typeErrors(opened)[0]?.range?.start, BROKEN, summary([opened]));
    assert.equal(vForKeyWarnings(opened).length, 1, summary([opened]));

    for (const { version, edit } of STEPS) {
      session.notify("textDocument/didChange", {
        textDocument: { uri, version },
        contentChanges: [{ range: { start: edit.start, end: edit.end }, text: edit.text }],
      });
      // The lint publication, then the complete one after the type pass.
      for (let index = 0; index < 2; index += 1) {
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, uri) &&
            (params as PublishDiagnosticsParams).version === version,
          60_000,
        );
      }
    }
    await session.shutdown();
    stopped = true;
    await session.waitForClosed();

    for (const { version, type, lint } of STEPS) {
      const published = publications.filter((params) => params.version === version);
      assert.ok(published.length >= 2, summary(publications));
      const complete = published.at(-1)!;
      if (type) {
        for (const params of published) {
          assert.deepEqual(
            typeErrors(params).map((diagnostic) => diagnostic.range?.start),
            [type],
            `v${version} dropped or moved the type error:\n${summary(publications)}`,
          );
        }
      } else {
        assert.deepEqual(typeErrors(complete), [], summary(publications));
      }
      assert.equal(vForKeyWarnings(published[0]).length, lint ? 1 : 0, summary(publications));
      assert.equal(vForKeyWarnings(complete).length, lint ? 1 : 0, summary(publications));
    }
  } finally {
    if (!stopped) await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
