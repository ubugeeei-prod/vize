/** Separate duplicate-tree profiling; never a timed speed or cache experiment. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync, realpathSync } from "node:fs";
import { basename, dirname, isAbsolute, join, resolve, sep } from "node:path";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
function virtualKey(root) {
  assert.equal(Buffer.from(root).toString("utf8"), root, "unsupported root encoding");
  return sha(Buffer.concat([Buffer.from("vize-canon-project-key:v1\0"), Buffer.from(root)]));
}
function graphVector(shards, authoredRoot) {
  authoredRoot = realpathSync(authoredRoot);
  return shards.map((shard) => {
    assert(isAbsolute(shard.cwd), "absolute native cwd required");
    assert.equal(
      basename(shard.cwd),
      virtualKey(authoredRoot),
      "unexpected virtual root ownership",
    );
    assert.equal(basename(dirname(shard.cwd)), "projects", "unsupported native storage topology");
    const raw = readFileSync(shard.nativeGraphArchive.record);
    assert.equal(sha(raw), shard.nativeGraphArchive.sha256, "native graph receipt changed");
    const archive = JSON.parse(raw);
    assert.equal(archive.cwd, shard.cwd, "native graph cwd mismatch");
    assert.deepEqual(archive.args, shard.args, "native graph argv mismatch");
    assert.equal(archive.config.path, shard.config, "native graph config mismatch");
    assert.equal(archive.config.sha256, shard.configSha256, "native graph config digest mismatch");
    assert.equal(
      archive.runtime.sha256,
      shard.runtimeSha256,
      "native graph runtime digest mismatch",
    );
    assert.deepEqual(
      archive.membersBefore.map((member) => ({
        path: resolve(shard.cwd, member.reportedPath),
        bytes: member.bytes,
        sha256: member.sha256,
      })),
      shard.members,
      "native graph member summary mismatch",
    );
    assert.deepEqual(archive.membersAfter, archive.membersBefore, "unstable native graph receipt");
    const relocate = (path) => {
      for (const [root, marker] of [
        [shard.cwd, "<virtual>"],
        [authoredRoot, "<authored>"],
      ]) {
        if (path === root) return marker;
        if (!path.startsWith(root + sep)) continue;
        const suffix = path.slice(root.length + 1);
        // Installed dependencies inside an authored tree retain their external identity.
        if (marker === "<authored>" && suffix.split(sep).includes("node_modules")) continue;
        return marker + "/" + suffix;
      }
      return path;
    };
    const identity = (record) => ({
      path: relocate(record.path),
      realPath: relocate(record.realPath),
      revision: relocate(record.realPath) === record.realPath ? record.revision : null,
      links: record.links.map((link) => ({
        path: relocate(link.path),
        target: isAbsolute(link.target) ? relocate(link.target) : link.target,
        revision: relocate(link.path) === link.path ? link.revision : null,
      })),
      bytes: record.bytes,
      sha256: record.sha256,
    });
    return {
      virtualParent: dirname(shard.cwd),
      config: identity(archive.config),
      configSha256: shard.configSha256,
      declaredRootSelection: archive.config.declaredRootSelection,
      args: shard.args.map((arg, index, args) =>
        args[index - 1] === "--project" ? relocate(arg) : arg,
      ),
      compilerOptions: shard.compilerOptions,
      runtime: archive.runtime,
      members: archive.membersBefore.map((member) => ({
        index: member.index,
        reportedPath: isAbsolute(member.reportedPath)
          ? relocate(member.reportedPath)
          : member.reportedPath,
        ...identity(member),
      })),
    };
  });
}

export function profileDuplicateCorpus({
  corpus,
  mode,
  label,
  directory,
  reference,
  project,
  pair,
}) {
  const copyRoot = join(directory, "work", label + "-profile-corpus");
  assert(!existsSync(copyRoot), "profile corpus must be a fresh duplicate tree");
  mkdirSync(join(directory, "work"), { recursive: true });
  const manifest = corpusManifest(corpus.dir);
  cpSync(corpus.dir, copyRoot, {
    recursive: true,
    dereference: false,
    verbatimSymlinks: true,
    errorOnExist: true,
    force: false,
  });
  assert.deepEqual(corpusManifest(copyRoot), manifest, "duplicate authored bytes differ");
  const duplicate = { ...corpus, dir: copyRoot };
  const before = project(duplicate, label + "-profile-before");
  const checked = pair(duplicate, mode, label + "-profile", true, true);
  assert.equal(
    checked.direct.fingerprint,
    reference.direct.fingerprint,
    "duplicate-tree full ordered product report differs",
  );
  assert.deepEqual(
    checked.direct.virtualFiles,
    reference.direct.virtualFiles,
    "duplicate-tree virtual bytes differ",
  );
  assert.deepEqual(
    graphVector(checked.wrapped.nativeShards, copyRoot),
    graphVector(reference.wrapped.nativeShards, corpus.dir),
    "duplicate-tree config/member bytes, order or canonical external context differ",
  );
  const profiles = checked.wrapped.nativeShards.map((shard) => {
    assert.equal(shard.nativeProfile?.validation, "passed", "missing successful native CPU replay");
    return { config: shard.config, ...shard.nativeProfile };
  });
  assert(profiles.length > 0, "profile replay produced no native commands");
  const after = project(duplicate, label + "-profile-after");
  assert.equal(after.text, before.text, "profile replay changed generated code/maps/links");
  assert.deepEqual(corpusManifest(corpus.dir), manifest, "profile changed original corpus");
  assert.deepEqual(corpusManifest(copyRoot), manifest, "profile changed duplicate corpus");
  return {
    kind: "untimed-duplicate-corpus-native-profile",
    originalRoot: corpus.dir,
    duplicateRoot: copyRoot,
    inputManifest: manifest,
    directReceipt: checked.direct.id,
    wrappedReceipt: checked.wrapped.id,
    profiles,
    orderedProductReportEqual: true,
    virtualBytesEqual: true,
    orderedNativeGraphBytesEqual: true,
    canonicalExternalContextEqual: true,
    rootRelocation:
      "only corresponding authored non-node_modules roots and proven sibling Canon virtual roots",
    projectionBeforeSha256: before.sha256,
    projectionAfterSha256: after.sha256,
    excludedFromTimings: true,
    semanticProfileDecoding: "not implemented; gzip container validation only",
    cpuSamples: null,
    allocationSamples: null,
    phaseAttribution: null,
    startupMs: null,
    programConstructionMs: null,
  };
}

/** Original clean/minimal/full planted populations each get a separate duplicate. */
export function profileNativeReferences({
  corpus,
  checked,
  gatePairs,
  fullPlant,
  mode,
  label,
  directory,
  project,
  pair,
}) {
  const references = [
    [corpus, checked],
    ...[...gatePairs].map(([cwd, value]) => [
      { ...corpus, dir: cwd, args: cwd === fullPlant.dir ? corpus.args : ["."] },
      value,
    ]),
  ];
  return references.map(([input, reference], index) =>
    profileDuplicateCorpus({
      corpus: input,
      mode,
      label: label + "-" + index,
      directory,
      reference,
      project,
      pair,
    }),
  );
}
