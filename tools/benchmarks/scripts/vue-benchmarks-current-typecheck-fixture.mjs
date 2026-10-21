import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

import { fileEvidence, sha256 } from "./vue-benchmarks-current-typecheck-capture.mjs";

export const UPSTREAM_REVISION = "5489aee433cd1054b9d72973457498544da7c467";
export const FIXTURE_PATH = "tests/_fixtures/_git/vue-benchmarks";
export const VUE_VERSION = "3.5.43";
export const GOLAR_VERSION = "0.1.10";

export function git(directory, ...args) {
  return execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trimEnd();
}

function escapesRoot(relativePath) {
  return relativePath === ".." || relativePath.startsWith(`..${sep}`) || isAbsolute(relativePath);
}

export function assertOutside(root, protectedRoot) {
  const rel = relative(resolve(protectedRoot), resolve(root));
  assert.ok(escapesRoot(rel), "work output overlaps read-only fixture");
  const inverse = relative(resolve(root), resolve(protectedRoot));
  assert.ok(escapesRoot(inverse), "work output contains fixture");
}

export function assertPristine(upstream) {
  assert.equal(git(upstream, "rev-parse", "HEAD"), UPSTREAM_REVISION);
  assert.equal(git(upstream, "status", "--porcelain", "--untracked-files=all"), "");
}

export function sourceInventory(upstream) {
  const selected = [
    "LICENSE",
    "package.json",
    "pnpm-lock.yaml",
    "results/benchmarks/confirm.json",
    "tests/confirm/fixtures/typecheck",
    "tests/confirm/suites/typecheck.mjs",
    "tests/confirm/lib",
    "scripts/lib",
  ];
  return git(upstream, "ls-tree", "-r", "HEAD", "--", ...selected)
    .split("\n")
    .map((row) => {
      const match = row.match(/^100644 blob ([0-9a-f]{40})\t(.+)$/);
      assert.ok(match, `unexpected upstream source entry: ${row}`);
      const [, gitBlob, path] = match;
      const bytes = readFileSync(join(upstream, path));
      assert.equal(git(upstream, "hash-object", "--", path), gitBlob);
      return { path, gitBlob, bytes: bytes.length, sha256: sha256(bytes) };
    });
}

export function workspaceInventory(root, allowSymlinks = false) {
  const files = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const absolute = join(directory, entry.name);
      if (entry.isDirectory()) visit(absolute);
      else if (allowSymlinks && entry.isSymbolicLink()) {
        const target = realpathSync(absolute);
        const rel = relative(root, target);
        assert.ok(!escapesRoot(rel), "provider link escapes pinned installation");
        assert.ok(
          statSync(target).isFile(),
          "provider directory link requires an explicit closure audit",
        );
        files.push({
          ...fileEvidence(target),
          path: relative(root, absolute),
          kind: "symbolic-link",
          target: readlinkSync(absolute),
          resolved: rel,
        });
      } else {
        assert.ok(entry.isFile(), `source input is not an ordinary file: ${absolute}`);
        files.push({ ...fileEvidence(absolute), path: relative(root, absolute) });
      }
    }
  };
  visit(root);
  return files.sort((left, right) =>
    left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
  );
}

function providerLinkEvidence(workRoot, providerRoot) {
  const versions = [
    ["golar", "golar/package.json"],
    ["@golar/vue", "@golar/vue/package.json"],
  ].map(([name, manifest]) => {
    const absolute = join(providerRoot, ...manifest.split("/"));
    const parsed = JSON.parse(readFileSync(absolute));
    assert.equal(parsed.name, name);
    assert.equal(parsed.version, GOLAR_VERSION);
    return { name, version: parsed.version, manifest: fileEvidence(absolute) };
  });
  const links = ["golar", "@golar"].map((name) => {
    const link = join(workRoot, "node_modules", name);
    const source = join(providerRoot, name);
    assert.equal(realpathSync(link), realpathSync(source));
    return {
      path: relative(workRoot, link),
      target: relative(workRoot, source),
      resolved: realpathSync(link),
    };
  });
  return { versions, links };
}

export function materializePinnedProvider(workRoot, providerRoot) {
  assertOutside(workRoot, providerRoot);
  const nodeModules = join(workRoot, "node_modules");
  assert.equal(existsSync(nodeModules), false, "copied workspace already has node_modules");
  mkdirSync(nodeModules);
  for (const name of ["golar", "@golar"]) {
    const source = join(providerRoot, name);
    assert.ok(statSync(source).isDirectory(), `missing pinned provider package: ${name}`);
    symlinkSync(source, join(nodeModules, name), "dir");
  }
  return providerLinkEvidence(workRoot, providerRoot);
}

export function inspectPinnedProvider(workRoot, providerRoot) {
  return providerLinkEvidence(workRoot, providerRoot);
}

export function archiveInputs(upstream, inventory, destination) {
  for (const { path, sha256: expected } of inventory) {
    const bytes = readFileSync(join(upstream, path));
    assert.equal(sha256(bytes), expected);
    const target = join(destination, path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
  }
}

export async function prepareFixture(repository, workRoot, vueDirectory) {
  const manifest = JSON.parse(
    readFileSync(join(repository, "tests/_fixtures/vue-benchmarks-upstream.json")),
  );
  assert.equal(manifest.revision, UPSTREAM_REVISION);
  assert.equal(manifest.fixturePath, FIXTURE_PATH);
  const upstream = join(repository, FIXTURE_PATH);
  assert.equal(
    git(repository, "ls-tree", "HEAD", "--", FIXTURE_PATH),
    `160000 commit ${UPSTREAM_REVISION}\t${FIXTURE_PATH}`,
  );
  assertPristine(upstream);
  assertOutside(workRoot, upstream);
  const vueManifest = JSON.parse(readFileSync(join(vueDirectory, "package.json")));
  assert.equal(vueManifest.name, "vue");
  assert.equal(vueManifest.version, VUE_VERSION);
  const upstreamPackage = JSON.parse(readFileSync(join(upstream, "package.json")));
  assert.equal(upstreamPackage.dependencies.vue, VUE_VERSION);
  const inventory = sourceInventory(upstream);
  const suite = await import(pathToFileURL(join(upstream, "tests/confirm/suites/typecheck.mjs")));
  const diagnostics = await import(
    pathToFileURL(join(upstream, "tests/confirm/lib/diagnostics.mjs"))
  );
  const ansi = await import(pathToFileURL(join(upstream, "scripts/lib/real-world/ansi.mjs")));
  const fixture = suite.prepareAllPlants(workRoot);
  assert.equal(fixture.cases.length, 154);
  const published = JSON.parse(
    readFileSync(join(upstream, "results/benchmarks/confirm.json")),
  ).results.find((row) => row.suite === "typecheck-all" && row.tool === "vize-check");
  assert.ok(published);
  assert.deepEqual(
    fixture.cases.map(({ caseId }) => caseId),
    published.detail.plants.map(({ caseId }) => caseId),
  );
  // Upstream already transports the Vue package through paths. Change only
  // that physical installation address in copied configs, never the sources.
  const configs = ["tsconfig.json", fixture.fallthroughTsconfig].map((path) => {
    const absolute = join(workRoot, path);
    const config = JSON.parse(readFileSync(absolute));
    const original = structuredClone(config);
    config.compilerOptions.paths = {
      vue: [vueDirectory.replaceAll("\\", "/")],
      "vue/*": [join(vueDirectory, "*").replaceAll("\\", "/")],
    };
    const expected = structuredClone(original);
    expected.compilerOptions.paths = config.compilerOptions.paths;
    assert.deepEqual(config, expected);
    writeFileSync(absolute, `${JSON.stringify(config, null, 2)}\n`);
    return { ...fileEvidence(absolute), original, effective: config };
  });
  const packagePath = join(workRoot, "package.json");
  const originalPackage = JSON.parse(readFileSync(packagePath));
  assert.equal(
    originalPackage.dependencies.vue,
    `file:${join(upstream, "node_modules/vue").replaceAll("\\", "/")}`,
  );
  const effectivePackage = structuredClone(originalPackage);
  effectivePackage.dependencies.vue = `file:${vueDirectory.replaceAll("\\", "/")}`;
  writeFileSync(packagePath, `${JSON.stringify(effectivePackage, null, 2)}\n`);
  configs.push({
    ...fileEvidence(packagePath),
    original: originalPackage,
    effective: effectivePackage,
  });
  assertPristine(upstream);
  return {
    ...fixture,
    upstream,
    inventory,
    configs,
    vue: { ...fileEvidence(join(vueDirectory, "package.json")), version: VUE_VERSION },
    published,
    score: suite.scoreCombinedRun,
    parse: diagnostics.parseDiagnostics,
    bootstrapFailure: diagnostics.isToolBootstrapFailure,
    stripAnsi: ansi.stripAnsi,
  };
}
