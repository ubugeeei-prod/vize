import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { RawGitRepository } from "./raw-git.ts";
import { runtimeGraph } from "../../../../performance/support/warm-type-backed-runtime.ts";
import {
  loadVueFixtureAuthority,
  originalFiles,
  originalHead,
  stockGraph,
  type VueFixtureAuthority,
} from "./vue.ts";

type PreparationOptions = {
  nodePath: string;
  nodeSha256: string;
  npmPath: string;
  npmSha256: string;
  sourceRoot: string;
  outputRoot: string;
};
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
function authenticate(file: string, sha: string) {
  assert.ok(path.isAbsolute(file));
  assert.equal(fs.realpathSync(file), file);
  assert.ok(fs.statSync(file).isFile());
  assert.match(sha, /^[a-f0-9]{64}$/u);
  assert.equal(digest(fs.readFileSync(file)), sha);
}

/** Only the original Vue fixture closure is installed; no provider packages or build scripts. */
export function prepareVueFixture(options: PreparationOptions) {
  authenticate(options.nodePath, options.nodeSha256);
  authenticate(options.npmPath, options.npmSha256);
  assert.ok(path.isAbsolute(options.sourceRoot));
  assert.equal(fs.realpathSync(options.sourceRoot), options.sourceRoot);
  assert.ok(path.isAbsolute(options.outputRoot));
  assert.equal(path.normalize(options.outputRoot), options.outputRoot);
  assert.equal(fs.existsSync(options.outputRoot), false, "fresh fixture output required");
  const parent = path.dirname(options.outputRoot);
  assert.equal(fs.realpathSync(parent), parent, "canonical output parent required");
  const source = new Map<string, Buffer>();
  const repository = new RawGitRepository(options.sourceRoot);
  for (const [file, sha] of Object.entries(originalFiles)) {
    const bytes = repository.file(originalHead, file);
    assert.equal(digest(bytes), sha, `original stock source changed: ${file}`);
    source.set(file, bytes);
  }
  const lockedPackages = stockGraph(source.get("pnpm-lock.yaml")!.toString("utf8"));
  assert.equal(new Set(lockedPackages.map((record) => record.name)).size, 26);
  const packageManifest = {
    name: "stock8011-vue-fixture",
    version: "1.0.0",
    private: true,
    type: "module",
    dependencies: { vue: "3.6.0-rc.6", typescript: "6.0.3" },
    overrides: Object.fromEntries(lockedPackages.map((record) => [record.name, record.version])),
  };
  fs.mkdirSync(options.outputRoot);
  for (const [file, bytes] of source) {
    const target = path.join(options.outputRoot, "source", file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes, { flag: "wx" });
  }
  const packageManifestPath = path.join(options.outputRoot, "package.json");
  fs.writeFileSync(packageManifestPath, `${JSON.stringify(packageManifest, null, 2)}\n`, {
    flag: "wx",
  });
  const emptyConfig = path.join(options.outputRoot, "npm-empty.ini");
  const emptyGlobalConfig = path.join(options.outputRoot, "npm-global-empty.ini");
  fs.writeFileSync(emptyConfig, "", { flag: "wx" });
  fs.writeFileSync(emptyGlobalConfig, "", { flag: "wx" });
  const environment = { PATH: path.dirname(options.nodePath) };
  const version = (arguments_: string[]) => {
    const result = spawnSync(options.nodePath, arguments_, {
      cwd: options.outputRoot,
      env: environment,
      encoding: "utf8",
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    return result.stdout.trim();
  };
  const nodeVersion = version(["--version"]);
  const npmVersion = version([
    options.npmPath,
    "--version",
    `--userconfig=${emptyConfig}`,
    `--globalconfig=${emptyGlobalConfig}`,
  ]);
  const npmArguments = [
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--registry=https://registry.npmjs.org/",
    `--cache=${path.join(options.outputRoot, "npm-cache")}`,
    `--userconfig=${emptyConfig}`,
    `--globalconfig=${emptyGlobalConfig}`,
  ];
  const stdout = fs.openSync(path.join(options.outputRoot, "npm-install.stdout"), "wx");
  const stderr = fs.openSync(path.join(options.outputRoot, "npm-install.stderr"), "wx");
  let installed: SpawnSyncReturns<Buffer>;
  try {
    installed = spawnSync(options.nodePath, [options.npmPath, ...npmArguments], {
      cwd: options.outputRoot,
      env: environment,
      stdio: ["ignore", stdout, stderr],
      timeout: 120_000,
    });
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  fs.writeFileSync(
    path.join(options.outputRoot, "npm-install.json"),
    `${JSON.stringify(
      {
        nodePath: options.nodePath,
        npmPath: options.npmPath,
        arguments: npmArguments,
        cwd: options.outputRoot,
        environment,
        status: installed.status,
        signal: installed.signal,
        error: installed.error ? String(installed.error) : null,
      },
      null,
      2,
    )}\n`,
    { flag: "wx" },
  );
  assert.equal(installed.error, undefined);
  assert.equal(installed.status, 0, "failed install retained; no fixture authority issued");
  assert.equal(installed.signal, null);
  authenticate(options.nodePath, options.nodeSha256);
  authenticate(options.npmPath, options.npmSha256);
  const packageLockPath = path.join(options.outputRoot, "package-lock.json");
  const lock = JSON.parse(fs.readFileSync(packageLockPath, "utf8"));
  assert.equal(Object.keys(lock.packages).length, 27);
  const registry = lockedPackages.map((record) => {
    const actual = lock.packages[`node_modules/${record.name}`];
    assert.equal(actual.version, record.version);
    assert.equal(
      actual.integrity,
      record.integrity,
      `registry differs from stock lock: ${record.name}`,
    );
    assert.ok(actual.resolved.startsWith("https://registry.npmjs.org/"));
    return {
      name: record.name,
      version: actual.version,
      resolved: actual.resolved,
      integrity: actual.integrity,
    };
  });
  const vueManifestPath = path.join(options.outputRoot, "node_modules/vue/package.json");
  const graph = runtimeGraph([vueManifestPath]);
  const receipt: VueFixtureAuthority = {
    schema: "vize-stock-alias-vue-install-v1",
    source: { head: originalHead, files: originalFiles },
    installRoot: options.outputRoot,
    packageManifestPath,
    packageManifestSha256: digest(fs.readFileSync(packageManifestPath)),
    packageLockPath,
    packageLockSha256: digest(fs.readFileSync(packageLockPath)),
    vueManifestPath,
    node: { path: options.nodePath, version: nodeVersion, sha256: options.nodeSha256 },
    npm: {
      path: options.npmPath,
      version: npmVersion,
      sha256: options.npmSha256,
      arguments: npmArguments,
      exitCode: 0,
    },
    lockedPackages,
    registry,
    runtimeGraph: graph,
    runtimeGraphSha256: digest(JSON.stringify(graph)),
  };
  const receiptPath = path.join(options.outputRoot, "receipt.json");
  const bytes = `${JSON.stringify(receipt, null, 2)}\n`;
  fs.writeFileSync(receiptPath, bytes, { flag: "wx" });
  const receiptSha256 = digest(bytes);
  const authority = loadVueFixtureAuthority({
    receiptPath,
    receiptSha256,
    sourceRoot: options.sourceRoot,
    sourceHead: originalHead,
  });
  authority.recheck();
  return { receiptPath, receiptSha256, vuePath: authority.vuePath, packages: graph.length };
}

if (process.argv[1] && pathToFileURL(fs.realpathSync(process.argv[1])).href === import.meta.url) {
  const { values } = parseArgs({
    options: {
      "node-path": { type: "string" },
      "node-sha256": { type: "string" },
      "npm-path": { type: "string" },
      "npm-sha256": { type: "string" },
      "source-root": { type: "string" },
      "output-root": { type: "string" },
    },
  });
  const required = (key: keyof typeof values): string => {
    const value = values[key];
    assert.ok(typeof value === "string" && value.length > 0, `--${key} is required`);
    return value;
  };
  console.log(
    JSON.stringify(
      prepareVueFixture({
        nodePath: required("node-path"),
        nodeSha256: required("node-sha256"),
        npmPath: required("npm-path"),
        npmSha256: required("npm-sha256"),
        sourceRoot: required("source-root"),
        outputRoot: required("output-root"),
      }),
    ),
  );
}
