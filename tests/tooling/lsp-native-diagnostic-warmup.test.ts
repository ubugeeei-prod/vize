import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { NativeDiagnosticsGate } from "./support/lsp/responsive-lint-native-gate.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/lsp/native-diagnostic-warmup-10604",
);
const fixture = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "case.json"), "utf8"));
const erroneous = readPinnedArtifact(fixtureRoot, {
  path: fixture.files[0].source,
  sha256: fixture.files[0].sha256,
}).toString("utf8");
const localColors = JSON.parse(
  readPinnedArtifact(fixtureRoot, {
    path: fixture.expected.source,
    sha256: fixture.expected.sha256,
  }).toString("utf8"),
);
const repaired = erroneous.replaceAll("needsRef(items)", "items.length");
const nativeErrors = (packet: PublishDiagnosticsParams) =>
  packet.diagnostics.filter(
    (diagnostic) => diagnostic.source === "vize/types" && Number(diagnostic.code) === 2739,
  );
const pending = (packet: PublishDiagnosticsParams) =>
  packet.diagnostics.some((diagnostic) => diagnostic.code === "typecheck-pending");

await test(
  "native warm-up retains late type errors, shares its project, and fences stale/config/closed results",
  { timeout: 120_000 },
  async (t) => {
    if (process.platform !== "linux") {
      t.skip("actual Linux native process custody is required for the bounded gate");
      return;
    }
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "genuine native TypeScript runtime",
      "native runtime unavailable",
    );
    if (!runtime) return;
    const directory = workspace("native-diagnostic-warmup-10604-");
    const role = process.env.VIZE_WARMUP_ROLE ?? "candidate";
    const output = path.join(testOutputRoot, "native-diagnostic-warmup-10604", `${role}.json`);
    const sourceRoot = process.env.VIZE_WARMUP_SOURCE_ROOT ?? root;
    const binary = process.env.VIZE_LSP_BIN ?? path.join(sourceRoot, "target/ci/vize");
    const session = new LspSession({ repoRoot: sourceRoot, binary });
    const first = pathToFileURL(path.join(directory, "First.vue")).href;
    const second = pathToFileURL(path.join(directory, "Second.vue")).href;
    const third = pathToFileURL(path.join(directory, "Third.vue")).href;
    const began = performance.now();
    const notifications: { elapsedMs: number; params: PublishDiagnosticsParams }[] = [];
    const gates: NativeDiagnosticsGate[] = [];
    let failure: string | undefined;
    let coldFinishedMs: number | undefined;
    const wait = (
      uri: string,
      predicate: (params: PublishDiagnosticsParams) => boolean,
      after = 0,
    ) =>
      session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) =>
          isDiagnosticsForUri(params, uri) &&
          notifications.findIndex((row) => row.params === params) >= after &&
          predicate(params),
        45_000,
      ) as Promise<PublishDiagnosticsParams>;
    const open = (uri: string) =>
      session.notify("textDocument/didOpen", {
        textDocument: { uri, version: 1, languageId: "vue", text: erroneous },
      });
    const change = (uri: string, version: number, text: string) =>
      session.notify("textDocument/didChange", {
        textDocument: { uri, version },
        contentChanges: [{ text }],
      });
    const stop = async () => {
      const gate = new NativeDiagnosticsGate(session.processId, fs.realpathSync(runtime));
      gates.push(gate);
      await gate.stop();
      return gate;
    };
    try {
      // Exact report-size authored corpus: 487 SFCs + 1,414 TS files = 1,901.
      for (let index = 0; index < 1414; index++)
        fs.writeFileSync(
          path.join(directory, `Module${index}.ts`),
          `export const value${index} = ${index};\n`,
        );
      for (let index = 0; index < 484; index++)
        fs.writeFileSync(
          path.join(directory, `Component${index}.vue`),
          "<template><div /></template>\n",
        );
      for (const name of ["First", "Second", "Third"])
        fs.writeFileSync(path.join(directory, `${name}.vue`), erroneous);
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            skipLibCheck: true,
            target: "ESNext",
            module: "ESNext",
            moduleResolution: "Bundler",
          },
          include: ["*.ts", "*.vue"],
        }),
      );
      writeVueConfigFixture(
        directory,
        JSON.stringify({
          linter: { preset: "recommended" },
          languageServer: { lint: true, typecheck: true, editor: true },
        }),
      );
      session.notificationObservers.push((method, params) => {
        if (
          method === "textDocument/publishDiagnostics" &&
          typeof params === "object" &&
          params !== null &&
          "diagnostics" in params
        ) {
          notifications.push({
            elapsedMs: performance.now() - began,
            params: params as PublishDiagnosticsParams,
          });
        }
      });
      await session.initialize(directory, { lint: true, editor: true, typecheckTimeoutMs: 1 });
      open(first);
      const lint = await wait(
        first,
        (packet) =>
          packet.version === undefined &&
          packet.diagnostics.some((diagnostic) => diagnostic.source === "vize/lint"),
      );
      assert.equal(nativeErrors(lint).length, 0);
      const initial = await wait(first, (packet) => packet.version === 1);
      coldFinishedMs = performance.now() - began;
      if (role === "baseline") {
        assert.ok(
          initial.diagnostics.some((diagnostic) => diagnostic.code === "typecheck-timed-out"),
          "the original source must reproduce the dropped native answer at the bounded threshold",
        );
        assert.equal(nativeErrors(initial).length, 0);
        return;
      }
      assert.equal(
        nativeErrors(initial).length,
        8,
        "all eight genuine Ref unwrapping errors survive the soft threshold",
      );
      assert.ok(
        notifications.some(({ params }) => params.uri === first && pending(params)),
        "slow startup must disclose that its retained native work continues",
      );

      const gate = await stop();
      const secondPass = notifications.length;
      change(first, 2, erroneous);
      await wait(first, pending, secondPass);
      gate.assertStopped();
      assert.deepEqual(
        await session.request("textDocument/documentColor", { textDocument: { uri: first } }),
        localColors,
      );
      gate.assertStopped();
      open(second);
      const secondLint = await wait(
        second,
        (packet) =>
          packet.version === undefined &&
          packet.diagnostics.some((diagnostic) => diagnostic.source === "vize/lint"),
      );
      assert.equal(nativeErrors(secondLint).length, 0);
      gate.assertStopped();
      gate.resume();
      assert.equal(
        nativeErrors(
          await wait(first, (packet) => packet.version === 2 && nativeErrors(packet).length === 8),
        ).length,
        8,
      );
      assert.equal(nativeErrors(await wait(second, (packet) => packet.version === 1)).length, 8);
      const afterWarm = new NativeDiagnosticsGate(session.processId, fs.realpathSync(runtime));
      assert.deepEqual(
        afterWarm.lives.map(({ pid, birthTicks, executable }) => ({ pid, birthTicks, executable })),
        gate.lives.map(({ pid, birthTicks, executable }) => ({ pid, birthTicks, executable })),
        "both documents reuse the same actual native process lifetimes",
      );

      const staleGate = await stop();
      const stalePass = notifications.length;
      change(first, 3, erroneous);
      await wait(first, pending, stalePass);
      const staleBoundary = notifications.length;
      change(first, 4, repaired);
      session.notify("textDocument/didClose", { textDocument: { uri: second } });
      await wait(second, (packet) => packet.diagnostics.length === 0);
      staleGate.resume();
      const final = await wait(
        first,
        (packet) =>
          packet.version === 4 &&
          packet.diagnostics.some(
            (diagnostic) => diagnostic.source === "vize/types" && Number(diagnostic.code) === 2322,
          ),
      );
      assert.equal(nativeErrors(final).length, 0);
      assert.ok(
        notifications
          .slice(staleBoundary)
          .every(({ params }) => params.uri !== first || params.version !== 3),
        "superseded native results cannot publish",
      );
      assert.ok(
        notifications
          .slice(staleBoundary)
          .every(({ params }) => params.uri !== second || nativeErrors(params).length === 0),
        "closed files cannot receive a late native result",
      );

      const configGate = await stop();
      const configPass = notifications.length;
      change(first, 5, erroneous);
      await wait(first, pending, configPass);
      writeVueConfigFixture(
        directory,
        JSON.stringify({ languageServer: { lint: true, typecheck: false, editor: true } }),
      );
      const configUri = pathToFileURL(path.join(directory, "vite.config.mjs")).href;
      session.notify("workspace/didChangeWatchedFiles", { changes: [{ uri: configUri, type: 2 }] });
      open(third);
      const disabled = await wait(third, (packet) => packet.version === 1);
      assert.ok(disabled.diagnostics.every((diagnostic) => diagnostic.source !== "vize/types"));
      const configBoundary = notifications.length;
      configGate.resume();
      // A foreground local request is the settlement barrier after revocation.
      await session.request("textDocument/documentSymbol", { textDocument: { uri: third } });
      assert.ok(
        notifications
          .slice(configBoundary)
          .every(({ params }) => nativeErrors(params).length === 0),
        "revoked type checking cannot publish its retained old result",
      );
      assert.ok(
        notifications.every(({ params }) =>
          params.diagnostics.every((diagnostic) => diagnostic.code !== "typecheck-timed-out"),
        ),
        "a soft threshold must never turn a real native result into a timeout",
      );
    } catch (error) {
      failure = error instanceof Error ? error.stack : String(error);
      throw error;
    } finally {
      for (const gate of gates) gate.resume();
      await session.shutdown();
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.writeFileSync(
        output,
        JSON.stringify(
          {
            schema: "vize.lsp.native-diagnostic-warmup.observation",
            role,
            sourceRoot,
            binary,
            authoredFiles: { vue: 487, ts: 1414 },
            softThresholdMs: 1,
            coldFinishedMs,
            notifications,
            nativeLifetimes: gates.map((gate) => ({ lives: gate.lives, signals: gate.signals })),
            stderr: session.stderrText,
            failure,
          },
          null,
          2,
        ),
      );
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
