import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { packageInventory } from "./archive-inventory.ts";
import { fileIdentity, hash, readAuthority } from "./authority.ts";

export function buildProvider(sourceRoot: string, outputRoot: string) {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "upstream builds run only in Actions");
  assert.equal(process.platform, "linux", "this provider recipe qualifies the Linux builder");
  assert.ok(path.isAbsolute(sourceRoot) && path.isAbsolute(outputRoot));
  const { authority, sha256: authoritySha256 } = readAuthority();
  fs.mkdirSync(outputRoot, { recursive: true });
  assert.deepEqual(fs.readdirSync(outputRoot), [], "refuse stale provider output");
  const records: unknown[] = [];
  const run = (
    label: string,
    command: string,
    args: string[],
    cwd = sourceRoot,
    timeout = 120_000,
  ) => {
    const start = performance.now();
    const result = spawnSync(command, args, {
      cwd,
      encoding: "utf8",
      timeout,
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    });
    const record = {
      label,
      command,
      args,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      elapsedMs: performance.now() - start,
    };
    records.push(record);
    fs.writeFileSync(
      path.join(outputRoot, "build-processes.json"),
      `${JSON.stringify(records, null, 2)}\n`,
    );
    assert.equal(record.error, null, label);
    assert.equal(record.signal, null, label);
    assert.equal(record.status, 0, `${label}: ${record.stderr}`);
    return record.stdout;
  };
  assert.equal(
    run("upstream-revision", "git", ["rev-parse", "HEAD"]).trim(),
    authority.provider.revision,
  );
  assert.equal(
    run("upstream-tree", "git", ["rev-parse", "HEAD^{tree}"]).trim(),
    authority.provider.tree,
  );
  assert.equal(
    run("upstream-clean-before", "git", ["status", "--porcelain", "--untracked-files=no"]),
    "",
  );
  const sourceInputs = authority.sourceInputs.map(({ gitBlob, ...expected }) => {
    const actual = fileIdentity(sourceRoot, expected.path);
    assert.deepEqual(actual, expected);
    return { ...actual, gitBlob };
  });
  const packageRoot = path.join(sourceRoot, "packages/router");
  const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
  assert.equal(manifest.name, authority.provider.packageName);
  assert.equal(manifest.version, authority.provider.packageVersion);
  const pnpmVersion = run("package-manager-version", "pnpm", ["--version"]).trim();
  assert.equal(pnpmVersion, "12.14.0");
  const npmVersion = run("npm-version", "npm", ["--version"]).trim();
  run("source-archive", "git", [
    "archive",
    "--format=tar",
    "--output",
    path.join(outputRoot, "upstream-source.tar"),
    "HEAD",
  ]);
  run(
    "frozen-upstream-install",
    authority.build.install[0],
    authority.build.install.slice(1),
    sourceRoot,
    900_000,
  );
  run(
    "original-upstream-build",
    authority.build.command[0],
    authority.build.command.slice(1),
    sourceRoot,
    900_000,
  );
  const packed = JSON.parse(
    run(
      "pack-built-provider",
      "npm",
      ["pack", "--ignore-scripts", "--json", "--pack-destination", outputRoot],
      packageRoot,
    ),
  );
  assert.equal(packed.length, 1);
  const item = packed[0];
  assert.equal(item.name, authority.provider.packageName);
  assert.equal(item.version, authority.provider.packageVersion);
  assert.equal(item.filename, "vue-router-5.4.0.tgz");
  const archiveBytes = fs.readFileSync(path.join(outputRoot, item.filename));
  const integrity = `sha512-${createHash("sha512").update(archiveBytes).digest("base64")}`;
  assert.equal(integrity, item.integrity);
  const archiveFiles = packageInventory(archiveBytes);
  assert.deepEqual(
    archiveFiles.map(({ path: file }: { path: string }) => file),
    item.files.map(({ path: file }: { path: string }) => file).sort(),
  );
  assert.equal(
    run("upstream-clean-after", "git", ["status", "--porcelain", "--untracked-files=no"]),
    "",
  );
  for (const expected of sourceInputs)
    assert.deepEqual(fileIdentity(sourceRoot, expected.path), {
      path: expected.path,
      bytes: expected.bytes,
      sha256: expected.sha256,
    });
  const receipt = {
    schema: "vize.router-preview-built-provider",
    version: 1,
    state: "provider-built-only",
    qualificationCredit: false,
    actor: {
      sourceRevision: process.env.GITHUB_SHA,
      runId: process.env.GITHUB_RUN_ID,
      runAttempt: process.env.GITHUB_RUN_ATTEMPT,
    },
    authoritySha256,
    upstream: authority.provider,
    sourceInputs,
    sourceArchive: fileIdentity(outputRoot, "upstream-source.tar"),
    builder: {
      node: process.version,
      nodeSha256: hash(fs.readFileSync(process.execPath)),
      pnpm: pnpmVersion,
      npm: npmVersion,
      install: authority.build.install,
      command: authority.build.command,
    },
    package: {
      name: item.name,
      version: item.version,
      filename: item.filename,
      bytes: archiveBytes.length,
      sha256: hash(archiveBytes),
      integrity,
      files: archiveFiles,
    },
    processes: fileIdentity(outputRoot, "build-processes.json"),
  };
  const bytes = Buffer.from(`${JSON.stringify(receipt, null, 2)}\n`);
  fs.writeFileSync(path.join(outputRoot, "provider-build.json"), bytes);
  if (process.env.GITHUB_OUTPUT)
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `receipt-sha256=${hash(bytes)}\n`);
  process.stdout.write(
    `${JSON.stringify({ state: receipt.state, source: authority.provider.revision, packageVersion: item.version, archiveSha256: hash(archiveBytes), receiptSha256: hash(bytes) })}\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [source, output] = process.argv.slice(2);
  assert.ok(source && output, "source and owned output paths are required");
  buildProvider(path.resolve(source), path.resolve(output));
}
