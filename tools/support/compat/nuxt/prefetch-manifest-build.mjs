// #7999: original SPA routes, whole manifest and physical Chromium prefetches.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { prepareNuxtSourceBinding } from "./source-binding.mjs";
import { assertPrefetchManifest, observePrefetchManifest } from "./prefetch-manifest-observe.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixtureDirectory = "tools/support/compat/nuxt/fixtures/nuxt-prefetch-manifest";
const fixture = path.join(root, fixtureDirectory);
const dependencies = path.join(
  root,
  "tests/_fixtures/_projects/nuxt-critical-css-build/node_modules",
);
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-prefetch"));
assert.equal(fs.existsSync(artifacts), false, "each proof must retain a new complete packet");
fs.mkdirSync(artifacts, { recursive: true });
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const write = (directory, name, value) =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n");
const git = (...args) => execFileSync("git", args, { cwd: root }).toString().trim();
const prior = "4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf";
const priorArchive = path.join(artifacts, "prior-nuxt-source.tar");
const priorRoot = path.join(artifacts, "prior-source");
const priorPackage = path.join(priorRoot, "npm/framework/nuxt");

function run(command, args, cwd, environment, log) {
  const fd = fs.openSync(log, "w");
  let result;
  try {
    result = spawnSync(command, args, {
      cwd,
      env: environment,
      stdio: ["ignore", fd, fd],
      timeout: 240_000,
    });
  } finally {
    fs.closeSync(fd);
  }
  write(path.dirname(log), path.basename(log) + ".process.json", {
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `actual command failed; retain ${log}`);
}
// Ordinary Actions checkouts are shallow; fetch the literal historical source.
run(
  "git",
  ["fetch", "--no-tags", "--depth=1", "origin", prior],
  root,
  process.env,
  path.join(artifacts, "prior-source-fetch.log"),
);
execFileSync(
  "git",
  ["archive", "--format=tar", `--output=${priorArchive}`, prior, "npm/framework/nuxt"],
  { cwd: root },
);
fs.mkdirSync(priorRoot);
execFileSync("tar", ["-xf", priorArchive, "-C", priorRoot]);
fs.symlinkSync(
  path.join(root, "npm/framework/nuxt/node_modules"),
  path.join(priorPackage, "node_modules"),
  "dir",
);
run("vp", ["pack"], priorPackage, process.env, path.join(artifacts, "prior-pack.log"));

const versions = {};
for (const [name, version] of [
  ["nuxt", "4.5.2"],
  ["vue", "3.5.43"],
  ["vite", "8.3.1"],
  ["@vizejs/nuxt", "0.429.0"],
  ["@vizejs/native", "0.429.0"],
]) {
  versions[name] = JSON.parse(
    fs.readFileSync(path.join(dependencies, name, "package.json")),
  ).version;
  assert.equal(versions[name], version);
}
const corpusDirectory = path.join(
  root,
  "tests/_fixtures/differential/compiler/nuxt-prefetch-manifest",
);
const corpus = JSON.parse(fs.readFileSync(path.join(corpusDirectory, "runtime.expected.json")));
assert.equal(corpus.issue, 7999);
assert.equal(corpus.priorSource, prior);
for (const input of corpus.fixtures)
  assert.equal(sha256(fs.readFileSync(path.join(root, input.path))), input.sha256);
assert.equal(
  sha256(fs.readFileSync(path.join(corpusDirectory, "original-issue.md"))),
  corpus.originalIssueSha256,
);
for (const name of ["runtime.expected.json", "original-issue.md"])
  fs.copyFileSync(path.join(corpusDirectory, name), path.join(artifacts, name));
const files = git("ls-files", "--", fixtureDirectory).split("\n").filter(Boolean);
const inputs = Object.fromEntries(
  files.map((name) => [
    name,
    {
      sha256: sha256(fs.readFileSync(path.join(root, name))),
      bytes: fs.readFileSync(path.join(root, name)).toString("base64"),
    },
  ]),
);
const fixtures = files.filter((name) => name.endsWith(".vue"));
assert.equal(fixtures.length, 4);
const packageHashes = (directory) =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => {
        const filename = path.join(entry.parentPath, entry.name);
        return [path.relative(directory, filename), sha256(fs.readFileSync(filename))];
      }),
  );
const candidateNuxt = path.join(root, "npm/framework/nuxt/dist");
const candidateVite = path.join(root, "npm/builder/vite/dist");
const packages = { nuxt: packageHashes(candidateNuxt), vite: packageHashes(candidateVite) };
const nodeModules = path.join(fixture, "node_modules");
assert.equal(
  fs.existsSync(nodeModules),
  false,
  "a previous generated fixture install cannot count",
);
fs.mkdirSync(path.join(nodeModules, "@vizejs"), { recursive: true });
for (const name of fs.readdirSync(dependencies)) {
  if (["@vizejs", ".cache"].includes(name)) continue;
  fs.symlinkSync(path.join(dependencies, name), path.join(nodeModules, name), "dir");
}
for (const name of fs.readdirSync(path.join(dependencies, "@vizejs"))) {
  const target = path.join(nodeModules, "@vizejs", name);
  const installed = path.join(dependencies, "@vizejs", name);
  if (["nuxt", "vite-plugin"].includes(name)) fs.cpSync(installed, target, { recursive: true });
  else fs.symlinkSync(installed, target, "dir");
}
const originalNuxt = packageHashes(path.join(dependencies, "@vizejs/nuxt"));
const originalVite = packageHashes(path.join(dependencies, "@vizejs/vite-plugin"));
const scopes = {},
  native = {};
try {
  for (const arm of ["stock", "prior", "candidate"]) {
    const directory = path.join(artifacts, arm);
    fs.mkdirSync(directory);
    for (const [name, source] of [
      ["nuxt", arm === "prior" ? path.join(priorPackage, "dist") : candidateNuxt],
      ["vite-plugin", candidateVite],
    ]) {
      const target = path.join(nodeModules, "@vizejs", name, "dist");
      fs.rmSync(target, { recursive: true });
      fs.cpSync(source, target, { recursive: true });
    }
    for (const name of [".nuxt", ".output", ".vize"])
      fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
    const binding =
      arm === "stock"
        ? null
        : prepareNuxtSourceBinding(root, directory, { fixtureDirectory, backends: ["client"] });
    run(
      process.execPath,
      [path.join(fixture, "node_modules/nuxt/bin/nuxt.mjs"), "generate"],
      fixture,
      {
        ...(binding?.environment ?? process.env),
        VIZE_COMPILER: arm === "stock" ? "0" : "1",
        NO_COLOR: "1",
        NUXT_TELEMETRY_DISABLED: "1",
      },
      path.join(directory, "generate.log"),
    );
    // Preserve the entire failed arm before assertions; no observation is waived.
    const result = await observePrefetchManifest(fixture, root, directory);
    write(directory, "observation.json", result);
    if (binding) native[arm] = binding.verify();
    scopes[arm] = assertPrefetchManifest(result, arm === "prior");
    for (const [name, original] of Object.entries(inputs))
      assert.equal(sha256(fs.readFileSync(path.join(root, name))), original.sha256);
  }
  assert.deepEqual(
    scopes.candidate.dynamicImports,
    scopes.stock.dynamicImports,
    "complete entry dynamic imports must match stock Nuxt",
  );
  assert.ok(scopes.prior.prefetch.length > scopes.candidate.prefetch.length);
  assert.equal(scopes.candidate.prefetch.length, scopes.stock.prefetch.length);
  assert.deepEqual(
    scopes.candidate.prefetchOwnership,
    scopes.stock.prefetchOwnership,
    "complete ordered manifest-owned prefetch vector must match stock Nuxt",
  );
  assert.deepEqual(packageHashes(path.join(dependencies, "@vizejs/nuxt")), originalNuxt);
  assert.deepEqual(packageHashes(path.join(dependencies, "@vizejs/vite-plugin")), originalVite);
  write(artifacts, "proof.json", {
    issue: 7999,
    sourceHead: git("rev-parse", "HEAD"),
    sourceTree: git("rev-parse", "HEAD^{tree}"),
    node: process.version,
    versions,
    reporterVue: "3.5.42",
    actualVue: "3.5.43",
    originalRoutes: ["/", "/reports"],
    inputs,
    sourceNative: native,
    prior: {
      head: prior,
      nuxtTree: git("rev-parse", `${prior}:npm/framework/nuxt`),
      archiveSha256: sha256(fs.readFileSync(priorArchive)),
      dist: packageHashes(path.join(priorPackage, "dist")),
    },
    candidatePackages: packages,
    scopes,
    oldPinnedPackagesUnchanged: true,
    reporterLargeAppCountsMeasured: false,
    ssrCredit: false,
  });
  console.log(
    "Original #7999 SPA: stock/prior/current full manifest, genuine source native and Chromium prefetch verified",
  );
} finally {
  fs.rmSync(nodeModules, { recursive: true, force: true });
}
