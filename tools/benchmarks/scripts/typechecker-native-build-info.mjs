/** Validate an owned cache filename before granting one argv relocation. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  existsSync,
  lstatSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join, parse, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { resolveVuePackageDir } from "./check-gate-env.mjs";

export const BUILD_INFO_SOURCE = {
  path: "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/config_anchor.rs",
  sha256: "47b951bf910b004a543bc2322307fe091efdd9dd6825cce9a2ba534628324b72",
};
const CORPORA = new Set(["shared-barrel500", "shared-leaf501-default", "generated500"]);
const NAMESPACE = ["compiler-sfc", "compiler-vapor"];
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function buildInfoTopologyFingerprint(links) {
  const digest = createHash("sha256").update("vize-cli-build-info-package-topology:v1\0");
  for (const { path, target } of links) {
    digest.update("link\0");
    for (const value of [path, target]) {
      const bytes = Buffer.from(value);
      const size = Buffer.alloc(8);
      size.writeBigUInt64LE(BigInt(bytes.length));
      digest.update(size).update(bytes);
    }
  }
  return digest.digest("hex");
}

function revision(path) {
  const stat = lstatSync(path, { bigint: true });
  return Object.fromEntries(
    ["dev", "ino", "mode", "size", "mtimeNs", "ctimeNs"].map((key) => [key, String(stat[key])]),
  );
}

function assertNoOtherInstalls(root) {
  for (let parent = dirname(root); ; parent = dirname(parent)) {
    assert(!existsSync(join(parent, "node_modules")), "unsupported ancestor package install");
    if (dirname(parent) === parent) break;
  }
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (directory === root && entry.name === "node_modules") continue;
      assert.notEqual(entry.name, "node_modules", "unsupported nested package install");
      assert(!entry.isSymbolicLink(), "unsupported authored source link");
      if (entry.isDirectory()) visit(join(directory, entry.name));
    }
  };
  visit(root);
}

function topology(authoredRoot, virtualRoot, selectedVue) {
  assert.equal(realpathSync(authoredRoot), authoredRoot, "canonical authored root required");
  assert.equal(realpathSync(virtualRoot), virtualRoot, "canonical virtual root required");
  assertNoOtherInstalls(authoredRoot);
  const modules = join(authoredRoot, "node_modules");
  assert(lstatSync(modules).isDirectory(), "real corpus node_modules directory required");
  assert.deepEqual(readdirSync(modules).sort(), ["@vue", "vue"], "unsupported root package shape");
  assert(isAbsolute(selectedVue) && basename(selectedVue) === "vue", "selected Vue path required");
  const namespace = join(dirname(selectedVue), "@vue");
  // The generator uses the unresolved Vue path's sibling namespace. Resolving
  // Vue first silently chooses a different pnpm namespace and cache identity.
  for (const [name, target] of [
    ["vue", selectedVue],
    ["@vue", namespace],
  ]) {
    const link = join(modules, name);
    assert(lstatSync(link).isSymbolicLink(), "corpus dependency link required");
    assert.equal(readlinkSync(link), target, "corpus dependency route changed");
    assert.equal(realpathSync(link), realpathSync(target), "corpus dependency target changed");
  }
  assert.deepEqual(readdirSync(namespace).sort(), NAMESPACE, "unsupported Vue namespace shape");
  const vueBytes = readFileSync(join(realpathSync(selectedVue), "package.json"));
  const vueMetadata = JSON.parse(vueBytes);
  assert.equal(vueMetadata.name, "vue", "selected Vue package identity changed");
  assert.equal(typeof vueMetadata.version, "string", "selected Vue version required");
  const links = NAMESPACE.map((name) => {
    const path = join(virtualRoot, "node_modules", "@vue", name);
    const target = realpathSync(join(namespace, name));
    assert(lstatSync(path).isSymbolicLink(), "producer package link required");
    assert.equal(readlinkSync(path), target, "producer package link retargeted");
    assert.equal(realpathSync(path), target, "producer canonical target changed");
    const packageBytes = readFileSync(join(target, "package.json"));
    const metadata = JSON.parse(packageBytes);
    assert.equal(metadata.name, "@vue/" + name, "dependency package identity changed");
    assert.equal(metadata.version, vueMetadata.version, "dependency package version changed");
    return {
      path,
      target,
      packageSha256: sha(packageBytes),
      revision: revision(path),
      targetRevision: revision(target),
    };
  });
  const receiptPath = join(virtualRoot, ".vize-buildinfo", "package-links.json");
  assert.equal(
    realpathSync(dirname(receiptPath)),
    dirname(receiptPath),
    "owned receipt directory required",
  );
  assert(lstatSync(receiptPath).isFile(), "regular producer receipt required");
  const receiptBytes = readFileSync(receiptPath);
  const receipt = JSON.parse(receiptBytes);
  assert.deepEqual(Object.keys(receipt).sort(), ["links", "version"], "unknown receipt schema");
  assert.equal(receipt.version, 1, "unknown producer receipt version");
  assert.deepEqual(
    receipt.links,
    links.map(({ path, target }) => ({ path, target })),
    "producer topology differs",
  );
  return {
    links,
    receiptSha256: sha(receiptBytes),
    receiptRevision: revision(receiptPath),
    namespaceRevision: revision(namespace),
    modulesRevision: revision(modules),
    vueSha256: sha(vueBytes),
    dependencyRoutes: [readlinkSync(join(modules, "vue")), readlinkSync(join(modules, "@vue"))],
  };
}

export function canonicalBuildInfoArgument({
  shard,
  authoredRoot,
  corpusId,
  selectedVuePackageDir,
  sourceRoot = process.env.NATIVE_PHASE_SOURCE_ROOT ?? ROOT,
}) {
  const args = shard.args;
  assert(Array.isArray(args) && args.every((arg) => typeof arg === "string"));
  const positions = args.flatMap((arg, index) => (arg === "--tsBuildInfoFile" ? [index] : []));
  if (positions.length === 0 && !CORPORA.has(corpusId)) return null;
  assert(CORPORA.has(corpusId), "unsupported build-info corpus");
  assert.deepEqual(positions, [3], "build-info flag position changed");
  assert.equal(args.length, 9, "unknown native argument shape");
  assert.equal(args[0], "--checkers");
  assert.match(args[1], /^[1-9]\d*$/u);
  assert.equal(args[2], "--incremental");
  assert.deepEqual(
    args.slice(5, 8),
    ["--pretty", "false", "--project"],
    "adjacent native flags changed",
  );
  assert.equal(args[8], shard.config, "native config argument changed");
  assert.equal(
    sha(readFileSync(join(sourceRoot, BUILD_INFO_SOURCE.path))),
    BUILD_INFO_SOURCE.sha256,
    "unknown build-info derivation source",
  );
  const virtualRoot = shard.cwd;
  assert(isAbsolute(virtualRoot) && basename(dirname(virtualRoot)) === "projects");
  const owner = sha(
    Buffer.concat([Buffer.from("vize-canon-project-key:v1\0"), Buffer.from(authoredRoot)]),
  );
  assert.equal(basename(virtualRoot), owner, "build-info virtual owner changed");
  assert(isAbsolute(shard.config) && resolve(shard.config) === shard.config);
  const config = relative(virtualRoot, shard.config);
  assert(
    config !== "" && !isAbsolute(config) && !config.split(sep).includes(".."),
    "foreign build-info config",
  );
  const parsed = parse(config);
  const stem = join(parsed.dir, parsed.name + ".tsbuildinfo")
    .split(sep)
    .join("__")
    .slice(0, -".tsbuildinfo".length);
  const selectedVue = selectedVuePackageDir ?? resolveVuePackageDir();
  assert(selectedVue, "selected Vue dependency missing");
  const observed = topology(authoredRoot, virtualRoot, selectedVue);
  const file = stem + "." + buildInfoTopologyFingerprint(observed.links) + ".tsbuildinfo";
  assert.equal(
    args[4],
    join(virtualRoot, ".vize-buildinfo", file),
    "source-derived build-info filename differs",
  );
  assert.deepEqual(
    topology(authoredRoot, virtualRoot, selectedVue),
    observed,
    "build-info topology changed during validation",
  );
  const relocated = observed.links.map(({ path, target }) => ({
    path: relative(virtualRoot, path).split(sep).join("/"),
    target,
  }));
  // Only the proven owner-dependent cache name moves. Literal canonical
  // external targets participate in this identity; cache contents do not.
  return (
    "<virtual>/.vize-buildinfo/" +
    stem +
    "." +
    buildInfoTopologyFingerprint(relocated) +
    ".tsbuildinfo"
  );
}
