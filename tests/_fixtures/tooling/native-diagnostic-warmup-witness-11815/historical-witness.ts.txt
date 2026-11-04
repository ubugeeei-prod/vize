import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { readPinnedArtifact, sha256 } from "../differential/harness.mjs";
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
const originalIssue = JSON.parse(
  readPinnedArtifact(fixtureRoot, fixture.provenance.originalIssue).toString("utf8"),
);
assert.equal(originalIssue.number, 10604);
assert.equal(
  sha256(fs.readFileSync(path.join(root, fixture.provenance.warmupWitness.path))),
  fixture.provenance.warmupWitness.sha256,
  "the full actual stdio transition witness must retain its pinned source custody",
);
const repaired = erroneous.replaceAll("needsRef(items)", "() => items.length");
const changedSource = (value: number) =>
  erroneous.replace("persistent: string = 1", `persistent: string = ${value}`);
const nativeErrors = (packet: PublishDiagnosticsParams) =>
  packet.diagnostics.filter(
    (diagnostic) => diagnostic.source === "vize/types" && Number(diagnostic.code) === 2739,
  );
const pending = (packet: PublishDiagnosticsParams) =>
  packet.diagnostics.some((diagnostic) => diagnostic.code === "typecheck-pending");

await test(
  "native warm-up retains late type errors and fences stale/config/closed results with unchanged-root reuse",
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
    const open = (uri: string, version = 1, text = erroneous) =>
      session.notify("textDocument/didOpen", {
        textDocument: { uri, version, languageId: "vue", text },
      });
    const change = (uri: string, version: number, text: string) =>
      session.notify("textDocument/didChange", {
        textDocument: { uri, version },
        contentChanges: [{ text }],
      });
    const logAfter = (needle: string, after: number) =>
      new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(
          () =>
            reject(
              new Error(
                `native pass did not drain after revocation: ${needle}\n${session.stderrText}`,
              ),
            ),
          20_000,
        );
        const observer = (text: string) => {
          if (text.slice(after).includes(needle)) {
            clearTimeout(timeout);
            session.stderrObservers.splice(session.stderrObservers.indexOf(observer), 1);
            resolve();
          }
        };
        session.stderrObservers.push(observer);
        observer(session.stderrText);
      });
    const finishedAfter = (uri: string, version: number, after: number) =>
      logAfter(`finished initial type diagnostics for ${uri} version ${version}`, after);
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
      change(first, 2, changedSource(2));
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
      await t.test(
        "UNRESOLVED #10622: unrelated generated tsconfig root expansion restarts the native process",
        { todo: "Known root-expansion transport boundary; no reuse claim across changed roots" },
        () => {
          assert.deepEqual(
            afterWarm.lives.map(({ pid, birthTicks, executable }) => ({
              pid,
              birthTicks,
              executable,
            })),
            gate.lives.map(({ pid, birthTicks, executable }) => ({ pid, birthTicks, executable })),
            "both documents reuse the same actual native process lifetimes",
          );
        },
      );
      const admittedGate = await stop();
      const admittedPass = notifications.length;
      change(first, 3, changedSource(7));
      await wait(first, pending, admittedPass);
      change(second, 2, changedSource(8));
      await wait(second, (packet) => packet.version === 2, admittedPass);
      admittedGate.assertStopped();
      admittedGate.resume();
      await wait(
        first,
        (packet) => packet.version === 3 && nativeErrors(packet).length === 8,
        admittedPass,
      );
      await wait(
        second,
        (packet) => packet.version === 2 && nativeErrors(packet).length === 8,
        admittedPass,
      );
      const admittedAfter = new NativeDiagnosticsGate(session.processId, fs.realpathSync(runtime));
      assert.deepEqual(
        admittedAfter.lives.map(({ pid, birthTicks, executable }) => ({
          pid,
          birthTicks,
          executable,
        })),
        admittedGate.lives.map(({ pid, birthTicks, executable }) => ({
          pid,
          birthTicks,
          executable,
        })),
        "both pre-admitted roots strictly reuse the same native process lifetime while the generated config is unchanged",
      );

      const staleGate = await stop();
      const stalePass = notifications.length;
      change(first, 4, changedSource(3));
      await wait(first, pending, stalePass);
      const staleBoundary = notifications.length;
      change(first, 5, repaired);
      session.notify("textDocument/didClose", { textDocument: { uri: second } });
      await wait(second, (packet) => packet.diagnostics.length === 0);
      staleGate.resume();
      const final = await wait(
        first,
        (packet) =>
          packet.version === 5 &&
          packet.diagnostics.some(
            (diagnostic) => diagnostic.source === "vize/types" && Number(diagnostic.code) === 2322,
          ),
      );
      assert.equal(nativeErrors(final).length, 0);
      assert.ok(
        notifications
          .slice(staleBoundary)
          .every(({ params }) => params.uri !== first || params.version !== 4),
        "superseded native results cannot publish",
      );
      assert.ok(
        notifications
          .slice(staleBoundary)
          .every(({ params }) => params.uri !== second || nativeErrors(params).length === 0),
        "closed files cannot receive a late native result",
      );

      const closePass = notifications.length;
      const closeLog = session.stderrText.length;
      const closePrepared = logAfter(
        `prepared initial native project for ${second} version 3`,
        closeLog,
      );
      open(second, 3, changedSource(6));
      await closePrepared;
      const closeGate = await stop();
      await wait(second, pending, closePass);
      closeGate.assertStopped();
      const closeBoundary = notifications.length;
      session.notify("textDocument/didClose", { textDocument: { uri: second } });
      await wait(second, (packet) => packet.diagnostics.length === 0, closeBoundary);
      const closedDrained = finishedAfter(second, 3, closeLog);
      closeGate.resume();
      await closedDrained;
      assert.ok(
        notifications
          .slice(closeBoundary)
          .every(({ params }) => params.uri !== second || nativeErrors(params).length === 0),
        "the actually pending native read must drain without publishing to its closed document",
      );

      const configGate = await stop();
      const configPass = notifications.length;
      const configLog = session.stderrText.length;
      change(first, 6, changedSource(5));
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
      const revokedDrained = finishedAfter(first, 6, configLog);
      configGate.resume();
      await revokedDrained;
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
            originalIssue: fixture.provenance.originalIssue,
            warmupWitness: fixture.provenance.warmupWitness,
            fixtureFiles: fixture.files,
            qualification:
              "Full genuine native diagnostic transitions are checked here; manifest documentColor is a separate structural control only.",
            authoredFiles: { vue: 487, ts: 1414 },
            softThresholdMs: 1,
            coldFinishedMs,
            rootExpansion: {
              state: "UNRESOLVED",
              issue: 10622,
              scope:
                "Generated tsconfig root expansion still replaces the native process; same-process reuse is qualified only for unchanged, pre-admitted roots.",
              originalStrictFailure:
                "https://github.com/ubugeeei-prod/vize/actions/runs/40183984382",
            },
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
