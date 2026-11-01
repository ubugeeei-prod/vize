import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { posix as path } from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { lockPackageIntegrity } from "./lock-integrity.ts";
import { gitBlob, object, sha256, text, wholeJson } from "./value.ts";
import { absoluteUnix, derivedNodePath, pnpmSource, within } from "./pnpm-shim-render.ts";
import type {
  DerivedBin,
  FileFacts,
  HumanBinFacts,
  PublicPackageFacts,
} from "./dependency-bin-types.ts";
import { validateBinConfiguration } from "./bin-configuration.ts";

export function regular(file: FileFacts, expected: string): Buffer {
  assert.equal(file.path, expected);
  assert.equal(file.realPath, expected);
  assert.equal(file.kind, "file");
  assert(file.body !== null);
  return file.body;
}
export function publicFiles(pkg: PublicPackageFacts): Map<string, Buffer> {
  assert(pkg.archive, "actual public tarball required in the live installer");
  absoluteUnix(pkg.directory);
  assert.match(pkg.resolved, /^https:\/\/registry\.npmjs\.org\/[^?#]+$/u);
  assert.equal(new URL(pkg.resolved).origin, "https://registry.npmjs.org");
  const integrity = "sha512-" + createHash("sha512").update(pkg.archive).digest("base64");
  assert.equal(integrity, pkg.integrity);
  const files = new Map<string, Buffer>();
  for (const member of readTarGz(pkg.archive)) {
    assert(member.name.startsWith("package/") && !member.name.includes("\\"));
    const relative = member.name.slice(8);
    assert(relative && relative.split("/").every((part) => part && part !== "." && part !== ".."));
    assert(member.kind === "file" || member.kind === "directory");
    if (member.kind === "directory") continue;
    assert(!files.has(relative));
    files.set(relative, member.data);
  }
  const manifest = files.get("package.json");
  assert(manifest && manifest.equals(pkg.manifest));
  const value = object(wholeJson(manifest));
  assert.equal(value.name, pkg.name);
  assert.equal(value.version, pkg.version);
  return files;
}
function completeOwnerCensus(facts: HumanBinFacts): void {
  const parent = object(wholeJson(facts.parent.manifest));
  assert.equal(parent.bin, undefined, "parent-owned bins need a separately reviewed branch");
  const declarations = new Map<string, string>();
  for (const kind of ["dependencies", "optionalDependencies", "peerDependencies"]) {
    if (parent[kind] === undefined) continue;
    for (const [name, constraint] of Object.entries(object(parent[kind]))) {
      const value = text(constraint),
        prior = declarations.get(name);
      assert(prior === undefined || prior === value, "conflicting parent declarations");
      declarations.set(name, value);
    }
  }
  const names = [...declarations.keys()].sort();
  assert.deepEqual(
    facts.dependencyOwners.map((row) => row.name).sort(),
    names,
    "complete authenticated parent dependency/optional/peer owner census required",
  );
  assert.equal(facts.graph.length, names.length);
  const candidates = new Map<string, string[]>();
  for (const owner of facts.dependencyOwners) {
    assert(within(facts.root, owner.directory));
    assert.equal(
      lockPackageIntegrity(facts.lock, owner.name, owner.version).integrity,
      owner.integrity,
    );
    assert(
      declarations.get(owner.name) === "*" || declarations.get(owner.name) === owner.version,
      "unsupported or mismatched source declaration constraint",
    );
    const edges = facts.graph.filter((row) => row.dependencyName === owner.name);
    assert.equal(edges.length, 1);
    assert.deepEqual(edges[0], {
      parentDirectory: facts.parent.directory,
      dependencyName: owner.name,
      version: owner.version,
      directory: owner.directory,
    });
    const manifest = object(wholeJson(owner.manifest));
    if (manifest.bin === undefined) continue;
    for (const name of Object.keys(object(manifest.bin))) {
      const owners = candidates.get(name) ?? [];
      owners.push(owner.directory);
      candidates.set(name, owners);
    }
  }
  const selected = facts.dependencyOwners.filter((row) => row.name === facts.owner.name);
  assert.equal(selected.length, 1);
  assert.equal(selected[0]!.directory, facts.owner.directory);
  assert(selected[0]!.manifest.equals(facts.owner.manifest));
  assert.equal(selected[0]!.integrity, facts.owner.integrity);
  const declared = object(object(wholeJson(facts.owner.manifest)).bin);
  assert.deepEqual(
    [...candidates.keys()].sort(),
    Object.keys(declared).sort(),
    "another dependency declares an unregistered executable",
  );
  for (const owners of candidates.values())
    assert.deepEqual(owners, [facts.owner.directory], "ambiguous executable owner/winner");
}
function observeManager(facts: HumanBinFacts): void {
  const manager = facts.manager;
  assert(manager, "manager route UNKNOWN");
  assert.equal(manager.reportedVersion, "12.16.1");
  assert(manager.versionStdout.equals(Buffer.from("12.16.1\n")));
  assert.equal(manager.versionStatus, 0);
  assert.equal(manager.versionSignal, null);
  assert.equal(manager.versionError, null);
  assert.equal(manager.versionStderr.length, 0);
  assert.deepEqual(manager.originalArgv, ["install"]);
  const current = regular(manager.currentImage, manager.originalImage.path);
  assert.equal(manager.originalImage.path, manager.currentImage.path);
  assert.equal(manager.originalImage.bytes, current.length);
  assert.equal(manager.originalImage.sha256, sha256(current));
  // A source-conformant derivative does not prove which writer ran. Retain
  // the actual launcher observation without requiring a guessed JS/native
  // owner or attributing its target image to the generator's source tag.
}
function inputBinding(facts: HumanBinFacts) {
  return {
    role: facts.role,
    host: facts.host,
    parent: {
      name: facts.parent.name,
      version: facts.parent.version,
      manifestSha256: sha256(facts.parent.manifest),
    },
    owner: {
      name: facts.owner.name,
      version: facts.owner.version,
      manifestSha256: sha256(facts.owner.manifest),
    },
    dependencyNames: facts.dependencyOwners.map((row) => row.name).sort(),
    rootConstruction: "OUTPUT/consumer-KIND",
    generatorOutputObservedAsInput: false,
  };
}

/** Input-only policy from the independently authenticated reviewed-controller
 * source closure. No shim bytes/hash/diagnostic/output is permitted in it.
 * Missing authority or an unknown relevant input refuses body conformance. */
function deriveMetadataBins(facts: HumanBinFacts): DerivedBin[] {
  assert.equal(facts.configuration.root, facts.root);
  assert.equal(facts.configuration.platform, facts.host.platform);
  validateBinConfiguration(facts.configuration, facts.settings.rootPackage);
  absoluteUnix(facts.root);
  assert(
    [
      "consumer-original",
      ...(facts.role === "runtime-prop-null-11252" ? ["consumer-tsrs-supplement"] : []),
    ].includes(path.basename(facts.root)),
    "source-owned consumer root construction required",
  );
  absoluteUnix(facts.parent.directory);
  absoluteUnix(facts.owner.directory);
  assert(within(facts.root, facts.parent.directory) && within(facts.root, facts.owner.directory));
  const version = facts.role === "runtime-prop-null-11252" ? "7.1.0-beta" : "5.9.3";
  assert(["runtime-prop-null-11252", "imports-vue-ts-11253"].includes(facts.role));
  assert.deepEqual(
    facts.host,
    facts.role === "runtime-prop-null-11252"
      ? { platform: "darwin", arch: "arm64" }
      : { platform: "linux", arch: "x64" },
  );
  assert.equal(facts.parent.name, "vue");
  assert.equal(facts.parent.version, "3.5.44");
  assert.equal(facts.owner.name, "typescript");
  assert.equal(facts.owner.version, version);
  assert.equal(lockPackageIntegrity(facts.lock, "vue", "3.5.44").integrity, facts.parent.integrity);
  assert.equal(
    lockPackageIntegrity(facts.lock, "typescript", version).integrity,
    facts.owner.integrity,
  );
  const parent = object(wholeJson(facts.parent.manifest));
  assert(
    [parent.dependencies, parent.optionalDependencies, parent.peerDependencies].some(
      (value) => value !== undefined && typeof object(value).typescript === "string",
    ),
    "parent has no authenticated TypeScript dependency declaration",
  );
  completeOwnerCensus(facts);
  const owner = object(wholeJson(facts.owner.manifest)),
    declared = object(owner.bin);
  const names = Object.keys(declared).sort();
  assert(names.length > 0 && names.every((name) => ["tsc", "tsserver"].includes(name)));
  const bins = names.map((name) => {
    const relative = text(declared[name]).replace(/^\.\//u, "");
    assert(
      relative &&
        relative.split("/").every((part) => part && part !== "." && part !== "..") &&
        !relative.includes("\\"),
    );
    const target = path.join(facts.owner.directory, relative);
    const matches = facts.targets.filter((file) => file.path === target);
    assert.equal(matches.length, 1);
    const actual = regular(matches[0]!, target);
    const firstLine = actual
      .subarray(0, actual.indexOf(10) === -1 ? actual.length : actual.indexOf(10))
      .toString("utf8");
    const runtime: "node" | null = firstLine === "#!/usr/bin/env node" ? "node" : null;
    assert(
      runtime === "node" || (!firstLine.startsWith("#!") && !/\.(?:[cm]?js)$/u.test(relative)),
      "unsupported or unknown target runtime",
    );
    return { name, target, hash: sha256(actual), runtime };
  });
  assert.equal(facts.targets.length, bins.length, "extra/duplicate unowned target");
  observeManager(facts);
  assert.equal(facts.settings.userConfig.length, 0);
  assert.equal(facts.settings.globalConfig.length, 0);
  assert.equal(facts.settings.rootNpmrc, null, "unowned .npmrc options refuse");
  const rootPackage = object(wholeJson(facts.settings.rootPackage));
  assert.equal(rootPackage.pnpm, undefined);
  const workspace = facts.settings.rootWorkspace.toString("utf8");
  assert(
    workspace === "" || /^minimumReleaseAgeExclude:\n(?:  - '[^'\n]+'\n)+$/u.test(workspace),
    "unknown workspace generator options",
  );
  assert(
    facts.inputPolicyRaw && facts.expectedInputPolicyGitBlob,
    "source-owned input/options authority UNKNOWN",
  );
  assert.match(facts.expectedInputPolicyGitBlob, /^[a-f0-9]{40}$/u);
  assert.equal(gitBlob(facts.inputPolicyRaw), facts.expectedInputPolicyGitBlob);
  const plan = object(wholeJson(facts.inputPolicyRaw));
  assert.deepEqual(
    Object.keys(plan).sort(),
    ["bindings", "options", "schema", "sourceCommit"].sort(),
  );
  assert.equal(plan.schema, "vize.humanPnpm12161InputPolicy.v1");
  assert.equal(plan.sourceCommit, pnpmSource.sourceCommit);
  const bindings = object(plan.bindings);
  assert(
    Object.keys(bindings).every((role) =>
      ["runtime-prop-null-11252", "imports-vue-ts-11253"].includes(role),
    ),
  );
  assert.deepEqual(bindings[facts.role], inputBinding(facts));
  const options = object(plan.options);
  assert.deepEqual(
    options,
    {
      extraNodePaths: ["CONSUMER_VIRTUAL_NODE_MODULES"],
      preferSymlinkedExecutables: false,
      projectModulesDirName: null,
      relocatableRoot: "CONSUMER_ROOT",
    },
    "options must follow the closed pre-install source inputs",
  );
  assert.deepEqual(
    Object.keys(options).sort(),
    [
      "extraNodePaths",
      "preferSymlinkedExecutables",
      "projectModulesDirName",
      "relocatableRoot",
    ].sort(),
  );
  assert.equal(options.preferSymlinkedExecutables, false);
  assert.equal(options.projectModulesDirName, null);
  assert(Array.isArray(options.extraNodePaths));
  const extra = options.extraNodePaths.map((value) => {
    assert.equal(value, "CONSUMER_VIRTUAL_NODE_MODULES");
    return path.join(facts.root, "node_modules/.pnpm/node_modules");
  });
  assert(new Set(extra).size === extra.length && extra.every((value) => within(facts.root, value)));
  const relocatableRoot =
    options.relocatableRoot === null
      ? null
      : (() => {
          assert.equal(options.relocatableRoot, "CONSUMER_ROOT");
          return facts.root;
        })();
  assert(relocatableRoot === null || relocatableRoot === facts.root);
  return bins.map((bin) => ({
    name: bin.name,
    target: bin.target,
    publicTargetSha256: bin.hash,
    runtime: bin.runtime,
    shimPath: path.join(facts.parent.directory, "node_modules/.bin", bin.name),
    nodePath: derivedNodePath(facts.owner.directory, extra),
    relocatableRoot,
  }));
}

/** Live path: every owner manifest/target is authenticated against the actual
 * complete public tarball before the shared metadata derivation is reached. */
export function deriveHumanBins(facts: HumanBinFacts): DerivedBin[] {
  assert.equal(publicFiles(facts.parent).size, 37);
  for (const owner of facts.dependencyOwners) publicFiles(owner);
  const files = publicFiles(facts.owner);
  for (const target of facts.targets) {
    const relative = path.relative(facts.owner.directory, target.path);
    const expected = files.get(relative);
    assert(
      expected && regular(target, target.path).equals(expected),
      "target differs from its whole public member",
    );
  }
  return deriveMetadataBins(facts);
}

/** Archive-only path. Its caller must join all manifests/targets to the
 * authenticated controller's recorded archive/file witnesses; no historical
 * path is read and no omitted tarball or payload is claimed rehashed here. */
export function deriveRecordedHumanBins(facts: HumanBinFacts): DerivedBin[] {
  for (const owner of [facts.parent, facts.owner, ...facts.dependencyOwners])
    assert.equal(owner.archive, null);
  return deriveMetadataBins(facts);
}

/** Pure authoring/fixture binding from declared public inputs. This does not
 * authenticate a policy or create its expected reviewed-controller Git blob. */
export function sourceInputBinding(facts: HumanBinFacts) {
  return inputBinding(facts);
}
