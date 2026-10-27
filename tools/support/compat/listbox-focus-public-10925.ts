import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const [consumer, fixtures, evidenceFile] = process.argv.slice(2);
assert.ok(consumer && fixtures && evidenceFile);
const consumerRoot = fs.realpathSync(consumer);
const pluginManifest = fs.realpathSync(path.join(consumer, "node_modules/@vizejs/vite-plugin/package.json"));
const require = createRequire(pluginManifest);
let vizeRoot = path.dirname(require.resolve("vize"));
while (!fs.existsSync(path.join(vizeRoot, "package.json"))) vizeRoot = path.dirname(vizeRoot);
const vizeManifest = fs.realpathSync(path.join(vizeRoot, "package.json"));
assert.ok(vizeManifest.startsWith(consumerRoot + path.sep));
assert.ok(pluginManifest.startsWith(consumerRoot + path.sep));
const pluginPackage = JSON.parse(fs.readFileSync(pluginManifest, "utf8"));
const vizePackage = JSON.parse(fs.readFileSync(vizeManifest, "utf8"));
assert.equal(vizePackage.name, "vize");
assert.equal(pluginPackage.version, vizePackage.version);
const cli = path.join(vizeRoot, "bin/vize");
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const read = (name: string) => JSON.parse(fs.readFileSync(path.join(fixtures, name), "utf8"));
const corpusBytes = fs.readFileSync(path.join(fixtures, "cases.json"));
const corpus = JSON.parse(corpusBytes.toString());
assert.equal(corpus.cases.length, 37);
const source = read("original-source.json");
const before = read("original-before.json");
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-listbox-public-10925-"));
const observations: unknown[] = [];
const evidence: Record<string, unknown> = {
  issue: 10925, pr: 10971, consumer: consumerRoot, node: process.version,
  packages: { plugin: { version: pluginPackage.version, manifest: pluginManifest }, vize: { version: vizePackage.version, manifest: vizeManifest } },
  corpusSha256: hash(corpusBytes), startedAt: new Date().toISOString(), observations,
};
const persist = () => fs.writeFileSync(evidenceFile, JSON.stringify(evidence, null, 2) + "\n");
function run(id: string, route: string, filename: string, bytes: Buffer, argv: string[], expected: unknown, status: number) {
  const file = path.join(temporary, filename);
  fs.writeFileSync(file, bytes);
  for (let repetition = 0; repetition < 2; repetition++) {
    const child = spawnSync(process.execPath, [cli, ...argv], {
      cwd: temporary, encoding: "utf8", timeout: 30_000, maxBuffer: 2 * 1024 * 1024,
      env: { ...process.env, NO_COLOR: "1", PATH: path.dirname(process.execPath) + path.delimiter + (process.env.PATH ?? "") },
    });
    let actual: unknown;
    try { actual = JSON.parse(child.stdout); } catch { actual = { invalidJson: child.stdout }; }
    const row = { id, route, repetition, sourceSha256: hash(bytes), argv, expected, expectedStatus: status, status: child.status, signal: child.signal, error: child.error?.message ?? null, stdout: child.stdout, stderr: child.stderr, actual };
    observations.push(row);
    persist();
    assert.equal(child.error, undefined, `${id}/${route}`);
    assert.equal(child.signal, null, `${id}/${route}`);
    assert.equal(child.status, status, `${id}/${route}: ${child.stdout}${child.stderr}`);
    assert.equal(child.stderr, "", `${id}/${route}`);
    assert.deepEqual(actual, expected, `${id}/${route}`);
    assert.deepEqual(fs.readFileSync(file), bytes);
  }
}
try {
  const bytes = fs.readFileSync(path.join(fixtures, source.sourceFile));
  assert.equal(hash(bytes), source.sourceSha256);
  run("original-issue", "opinionated", "CompositeWidgets.vue", bytes, before.argv, [{ ...before.packet[0], messages: [], warningCount: 0 }], 0);
  fs.writeFileSync(path.join(temporary, "vite.config.mjs"), 'export default { vue: { linter: { preset: "incremental", rules: { "a11y/interactive-supports-focus": "warn" } } } };\n');
  for (const entry of corpus.cases) {
    for (const name of ["vue", "html", "jsx", "tsx"]) {
      const route = entry.routes[name];
      const bytes = fs.readFileSync(path.join(fixtures, route.sourceFile));
      assert.equal(hash(bytes), route.sourceSha256);
      run(entry.id, name, route.filename, bytes, ["lint", "--config", "vite.config.mjs", route.filename, "--format", "json", "--locale", "en", "--quiet"], route.afterPacket, route.afterStatus);
    }
  }
  assert.equal(observations.length, 298);
  evidence.status = "passed";
} catch (error) {
  evidence.status = "failed";
  evidence.error = error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
  process.exitCode = 1;
} finally {
  evidence.finishedAt = new Date().toISOString();
  persist();
  fs.rmSync(temporary, { recursive: true, force: true });
  console.log(JSON.stringify({ status: evidence.status, version: vizePackage.version, observations: observations.length, evidenceFile }));
}
