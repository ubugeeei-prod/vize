/** Qualify retained release CLIs against one frozen Nuxt consumer without rebuilding either side. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { arch, cpus, platform, release } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { publish } from "./shipped-cli-nuxt-report.ts";

import {
  labels,
  balancedOrder,
  type Label,
  type Snapshot,
  type Receipt,
  type RecordRow,
} from "./shipped-cli-nuxt-protocol.ts";
import {
  fixture,
  loadSnapshot,
  paths,
  readJson,
  writeJson,
  sha256,
  relocate,
  validateOutput,
  validateProducer,
} from "./shipped-cli-nuxt-capsule.ts";
export {
  labels,
  balancedOrder,
  type Receipt,
  type RecordRow,
} from "./shipped-cli-nuxt-protocol.ts";
export {
  loadSnapshot,
  relocate,
  safeRelative,
  sha256,
  validateOutput,
  validateProducer,
} from "./shipped-cli-nuxt-capsule.ts";

async function download(out: string, receipt: Receipt) {
  mkdirSync(out, { recursive: true });
  const p = receipt.producer;
  const headers = {
    Authorization: `Bearer ${process.env.GH_TOKEN ?? assert.fail("GH_TOKEN is required")}`,
    Accept: "application/vnd.github+json",
  };
  const api = `https://api.github.com/repos/${p.repository}`;
  const get = async (suffix: string) => {
    const response = await fetch(api + suffix, { headers });
    assert.ok(response.ok, `GitHub metadata request failed (${response.status})`);
    const payload: unknown = await response.json();
    assert.ok(
      payload !== null && typeof payload === "object" && !Array.isArray(payload),
      "invalid GitHub metadata object",
    );
    return payload as Record<string, unknown>;
  };
  const [run, artifact] = await Promise.all([
    get(`/actions/runs/${p.runId}`),
    get(`/actions/artifacts/${p.artifactId}`),
  ]);
  validateProducer(run, artifact, receipt);
  writeJson(join(out, "producer-run.json"), run);
  writeJson(join(out, "producer-artifact.json"), artifact);
  const response = await fetch(api + `/actions/artifacts/${p.artifactId}/zip`, { headers });
  assert.ok(response.ok, `GitHub artifact download failed (${response.status})`);
  const zip = Buffer.from(await response.arrayBuffer());
  assert.equal(sha256(zip), p.artifactSha256, "downloaded artifact ZIP digest differs");
  writeFileSync(join(out, "release-measure.zip"), zip, { flag: "wx" });
}

function command(executable: string, args: string[], cwd: string, log?: string) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
    timeout: 600_000,
  });
  if (log) writeFileSync(log, (result.stdout ?? "") + (result.stderr ?? ""), { flag: "wx" });
  assert.equal(
    result.status,
    0,
    `${executable} failed: ${result.error?.message ?? result.stderr?.slice(-4000)}`,
  );
  return result.stdout;
}

function extractImages(out: string, receipt: Receipt) {
  const zip = join(out, "release-measure.zip");
  assert.equal(sha256(readFileSync(zip)), receipt.producer.artifactSha256);
  const extract = (file: string) => {
    const result = spawnSync("unzip", ["-p", zip, file], { maxBuffer: 128 * 1024 * 1024 });
    assert.equal(result.status, 0, `missing retained evidence ${file}`);
    return result.stdout;
  };
  const pairBytes = extract("pair.json");
  const pair = JSON.parse(pairBytes.toString("utf8"));
  assert.deepEqual(pair, {
    baseSha: receipt.producer.baseSha,
    headSha: receipt.producer.headSha,
    sourceDateEpoch: receipt.producer.sourceDateEpoch,
  });
  writeFileSync(join(out, "pair.json"), pairBytes, { flag: "wx" });
  for (const label of labels) {
    const side = label.startsWith("base") ? "base" : "head";
    const expected = receipt.producer.cli[side];
    const identityBytes = extract(`${label}/cli/identity.json`);
    const identity = JSON.parse(identityBytes.toString("utf8"));
    assert.deepEqual(identity, {
      schema: "vize.release-measure.cli.v1",
      source: side === "base" ? pair.baseSha : pair.headSha,
      profile: "release",
      target: receipt.producer.target,
      ...expected,
    });
    const image = extract(`${label}/cli/vize`);
    assert.equal(image.length, expected.sizeBytes, `${label}: image size differs`);
    assert.equal(sha256(image), expected.sha256, `${label}: image digest differs`);
    assert.equal(
      image.subarray(0, 6).toString("hex"),
      "7f454c460201",
      "expected ELF64 little endian",
    );
    assert.equal(image.readUInt16LE(18), 62, "expected x86-64 ELF machine");
    const target = join(out, "images", label);
    mkdirSync(target, { recursive: true });
    writeFileSync(join(target, "identity.json"), identityBytes, { flag: "wx" });
    writeFileSync(join(target, "side.json"), extract(`${label}/side.json`), { flag: "wx" });
    writeFileSync(join(target, "vize"), image, { flag: "wx", mode: 0o755 });
    const versions = command("readelf", ["--version-info", join(target, "vize")], out);
    writeFileSync(join(target, "elf-versions.txt"), versions, { flag: "wx" });
    const glibc = [...versions.matchAll(/GLIBC_(\d+)\.(\d+)/gu)].map((m) => [
      Number(m[1]),
      Number(m[2]),
    ]);
    assert.ok(glibc.length > 0, `${label}: no ELF glibc version requirements`);
    assert.ok(
      glibc.every(([major, minor]) => major < 2 || (major === 2 && minor <= 17)),
      `${label}: ELF exceeds glibc 2.17`,
    );
  }
}

function prepare(work: string, out: string, receipt: Receipt, snapshot: Snapshot) {
  assert.ok(!existsSync(work), `refusing to reuse prepared workload ${work}`);
  const { project, dependencies } = paths(work);
  mkdirSync(project, { recursive: true });
  mkdirSync(dependencies);
  for (const [file, contents] of Object.entries(snapshot.files)) {
    const target = join(project, file);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, relocate(contents, receipt, work), { flag: "wx" });
  }
  copyFileSync(join(fixture, "package.json"), join(dependencies, "package.json"));
  copyFileSync(join(fixture, "package-lock.json"), join(dependencies, "package-lock.json"));
  // No prepare hook or Nuxt regeneration can alter the captured declarations.
  command(
    "npm",
    ["ci", "--ignore-scripts", "--no-audit", "--no-fund"],
    dependencies,
    join(out, "npm-ci.log"),
  );
  symlinkSync(join(dependencies, "node_modules"), join(project, "node_modules"), "dir");
  const installed: Record<string, string> = {};
  for (const [name, version] of Object.entries(receipt.runtime)) {
    if (name === "node") {
      assert.equal(process.version, `v${version}`, "Node runtime differs");
      installed.node = process.version;
    } else if (name !== "nativeTypeScript") {
      installed[name] = readJson(join(dependencies, "node_modules", name, "package.json")).version;
      assert.equal(installed[name], version, `${name}: installed version differs`);
    }
  }
  const native = "@typescript/typescript-linux-x64";
  installed[native] = readJson(join(dependencies, "node_modules", native, "package.json")).version;
  assert.equal(installed[native], receipt.runtime.nativeTypeScript);
  writeJson(join(out, "runtime.json"), {
    installed,
    platform: platform(),
    arch: arch(),
    release: release(),
    cpus: cpus(),
    nodeSha256: sha256(readFileSync(process.execPath)),
    driver: {
      source: process.env.GITHUB_SHA,
      workflow: process.env.GITHUB_WORKFLOW_REF,
      repository: process.env.GITHUB_REPOSITORY,
      runId: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
    },
    npmVersion: command("npm", ["--version"], dependencies).trim(),
    timeVersion: command("/usr/bin/time", ["--version"], work).trim(),
    nativeTypeScriptSha256: sha256(
      readFileSync(join(dependencies, "node_modules", native, "lib/tsc")),
    ),
    work,
    project,
    dependencies,
    command: ["vize", "check", "--format", "json"],
    scope: receipt.scope,
    substitutions: [
      [receipt.originalProject, project],
      [receipt.originalDependencies, dependencies],
    ],
  });
  assertInputs(work, receipt, snapshot);
}

function projectFiles(project: string, prefix = ""): string[] {
  return readdirSync(join(project, prefix), { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === "node_modules") return [];
      const path = prefix ? `${prefix}/${entry.name}` : entry.name;
      assert.ok(!entry.isSymbolicLink(), `unexpected project symlink ${path}`);
      return entry.isDirectory() ? projectFiles(project, path) : [path];
    })
    .sort();
}

export function assertInputs(work: string, receipt: Receipt, snapshot: Snapshot) {
  const { project, dependencies } = paths(work);
  assert.deepEqual(
    projectFiles(project),
    Object.keys(snapshot.files).sort(),
    "project input inventory changed",
  );
  const hashes: Record<string, string> = {};
  for (const [file, contents] of Object.entries(snapshot.files)) {
    assert.equal(
      readFileSync(join(project, file), "utf8"),
      relocate(contents, receipt, work),
      `frozen project input changed: ${file}`,
    );
  }
  assert.equal(sha256(readFileSync(join(dependencies, "package-lock.json"))), receipt.lockSha256);
  for (const [file, expected] of Object.entries(snapshot.programInputSha256)) {
    // Only generated project files contain the recorded absolute roots.
    const raw = readFileSync(join(project, file));
    const observed = file.startsWith("node_modules/")
      ? sha256(raw)
      : sha256(relocate(raw.toString("utf8"), receipt, work, true));
    assert.equal(observed, expected, `frozen program input changed: ${file}`);
    hashes[file] = sha256(raw);
  }
  return hashes;
}

function measure(work: string, out: string, receipt: Receipt, snapshot: Snapshot) {
  assert.equal(platform(), "linux");
  assert.equal(arch(), "x64");
  const { project, dependencies } = paths(work);
  const scratch = join(work, "invocation");
  mkdirSync(scratch);
  const executable = join(scratch, "vize");
  const records: RecordRow[] = [];
  let complete = false;
  try {
    writeJson(join(out, "inputs-before.json"), assertInputs(work, receipt, snapshot));
    const execute = (label: Label, name: string, index?: number) => {
      assertInputs(work, receipt, snapshot);
      copyFileSync(join(out, "images", label, "vize"), executable);
      chmodSync(executable, 0o755);
      const side = label.startsWith("base") ? "base" : "head";
      assert.equal(sha256(readFileSync(executable)), receipt.producer.cli[side].sha256);
      const time = join(out, `${name}.time.json`);
      const result = spawnSync(
        "/usr/bin/time",
        [
          "--output",
          time,
          "--format",
          '{"elapsed":%e,"user":%U,"system":%S,"maxRssKiB":%M}',
          executable,
          "check",
          "--format",
          "json",
        ],
        {
          cwd: project,
          env: {
            ...process.env,
            CORSA_PATH: join(dependencies, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
          },
          encoding: "utf8",
          maxBuffer: 128 * 1024 * 1024,
          timeout: 240_000,
        },
      );
      writeFileSync(join(out, `${name}.stdout.json`), result.stdout ?? "", { flag: "wx" });
      writeFileSync(join(out, `${name}.stderr`), result.stderr ?? "", { flag: "wx" });
      assert.equal(
        result.status,
        0,
        `${name}: CLI failed (${result.error?.message ?? result.status})`,
      );
      validateOutput(JSON.parse(result.stdout), snapshot, receipt, work);
      assertInputs(work, receipt, snapshot);
      if (index !== undefined) {
        const row = {
          index,
          label,
          ...readJson(time),
          outputSha256: sha256(result.stdout),
        } as RecordRow;
        for (const key of ["elapsed", "user", "system", "maxRssKiB"] as const)
          assert.ok(Number.isFinite(row[key]) && row[key] >= 0, `invalid time metric ${key}`);
        records.push(row);
        writeFileSync(join(out, "progress.json"), JSON.stringify(records, null, 2) + "\n");
        console.log(JSON.stringify(row));
      }
    };
    for (const [i, label] of labels.entries()) execute(label, `warmup-${i}-${label}`);
    for (const [i, label] of balancedOrder().entries())
      execute(label, `${String(i).padStart(2, "0")}-${label}`, i);
    writeJson(join(out, "inputs-after.json"), assertInputs(work, receipt, snapshot));
    complete = true;
  } finally {
    publish(out, receipt, records, complete);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { receipt, snapshot } = loadSnapshot();
  const root = realpathSync(process.env.RUNNER_TEMP ?? assert.fail("RUNNER_TEMP is required"));
  const out = join(root, "shipped-cli-nuxt");
  const work = join(root, "shipped-cli-nuxt-work");
  const mode = process.argv[2];
  if (mode === "--download") await download(out, receipt);
  else if (mode === "--prepare") {
    extractImages(out, receipt);
    prepare(work, out, receipt, snapshot);
  } else if (mode === "--measure") measure(work, out, receipt, snapshot);
  else if (mode === "--replay") {
    const { replay } = await import("./shipped-cli-nuxt-replay.ts");
    replay(out);
  } else assert.fail("expected --download, --prepare, --measure, or --replay");
}
