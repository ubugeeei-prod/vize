import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const [directory, fixtures, output] = process.argv.slice(2);
assert.ok(directory && fixtures && output);
const root = fs.realpathSync(directory);
const rootRequire = createRequire(path.join(root, "package.json"));
const pluginManifest = fs.realpathSync(
  path.join(root, "node_modules/@vizejs/vite-plugin/package.json"),
);
assert.ok(pluginManifest.startsWith(root + path.sep));
const pluginRequire = createRequire(pluginManifest);
let vizeRoot = path.dirname(pluginRequire.resolve("vize"));
while (!fs.existsSync(path.join(vizeRoot, "package.json"))) vizeRoot = path.dirname(vizeRoot);
assert.ok(fs.realpathSync(vizeRoot).startsWith(root + path.sep));
const vizePackage = JSON.parse(fs.readFileSync(path.join(vizeRoot, "package.json"), "utf8"));
const pluginPackage = JSON.parse(fs.readFileSync(pluginManifest, "utf8"));
assert.equal(vizePackage.version, "0.602.0");
assert.equal(pluginPackage.version, "0.602.0");
const cli = path.join(vizeRoot, "bin/vize");
const vp = path.join(path.dirname(rootRequire.resolve("vite-plus/package.json")), "bin/vp");
const sha256 = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
const manifestBytes = fs.readFileSync(path.join(fixtures, "cases.json"));
const corpus = JSON.parse(manifestBytes.toString());
const observations: any[] = [];
const evidence: any = {
  schema: "vize.cross-file-directives-public-before",
  issue: 10912,
  version: "0.602.0",
  acceptanceHead: process.env.GITHUB_SHA,
  preparedTag: "92223735e9c41cefbfe584d7f364e4847569c92b",
  producerSource: "3fa403f8cf4edbd851f7ca91c0f0648200f5777f",
  startedAt: new Date().toISOString(),
  packages: { plugin: pluginPackage, vize: vizePackage },
  consumer: root,
  corpusSha256: sha256(manifestBytes),
  original: corpus.originalSources,
  observations,
};
const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
function invoke(entry: any, route: string, binary: string, args: string[], repetition: number) {
  const config = fs.readFileSync(path.join(root, "vite.config.ts"), "utf8");
  const before = Object.fromEntries(
    Object.keys(entry.files).map((file) => [file, sha256(fs.readFileSync(path.join(root, file)))]),
  );
  const result = spawnSync(process.execPath, [binary, ...args], {
    cwd: root,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 8 * 1024 * 1024,
    env: {
      ...process.env,
      NO_COLOR: "1",
      FORCE_COLOR: "0",
      PATH: path.dirname(process.execPath) + path.delimiter + (process.env.PATH ?? ""),
    },
  });
  let packet: unknown = null;
  if (route === "cli-json") {
    try {
      packet = JSON.parse(result.stdout);
    } catch {
      /* preserve invalid JSON and fail below */
    }
  }
  const row = {
    id: entry.id,
    route,
    repetition,
    binary,
    args,
    sourceHashes: before,
    sourceFiles: entry.files,
    config,
    configSha256: sha256(config),
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout,
    stderr: result.stderr,
    packet,
  };
  observations.push(row);
  persist();
  assert.equal(result.error, undefined, entry.id + "/" + route);
  assert.equal(result.signal, null, entry.id + "/" + route);
  assert.ok(
    result.status === 0 || result.status === 1,
    entry.id + "/" + route + ": " + result.stdout + result.stderr,
  );
  if (route === "cli-json") assert.ok(Array.isArray(packet), entry.id + ": invalid JSON");
  for (const [file, bytes] of Object.entries(entry.files))
    assert.equal(fs.readFileSync(path.join(root, file), "utf8"), bytes);
  assert.equal(fs.readFileSync(path.join(root, "vite.config.ts"), "utf8"), config);
  if (entry.original) {
    assert.equal(result.status, 1);
    const full = result.stdout + result.stderr;
    assert.ok(
      full.includes("croquis/cf/spread-breaks-reactivity"),
      entry.id + "/" + route + ": missing original spread",
    );
    assert.ok(
      full.includes("croquis/cf/duplicate-id"),
      entry.id + "/" + route + ": missing original ID",
    );
  }
}
try {
  assert.equal(
    sha256(fs.readFileSync(path.join(fixtures, "original-issue.md"))),
    corpus.bodySha256,
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(fixtures, "original-reported.txt"))),
    corpus.reportedSha256,
  );
  for (const original of corpus.originalSources) {
    const bytes = fs.readFileSync(path.join(fixtures, "original", original.path + ".txt"));
    assert.equal(bytes.length, original.bytes);
    assert.equal(sha256(bytes), original.sha256);
    assert.equal(corpus.cases[0].files[original.path], bytes.toString());
  }
  for (const entry of corpus.cases) {
    fs.rmSync(path.join(root, "src"), { recursive: true, force: true });
    for (const [file, bytes] of Object.entries(entry.files)) {
      fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
      fs.writeFileSync(path.join(root, file), bytes as string);
    }
    const linter = {
      crossFile: entry.crossFile,
      reportUnusedDisableDirectives: entry.reportUnused,
    };
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nexport default defineConfig(' +
        JSON.stringify({ lint: { vue: linter } }) +
        ", { plugin: false, conflicts: false });\n",
    );
    const options = [
      ...(entry.crossFile ? ["--cross-file"] : []),
      ...(entry.reportUnused ? ["--report-unused-disable-directives"] : []),
    ];
    for (let repetition = 0; repetition < 2; repetition++) {
      invoke(
        entry,
        "cli-plain",
        cli,
        ["lint", "-f", "plain", "--no-config", ...options, "src/**/*.vue"],
        repetition,
      );
      invoke(
        entry,
        "cli-json",
        cli,
        ["lint", "-f", "json", "--no-config", ...options, "src/**/*.vue"],
        repetition,
      );
      invoke(entry, "installed-vp-task", vp, ["run", "lint", "src/**/*.vue"], repetition);
    }
  }
  evidence.status = "captured";
  evidence.caseCount = corpus.cases.length;
  evidence.packetCount = observations.length;
  evidence.originalIssueReproduced = true;
  assert.equal(observations.length, corpus.cases.length * 6);
} catch (error: any) {
  evidence.status = "failed";
  evidence.error = { message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  evidence.finishedAt = new Date().toISOString();
  persist();
  console.log(
    JSON.stringify({
      status: evidence.status,
      caseCount: corpus.cases.length,
      packets: observations.length,
      output,
    }),
  );
}
