import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { AutoInsertParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

type OracleCase = {
  name: string;
  source: string;
  selection: { line: number; character: number };
  change: { rangeOffset: number; rangeLength: number; text: string };
  response: string | null;
};

const oracle = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/vue-language-server-auto-insertion.json"),
    "utf8",
  ),
) as {
  oracle: { package: string; version: string; method: string; measuredAt: string };
  capability: {
    triggerCharacters: string[];
    configurationSections: Array<string[] | null>;
  };
  cases: OracleCase[];
  dotValue: OracleCase;
};

function resolveCorsaBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

async function request(
  session: LspSession,
  uri: string,
  fixture: OracleCase,
  options: { settle?: boolean } = {},
): Promise<unknown> {
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: fixture.source },
  });
  if (options.settle) {
    // The `.value` decision asks the type backend; before the opened document
    // has produced its first diagnostics the backend can answer cold and the
    // gate degrades to no-insert. A real editing session types after the file
    // settles, so the probe waits for that signal — the recurring release
    // flake was exactly this race on slow runners.
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => (params as { uri: string }).uri === uri,
      60000,
    );
  }
  const params: AutoInsertParams = {
    textDocument: { uri },
    selection: fixture.selection,
    change: fixture.change,
  };
  const response = await session.request(oracle.oracle.method, params);
  session.notify("textDocument/didClose", { textDocument: { uri } });
  return response;
}

test("real stdio server matches the pinned Vue LS auto-insertion responses", async () => {
  assert.deepEqual(oracle.oracle, {
    package: "@vue/language-server",
    version: "3.3.8",
    method: "volar/client/autoInsert",
    measuredAt: "2026-08-01",
  });

  const testRootDir = path.join(testOutputRoot, "lsp-auto-insertion-markup");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, {
      editor: true,
      typecheck: false,
      lint: false,
      autoInsert: true,
    });
    for (const [index, fixture] of oracle.cases.entries()) {
      const uri = pathToFileURL(path.join(workspaceDir, `Case${index}.vue`)).href;
      assert.deepEqual(await request(session, uri, fixture), fixture.response, fixture.name);
    }

    const hostile = oracle.cases[0];
    const uri = pathToFileURL(path.join(workspaceDir, "Hostile.vue")).href;
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: hostile.source },
    });
    assert.deepEqual(
      await session.request(oracle.oracle.method, {
        textDocument: { uri },
        selection: hostile.selection,
        change: { ...hostile.change, rangeOffset: Number.MAX_SAFE_INTEGER },
      }),
      null,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("real stdio server asks Corsa type information before inserting .value", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveCorsaBinary(),
    "Corsa runtime for the auto-insertion gate",
    "Corsa runtime is unavailable",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-auto-insertion-dot-value");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  try {
    fs.mkdirSync(path.join(workspaceDir, "node_modules"), { recursive: true });
    fs.symlinkSync(
      path.join(root, "tests/node_modules/vue"),
      path.join(workspaceDir, "node_modules/vue"),
      process.platform === "win32" ? "junction" : "dir",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({ include: ["*.vue"], compilerOptions: { strict: true } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({ typeChecker: { corsaPath } }),
      "utf8",
    );

    await session.initialize(workspaceDir, {
      editor: true,
      typecheck: true,
      lint: false,
      autoInsert: true,
    });
    const uri = pathToFileURL(path.join(workspaceDir, "Ref.vue")).href;
    assert.deepEqual(
      await request(session, uri, oracle.dotValue, { settle: true }),
      oracle.dotValue.response,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
