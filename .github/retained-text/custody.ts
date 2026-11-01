import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  array,
  base,
  candidate,
  cliArtifact,
  fixtureRevision,
  object,
  recipe,
  type Json,
  type Role,
} from "./laws.ts";

export const here = path.dirname(fileURLToPath(import.meta.url));
export const manifest = object(
  JSON.parse(fs.readFileSync(path.join(here, "manifest.json"), "utf8")),
);
export const aliases = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];
export const sha = (bytes: string | Uint8Array): string =>
  createHash("sha256").update(bytes).digest("hex");
export const fileSha = (file: string): string => sha(fs.readFileSync(file));
export function physicalImage(file: string): Json {
  const physical = fs.realpathSync(file),
    bytes = fs.readFileSync(physical);
  assert.deepEqual(
    [...bytes.subarray(0, 4)],
    [0x7f, 0x45, 0x4c, 0x46],
    "actual Linux ELF required",
  );
  const stat = fs.statSync(physical, { bigint: true });
  assert.ok(stat.isFile());
  return {
    path: physical,
    sha256: sha(bytes),
    bytes: String(stat.size),
    mode: String(stat.mode),
    device: String(stat.dev),
    inode: String(stat.ino),
    mtimeNs: String(stat.mtimeNs),
    elf: true,
  };
}
export const readJson = (file: string): Json => object(JSON.parse(fs.readFileSync(file, "utf8")));
export function writeJson(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function git(root: string, ...args: string[]): string {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trimEnd();
}
export function run(root: string, argv: string[], prefix: string, env: NodeJS.ProcessEnv): Json {
  fs.mkdirSync(path.dirname(prefix), { recursive: true });
  writeJson(prefix + ".command.json", {
    argv,
    cwd: root,
    started: new Date().toISOString(),
    environment: Object.fromEntries(
      [
        ...aliases,
        "EXPECTED_SOURCE",
        "RAYON_NUM_THREADS",
        "VIZE_PERF_BUDGET_SCALE",
        "VIZE_TRACE_EDITOR_PREPARATION",
        "VIZE_LSP_BIN",
        "CARGO_INCREMENTAL",
        "RUSTFLAGS",
        "CARGO_TARGET_DIR",
        "RUSTC_WRAPPER",
      ].map((name) => [name, env[name] ?? null]),
    ),
  });
  const stdout = fs.openSync(prefix + ".stdout", "wx"),
    stderr = fs.openSync(prefix + ".stderr", "wx");
  let result;
  try {
    result = spawnSync(argv[0], argv.slice(1), {
      cwd: root,
      env,
      stdio: ["ignore", stdout, stderr],
    });
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  const row = {
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    finished: new Date().toISOString(),
    stdoutSha256: fileSha(prefix + ".stdout"),
    stderrSha256: fileSha(prefix + ".stderr"),
  };
  writeJson(prefix + ".result.json", row);
  return row;
}
export function success(result: Json): void {
  assert.equal(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.error, null);
}
export function source(root: string, role: Role): Json {
  const commit = role === "B" ? base : candidate;
  assert.equal(git(root, "rev-parse", "HEAD"), commit);
  assert.equal(git(root, "status", "--porcelain", "--untracked-files=no"), "");
  const files = array(manifest.sources)
    .map(object)
    .map((row) => {
      const relative = String(row.path),
        expected = row[role];
      const file = path.join(root, relative);
      if (expected === null) {
        assert.equal(fs.existsSync(file), false, relative);
        return { path: relative, sha256: null };
      }
      const actual = fileSha(file);
      assert.equal(actual, expected, relative);
      return { path: relative, sha256: actual };
    });
  return { commit, tree: git(root, "rev-parse", "HEAD^{tree}"), files };
}
export function provider(root: string): Json {
  for (const name of aliases)
    assert.equal(process.env[name], undefined, `ambient runtime override: ${name}`);
  const directory = path.join(root, "node_modules/@typescript/typescript-linux-x64");
  const metadata = readJson(path.join(directory, "package.json"));
  assert.equal(metadata.name, "@typescript/typescript-linux-x64");
  assert.equal(metadata.version, "7.0.2");
  const binary = fs.realpathSync(path.join(directory, "lib/tsc"));
  fs.accessSync(binary, fs.constants.X_OK);
  const version = spawnSync(binary, ["--version"], { encoding: "utf8" });
  assert.equal(version.status, 0);
  assert.equal(version.signal, null);
  assert.equal(version.error, undefined);
  assert.equal(version.stderr, "");
  assert.match(version.stdout.trim(), /^Version 7\.0\.2$/);
  return {
    package: metadata.name,
    version: metadata.version,
    binary,
    sha256: fileSha(binary),
    bytes: fs.statSync(binary).size,
    packageJsonSha256: fileSha(path.join(directory, "package.json")),
    versionStdout: version.stdout,
    versionStderr: version.stderr,
    physicalImage: physicalImage(binary),
    lockIntegrity: object(manifest.provider).lockIntegrity,
  };
}
export function corpus(root: string): Json {
  const fixture = path.join(root, "tests/_fixtures/_git/vue-vben-admin");
  assert.equal(git(fixture, "rev-parse", "HEAD"), fixtureRevision);
  assert.equal(git(fixture, "status", "--porcelain", "--untracked-files=no"), "");
  const tracked = git(fixture, "ls-files", "-s");
  const counts: Record<string, number> = {};
  for (const line of tracked.split("\n")) {
    const extension = path.extname(line.slice(line.indexOf("\t") + 1));
    counts[extension] = (counts[extension] ?? 0) + 1;
  }
  return {
    revision: fixtureRevision,
    trackedCount: tracked.split("\n").length,
    trackedIndexSha256: sha(tracked),
    counts,
    selected: array(object(manifest.fixture).selectedApplicationPaths).map((relative) => ({
      path: relative,
      sha256: fileSha(path.join(fixture, String(relative))),
    })),
  };
}
export function snapshot(root: string, role: Role): Json {
  return { source: source(root, role), provider: provider(root), corpus: corpus(root) };
}
export function sameSnapshot(before: Json, after: Json): void {
  assert.deepEqual(after, before);
}
export function retainFirst(root: string, role: Role, images: string, output: string): Json {
  const cli = path.join(root, "target/ci/vize");
  const artifact = cliArtifact(fs.readFileSync(path.join(output, "build.stdout"), "utf8"));
  assert.equal(fs.realpathSync(String(artifact.executable)), fs.realpathSync(cli));
  const receipt = readJson(cli + ".differential-build.json");
  assert.equal(receipt.recipe, recipe);
  assert.equal(receipt.sourceRevision, role === "B" ? base : candidate);
  assert.equal(receipt.binaryPath, "target/ci/vize");
  assert.equal(receipt.binarySha256, fileSha(cli));
  const prefix = path.join(images, role);
  fs.mkdirSync(prefix, { recursive: true });
  fs.copyFileSync(cli, path.join(prefix, "first-vize"), fs.constants.COPYFILE_EXCL);
  fs.copyFileSync(
    cli + ".differential-build.json",
    path.join(prefix, "first-receipt.json"),
    fs.constants.COPYFILE_EXCL,
  );
  const row = {
    role,
    receipt,
    cargoArtifact: artifact,
    sha256: fileSha(cli),
    bytes: fs.statSync(cli).size,
    physicalImage: physicalImage(cli),
    sourceTree: git(root, "rev-parse", "HEAD^{tree}"),
    hostOnlyExecutable: path.join(prefix, "first-vize"),
  };
  writeJson(path.join(output, "first-cli.json"), row);
  return row;
}
export function restoreFirst(root: string, role: Role, images: string, first: Json): void {
  const saved = path.join(images, role, "first-vize");
  assert.equal(fileSha(saved), first.sha256);
  const cli = path.join(root, "target/ci/vize");
  fs.copyFileSync(saved, cli);
  fs.chmodSync(cli, 0o755);
  fs.copyFileSync(path.join(images, role, "first-receipt.json"), cli + ".differential-build.json");
  assert.equal(fileSha(cli), first.sha256);
  assert.deepEqual(readJson(cli + ".differential-build.json"), first.receipt);
}
/** Explicit source-logger/metric filenames only; executable copies remain hosted. */
export function saveAttempt(root: string, destination: string): void {
  const logs = path.join(root, "target/vize-tests/lsp-config-diagnostics");
  const metric = path.join(root, "target/vize-tests/metrics/vben-lsp-incremental");
  for (const [directory, names, sub] of [
    [
      logs,
      [
        "command.json",
        "result.json",
        "test.stdout",
        "test.stderr",
        "native-lsp.log",
        "process-lsp.log",
        "measured-vize.differential-build.json",
      ],
      "original",
    ],
    [metric, ["metrics.json", "summary.md"], "metrics"],
  ] as const) {
    for (const name of names) {
      const input = path.join(directory, name);
      if (!fs.existsSync(input)) continue;
      assert.ok(fs.lstatSync(input).isFile() && !fs.lstatSync(input).isSymbolicLink());
      const output = path.join(destination, sub, name);
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.copyFileSync(input, output, fs.constants.COPYFILE_EXCL);
    }
  }
}

export function inventoryEvidence(root: string): Json {
  const files: Json[] = [];
  function walk(directory: string): void {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name),
        stat = fs.lstatSync(file);
      assert.equal(stat.isSymbolicLink(), false, "no evidence symlinks");
      if (stat.isDirectory()) {
        walk(file);
        continue;
      }
      assert.ok(stat.isFile());
      assert.match(entry.name, /\.(?:json|stdout|stderr|log|md|status)$/);
      const bytes = fs.readFileSync(file);
      assert.notDeepEqual(
        [...bytes.subarray(0, 4)],
        [0x7f, 0x45, 0x4c, 0x46],
        "no ELF in uploaded evidence",
      );
      files.push({ path: path.relative(root, file), bytes: stat.size, sha256: sha(bytes) });
    }
  }
  walk(root);
  return {
    files,
    totalBytes: files.reduce((sum, row) => sum + Number(row.bytes), 0),
    noExecutables: true,
    transport:
      "Authenticate Actions artifact metadata and this complete member inventory before bounded local acquisition; no full target/image archive.",
  };
}
