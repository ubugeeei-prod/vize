import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { gzipSync } from "node:zlib";
import { verifyBracesDepth, type Braces } from "./braces-capture-laws.ts";
const pins = {
  source: "eae9dde61590ee4c803c66159c1fb01a4b6b4245",
  tree: "59dac96e7f49d7c72977ab62777468f84c880712",
  files: {
    "package.json": {
      bytes: 1736,
      sha256: "d8fcbdb8b01ce5837a2ff1691a8f45649cb6dc936b49b4883f8cf6d04ea722e2",
    },
    "pnpm-workspace.yaml": {
      bytes: 16812,
      sha256: "8d56e626002a50be4c9526333af15dc1a3dcd882b05ac22c42bc0633958b664e",
    },
    "pnpm-lock.yaml": {
      bytes: 921981,
      sha256: "c16e5e1370026b63cfb4603ad06cf6c0425e6dc4578149efa53eca1e47a5578e",
    },
    "patches/braces@3.0.3.patch": {
      bytes: 6200,
      sha256: "4164fc10a66e95450b372dda14add9e336b8daa6ef49d472caed0f6d9bc28d74",
    },
    "patches/node-forge@1.4.0.patch": {
      bytes: 1413,
      sha256: "8877267668e88fba4baeb2b574509e2d13a9cff4170179473cb808be833d2b1b",
    },
  },
  package: [
    {
      path: "README.md",
      bytes: 21505,
      sha256: "947b0fc3cc12eaaa070207126213fbdf9ab2bf8cd13dcc6e4007b36b79309866",
      gitBlob: "f59dd604566f1239274c4caf6d88b90b4ddeb25d",
    },
    {
      path: "LICENSE",
      bytes: 1091,
      sha256: "35bdd8a44339719441900fb50fbefc5e2dca1ca662cbaed7a687de842c8b70f2",
      gitBlob: "9af4a67d206f24ecdbb5fdff2839041ca0bbd346",
    },
    {
      path: "index.js",
      bytes: 4380,
      sha256: "332ea07c7b006361aad12aa994ca75dc1db8e8382b884909e2f38f10b85c88a4",
      gitBlob: "d222c13b579d2d476dc26d692219dbaee1665b88",
    },
    {
      path: "package.json",
      bytes: 1647,
      sha256: "56f08b888a4f30dc7cf8a7dbb36ffe92b737912ba36abe9d069d32167c957ac7",
      gitBlob: "c3c056e469726cdb0d0cb6cec73b3f79abf0eb23",
    },
    {
      path: "lib/compile.js",
      bytes: 1903,
      sha256: "c356bd41e07ce3390dd6241fd2d5a799b55872c3116f8eb16717ed6b207cf517",
      gitBlob: "650161895557bc5226bc3a61ba3997962e96dedc",
    },
    {
      path: "lib/constants.js",
      bytes: 1607,
      sha256: "f9fb688959232eee3e6ad7906a5b0e3234815db49ee857ef86983d65b917dc7c",
      gitBlob: "c23709951666ff6bb0d726084d89e62948446bd7",
    },
    {
      path: "lib/expand.js",
      bytes: 3198,
      sha256: "3dab9583d58d02de2ac078ef4d6431796bf994ef7e06ad2ffc2d6c62b01fb19f",
      gitBlob: "a5d5f957e4e7827ebef604f165c920ee57c1aabc",
    },
    {
      path: "lib/parse.js",
      bytes: 7415,
      sha256: "b278f82a3adea41687f7f66b171270ed91c319bf74caa79180c5414efa7a2cdd",
      gitBlob: "1b519c853bb6c16cce62835b89c2774921f2ec3d",
    },
    {
      path: "lib/stringify.js",
      bytes: 1113,
      sha256: "11dbf96c8981431dfadae5656751860a7f384df7cac4f47c5e1f5571a81045f4",
      gitBlob: "e73f99175cbbb067194ec1fa50b35cc06ef7d0cc",
    },
    {
      path: "lib/utils.js",
      bytes: 2518,
      sha256: "b5a7596aa67730412b3c029ef09e84e6b67b8e445cffd35d1d295549c89066c7",
      gitBlob: "d19311fe044ad5157624077670dc297e8b53da49",
    },
  ],
  patch: "4164fc10a66e95450b372dda14add9e336b8daa6ef49d472caed0f6d9bc28d74",
  lockProjection: {
    kind: "BRACES_ONLY_PROJECTION_OF_ACTUAL_RESOLVER_OUTPUT",
    producerRun: 37087472637,
    producerJob: 111100641369,
    producerAttempt: 1,
    producerLockSha256: "7b3c3e33f0c8734584aa10e41ed4d1dde73f48f6cce4242a64c0675a174d2b3a",
    originalLock: {
      bytes: 921669,
      sha256: "add5b07af83e9549188621e211d5a8158ce4addb5fe9d13f1bc54e323300bc4a",
    },
    qualification:
      "Not a verbatim full resolver output; only Braces hash, snapshot header and two dependency refs projected; all original unrelated bytes and Forge object retained.",
  },
};
const root = process.cwd();
const phase = process.argv[2];
const hash = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const command = (bin: string, args: string[]) => {
  const result = spawnSync(bin, args, { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
};
const git = (args: string[]) => {
  const result = command("git", args);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
};
function guard(before = false): void {
  assert.equal(process.version, "v24.14.0");
  assert.equal(git(["rev-parse", pins.source + "^{tree}"]).trim(), pins.tree);
  for (const [file, pin] of Object.entries(pins.files)) {
    if (file === "pnpm-lock.yaml" && !before) continue;
    const bytes = fs.readFileSync(path.join(root, file));
    assert.equal(bytes.length, pin.bytes);
    assert.equal(hash(bytes), pin.sha256, file);
  }
  const changed = git(["diff", "--name-only", "HEAD", "--"]).trim().split("\n").filter(Boolean);
  assert.deepEqual(changed, [], "only generated root lock may change");
  assert.equal(git(["diff", "--cached", "--name-only"]).trim(), "");
}
function emit(data: unknown): void {
  const raw = Buffer.from(JSON.stringify(data));
  const gzip = gzipSync(raw);
  const encoded = gzip.toString("base64");
  const chunkSize = 4096;
  const header = {
    phase,
    rawBytes: raw.length,
    rawSha256: hash(raw),
    gzipBytes: gzip.length,
    gzipSha256: hash(gzip),
    chunks: Math.ceil(encoded.length / chunkSize),
  };
  console.log("VIZE_BRACES_LOCK_CAPTURE_V1 HEADER " + JSON.stringify(header));
  for (let index = 0; index < header.chunks; index++)
    console.log(
      "VIZE_BRACES_LOCK_CAPTURE_V1 CHUNK " +
        index +
        " " +
        encoded.slice(index * chunkSize, (index + 1) * chunkSize),
    );
  console.log("VIZE_BRACES_LOCK_CAPTURE_V1 END " + header.rawSha256);
}
function file(filePath: string) {
  const bytes = fs.readFileSync(filePath);
  return { bytes: bytes.length, sha256: hash(bytes), base64: bytes.toString("base64") };
}
function inventory(
  directory: string,
): { path: string; bytes: number; sha256: string; base64: string }[] {
  const rows: { path: string; bytes: number; sha256: string; base64: string }[] = [];
  const walk = (current: string) => {
    for (const name of fs.readdirSync(current).sort()) {
      const next = path.join(current, name);
      const stat = fs.lstatSync(next);
      if (stat.isDirectory()) walk(next);
      else {
        assert.ok(stat.isFile(), "unexpected package symlink");
        rows.push({
          path: path.relative(directory, next).split(path.sep).join("/"),
          ...file(next),
        });
      }
    }
  };
  walk(directory);
  return rows.sort((a, b) => a.path.localeCompare(b.path));
}
function dependencies(packageRoot: string): unknown[] {
  const seen = new Set<string>();
  const rows: unknown[] = [];
  const walk = (directory: string) => {
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    const require = createRequire(path.join(directory, "package.json"));
    for (const name of Object.keys(manifest.dependencies ?? {}).sort()) {
      const resolved = fs.realpathSync(path.dirname(require.resolve(name + "/package.json")));
      if (seen.has(resolved)) continue;
      seen.add(resolved);
      rows.push({
        from: directory,
        name,
        declared: manifest.dependencies[name],
        resolved,
        rows: inventory(resolved),
      });
      walk(resolved);
    }
  };
  walk(packageRoot);
  return rows;
}
const identity = {
  schema: "vize.braces-hosted-lock-install-capture",
  phase,
  source: pins.source,
  sourceTree: pins.tree,
  checkout: git(["rev-parse", "HEAD"]).trim(),
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  job: process.env.GITHUB_JOB,
  ref: process.env.GITHUB_REF,
  node: process.version,
};
if (phase === "before") {
  guard(true);
  const pnpm = command("vp", ["exec", "pnpm", "--version"]);
  assert.equal(pnpm.status, 0, pnpm.stderr);
  assert.equal(pnpm.stdout.trim(), "12.1.0");
  emit({
    ...identity,
    pins,
    pnpm,
    beforeLock: file("pnpm-lock.yaml"),
    originalSourceLock: {
      ...pins.lockProjection.originalLock,
      base64: Buffer.from(git(["show", pins.source + ":pnpm-lock.yaml"])).toString("base64"),
    },
  });
} else if (phase === "after-lock") {
  guard();
  emit({
    ...identity,
    generatedLock: file("pnpm-lock.yaml"),
    completeDiff: git(["diff", pins.source, "HEAD", "--", "pnpm-lock.yaml"]),
  });
} else if (phase === "audit") {
  const audit = command("vp", [
    "exec",
    "pnpm",
    "audit",
    "--prod",
    "--audit-level",
    "moderate",
    "--json",
  ]);
  emit({
    ...identity,
    originalProductionAudit: { ...audit, stdoutSha256: hash(audit.stdout) },
    accepted: false,
  });
  assert.ok(audit.status === 0 || audit.status === 1);
  const report = JSON.parse(audit.stdout);
  assert.ok(report.advisories && report.metadata && !report.error);
} else if (phase === "installed") {
  guard();
  const store = path.join(root, "node_modules/.pnpm");
  const packages = [];
  const consumers = [];
  for (const directory of fs.readdirSync(store).sort()) {
    if (directory.startsWith("braces@")) {
      const packageRoot = fs.realpathSync(path.join(store, directory, "node_modules/braces"));
      const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
      const rows = inventory(packageRoot);
      assert.equal(manifest.name, "braces");
      assert.equal(manifest.version, "3.0.3", "unexpected installed braces version");
      assert.deepEqual(
        rows.map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 })),
        pins.package
          .map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 }))
          .sort((a, b) => a.path.localeCompare(b.path)),
      );
      const require = createRequire(path.join(packageRoot, "package.json"));
      assert.equal(require.resolve(packageRoot), path.join(packageRoot, "index.js"));
      packages.push({
        directory,
        packageRoot,
        manifest,
        rows,
        runtimeDependencies: dependencies(packageRoot),
        laws: verifyBracesDepth(require(packageRoot) as Braces),
      });
    }
    if (!/^(chokidar@|micromatch@)/.test(directory)) continue;
    const name = directory.startsWith("chokidar@") ? "chokidar" : "micromatch";
    const packageRoot = fs.realpathSync(path.join(store, directory, "node_modules", name));
    const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
    if (!manifest.dependencies?.braces) continue;
    const require = createRequire(path.join(packageRoot, "package.json"));
    const resolved = fs.realpathSync(path.dirname(require.resolve("braces/package.json")));
    const braces = require("braces") as Braces;
    assert.deepEqual(braces.expand("src/{app,lib}/{1..3}.ts"), [
      "src/app/1.ts",
      "src/app/2.ts",
      "src/app/3.ts",
      "src/lib/1.ts",
      "src/lib/2.ts",
      "src/lib/3.ts",
    ]);
    consumers.push({
      directory,
      packageRoot,
      manifest,
      resolvedBraces: resolved,
      packageManifest: file(path.join(packageRoot, "package.json")),
      rows: inventory(packageRoot),
      linkedRuntimeLaws: verifyBracesDepth(braces),
    });
  }
  assert.ok(packages.length > 0);
  assert.ok(consumers.some((row) => row.manifest.name === "chokidar"));
  assert.ok(consumers.some((row) => row.manifest.name === "micromatch"));
  for (const consumer of consumers)
    assert.ok(packages.some((row) => row.packageRoot === consumer.resolvedBraces));
  const { verifyInstalledForge } = await import("../security/node-forge-proof.ts");
  const forge = verifyInstalledForge(root);
  emit({
    ...identity,
    generatedLock: file("pnpm-lock.yaml"),
    packages,
    consumers,
    forge,
    registryReleaseRemainsUnpatched: true,
    detachedNuxtFixtureLocksRepaired: false,
  });
} else throw Error("unknown capture phase");
