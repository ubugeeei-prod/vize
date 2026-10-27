import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { controls, negative, type Case, type Qs } from "./qs-runtime-contracts.ts";

type Package = {
  name: string;
  version: string;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  overrides?: unknown;
  pnpm?: unknown;
  resolutions?: unknown;
  dist?: { integrity: string; shasum: string; tarball: string };
};
type Packet = {
  command: string;
  args: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  nodePath: string | null;
  registry: string;
};
const root = fileURLToPath(new URL("../../../", import.meta.url));
const registry = "https://registry.npmjs.org";
const output = path.resolve(process.argv[2] ?? "");
const archive = process.argv[3];
assert(process.argv[2] && archive, "proof directory and actual packed MCP archive are required");
fs.mkdirSync(output, { recursive: true });
const report: {
  schema: string;
  advisory: string;
  authoredHead: string | null;
  source: unknown;
  processes: Record<string, Packet>;
  producers: unknown[];
  matrices: unknown[];
  scope: string;
} = {
  schema: "vize.qs.runtime-security.v1",
  advisory: "https://github.com/advisories/GHSA-6pq8-jhm2-2vcw",
  authoredHead: process.env.EXPECTED_SOURCE_SHA ?? null,
  source: null,
  processes: {},
  producers: [],
  matrices: [],
  scope:
    "source and fresh packed-MCP dependency graphs; existing consumer locks require their own update",
};
const hash = (value: string | Buffer, algorithm = "sha256") =>
  createHash(algorithm).update(value).digest("hex");
const save = () =>
  fs.writeFileSync(path.join(output, "proof.json"), JSON.stringify(report, null, 2) + "\n");
function run(name: string, command: string, args: string[], cwd: string, env = process.env) {
  const result = spawnSync(command, args, {
    cwd,
    env: { ...env, npm_config_registry: registry, NPM_CONFIG_REGISTRY: registry },
    encoding: "utf8",
    timeout: 300_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const packet: Packet = {
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    nodePath: env.NODE_PATH ?? null,
    registry,
  };
  report.processes[name] = packet;
  save();
  assert.equal(packet.error, null, name);
  assert.equal(packet.signal, null, name);
  return packet;
}
function success(packet: Packet, name: string) {
  assert.equal(packet.status, 0, name + ": " + packet.stderr.slice(-4000));
}
function readPackage(filename: string) {
  return JSON.parse(fs.readFileSync(filename, "utf8")) as Package;
}
function packageRoot(entry: string, expectedName: string) {
  let directory = path.dirname(fs.realpathSync(entry));
  while (true) {
    const manifest = path.join(directory, "package.json");
    if (fs.existsSync(manifest) && readPackage(manifest).name === expectedName) return directory;
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory, "unresolved actual package " + expectedName);
    directory = parent;
  }
}
function resolvePackage(parent: string, name: string, specifier = name) {
  return packageRoot(createRequire(path.join(parent, "package.json")).resolve(specifier), name);
}
function tree(directory: string, relative = ""): Record<string, string> {
  const files: Record<string, string> = {};
  for (const entry of fs.readdirSync(path.join(directory, relative), { withFileTypes: true })) {
    const name = path.posix.join(relative, entry.name);
    if (entry.isDirectory()) Object.assign(files, tree(directory, name));
    else if (entry.isFile()) files[name] = hash(fs.readFileSync(path.join(directory, name)));
  }
  return Object.fromEntries(Object.entries(files).sort(([a], [b]) => a.localeCompare(b, "en")));
}
function routes(mcp: string) {
  const sdk = resolvePackage(
    mcp,
    "@modelcontextprotocol/sdk",
    "@modelcontextprotocol/sdk/server/index.js",
  );
  const express = resolvePackage(sdk, "express");
  const body = resolvePackage(express, "body-parser");
  const limit = resolvePackage(sdk, "express-rate-limit");
  const limitExpress = resolvePackage(limit, "express");
  const limitBody = resolvePackage(limitExpress, "body-parser");
  return [
    ["mcp", mcp, "sdk", sdk, "express", express],
    ["mcp", mcp, "sdk", sdk, "express", express, "body-parser", body],
    ["mcp", mcp, "sdk", sdk, "express-rate-limit", limit, "express", limitExpress],
    [
      "mcp",
      mcp,
      "sdk",
      sdk,
      "express-rate-limit",
      limit,
      "express",
      limitExpress,
      "body-parser",
      limitBody,
    ],
  ].map((route) => {
    const parent = route.at(-1);
    assert(parent);
    const qs = resolvePackage(parent, "qs");
    return {
      route,
      qs,
      manifest: readPackage(path.join(qs, "package.json")),
      entry: fs.realpathSync(createRequire(path.join(parent, "package.json")).resolve("qs")),
    };
  });
}
function observe(qs: Qs, item: Case) {
  try {
    return { value: qs.stringify(item.make(qs), item.options), error: null };
  } catch (error) {
    return {
      value: null,
      error: {
        name: error instanceof Error ? error.name : "unknown",
        message: error instanceof Error ? error.message : String(error),
      },
    };
  }
}
function matrix(
  directory: string,
  role: string,
  vulnerable: boolean,
  normalBefore?: ReturnType<typeof observe>[],
) {
  const qs = createRequire(path.join(directory, "package.json"))("./") as Qs;
  const bad = negative.map((item) => ({
    name: item.name,
    factory: item.make.toString(),
    options: item.options,
    result: observe(qs, item),
  }));
  const normal = controls.map((item) => ({
    name: item.name,
    factory: item.make.toString(),
    options: item.options,
    result: observe(qs, item),
  }));
  report.matrices.push({ role, directory, vulnerable, negative: bad, controls: normal });
  save();
  for (const item of bad) {
    if (vulnerable) assert.equal(item.result.error?.name, "TypeError", item.name);
    else {
      assert.equal(item.result.error, null, item.name);
      assert.equal(item.result.value, "a=%5Bobject%20Object%5D", item.name);
    }
  }
  if (normalBefore)
    assert.deepEqual(
      normal.map((item) => item.result),
      normalBefore,
      role + ": complete normal controls",
    );
  const authoredThrow = normal.find((item) => item.name === "authored-throw-propagates");
  assert.equal(authoredThrow?.result.error?.name, "SyntaxError");
  assert.equal(authoredThrow?.result.error?.message, "authored-error");
  return normal.map((item) => item.result);
}
function originalProcesses(directory: string, role: string, vulnerable: boolean) {
  const cases = [
    "q.stringify(q.parse('a[0][toString]=x', { allowPrototypes: true }), { arrayFormat: 'comma' })",
    'q.stringify(JSON.parse(\'{"a":[{"toString":"x"}]}\'), { arrayFormat: \'comma\' })',
    'q.stringify(JSON.parse(\'{"a":[{"toString":"x"}]}\'), { arrayFormat: \'comma\', encodeValuesOnly: true })',
  ];
  for (const [index, expression] of cases.entries()) {
    const program =
      "const q=require(" + JSON.stringify(directory) + ");console.log(" + expression + ");";
    const packet = run(
      role + "-uncaught-original-" + index,
      process.execPath,
      ["-e", program],
      root,
    );
    if (vulnerable) {
      assert.equal(packet.status, 1, "the original public payload must fail uncaught");
      assert.match(packet.stderr, /TypeError: Cannot convert object to primitive value/);
    } else {
      success(packet, "the actual fixed runtime handles the original payload");
      assert.equal(packet.stdout, "a=%5Bobject%20Object%5D\n");
    }
  }
}
async function published(version: string) {
  assert.match(version, /^6\.\d+\.\d+$/u, "exact public qs version");
  const response = await fetch("https://registry.npmjs.org/qs/" + version);
  assert.equal(response.status, 200);
  const raw = await response.text();
  fs.writeFileSync(path.join(output, "registry-" + version + ".json"), raw);
  const metadata = JSON.parse(raw) as Package;
  assert.equal(metadata.name, "qs");
  assert.equal(metadata.version, version);
  assert(metadata.dist);
  assert.equal(metadata.dist.tarball, "https://registry.npmjs.org/qs/-/qs-" + version + ".tgz");
  const tar = await fetch(metadata.dist.tarball);
  assert.equal(tar.status, 200);
  const bytes = Buffer.from(await tar.arrayBuffer());
  const integrity = "sha512-" + createHash("sha512").update(bytes).digest("base64");
  const historicalIntegrity = (
    {
      "6.16.0":
        "sha512-h6fhOIaRrID2CbEY2fqs+7t+UXZo+MLAnU5gRIq85uFtdiUPCdsApMlHhXogKVM4HM2DVbIjGNTTYH2OcmP1vA==",
      "6.16.1":
        "sha512-5Px6J7rfHBAGUhNQ3es4YzcOdBBw0SDcVtThJml4RdnKw4tKs2epA0kPZ+K8X+WiHktT93RHVcQ/3ym5aTCgtg==",
    } as Record<string, string>
  )[version];
  if (historicalIntegrity) assert.equal(integrity, historicalIntegrity);
  assert.equal(integrity, metadata.dist.integrity);
  assert.equal(hash(bytes, "sha1"), metadata.dist.shasum);
  const archivePath = path.join(output, "qs-" + version + ".tgz");
  fs.writeFileSync(archivePath, bytes);
  const project = path.join(path.dirname(output), "qs-public-" + version);
  fs.mkdirSync(project, { recursive: true });
  success(
    run(
      "install-" + version,
      "npm",
      ["install", "--prefix", project, "--ignore-scripts", "--no-audit", "--no-fund", archivePath],
      project,
    ),
    "public producer install",
  );
  fs.copyFileSync(
    path.join(project, "package-lock.json"),
    path.join(output, "lock-" + version + ".json"),
  );
  const installed = resolvePackage(project, "qs");
  report.producers.push({
    version,
    registry: metadata.dist,
    archiveSha256: hash(bytes),
    installed,
    manifest: readPackage(path.join(installed, "package.json")),
    lib: tree(path.join(installed, "lib")),
    tests: tree(path.join(installed, "test")),
  });
  save();
  return { metadata, installed };
}
async function main() {
  const revision = run("source-revision", "git", ["rev-parse", "HEAD"], root);
  success(revision, "source revision");
  report.source = {
    revision: revision.stdout.trim(),
    lockSha256: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
    workspaceSha256: hash(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"))),
  };
  save();
  success(
    run(
      "source-security-attestation",
      process.execPath,
      ["tools/commands/ci/npm-audit-retry.ts"],
      root,
    ),
    "original full source security laws",
  );
  const before = await published("6.16.0"),
    fixed = await published("6.16.1");
  assert.deepEqual(before.metadata.dependencies, fixed.metadata.dependencies);
  originalProcesses(before.installed, "immutable-public-before", true);
  originalProcesses(fixed.installed, "immutable-public-fixed", false);
  const normalBefore = matrix(before.installed, "immutable-public-before", true);
  matrix(fixed.installed, "immutable-public-fixed", false, normalBefore);
  const consumer = path.join(path.dirname(output), "qs-packed-mcp-consumer");
  fs.mkdirSync(consumer, { recursive: true });
  fs.writeFileSync(
    path.join(consumer, "package.json"),
    JSON.stringify({ name: "qs-packed-mcp-proof", private: true, version: "0.0.0" }) + "\n",
  );
  success(
    run(
      "packed-mcp-install",
      "npm",
      ["install", "--ignore-scripts", "--no-audit", "--no-fund", path.resolve(archive)],
      consumer,
    ),
    "fresh packed MCP installation",
  );
  fs.copyFileSync(
    path.join(consumer, "package-lock.json"),
    path.join(output, "packed-mcp-lock.json"),
  );
  const consumerManifest = readPackage(path.join(consumer, "package.json"));
  for (const field of ["overrides", "pnpm", "resolutions"] as const)
    assert.equal(consumerManifest[field], undefined, "consumer has no workspace policy " + field);
  // The MCP package exposes only its ESM import entry; inspect the installed
  // root manifest directly instead of asking CommonJS export resolution.
  const packedMcp = fs.realpathSync(path.join(consumer, "node_modules/@vizejs/musea-mcp-server"));
  assert.equal(readPackage(path.join(packedMcp, "package.json")).name, "@vizejs/musea-mcp-server");
  const sourceRoutes = routes(path.join(root, "npm/mcp-musea")),
    packedRoutes = routes(packedMcp);
  const publicProducers = new Map<string, Awaited<ReturnType<typeof published>>>([
    [before.metadata.version, before],
    [fixed.metadata.version, fixed],
  ]);
  report.producers.push(
    { role: "source-routes", routes: sourceRoutes },
    {
      role: "fresh-packed-MCP-routes",
      manifest: readPackage(path.join(packedMcp, "package.json")),
      archiveSha256: hash(fs.readFileSync(archive)),
      routes: packedRoutes,
    },
  );
  save();
  const instances = [...new Set([...sourceRoutes, ...packedRoutes].map((route) => route.qs))];
  for (const [index, directory] of instances.entries()) {
    const version = readPackage(path.join(directory, "package.json")).version;
    assert.match(version, /^6\.\d+\.\d+$/u);
    const [, minor, patch] = version.split(".").map(Number);
    assert(
      minor > 16 || (minor === 16 && patch >= 1),
      "every runtime must exclude the vulnerable range",
    );
    if (directory.startsWith(root)) assert.equal(version, "6.16.1", "exact source override");
    let producer = publicProducers.get(version);
    if (!producer) {
      producer = await published(version);
      publicProducers.set(version, producer);
    }
    const expectedLib = tree(path.join(producer.installed, "lib"));
    const expectedTests = tree(path.join(producer.installed, "test"));
    assert(
      Object.keys(expectedTests).length > 0,
      "the complete published runtime suite is required",
    );
    assert.deepEqual(
      tree(path.join(directory, "lib")),
      expectedLib,
      "actual runtime equals its authenticated public version",
    );
    assert.deepEqual(tree(path.join(directory, "test")), expectedTests);
    const role =
      (directory.startsWith(root) ? "actual-source-runtime-" : "actual-packed-runtime-") + index;
    originalProcesses(directory, role, false);
    matrix(directory, role, false, normalBefore);
  }
  const toolsByVersion = new Map<string, string>();
  for (const [index, directory] of [before.installed, fixed.installed, ...instances].entries()) {
    const version = readPackage(path.join(directory, "package.json")).version;
    const producer = publicProducers.get(version);
    assert(producer?.metadata.devDependencies);
    let tools = toolsByVersion.get(version);
    if (!tools) {
      tools = path.join(path.dirname(output), "qs-upstream-tools-" + version);
      fs.mkdirSync(tools, { recursive: true });
      fs.writeFileSync(
        path.join(tools, "package.json"),
        JSON.stringify({ private: true, dependencies: producer.metadata.devDependencies }) + "\n",
      );
      success(
        run(
          "install-upstream-tools-" + version,
          "npm",
          ["install", "--ignore-scripts", "--no-audit", "--no-fund"],
          tools,
        ),
        "published upstream test prerequisites",
      );
      fs.copyFileSync(
        path.join(tools, "package-lock.json"),
        path.join(output, "upstream-tools-lock-" + version + ".json"),
      );
      toolsByVersion.set(version, tools);
    }
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      NODE_PATH: path.join(tools, "node_modules"),
      PATH: path.join(tools, "node_modules/.bin") + path.delimiter + (process.env.PATH ?? ""),
    };
    success(
      run("complete-upstream-tests-" + index, "npm", ["run", "tests-only"], directory, env),
      "complete unchanged published upstream runtime suite",
    );
  }
  const audit = run("fresh-packed-mcp-audit", "npm", ["audit", "--omit=dev", "--json"], consumer);
  JSON.parse(audit.stdout);
  success(audit, "fresh packed MCP production audit");
  save();
  console.log(
    JSON.stringify({
      qualified: true,
      sourceRevision: revision.stdout.trim(),
      runtimePaths: sourceRoutes.length + packedRoutes.length,
      uniqueRuntimeInstances: instances.length,
      negativeBeforeCases: negative.length,
      controls: controls.length,
      fullUpstreamSuites: 2 + instances.length,
      freshPackedAudit: "clean",
    }),
  );
}
try {
  await main();
} finally {
  save();
}
