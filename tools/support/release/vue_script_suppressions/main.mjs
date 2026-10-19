import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { extraOverrides } from "../lint_reports/custody.mjs";
import { installPackageGraph } from "../lint_reports/package_graph.mjs";
import { overrides, sha256 } from "../typed_defaults/custody.mjs";
import { fixtureSha256, loadFixtures } from "./fixtures.mjs";
import { errorRecord, observeAll } from "./observe.mjs";

export function rejectOverrides(env) {
  for (const key of [
    ...overrides,
    ...extraOverrides,
    "NODE_PATH",
    "VIZE_BIN",
    "VIZE_BINARY",
    "VIZE_LSP_BIN",
    "VIZE_LSP_REQUIRE_SOURCE_BUILD",
  ]) {
    assert(!env[key], `runtime override must be absent: ${key}`);
  }
}

export function prepareOutput(value) {
  const output = path.resolve(value);
  const checkout = fs.realpathSync(fileURLToPath(new URL("../../../..", import.meta.url)));
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const canonical = path.join(fs.realpathSync(path.dirname(output)), path.basename(output));
  assert(
    canonical !== checkout && !canonical.startsWith(checkout + path.sep),
    "public consumer and evidence must be outside checkout",
  );
  // Exclusive mkdir refuses existing evidence, including an empty directory or symlink.
  fs.mkdirSync(canonical);
  return fs.realpathSync(canonical);
}

const write = (output, name, value) =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

function fileIdentity(filename) {
  assert(fs.lstatSync(filename).isFile() && !fs.lstatSync(filename).isSymbolicLink());
  return { filename, bytes: fs.statSync(filename).size, sha256: sha256(fs.readFileSync(filename)) };
}

function installedIdentity(consumer, version, records) {
  const require = createRequire(path.join(consumer, "package.json"));
  const native = require("@vizejs/native");
  const loaded = Object.values(require.cache).filter(
    (entry) => entry.filename.endsWith(".node") && entry.exports === native,
  );
  assert.equal(loaded.length, 1, "one actually loaded public native binding required");
  const binding = fileIdentity(loaded[0].filename);
  const record = records.find(
    ({ directory, members }) =>
      binding.filename.startsWith(directory + path.sep) &&
      members.some(
        (member) =>
          path.join(directory, member.path) === binding.filename &&
          member.sha256 === binding.sha256,
      ),
  );
  assert(record && record.metadata.name.startsWith("@vizejs/native-"));
  assert.equal(record.metadata.version, version);
  const packageRecord = (name) => {
    const row = records.find(({ location }) => location === "node_modules/" + name);
    assert.equal(row?.metadata.version, version);
    return row;
  };
  const cli = packageRecord("vize"),
    umbrella = packageRecord("@vizejs/native");
  const launcher = path.join(cli.directory, "bin/vize");
  return {
    version,
    node: { version: process.version, executable: process.execPath },
    launcher,
    files: [
      fileIdentity(launcher),
      fileIdentity(path.join(cli.directory, "dist/cli.mjs")),
      fileIdentity(require.resolve("@vizejs/native")),
      binding,
    ],
    packages: [cli, umbrella, record].map(({ location, metadata, archiveSha256 }) => ({
      location,
      name: metadata.name,
      version: metadata.version,
      archiveSha256,
      tarball: metadata.dist.tarball,
      integrity: metadata.dist.integrity,
    })),
  };
}

export async function run(version, target) {
  assert.match(version ?? "", /^0\.[1-9][0-9]*\.0$/, "exact published pre-1.0 minor required");
  rejectOverrides(process.env);
  const output = prepareOutput(target);
  const receipt = {
    version,
    output,
    fixtureSha256,
    startedAt: new Date().toISOString(),
    qualification:
      "Published npm CLI/native; only the four frozen script suppression regression contracts",
  };
  write(output, "started.json", receipt);
  try {
    const { fixture, bytes } = loadFixtures();
    fs.writeFileSync(path.join(output, "fixtures.json"), bytes, { flag: "wx" });
    const helperNames = [
      "main.mjs",
      "fixtures.mjs",
      "observe.mjs",
      "../lint_reports/package_graph.mjs",
      "../lint_reports/package_members.ts",
      "../../archive/tar.ts",
      "../typed_defaults/custody.mjs",
      "../typed_defaults/packet.mjs",
    ];
    receipt.helpers = helperNames.map((name) =>
      fileIdentity(fileURLToPath(new URL(name, import.meta.url))),
    );
    const consumer = path.join(output, "consumer");
    fs.mkdirSync(consumer);
    const graph = await installPackageGraph(
      consumer,
      output,
      { vize: version, "@vizejs/native": version },
      (name) => name === "vize" || name.startsWith("@vizejs/native"),
    );
    graph.assertDependencies();
    const identity = installedIdentity(consumer, version, graph.records);
    receipt.installed = identity;
    write(output, "installed.json", identity);
    const assertInstalled = () => {
      graph.assertDependencies();
      for (const original of identity.files)
        assert.deepEqual(fileIdentity(original.filename), original);
    };
    const probe = spawnSync(process.execPath, [identity.launcher, "--version"], {
      cwd: consumer,
      timeout: 60_000,
      maxBuffer: 1024 * 1024,
    });
    write(output, "version-process.json", {
      executable: process.execPath,
      argv: [identity.launcher, "--version"],
      cwd: consumer,
      status: probe.status,
      signal: probe.signal,
      error: errorRecord(probe.error),
      captureLimitBytes: 1024 * 1024,
      rawCaptureComplete: probe.error === undefined,
      stdoutBase64: probe.stdout?.toString("base64"),
      stderrBase64: probe.stderr?.toString("base64"),
    });
    assert.equal(probe.error, undefined);
    assert.equal(probe.signal, null);
    assert.equal(probe.status, 0);
    assert.equal(probe.stderr.toString("utf8"), "");
    assert.equal(probe.stdout.toString("utf8"), `vize ${version}\n`);
    assertInstalled();
    const observations = observeAll(fixture, identity, consumer, output, assertInstalled);
    receipt.counts = {
      total: observations.length,
      passed: observations.filter(({ passed }) => passed).length,
      failed: observations.filter(({ passed }) => !passed).length,
    };
    assertInstalled();
    assert.equal(observations.length, 72);
    assert.equal(
      receipt.counts.failed,
      0,
      "all frozen contracts must pass; complete RED packets are retained",
    );
    receipt.passed = true;
  } catch (error) {
    receipt.passed = false;
    receipt.failure = errorRecord(error);
  }
  receipt.finishedAt = new Date().toISOString();
  write(output, "receipt.json", receipt);
  return receipt;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const receipt = await run(process.argv[2], process.argv[3]);
  console.log(
    JSON.stringify({ output: receipt.output, passed: receipt.passed, counts: receipt.counts }),
  );
  if (!receipt.passed) process.exitCode = 1;
}
