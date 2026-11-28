/** Authenticated upstream checker builds, separate from the fixture and published oracle. */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { packageManagerRunner, runPackageManager } from "./typecheck-package-manager.ts";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
const supportedProvider = "vue-language-tools-6371";
const executable = "vize-vue-tsc.cjs";
const sha256 = (value) => createHash("sha256").update(value).digest("hex");

export function typecheckProvider(value) {
  if (value === undefined) return null;
  if (value !== supportedProvider) throw new Error("Unknown pinned typecheck baseline provider");
  const descriptor = join(repoRoot, "tests/_fixtures/typecheck-providers", `${value}.json`);
  const bytes = readFileSync(descriptor);
  return { ...JSON.parse(bytes), descriptorSha256: sha256(bytes) };
}

function providerRoot(fixtureRoot, provider) {
  return join(fixtureRoot, ".vize-typecheck-provider", provider.id);
}

function git(root, ...args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", timeout: 120_000 }).trim();
}

/** The whole authored checkout, including its lockfile, must remain at the sealed commit. */
export function verifyTypecheckProviderSource(root, provider) {
  if (realpathSync(git(root, "rev-parse", "--show-toplevel")) !== realpathSync(root)) {
    throw new Error("Typecheck provider Git root differs");
  }
  if (git(root, "rev-parse", "HEAD") !== provider.revision) {
    throw new Error("Typecheck provider revision differs");
  }
  if (git(root, "status", "--porcelain=v1", "--untracked-files=no") !== "") {
    throw new Error("Typecheck provider authored source changed");
  }
  for (const [path, blob] of Object.entries(provider.sourceBlobs)) {
    if (git(root, "rev-parse", `HEAD:${path}`) !== blob) {
      throw new Error(`Typecheck provider source blob differs: ${path}`);
    }
  }
}

function providerOutputs(root, provider) {
  const paths = new Set([executable, provider.bin, ...provider.outputs]);
  const collect = (directory) => {
    for (const entry of readdirSync(join(root, directory), { withFileTypes: true })) {
      if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      const path = `${directory}/${entry.name}`;
      if (entry.isDirectory()) collect(path);
      else if (entry.name.endsWith(".js") || entry.name.endsWith(".d.ts")) paths.add(path);
    }
  };
  collect("packages/tsc");
  collect("packages/language-core");
  return [...paths].sort().map((path) => {
    const file = join(root, path);
    const stat = lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size === 0) {
      throw new Error(`Typecheck provider output is not a nonempty regular file: ${path}`);
    }
    return { path, sizeBytes: stat.size, sha256: sha256(readFileSync(file)) };
  });
}

function runtimeCompiler(provider) {
  // The upstream source build dogfoods a different compiler through its
  // workspace override. Select the same supported TypeScript peer as the
  // published matrix oracle through vue-tsc's public run(tscPath) argument.
  const require = createRequire(join(repoRoot, "tests/package.json"));
  const manifest = require.resolve("typescript/package.json");
  const root = dirname(manifest);
  const metadata = JSON.parse(readFileSync(manifest, "utf8"));
  if (metadata.version !== provider.typescriptPeerVersion) {
    throw new Error("Pinned typecheck provider TypeScript peer version differs");
  }
  const paths = ["package.json", "lib/tsc.js", "lib/_tsc.js", "lib/typescript.js"];
  return {
    path: join(root, "lib/tsc.js"),
    name: metadata.name,
    version: metadata.version,
    files: paths.map((path) => ({ path, sha256: sha256(readFileSync(join(root, path))) })),
  };
}

function entrySource(root) {
  return (
    `#!/usr/bin/env node\nrequire(${JSON.stringify(join(root, "packages/tsc/index.js"))})` +
    `.run(${JSON.stringify(join(repoRoot, "tests/node_modules/typescript/lib/tsc.js"))});\n`
  );
}

function identity(root, provider) {
  verifyTypecheckProviderSource(root, provider);
  if (readFileSync(join(root, executable), "utf8") !== entrySource(root)) {
    throw new Error("Typecheck provider entrypoint differs");
  }
  return {
    id: provider.id,
    repository: provider.repository,
    revision: provider.revision,
    upstreamFix: provider.upstreamFix,
    descriptorSha256: provider.descriptorSha256,
    lockfileSha256: sha256(readFileSync(join(root, "pnpm-lock.yaml"))),
    typescriptPeer: {
      version: provider.typescriptPeerVersion,
      repositoryLockfileSha256: sha256(readFileSync(join(repoRoot, "pnpm-lock.yaml"))),
    },
    outputs: providerOutputs(root, provider),
  };
}

/** Build only from the official frozen tree; never patch the published checker or compared source. */
export async function prepareTypecheckProvider(fixtureRoot, value, timeoutMs) {
  const provider = typecheckProvider(value);
  if (provider == null) return null;
  const root = providerRoot(fixtureRoot, provider);
  if (!existsSync(root)) {
    mkdirSync(root, { recursive: true });
    git(root, "init");
    git(root, "fetch", "--depth=1", provider.repository, provider.revision);
    git(root, "checkout", "--detach", "FETCH_HEAD");
  }
  verifyTypecheckProviderSource(root, provider);
  const manager = packageManagerRunner("pnpm", provider.packageManagerVersion);
  const startedAt = Date.now();
  let stdout = "";
  let stderr = "";
  const command = [provider.install, provider.build].map((args) => ["pnpm", ...args]);
  for (const args of [provider.install, provider.build]) {
    const result = await runPackageManager(manager, args, {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, CI: "true", npm_config_ignore_scripts: "true" },
      timeout: timeoutMs,
      maxBuffer: 1024 * 1024 * 1024,
    });
    if (result.error != null || result.status !== 0) {
      throw new Error(
        `Pinned typecheck provider build failed: ${result.error ?? result.status}\n` +
          `${result.stderr ?? ""}\n${result.stdout ?? ""}`,
      );
    }
    stdout += result.stdout ?? "";
    stderr += result.stderr ?? "";
  }
  writeFileSync(join(root, executable), entrySource(root));
  chmodSync(join(root, executable), 0o755);
  return {
    ...identity(root, provider),
    command,
    durationMs: Date.now() - startedAt,
    exitCode: 0,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
  };
}

/** Re-read the actual source and generated modules immediately before baseline execution. */
export function verifyPreparedTypecheckProvider(fixtureRoot, value, evidence) {
  const provider = typecheckProvider(value);
  if (provider == null) {
    if (evidence != null) throw new Error("Unexpected typecheck provider evidence");
    return null;
  }
  const root = providerRoot(fixtureRoot, provider);
  const actual = identity(root, provider);
  for (const [key, value] of Object.entries(actual)) {
    if (JSON.stringify(evidence?.[key]) !== JSON.stringify(value)) {
      throw new Error(`Typecheck provider evidence differs: ${key}`);
    }
  }
  const command = [provider.install, provider.build].map((args) => ["pnpm", ...args]);
  if (
    evidence.exitCode !== 0 ||
    JSON.stringify(evidence.command) !== JSON.stringify(command) ||
    !/^[a-f0-9]{64}$/.test(evidence.stdoutSha256 ?? "") ||
    !/^[a-f0-9]{64}$/.test(evidence.stderrSha256 ?? "")
  )
    throw new Error("Typecheck provider build evidence differs");
  // Root JavaScript dependencies are installed after fixture preparation in
  // Actions. Bind their actual runtime bytes here, immediately before use.
  const compiler = runtimeCompiler(provider);
  if (
    realpathSync(compiler.path) !==
    realpathSync(join(repoRoot, "tests/node_modules/typescript/lib/tsc.js"))
  ) {
    throw new Error("Typecheck provider TypeScript peer path differs");
  }
  return { path: join(root, executable), identity: { ...actual, typescriptRuntime: compiler } };
}

if (process.argv[1] != null && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, fixtureRoot, provider, artifactPath] = process.argv.slice(2);
  if (command !== "verify" || artifactPath == null)
    throw new Error("Expected provider verify inputs");
  const artifact = JSON.parse(readFileSync(artifactPath, "utf8"));
  const verified = verifyPreparedTypecheckProvider(
    resolve(fixtureRoot),
    provider,
    artifact.baselinePrepare?.provider,
  );
  process.stdout.write(`${JSON.stringify(verified)}\n`);
}
