import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  CapacityCoverage,
  capacityPacket,
  capacitySource,
  waitUntil,
} from "./support/lsp/diagnostic-capacity.ts";
import { CapacityNativeCoverage } from "./support/lsp/diagnostic-capacity-native.ts";
import {
  cleanupCapacitySession,
  nativeIpcEvidence,
  nativeIpcGrew,
} from "./support/lsp/diagnostic-capacity-pipe.ts";
import { NativeDiagnosticsGate } from "./support/lsp/responsive-lint-native-gate.ts";
import { LspSession } from "./support/lsp/session.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

test(
  "an old native pass cannot contaminate or retire a same-version reopened document",
  {
    skip: process.platform !== "linux",
    timeout: 120_000,
  },
  async (t) => {
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "genuine native backend",
      "native runtime unavailable",
    );
    if (!runtime) return;
    const sourceRoot = process.env.VIZE_CAPACITY_SOURCE_ROOT ?? root;
    const binary = process.env.VIZE_LSP_BIN ?? path.join(sourceRoot, "target/ci/vize");
    const producer: unknown = JSON.parse(
      fs.readFileSync(binary + ".differential-build.json", "utf8"),
    );
    const directory = workspace("diagnostic-capacity-reopen-");
    const uri = (file: string) => pathToFileURL(path.join(directory, file)).href;
    const warm = { ...capacitySource(9000, uri("Warm.vue")), file: "Warm.vue" };
    const old = { ...capacitySource(9001, uri("Target.vue")), file: "Target.vue" };
    const next = { ...capacitySource(9002, old.uri), file: old.file };
    next.valid = next.valid.replace("#123456", "#abcdef");
    next.invalid = next.invalid.replace("#123456", "#abcdef");
    fs.writeFileSync(path.join(directory, warm.file), warm.invalid);
    fs.writeFileSync(path.join(directory, old.file), old.valid);
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          skipLibCheck: true,
          module: "ESNext",
          moduleResolution: "Bundler",
          target: "ES2024",
          lib: ["ES2024", "DOM"],
        },
        include: ["*.vue"],
      }),
    );
    writeVueConfigFixture(directory, {
      languageServer: { lint: true, typecheck: true, editor: true },
      linter: { rules: { "vue/require-v-for-key": "error" } },
      typeChecker: { corsaPath: runtime },
    });
    const session = new LspSession({ repoRoot: sourceRoot, binary });
    const warmPackets = new CapacityCoverage([warm]);
    const oldPackets = new CapacityCoverage([old]);
    const newPackets = new CapacityCoverage([next]);
    const warmNative = new CapacityNativeCoverage(warmPackets, () => session.stderrText);
    const oldNative = new CapacityNativeCoverage(oldPackets, () => session.stderrText);
    let epochOffset = 0;
    const newNative = new CapacityNativeCoverage(newPackets, () =>
      session.stderrText.slice(epochOffset),
    );
    let gate: NativeDiagnosticsGate | undefined, failure: unknown;
    const events: { stage: string; value?: unknown }[] = [];
    const report: Record<string, unknown> = {
      scope: "same URI and reused version1 with a genuine old native pass held in flight",
      producer,
      input: { warm, old, next },
      events,
      qualified: false,
    };
    session.notificationObservers.push((method, value) => {
      if (method === "textDocument/publishDiagnostics" && capacityPacket(value)) {
        const at = performance.now();
        warmPackets.record(value, at);
        oldPackets.record(value, at);
        newPackets.record(value, at);
      }
    });
    const open = (probe: typeof warm, version: number, text: string) =>
      session.notify("textDocument/didOpen", {
        textDocument: { uri: probe.uri, languageId: "vue", version, text },
      });
    const change = (version: number, text: string) =>
      session.notify("textDocument/didChange", {
        textDocument: { uri: old.uri, version },
        contentChanges: [{ text }],
      });
    try {
      await session.initialize(directory, {
        editor: true,
        lint: true,
        typecheck: true,
        typecheckTimeoutMs: 120_000,
      });
      open(warm, 1, warm.invalid);
      assert.ok(
        await waitUntil(() => warmNative.rows(1, false).every((row) => row.complete), 30_000),
      );
      open(old, 0, old.valid);
      assert.ok(
        await waitUntil(() => oldNative.rows(0, true).every((row) => row.complete), 30_000),
      );
      gate = new NativeDiagnosticsGate(session.processId, fs.realpathSync(runtime));
      await gate.stop();
      const before = nativeIpcEvidence(gate.lives);
      epochOffset = session.stderrText.length;
      change(1, old.invalid);
      assert.ok(
        await waitUntil(
          () =>
            session.stderrText
              .slice(epochOffset)
              .includes("starting initial type diagnostics for " + old.uri + " version 1"),
          10_000,
        ),
      );
      let after = before;
      assert.ok(
        await waitUntil(() => {
          gate!.assertStopped();
          after = nativeIpcEvidence(gate!.lives);
          return nativeIpcGrew(before, after);
        }, 10_000),
      );
      events.push({ stage: "old-version1-native-inflight", value: { before, after } });
      const closingOffset = newPackets.packets.length;
      session.notify("textDocument/didClose", { textDocument: { uri: old.uri } });
      assert.ok(
        await waitUntil(
          () =>
            newPackets.packets
              .slice(closingOffset)
              .some(
                ({ packet }) =>
                  packet.uri === old.uri &&
                  packet.version === undefined &&
                  packet.diagnostics.length === 0,
              ),
          10_000,
        ),
        "the real close must be accepted before version1 is reopened",
      );
      const reopenedOffset = newPackets.packets.length;
      open(next, 1, next.valid);
      const colours = await session.request(
        "textDocument/documentColor",
        { textDocument: { uri: next.uri } },
        10_000,
      );
      assert.ok(Array.isArray(colours) && colours.length === 1);
      const first: unknown = colours[0];
      assert.ok(
        first &&
          typeof first === "object" &&
          "color" in first &&
          first.color &&
          typeof first.color === "object" &&
          "red" in first.color &&
          "green" in first.color &&
          "blue" in first.color,
      );
      for (const [name, value] of [
        ["red", 171],
        ["green", 205],
        ["blue", 239],
      ] as const) {
        const actual = first.color[name];
        assert.ok(typeof actual === "number" && Math.abs(actual - value / 255) < 1e-6);
      }
      gate.assertStopped();
      events.push({
        stage: "same-version-reopen-local-buffer-confirmed",
        value: { colours, reopenedOffset },
      });
      gate.resume();
      assert.ok(
        await waitUntil(() => newNative.rows(1, true).every((row) => row.complete), 30_000),
        "the reopened version1 needs its own source-owned complete native result",
      );
      change(2, next.invalid);
      assert.ok(
        await waitUntil(() => newNative.rows(2, false).every((row) => row.complete), 30_000),
        "the new generation must diagnose its own unique invalid token",
      );
      change(3, next.valid);
      assert.ok(
        await waitUntil(
          () => newNative.rows(3, true).every((row) => row.complete) && newNative.idle(),
          30_000,
        ),
      );
      let quietSince = performance.now(),
        seenPackets = newPackets.packets.length;
      assert.ok(
        await waitUntil(() => {
          if (seenPackets !== newPackets.packets.length) {
            seenPackets = newPackets.packets.length;
            quietSince = performance.now();
          }
          return (
            newNative.rows(3, true).every((row) => row.complete) &&
            newNative.idle() &&
            performance.now() - quietSince >= 1500
          );
        }, 5000),
      );
      const stale = newPackets.packets
        .slice(reopenedOffset)
        .filter(
          ({ packet }) =>
            packet.uri === old.uri &&
            packet.diagnostics.some(
              (row) =>
                row.source === "vize/types" &&
                (packet.version === 1 || row.message?.includes(old.token)),
            ),
        );
      assert.deepEqual(stale, [], "no old-generation native error may reach the reopened URI");
      newNative.assertComplete(1, true);
      newNative.assertComplete(2, false);
      newNative.assertComplete(3, true);
      report.current = {
        clean: newNative.rows(1, true),
        mutation: newNative.rows(2, false),
        repair: newNative.rows(3, true),
      };
      report.qualified = true;
    } catch (error) {
      failure = error;
      report.error = String(error);
    } finally {
      const cleanup = await cleanupCapacitySession(session, gate);
      report.cleanup = cleanup;
      if (!cleanup.complete) {
        report.qualified = false;
        failure ??= new Error("reopen process tree did not terminate");
      }
      const capture = path.join(testOutputRoot, "diagnostic-capacity-129");
      try {
        fs.mkdirSync(capture, { recursive: true });
        fs.writeFileSync(
          path.join(capture, "reopen.json"),
          JSON.stringify(
            {
              ...report,
              packets: newPackets.packets,
              stderr: session.stderrText,
            },
            null,
            2,
          ) + "\n",
        );
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
    if (failure !== undefined) throw failure;
  },
);
