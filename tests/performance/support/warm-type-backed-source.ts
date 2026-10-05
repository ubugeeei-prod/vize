import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const driverRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const fixtureRoot = path.join(
  driverRoot,
  "tests/_fixtures/differential/lsp/warm-type-backed-requests",
);
export const sha256 = (bytes: Buffer | string): string =>
  createHash("sha256").update(bytes).digest("hex");

export function git(repoRoot: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function inputAuthority(): Record<string, string> {
  const manifest = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "case.json"), "utf8")) as {
    originalSha256: Record<string, string>;
  };
  for (const [file, digest] of Object.entries(manifest.originalSha256)) {
    assert.equal(sha256(fs.readFileSync(path.join(fixtureRoot, file))), digest, file);
  }
  return manifest.originalSha256;
}

export function runtimeIdentity() {
  const fromTests = createRequire(path.join(driverRoot, "tests/package.json"));
  const vueManifest = fs.realpathSync(fromTests.resolve("vue/package.json"));
  const runtimeManifest = fs.realpathSync(
    createRequire(path.join(driverRoot, "package.json")).resolve(
      "@typescript/typescript-linux-x64/package.json",
    ),
  );
  const runtime = path.join(path.dirname(runtimeManifest), "lib/tsc");
  const read = (file: string) => ({
    path: file,
    sha256: sha256(fs.readFileSync(file)),
    content: JSON.parse(fs.readFileSync(file, "utf8")) as Record<string, unknown>,
  });
  assert.ok(fs.statSync(runtime).isFile());
  assert.equal(read(runtimeManifest).content.version, "7.0.2");
  const probe = spawnSync(runtime, ["--version"], { encoding: "utf8" });
  assert.equal(probe.status, 0, probe.stderr);
  return {
    vue: read(vueManifest),
    native: read(runtimeManifest),
    executable: fs.realpathSync(runtime),
    binarySha256: sha256(fs.readFileSync(runtime)),
    versionProbe: {
      status: probe.status,
      signal: probe.signal,
      stdout: probe.stdout,
      stderr: probe.stderr,
    },
    reportedVueVersion: "3.5.41",
    dependencyQualification:
      "actual locked dependency graph; reported package version is historical",
  };
}

export function generateWorkspace(
  workspace: string,
  runtime: ReturnType<typeof runtimeIdentity>,
  files: 20 | 400 = 400,
) {
  inputAuthority();
  assert.ok(["workspace", "workspace-other"].includes(path.basename(workspace)));
  assert.ok(fs.existsSync(path.join(path.dirname(workspace), "workflow-source.json")));
  // This directory is created solely by this driver and reused at the same
  // absolute path for both sides. The original generator itself is unchanged.
  fs.rmSync(workspace, { recursive: true, force: true });
  fs.mkdirSync(workspace, { recursive: true });
  for (const file of ["gen.mjs", "package.json", "tsconfig.json", "vize.config.json"]) {
    fs.copyFileSync(path.join(fixtureRoot, file), path.join(workspace, file));
  }
  const generated = spawnSync(process.execPath, [path.join(workspace, "gen.mjs"), String(files)], {
    encoding: "utf8",
  });
  assert.equal(generated.status, 0, generated.stderr);
  const source = Object.fromEntries(
    ["c", "lib"].flatMap((directory) =>
      fs
        .readdirSync(path.join(workspace, "src", directory))
        .toSorted()
        .map((name) => {
          const relative = `src/${directory}/${name}`;
          return [relative, fs.readFileSync(path.join(workspace, relative), "utf8")];
        }),
    ),
  ) as Record<string, string>;
  assert.equal(Object.keys(source).filter((name) => name.endsWith(".vue")).length, files);
  assert.equal(
    Object.keys(source).filter((name) => name.endsWith(".ts")).length,
    Math.ceil(files / 3),
  );
  for (const relative of [...Object.keys(source), "tsconfig.json"]) {
    fs.utimesSync(path.join(workspace, relative), 1_700_000_000, 1_700_000_000);
  }
  const modules = path.join(workspace, "node_modules");
  fs.mkdirSync(path.join(modules, ".bin"), { recursive: true });
  fs.symlinkSync(path.dirname(runtime.vue.path), path.join(modules, "vue"), "dir");
  const vueScope = fs.realpathSync(path.join(path.dirname(path.dirname(runtime.vue.path)), "@vue"));
  fs.symlinkSync(vueScope, path.join(modules, "@vue"), "dir");
  // Runtime discovery is an owned filesystem link, not a config/source edit.
  fs.symlinkSync(runtime.executable, path.join(modules, ".bin/tsgo"));
  return {
    source,
    sourceSha256: sha256(JSON.stringify(source)),
    generatorProcess: generated,
    workspace,
    files,
    sourceMtimeSeconds: 1_700_000_000,
  };
}

export function sourceIdentity(repoRoot: string) {
  const files = [
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "pnpm-lock.yaml",
    "rust-toolchain.toml",
  ];
  return {
    revision: git(repoRoot, ["rev-parse", "HEAD"]),
    tree: git(repoRoot, ["rev-parse", "HEAD^{tree}"]),
    locks: Object.fromEntries(
      files.map((file) => [file, sha256(fs.readFileSync(path.join(repoRoot, file)))]),
    ),
    production: git(repoRoot, ["rev-parse", "HEAD:crates"]),
    dirty: git(repoRoot, ["diff", "--name-only", "HEAD"]),
    recipe: "cargo build --profile ci -p vize",
  };
}
