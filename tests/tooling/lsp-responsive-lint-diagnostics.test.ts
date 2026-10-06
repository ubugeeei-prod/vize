import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { readPinnedArtifact, sha256 } from "../differential/harness.mjs";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { NativeDiagnosticsGate } from "./support/lsp/responsive-lint-native-gate.ts";

const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/lsp-regressions/responsive-lint-diagnostics-8002",
);
const corpus = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "case.json"), "utf8"));
const files = new Map<string, Buffer>(
  [...corpus.originalFiles, ...corpus.authoredFiles].map(
    (pin: { path: string; sha256: string }) => [pin.path, readPinnedArtifact(fixtureRoot, pin)],
  ),
);
const original = files.get(corpus.originalSource)!.toString("utf8");
const removed = original.replace(' alt=""', "");
assert.notEqual(removed, original);
const sourceHash = sha256(files.get(corpus.originalSource)!);

function wait(session: LspSession, uri: string, version: number | undefined) {
  return session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    15_000,
  ) as Promise<PublishDiagnosticsParams>;
}

function change(session: LspSession, uri: string, version: number, text: string) {
  session.notify("textDocument/didChange", {
    textDocument: { uri, version },
    contentChanges: [{ text }],
  });
}

function alt(params: PublishDiagnosticsParams): boolean {
  return params.diagnostics.some((diagnostic) => diagnostic.code === "a11y/alt-text");
}

async function run(typecheck: boolean, corsaPath: string) {
  const directory = workspace(`responsive-lint-${typecheck}-`);
  const file = path.join(directory, "src/App.vue");
  const uri = pathToFileURL(file).href;
  let session: LspSession | undefined;
  let gate: NativeDiagnosticsGate | undefined;
  const gates: NativeDiagnosticsGate[] = [];
  let initialization: unknown;
  const notifications: { elapsedMs: number; params: unknown }[] = [];
  const rows: {
    version: number;
    source: string;
    early: PublishDiagnosticsParams;
    elapsedMs: number;
    terminal: PublishDiagnosticsParams;
  }[] = [];
  let started = performance.now();
  try {
    fs.mkdirSync(path.dirname(file));
    fs.writeFileSync(file, files.get(corpus.originalSource)!);
    fs.writeFileSync(path.join(directory, "vize.config.json"), files.get(corpus.originalConfig)!);
    fs.writeFileSync(path.join(directory, "tsconfig.json"), files.get(corpus.runtimeConfig)!);
    session = new LspSession();
    session.notificationObservers.push((method, params) => {
      if (method === "textDocument/publishDiagnostics" && isDiagnosticsForUri(params, uri)) {
        notifications.push({ elapsedMs: performance.now() - started, params });
      }
    });
    initialization = await session.initialize(directory, { lint: true, typecheck, editor: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: original },
    });
    const initial = await wait(session, uri, 1);
    assert.equal(alt(initial), false);
    for (let cycle = 0; cycle < 3; cycle++) {
      if (typecheck) {
        gate = new NativeDiagnosticsGate(session.processId, fs.realpathSync(corsaPath));
        gates.push(gate);
        await gate.stop();
      }
      const version = cycle * 2 + 2;
      const before = notifications.length;
      started = performance.now();
      change(session, uri, version, removed);
      const early = await wait(session, uri, version);
      const elapsedMs = performance.now() - started;
      assert.equal(alt(early), true);
      if (typecheck) {
        gate!.assertStopped();
        assert.deepEqual(
          notifications.slice(before).map(({ params }) => params),
          [early],
          "exact current-version feedback arrives while native remains stopped",
        );
      }
      started = performance.now();
      change(session, uri, version + 1, original);
      const repaired = await wait(session, uri, version + 1);
      const repairElapsedMs = performance.now() - started;
      assert.equal(
        alt(repaired),
        false,
        "repair must clear lint while the previous native pass remains gated",
      );
      if (typecheck) {
        gate!.assertStopped();
        gate!.resume();
      }
      const terminal = typecheck ? await wait(session, uri, version + 1) : repaired;
      assert.equal(alt(terminal), false);
      assert.deepEqual(terminal.diagnostics, initial.diagnostics, "full restored original result");
      if (typecheck)
        assert.deepEqual(
          notifications.slice(before).map(({ params }) => params),
          [early, repaired, terminal],
          "exactly one partial per edit and one latest complete; no superseded native result",
        );
      rows.push({ version, source: removed, early, elapsedMs, terminal });
      rows.push({
        version: version + 1,
        source: original,
        early: repaired,
        elapsedMs: repairElapsedMs,
        terminal,
      });
    }
    // Authored positive: a real TS2322 must arrive in the second publication.
    if (typecheck) {
      const typed = files.get(corpus.positiveNativeControl.source)!.toString("utf8");
      const expected = JSON.parse(
        files.get(corpus.positiveNativeControl.expected)!.toString("utf8"),
      );
      gate = new NativeDiagnosticsGate(session.processId, fs.realpathSync(corsaPath));
      gates.push(gate);
      await gate.stop();
      started = performance.now();
      const before = notifications.length;
      change(session, uri, 8, typed);
      const early = await wait(session, uri, 8);
      const typeEarlyElapsedMs = performance.now() - started;
      gate.assertStopped();
      assert.deepEqual(
        early,
        { uri, version: 8, diagnostics: expected.early },
        "whole authored early packet",
      );
      gate.resume();
      const terminal = await wait(session, uri, 8);
      assert.deepEqual(
        terminal,
        { uri, version: 8, diagnostics: expected.terminal },
        "entire independently authored native packet must merge",
      );
      assert.deepEqual(
        notifications.slice(before).map(({ params }) => params),
        [early, terminal],
        "positive native control retains both entire same-version packets",
      );
      rows.push({
        version: 8,
        source: typed,
        early,
        elapsedMs: typeEarlyElapsedMs,
        terminal,
      });
    }
    assert.deepEqual(fs.readFileSync(file), files.get(corpus.originalSource)!);
    assert.deepEqual(
      fs.readFileSync(path.join(directory, "vize.config.json")),
      files.get(corpus.originalConfig)!,
    );
    return {
      initialization,
      initial,
      uri,
      rows,
      notifications,
      gates: gates.map(({ lives, signals }) => ({ lives, signals })),
      runtime: {
        vue: JSON.parse(
          fs.readFileSync(
            createRequire(path.join(directory, "package.json")).resolve("vue/package.json"),
            "utf8",
          ),
        ),
        native: {
          executable: fs.realpathSync(corsaPath),
          sha256: sha256(fs.readFileSync(corsaPath)),
          manifest: JSON.parse(
            fs.readFileSync(
              path.join(path.dirname(path.dirname(fs.realpathSync(corsaPath))), "package.json"),
              "utf8",
            ),
          ),
        },
      },
      stderr: session.stderrText,
    };
  } catch (failure) {
    const output = path.join(
      root,
      `target/differential/responsive-lint-diagnostics-8002-${typecheck}-failure.json`,
    );
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(
      output,
      JSON.stringify(
        {
          sourceHash,
          initialization,
          uri,
          rows,
          notifications,
          gates: gates.map(({ lives, signals }) => ({ lives, signals })),
          stderr: session?.stderrText,
          failure: failure instanceof Error ? failure.stack : String(failure),
        },
        null,
        2,
      ),
    );
    throw failure;
  } finally {
    try {
      gate?.resume();
    } finally {
      try {
        await session?.shutdown();
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  }
}

await test("original edit lint and repairs publish before a gated genuine native type pass", async (t) => {
  if (process.platform !== "linux") {
    t.skip("real Linux native PID/birth/executable gate; other session laws remain portable");
    return;
  }
  const corsa = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "actual native diagnostics runtime",
    "native runtime unavailable",
  );
  if (!corsa) return;
  const binary = path.join(root, "target/ci/vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const prior = {
    binary: process.env.VIZE_LSP_BIN,
    required: process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    corsa: process.env.CORSA_PATH,
  };
  process.env.VIZE_LSP_BIN = binary;
  process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
  process.env.CORSA_PATH = corsa;
  const output = path.join(root, "target/differential/responsive-lint-diagnostics-8002.json");
  const observed: {
    control?: Awaited<ReturnType<typeof run>>;
    native?: Awaited<ReturnType<typeof run>>;
    failure?: string;
  } = {};
  try {
    observed.control = await run(false, corsa);
    observed.native = await run(true, corsa);
    assert.equal(observed.control.rows.length, 6);
    assert.equal(observed.native.rows.length, 7);
    for (let index = 0; index < 6; index++) {
      assert.deepEqual(
        observed.native.rows[index].early,
        {
          uri: observed.native.uri,
          version: observed.native.rows[index].version,
          diagnostics: observed.control.rows[index].early.diagnostics,
        },
        "entire early public packet matches the independent typecheck-disabled control",
      );
    }
  } catch (failure) {
    observed.failure = failure instanceof Error ? failure.stack : String(failure);
    throw failure;
  } finally {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(
      output,
      JSON.stringify(
        {
          schema: "vize.lsp.responsive-lint-diagnostics.observation",
          version: 1,
          sourceHash,
          originalAuthority: corpus,
          scope:
            "original three remove/restore cycles, whole public packets, real stopped native process; no universal speedup claim",
          ...observed,
        },
        null,
        2,
      ),
    );
    if (prior.binary === undefined) delete process.env.VIZE_LSP_BIN;
    else process.env.VIZE_LSP_BIN = prior.binary;
    if (prior.required === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
    else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = prior.required;
    if (prior.corsa === undefined) delete process.env.CORSA_PATH;
    else process.env.CORSA_PATH = prior.corsa;
  }
});
