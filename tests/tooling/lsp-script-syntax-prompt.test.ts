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

// #11626: the prompt publication after an edit shows the script parser's
// syntax error for the new text at once, beside the moved type error; it must
// not wait for the native pass of that text.
const FIXTURE = path.join(
  root,
  "tests/_fixtures/differential/lsp/script-syntax-carried-native-11816",
);
const SOURCE = fs.readFileSync(path.join(FIXTURE, "App.vue.txt"), "utf8");
const TSCONFIG = fs.readFileSync(path.join(FIXTURE, "tsconfig.json.txt"), "utf8");
const SYNTAX_LINE = 2;
const BROKEN_LINE = 3;

const diagnosticCode = (code: unknown) => {
  if (typeof code === "number") return code;
  if (typeof code !== "string" || !/^(?:TS)?[0-9]+$/.test(code)) return undefined;
  return Number(code.replace(/^TS/, ""));
};
const typeErrors = (params: PublishDiagnosticsParams) =>
  params.diagnostics.filter(
    (diagnostic) => diagnostic.source === "vize/types" && diagnosticCode(diagnostic.code) === 2322,
  );
// This pinned case must have no stale native TS1xxx output beside the parser.
// Semantic TS1xxx findings have a separate retention control below.
const scriptSyntax = (params: PublishDiagnosticsParams) =>
  params.diagnostics.filter(
    (diagnostic) =>
      diagnostic.source === "vize/script" ||
      (diagnostic.source === "vize/types" &&
        Math.floor((diagnosticCode(diagnostic.code) ?? 0) / 1000) === 1),
  );
const at = (start: number, end: number, text: string) => ({
  range: { start: { line: start, character: 0 }, end: { line: end, character: 0 } },
  text,
});

const summary = (publications: PublishDiagnosticsParams[]) =>
  publications
    .map(
      ({ version, diagnostics }) =>
        `v${version ?? "-"} [${diagnostics.map((d) => `${d.source}:${String(d.code)}@${d.range?.start?.line}`).join(", ")}]`,
    )
    .join("\n");

test("prompt filters retain the same stale diagnostic for every checker code spelling", () => {
  const diagnostic = {
    source: "vize/types",
    message: "Expression expected.",
    range: { start: { line: 2, character: 0 }, end: { line: 2, character: 5 } },
  };
  const parser = { ...diagnostic, source: "vize/script", code: "script-syntax" };
  for (const code of [1109, "1109", "TS1109"]) {
    const stale = { ...diagnostic, code };
    const publication = { uri: "file:///App.vue", version: 2, diagnostics: [parser, stale] };
    // A stale native copy must make the original parser-only expectation fail.
    assert.deepEqual(scriptSyntax(publication), [parser, stale]);
    assert.notDeepEqual(scriptSyntax(publication), [parser]);
  }
  for (const code of [2322, "2322", "TS2322"]) {
    const retained = { ...diagnostic, code };
    const unrelated = { ...diagnostic, code: "TS23220" };
    const malformed = { ...diagnostic, code: "TS2322suffix" };
    const publication = {
      uri: "file:///App.vue",
      version: 2,
      diagnostics: [retained, unrelated, malformed],
    };
    assert.deepEqual(typeErrors(publication), [retained]);
  }
});

test("a new script syntax error reaches the prompt publication", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7 runtime for prompt script syntax",
    "TypeScript 7 runtime not found; skipping prompt script syntax",
  );
  if (!runtime) return;
  const directory = workspace("lsp-script-syntax-prompt-");
  const filePath = path.join(directory, "src/App.vue");
  const uri = pathToFileURL(filePath).href;
  const publications: PublishDiagnosticsParams[] = [];
  const session = new LspSession();
  let stopped = false;
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(filePath, SOURCE);
    fs.writeFileSync(path.join(directory, "tsconfig.json"), TSCONFIG);
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
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).version === 1 &&
        typeErrors(params as PublishDiagnosticsParams).length > 0,
      60_000,
    );
    // v2 adds a syntax error on its own line above the type error, v3 edits
    // elsewhere while it stays, and v4 removes it.
    const steps = [
      at(SYNTAX_LINE, SYNTAX_LINE, "const = ;\n"),
      at(BROKEN_LINE + 2, BROKEN_LINE + 2, "\n"),
      at(SYNTAX_LINE, SYNTAX_LINE + 1, ""),
    ];
    for (const [index, change] of steps.entries()) {
      const version = index + 2;
      session.notify("textDocument/didChange", {
        textDocument: { uri, version },
        contentChanges: [change],
      });
      for (let seen = 0; seen < 2; seen += 1) {
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

    const prompt = (version: number) => publications.find((params) => params.version === version)!;
    const syntaxOf = (params: PublishDiagnosticsParams) =>
      scriptSyntax(params).map(({ source, range }) => `${source}@${range?.start?.line}`);
    for (const version of [2, 3]) {
      // The parser's error alone: TypeScript's copy from the earlier text is gone.
      assert.deepEqual(
        syntaxOf(prompt(version)),
        [`vize/script@${SYNTAX_LINE}`],
        `v${version}'s prompt publication must show the current syntax error once:\n${summary(publications)}`,
      );
      assert.deepEqual(
        typeErrors(prompt(version)).map((diagnostic) => diagnostic.range?.start?.line),
        [BROKEN_LINE],
        `the moved type error stays:\n${summary(publications)}`,
      );
    }
    // Negative control: removing the syntax error clears it at once, and the
    // type error moves back.
    const repaired = prompt(4);
    assert.deepEqual(scriptSyntax(repaired), [], summary(publications));
    assert.deepEqual(
      typeErrors(repaired).map((diagnostic) => diagnostic.range?.start?.line),
      [BROKEN_LINE - 1],
      summary(publications),
    );
  } finally {
    if (!stopped) await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("an unrelated edit retains a semantic TS1xxx diagnostic in the prompt", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7 runtime for semantic diagnostic retention",
    "TypeScript 7 runtime not found; skipping semantic diagnostic retention",
  );
  if (!runtime) return;
  const directory = workspace("lsp-semantic-retention-");
  const filePath = path.join(directory, "src/App.vue");
  const uri = pathToFileURL(filePath).href;
  const source = fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/typechecker/semantic-diagnostic-provenance-11626/App.vue.txt",
    ),
    "utf8",
  );
  const publications: PublishDiagnosticsParams[] = [];
  const session = new LspSession();
  let stopped = false;
  const semantic = (params: PublishDiagnosticsParams) =>
    params.diagnostics.filter(
      (diagnostic) =>
        diagnostic.source === "vize/types" && diagnosticCode(diagnostic.code) === 1355,
    );
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(filePath, source);
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
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).version === 1 &&
        semantic(params as PublishDiagnosticsParams).length === 1,
      60_000,
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [at(4, 4, "\n")],
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) && (params as PublishDiagnosticsParams).version === 2,
      60_000,
    );
    const prompt = publications.find((params) => params.version === 2)!;
    assert.deepEqual(
      semantic(prompt).map((row) => row.range?.start?.line),
      [2],
      summary(publications),
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 3 },
      contentChanges: [at(2, 3, "const result = make();\n")],
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) && (params as PublishDiagnosticsParams).version === 3,
      60_000,
    );
    assert.deepEqual(
      semantic(publications.find((params) => params.version === 3)!),
      [],
      summary(publications),
    );
    await session.shutdown();
    stopped = true;
    await session.waitForClosed();
  } finally {
    if (!stopped) await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
