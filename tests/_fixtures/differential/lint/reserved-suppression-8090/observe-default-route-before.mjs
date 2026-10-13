import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256, wholeJson } from "../../../../tooling/lint-default-correctness-reference.mjs";

const fixture = path.dirname(fileURLToPath(import.meta.url));
const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json")));
const successors = JSON.parse(fs.readFileSync(path.join(fixture, "default-route-successors.json")));
const projection = JSON.parse(fs.readFileSync(path.join(fixture, "../../../n8n-cli-adoption.json")));
const binary = process.argv[2];
assert.equal(sha256(fs.readFileSync(binary)), corpus.beforeBaseline.binarySha256);
assert.equal(spawnSync(binary, ["--version"], { encoding: "utf8" }).stdout, "vize 0.443.0\n");
const receipt = {
  schema: "vize.lint.reserved-suppression-default-route-public-before", version: 1,
  baseline: corpus.beforeBaseline,
  custody: ["cases.json", "default-route-successors.json", "observe-default-route-before.mjs"].map(name => ({ path: name, sha256: sha256(fs.readFileSync(path.join(fixture, name))) })),
  runs: [],
};
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-suppression-default-before-"));
try {
  for (const [index, entry] of corpus.cases.entries()) {
    const successor = successors.cases[index];
    assert.equal(successor.id, entry.id);
    assert.equal(successor.sourceSha256, entry.sourceSha256);
    const bytes = fs.readFileSync(path.join(fixture, entry.filename + ".fixture"));
    assert.equal(bytes.toString(), entry.source);
    assert.equal(sha256(bytes), entry.sourceSha256);
    const scenarios = [
      ...[null, "recommended", "essential", "ecosystem"].map(preset => ({ route: preset ?? "default", preset, config: null, findings: preset === "essential" ? successor.beforeEssentialDiagnostics : successor.beforeDefaultDiagnostics })),
      { route: "incremental", preset: "incremental", config: null, findings: successor.incrementalDiagnostics },
      { route: "explicit-off", preset: null, config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n", findings: successor.offDiagnostics },
      { route: "n8n-projection", preset: null, config: JSON.stringify({ linter: projection.linter }) + "\n", findings: successor.projectionDiagnostics },
    ];
    for (const { route, preset, config, findings } of scenarios) {
      fs.writeFileSync(path.join(directory, entry.filename), bytes);
      if (config) fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      const argv = ["lint", ...(config ? ["-c", "vize.config.json"] : ["--no-config"]), ...(preset ? ["--preset", preset] : []), "--format", "json", "--locale", "en", "--help-level", "none", entry.filename];
      const result = spawnSync(binary, argv, { cwd: directory, encoding: "utf8", timeout: 60000, maxBuffer: 4 * 1024 * 1024 });
      const run = {
        id: entry.id, route, argv, config, input: { text: entry.source, sha256: entry.sourceSha256 },
        status: result.status, error: result.error ? { name: result.error.name, message: result.error.message } : null,
        signal: result.signal, stdout: result.stdout ?? "", stderr: result.stderr ?? "",
        expected: wholeJson(entry, findings), expectedStatus: findings.some(f => f.severity === "error") ? 1 : 0,
        actual: JSON.parse(result.stdout ?? "null"),
      };
      receipt.runs.push(run);
      fs.writeFileSync(path.join(fixture, "default-route-public-before.json"), JSON.stringify(receipt, null, 2) + "\n");
      assert.equal(run.error, null); assert.equal(run.signal, null); assert.equal(run.stderr, "");
      assert.equal(run.status, run.expectedStatus, entry.id + "/" + route);
      assert.deepEqual(run.actual, run.expected, entry.id + "/" + route);
    }
  }
  assert.equal(receipt.runs.length, 168);
  console.log("168/168 full public-before packets match authored complete route vectors");
} finally { fs.rmSync(directory, { recursive: true, force: true }); }
