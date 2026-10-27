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
import { NativeDiagnosticsGate } from "./support/lsp/responsive-lint-native-gate.ts";
import {
  nativePipeEvidence,
  nativePipeGrew,
  cleanupCapacitySession,
} from "./support/lsp/diagnostic-capacity-pipe.ts";
import { LspSession } from "./support/lsp/session.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

test(
  "every independent document gets native errors after a blocked-worker open burst",
  {
    skip: process.platform !== "linux",
    timeout: 300_000,
  },
  async (t) => {
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "genuine native backend",
      "native runtime unavailable",
    );
    if (!runtime) return;
    const count = Number(process.env.VIZE_CAPACITY_FILES ?? "129");
    assert.ok(Number.isInteger(count) && count >= 65 && count <= 2500);
    const directory = workspace("diagnostic-capacity-");
    const capture = path.join(testOutputRoot, "diagnostic-capacity-" + count);
    fs.mkdirSync(capture, { recursive: true });
    const uri = (file: string) => pathToFileURL(path.join(directory, file)).href;
    const probes = Array.from({ length: count }, (_, index) => {
      const file = "Coverage" + String(index + 1).padStart(4, "0") + ".vue";
      return capacitySource(index + 1, uri(file));
    });
    const seed = { ...capacitySource(0, uri("Seed.vue")), file: "Seed.vue" };
    for (const probe of [seed, ...probes])
      fs.writeFileSync(path.join(directory, probe.file), probe.invalid);
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
    const sourceRoot = process.env.VIZE_CAPACITY_SOURCE_ROOT ?? root;
    const binary = process.env.VIZE_LSP_BIN ?? path.join(sourceRoot, "target/ci/vize");
    const session = new LspSession({ repoRoot: sourceRoot, binary });
    const coverage = new CapacityCoverage(probes);
    const seedCoverage = new CapacityCoverage([seed]);
    const events: { atMs: number; stage: string; value?: unknown }[] = [];
    const began = performance.now();
    let gate: NativeDiagnosticsGate | undefined;
    let failure: unknown;
    const report: Record<string, unknown> = {
      scope:
        "Linux real native backend paused to expose scheduling; correctness only, no cold latency claim",
      eventSource: process.env.GITHUB_SHA,
      producer: JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8")) as unknown,
      count,
      backend: runtime,
      input: [seed, ...probes],
      events,
      qualified: false,
    };
    const save = () =>
      fs.writeFileSync(
        path.join(capture, "result.json"),
        JSON.stringify(
          {
            ...report,
            packets: coverage.packets,
            seedPackets: seedCoverage.packets,
            invalid: coverage.rows(1, false),
            repaired: coverage.rows(2, true),
            recovery: coverage.rows(3, true),
            stderr: session.stderrText,
          },
          null,
          2,
        ) + "\n",
      );
    const note = (stage: string, value?: unknown) => {
      events.push({ atMs: performance.now() - began, stage, value });
      save();
    };
    session.notificationObservers.push((method, value) => {
      if (method === "textDocument/publishDiagnostics" && capacityPacket(value)) {
        coverage.record(value, performance.now() - began);
        seedCoverage.record(value, performance.now() - began);
      }
    });
    const open = (probe: typeof seed, version: number, text: string) =>
      session.notify("textDocument/didOpen", {
        textDocument: { uri: probe.uri, languageId: "vue", version, text },
      });
    const change = (probe: typeof seed, version: number, text: string) =>
      session.notify("textDocument/didChange", {
        textDocument: { uri: probe.uri, version },
        contentChanges: [{ text }],
      });
    const traceFinished = (): boolean => {
      const lines = session.stderrText.split("\n");
      const started = lines.filter((line) =>
        line.includes("starting initial type diagnostics for "),
      ).length;
      const finished = lines.filter((line) =>
        line.includes("finished initial type diagnostics for "),
      ).length;
      return started > 0 && started === finished;
    };
    try {
      await session.initialize(directory, {
        editor: true,
        lint: true,
        typecheck: true,
        typecheckTimeoutMs: 120_000,
      });
      open(seed, 1, seed.invalid);
      assert.ok(
        await waitUntil(() => seedCoverage.rows(1, false).every((row) => row.complete), 60_000),
        "the actual native seed must finish before signalling its exact process",
      );
      note("seed-native-complete");
      gate = new NativeDiagnosticsGate(session.processId, fs.realpathSync(runtime));
      await gate.stop();
      const before = nativePipeEvidence(gate.lives);
      note("native-stopped-before-blocker", {
        lives: gate.lives,
        signals: gate.signals,
        pipes: before,
      });
      const afterTrace = session.stderrText.length;
      change(seed, 2, seed.invalid + "\n");
      assert.ok(
        await waitUntil(
          () =>
            session.stderrText
              .slice(afterTrace)
              .includes("starting initial type diagnostics for " + seed.uri + " version 2"),
          10_000,
        ),
        "the edit pass must start while the real native process is stopped",
      );
      let after = before;
      assert.ok(
        await waitUntil(() => {
          gate!.assertStopped();
          after = nativePipeEvidence(gate!.lives);
          return nativePipeGrew(before, after);
        }, 10_000),
        "the stopped native stdin must gain real request bytes before the open burst",
      );
      note("seed-pass-inflight", { pipesBefore: before, pipesAfter: after });
      const accepted: { uri: string; atMs: number; syncPackets: number }[] = [];
      const startsBefore = session.stderrText
        .split("\n")
        .filter((line) => line.includes("starting initial type diagnostics for ")).length;
      for (const probe of probes) {
        open(probe, 1, probe.invalid);
        assert.ok(
          await waitUntil(
            () =>
              coverage.packets.some(
                ({ packet }) =>
                  packet.uri === probe.uri &&
                  packet.version === undefined &&
                  packet.diagnostics.some(
                    (row) => row.source === "vize/lint" && row.code === "vue/require-v-for-key",
                  ),
              ),
            10_000,
          ),
          "each URI must enter the real scheduler and finish its sync feedback while native is blocked",
        );
        gate.assertStopped();
        accepted.push({
          uri: probe.uri,
          atMs: performance.now() - began,
          syncPackets: coverage.packets.filter(
            ({ packet }) => packet.uri === probe.uri && packet.version === undefined,
          ).length,
        });
      }
      const startsAfter = session.stderrText
        .split("\n")
        .filter((line) => line.includes("starting initial type diagnostics for ")).length;
      assert.equal(
        startsAfter,
        startsBefore,
        "no burst URI starts a native pass behind the held seed",
      );
      assert.ok(
        !session.stderrText
          .slice(afterTrace)
          .includes("finished initial type diagnostics for " + seed.uri + " version 2"),
      );
      note("all-open-sync-jobs-accepted-native-still-blocked", {
        accepted,
        startsBefore,
        startsAfter,
        pipes: nativePipeEvidence(gate.lives),
      });
      const last = probes[probes.length - 1];
      assert.ok(last);
      const colours = await session.request(
        "textDocument/documentColor",
        { textDocument: { uri: last.uri } },
        30_000,
      );
      assert.ok(
        Array.isArray(colours) && colours.length === 1,
        "the final buffer must be visible to the local foreground service",
      );
      gate.assertStopped();
      note("bulk-open-local-barrier", {
        colours,
        distinction: "local visibility is not a native-job-start receipt",
      });
      gate.resume();
      note("native-resumed", { signals: gate.signals });
      const allInvalid = await waitUntil(
        () => coverage.rows(1, false).every((row) => row.complete),
        60_000,
      );
      note("initial-native-coverage", { complete: allInvalid, rows: coverage.rows(1, false) });
      // Repair every buffer. A silent old queue cannot be mistaken for a clean result.
      for (const probe of probes) {
        fs.writeFileSync(path.join(directory, probe.file), probe.valid);
        change(probe, 2, probe.valid);
      }
      const repaired = await waitUntil(
        () => coverage.rows(2, true).every((row) => row.complete),
        60_000,
      );
      note("bulk-repair-coverage", { complete: repaired, rows: coverage.rows(2, true) });
      // Sequential recovery is an independent health control, never bulk success.
      for (const probe of probes) {
        change(probe, 3, probe.valid);
        assert.ok(
          await waitUntil(
            () => coverage.rows(3, true).find((row) => row.uri === probe.uri)?.complete === true,
            10_000,
          ),
          "each repaired document must receive a terminal native result",
        );
      }
      change(seed, 3, seed.valid);
      assert.ok(
        await waitUntil(() => seedCoverage.rows(3, true).every((row) => row.complete), 30_000),
      );
      let quietAfter = performance.now(),
        lastPackets = coverage.packets.length;
      const idle = await waitUntil(() => {
        if (coverage.packets.length !== lastPackets) {
          lastPackets = coverage.packets.length;
          quietAfter = performance.now();
        }
        return (
          coverage.rows(3, true).every((row) => row.complete) &&
          seedCoverage.rows(3, true).every((row) => row.complete) &&
          traceFinished() &&
          performance.now() - quietAfter >= 1_500
        );
      }, 30_000);
      note("checked-repair-and-idle", { idle, complete: coverage.rows(3, true) });
      assert.ok(
        idle,
        "idle requires complete repaired coverage and no in-flight traced native pass",
      );
      report.reverseDependentCounts = session.stderrText
        .split("\n")
        .filter(
          (line) =>
            line.includes("refreshing ") && line.includes(" open typecheck dependents for "),
        );
      report.qualified = allInvalid && repaired && idle;
      save();
      console.log(
        JSON.stringify({
          scope: report.scope,
          count,
          qualified: report.qualified,
          invalid: coverage.rows(1, false),
          repaired: coverage.rows(2, true),
          recovery: coverage.rows(3, true),
          reverseDependentCounts: report.reverseDependentCounts,
        }),
      );
      coverage.assertComplete(1, false);
      coverage.assertComplete(2, true);
    } catch (error) {
      failure = error;
      report.error = String(error);
    } finally {
      const cleanup = await cleanupCapacitySession(session, gate);
      report.cleanup = cleanup;
      if (!cleanup.complete) failure ??= new Error("owned process tree did not terminate");
      try {
        save();
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
    if (failure !== undefined) throw failure;
  },
);
