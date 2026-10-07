import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import {
  effectiveRules,
  editorPrefix,
  manifest,
  replayConfig,
  repository,
  sha256,
  verifyCorpus,
} from "./n8n-replay-inputs.mjs";

export function qualifyN8nHost(engine, version, output, environment) {
  const inventory = verifyCorpus();
  const original = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-original-project-"));
  const plugin = path.join(repository, "npm/oxlint/dist/index.mjs");
  const wrapper = path.join(repository, "npm/oxlint/dist/cli.mjs");
  const capture = [];
  const capturePath = path.join(output, "host-results.json");
  const configPath = path.join(original, ".oxlintrc.mts");
  fs.mkdirSync(output, { recursive: true });
  const save = () => fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
  try {
    fs.mkdirSync(path.join(original, ".git"));
    fs.writeFileSync(path.join(original, ".gitignore"), "");
    fs.mkdirSync(path.join(original, "node_modules/oxlint/bin"), { recursive: true });
    fs.symlinkSync(engine, path.join(original, "node_modules/oxlint/bin/oxlint"));
    for (const { file, sha256: expected } of inventory.files) {
      const target = path.join(original, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(inventory.fixture, file), target, fs.constants.COPYFILE_EXCL);
      assert.equal(sha256(fs.readFileSync(target)), expected);
    }
    const reports = {};
    const modes = ["stock", "baseline", "shared", "effective"];
    for (const mode of modes) {
      const config = replayConfig(plugin, mode);
      // This is the functional frozen native/Vize object projection, not a
      // redistributed adoption-branch source or a JSONC approximation.
      const bytes = `export default ${JSON.stringify(config, null, 2)};\n`;
      fs.writeFileSync(configPath, bytes);
      fs.writeFileSync(path.join(output, `${mode}.mts`), bytes);
      const args = [
        mode === "stock" ? engine : wrapper,
        "-c",
        configPath,
        "-f",
        "json",
        "--threads",
        "1",
        "packages",
      ];
      const start = performance.now();
      const result = spawnSync(process.execPath, args, {
        cwd: original,
        encoding: "utf8",
        timeout: 180_000,
        maxBuffer: 64 * 1024 * 1024,
        env: { ...process.env, ...environment, VIZE_N8N_REPLAY_PHASE: `host:${version}:${mode}` },
      });
      capture.push({
        mode,
        engine,
        version,
        cwd: original,
        args,
        configBytes: bytes,
        configSha256: sha256(bytes),
        elapsedMs: performance.now() - start,
        ...result,
      });
      save(); // Preserve complete output/status/errors before any assertion.
      assert.equal(result.error, undefined, result.stderr);
      assert.equal(result.signal, null);
      assert.ok([0, 1].includes(result.status), result.stderr);
      assert.equal(result.stderr, "");
      assert.equal(fs.readFileSync(configPath, "utf8"), bytes);
      if (mode === "effective") {
        // Neither pinned host accepts settings inside an override. Preserve
        // this actual rejected packet; the native effective-map API remains
        // independently qualified, but cannot grant host batching credit.
        assert.equal(result.status, 1);
        assert.match(result.stdout, /unknown field `settings`/u);
        const originalStart = performance.now();
        const originalResult = spawnSync(process.execPath, [engine, ...args.slice(1)], {
          cwd: original,
          encoding: "utf8",
          timeout: 180_000,
          maxBuffer: 64 * 1024 * 1024,
          env: {
            ...process.env,
            ...environment,
            VIZE_N8N_REPLAY_PHASE: `host:${version}:effective-original`,
          },
        });
        capture.push({
          mode: "effective-original",
          engine,
          version,
          cwd: original,
          args: [engine, ...args.slice(1)],
          configBytes: bytes,
          configSha256: sha256(bytes),
          elapsedMs: performance.now() - originalStart,
          ...originalResult,
        });
        save();
        assert.equal(originalResult.error, undefined);
        assert.equal(originalResult.signal, null);
        assert.equal(originalResult.status, result.status);
        assert.equal(originalResult.stdout, result.stdout);
        assert.equal(originalResult.stderr, result.stderr);
        continue;
      }
      const report = (reports[mode] = JSON.parse(result.stdout));
      assert.ok(Array.isArray(report.diagnostics));
      assert.equal(report.number_of_files, inventory.files.length);
      if (mode !== "stock") {
        const foreign = report.diagnostics.filter((packet) => !packet.code?.startsWith("vize("));
        assert.deepEqual(
          foreign,
          reports.stock.diagnostics,
          `${version}/${mode}: complete original foreign packets/order/multiplicity`,
        );
        for (const packet of report.diagnostics.filter((packet) =>
          packet.code?.startsWith("vize("),
        )) {
          const rules = effectiveRules(packet.filename);
          const name = `vize/${packet.code.slice(5, -1)}`;
          const entry = rules[name];
          assert.ok(entry, `foreign selected rule ${name}`);
          const severity = Array.isArray(entry) ? entry[0] : entry;
          assert.ok(["error", "warn"].includes(severity));
          assert.equal(packet.severity, severity === "warn" ? "warning" : "error");
          assert.ok(inventory.files.some(({ file }) => file === packet.filename));
          assert.ok(packet.labels.length > 0);
        }
      }
    }
    for (const mode of ["shared"]) {
      assert.deepEqual(
        reports[mode].diagnostics,
        reports.baseline.diagnostics,
        `${version}/${mode}: whole ordered host vectors`,
      );
      // Keep timing fields in the raw reports; compare every other field.
      const { start_time: baselineTime, ...baseline } = reports.baseline;
      const { start_time: candidateTime, ...candidate } = reports[mode];
      assert.equal(typeof baselineTime, "number");
      assert.equal(typeof candidateTime, "number");
      assert.deepEqual(candidate, baseline);
      assert.equal(
        capture.find((row) => row.mode === mode).status,
        capture.find((row) => row.mode === "baseline").status,
      );
    }
    const nativeVectors = fs
      .readFileSync(path.join(path.dirname(output), "bridge/bridge-vectors.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line))
      .filter(({ mode }) => mode === "scopedPerRule");
    assert.equal(nativeVectors.length, inventory.files.length);
    // Bind every emitted native finding to the corresponding complete SDK
    // packet and exact original UTF-8 span; no count-only coverage claim.
    for (const row of nativeVectors) {
      for (const requested of row.cold) {
        const packets = reports.baseline.diagnostics.filter(
          (packet) => packet.filename === row.file && packet.code === `vize(${requested.rule})`,
        );
        assert.equal(packets.length, requested.diagnostics.length, `${row.file}/${requested.rule}`);
        for (const [index, diagnostic] of requested.diagnostics.entries()) {
          assert.equal(packets[index].message, diagnostic.message);
          const start = diagnostic.location.start.offset,
            end = diagnostic.location.end.offset;
          assert.deepEqual(
            packets[index].labels,
            [
              {
                span: {
                  offset: start,
                  length: end - start,
                  line: diagnostic.location.start.line,
                  column: diagnostic.location.start.column,
                },
              },
            ],
            `${row.file}/${requested.rule}: complete original span`,
          );
        }
      }
    }
    for (const { file, sha256: expected } of inventory.files)
      assert.equal(sha256(fs.readFileSync(path.join(original, file))), expected);
    assert.deepEqual(verifyCorpus(), inventory);
    const summary = {
      version,
      files: inventory.files.length,
      scriptless: 19,
      disabledFiles: inventory.disabledFiles,
      modes: capture.map(({ mode, elapsedMs, status }) => ({ mode, elapsedMs, status })),
      vectorSha256: sha256(JSON.stringify(reports.baseline.diagnostics)),
      limits: [
        "native/Vize layer and six scopes on owned exact master copies",
        "two n8n-local plugins and unrelated frontend/workspace layers excluded",
        "wrapper qualification does not repair direct SDK scriptless callbacks",
        "no-hint baseline retains per-rule calls; optional hint modes are separate",
        "per-file effective host batching is unmet: override.settings is rejected; native effective maps are separate",
      ],
    };
    fs.writeFileSync(
      path.join(output, "host-summary.json"),
      JSON.stringify(summary, null, 2) + "\n",
    );
    return { ...summary, original };
  } finally {
    save();
    fs.rmSync(original, { recursive: true, force: true });
  }
}

export function verifyNativeCalls(custody, expectedPhases) {
  const loaded = new Set();
  const counts = new Map();
  const calls = new Map();
  const inventory = verifyCorpus();
  const filesByName = new Map(inventory.files.map((entry) => [entry.file, entry]));
  const events = fs
    .readFileSync(custody.calls, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  for (const event of events) {
    assert.deepEqual(event.source, custody.source);
    assert.equal(event.binary, custody.binary.path);
    assert.equal(event.binarySha256, custody.binary.sha256);
    assert.ok(expectedPhases.includes(event.phase), `unexpected native phase ${event.phase}`);
    if (event.kind === "load") {
      loaded.add(event.pid);
      continue;
    }
    assert.equal(event.kind, "call");
    assert.ok(loaded.has(event.pid));
    assert.equal(event.outcome, "return");
    const source = fs.readFileSync(event.args[0].sourcePath);
    assert.equal(
      event.args[0].sourcePath,
      path.join(custody.sources, event.args[0].sourceSha256 + ".vue"),
    );
    assert.equal(sha256(source), event.args[0].sourceSha256);
    assert.ok(event.result && Array.isArray(event.result.diagnostics));
    const phase = counts.get(event.phase) ?? new Map();
    counts.set(event.phase, phase);
    const filename = event.args[1].filename;
    phase.set(filename, (phase.get(filename) ?? 0) + 1);
    const phaseCalls = calls.get(event.phase) ?? new Map();
    calls.set(event.phase, phaseCalls);
    const fileCalls = phaseCalls.get(filename) ?? [];
    phaseCalls.set(filename, fileCalls);
    fileCalls.push(event);
    const relative = filename.slice(filename.indexOf("/packages/") + 1);
    if (event.phase !== "cacheControls")
      assert.equal(event.args[0].sourceSha256, filesByName.get(relative)?.sha256, filename);
  }
  return { counts, calls, events: events.length, processes: loaded.size, inventory };
}

export function expectedHostCalls(file, mode) {
  if (["stock", "effective", "effective-original"].includes(mode)) return 0;
  if (mode === "baseline")
    return (
      51 -
      Number(
        manifest.adoption.packageOverrides["editor-ui"][
          "vize/vue/no-multiple-template-root"
        ].files.some((target) => file === editorPrefix + target) ||
          manifest.adoption.packageOverrides["design-system"][
            "vize/vue/require-v-for-key"
          ].files.some((target) => file === "packages/frontend/@n8n/design-system/" + target),
      )
    );
  return mode === "shared" && file.startsWith(editorPrefix) ? 2 : 1;
}
