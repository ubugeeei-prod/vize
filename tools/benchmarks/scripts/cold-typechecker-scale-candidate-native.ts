/** Archive a fully observed native block before reusing its exact physical paths for the other role. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { sealInputs } from "./cold-typechecker-scale-native-phase.ts";
import { validateNativeArgs } from "./cold-typechecker-scale-native-inputs.ts";

const inside = (root: string, file: string) => file === root || file.startsWith(root + path.sep);
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");

export function verifyRetainedCandidateBodies(
  index: { rows: Record<string, unknown>[]; textBytes: number },
  snapshot: { rows: Record<string, unknown>[] },
  images: { path: string; sha256: string; role: string }[],
  readBlob: (digest: string) => Buffer,
) {
  const files = snapshot.rows.filter((row) => row.kind === "file");
  assert.equal(
    index.rows.length,
    files.length,
    "every potential input file needs a body or exact image role",
  );
  let textBytes = 0;
  index.rows.forEach((entry, position) => {
    const { body, blob, imageRole, ...row } = entry;
    assert.deepEqual(row, files[position], "body index changed input order/stat/content identity");
    const image = images.filter((candidate) => candidate.path === row.path);
    assert(image.length <= 1);
    if (image.length === 1) {
      assert.equal(body, "metadata-only-native-image");
      assert.equal(imageRole, image[0].role);
      assert.equal(row.sha256, image[0].sha256);
      assert.equal(blob, undefined, "executable image bytes must not be copied into the artifact");
    } else {
      assert.equal(body, "retained-utf8-text");
      assert.equal(blob, row.sha256);
      assert.match(String(blob), /^[a-f0-9]{64}$/u);
      assert.equal(imageRole, undefined);
      const bytes = readBlob(blob as string);
      assert.equal(bytes.length, row.bytes);
      assert.equal(hash(bytes), blob);
      textBytes += bytes.length;
    }
  });
  assert.equal(index.textBytes, textBytes);
  return { files: files.length, textBytes };
}

export function archiveCandidateNativeBlock(capture: string, archive: string) {
  assert(path.isAbsolute(capture) && path.isAbsolute(archive));
  assert.equal(fs.realpathSync(capture), capture);
  assert.equal(fs.realpathSync(path.dirname(archive)), path.dirname(archive));
  assert(
    fs.lstatSync(archive, { throwIfNoEntry: false }) === undefined,
    "never replace a previous role, observation or symlink",
  );
  assert(!inside(capture, archive) && !inside(archive, capture));
  const read = (name: string) => JSON.parse(fs.readFileSync(path.join(capture, name), "utf8"));
  const request = read("request.json"),
    spec = request.spec;
  assert.equal(spec.out, capture);
  const validation = read("native-profile.validation.json");
  assert.equal(validation.passed, true);
  const cpu = read("native-profile-report/reader-and-denominator.json");
  const memory = read("native-memory-report/reader-and-denominators.json");
  assert.equal(cpu.goFullRawMatchesSampleDenominator, true);
  assert.equal(memory.goFullRawOrderedSamplesEqual, true);
  assert.equal(memory.allSamplesCountedOnce, true);
  const profiles = read("native-profile.files.json") as { path: string; sha256: string }[];
  assert.deepEqual(validation.profileFiles, profiles);
  assert.equal(profiles.length, 2);
  for (const [index, reader] of [cpu, memory].entries()) {
    const profile = profiles[index];
    assert.equal(fs.realpathSync(profile.path), profile.path);
    assert.equal(path.dirname(profile.path), path.join(capture, "native-profile-data"));
    assert.equal(reader.profileSha256, profile.sha256);
    assert.equal(hash(fs.readFileSync(profile.path)), profile.sha256);
  }
  const phase = read("phase-vize.status.json"),
    custody = read("vize.status.json"),
    bare = read("native-replay.status.json"),
    profile = read("native-profile.status.json");
  for (const outcome of [phase, custody, bare, profile]) {
    assert.equal(outcome.error, null);
    assert.equal(outcome.signal, null);
    assert(Number.isSafeInteger(outcome.invokedPid) && outcome.invokedPid > 0);
  }
  assert.equal(phase.status, 0);
  assert.equal(custody.status, 0);
  assert.equal(bare.status, profile.status);
  const rawStreams = [
    "phase-vize",
    "vize",
    "native-initial",
    "native-replay",
    "native-profile",
  ].flatMap((role) =>
    ["stdout", "stderr"].map((stream) => {
      const name = `${role}.${stream}.bin`,
        file = path.join(capture, name),
        stat = fs.lstatSync(file);
      assert(stat.isFile() && !stat.isSymbolicLink());
      const bytes = fs.readFileSync(file);
      return { file: name, bytes: bytes.length, sha256: hash(bytes) };
    }),
  );
  assert.deepEqual(
    profiles.map((row) => path.basename(row.path)),
    [`${profile.invokedPid}-cpuprofile.pb.gz`, `${profile.invokedPid}-memprofile.pb.gz`],
  );
  assert.deepEqual(
    fs.readdirSync(path.join(capture, "native-profile-data")).sort(),
    profiles.map((row) => path.basename(row.path)).sort(),
  );
  const native = read("native-profile.request.json");
  const args = validateNativeArgs(native.args.slice(0, 9), native.args[8]);
  assert.deepEqual(native.args.slice(9), ["--pprofDir", path.join(capture, "native-profile-data")]);
  assert(!fs.existsSync(args[4]), "build-info output must already be separately retained");
  const before = read("materialized-before.json"),
    after = read("native-profile.input-after.json");
  assert.deepEqual(
    after,
    before,
    "complete post-profile inputs must retain the captured bare program",
  );
  assert.deepEqual(after.roots.slice(0, 2), [spec.project, spec.provider]);
  const roots: string[] = spec.runtimeRoots ?? [
    path.join(capture, "runtime/xdg"),
    path.join(capture, "runtime/tmp"),
  ];
  assert.deepEqual(after.roots.slice(2), roots);
  assert.equal(after.roots.length, 4);
  assert(roots.some((root) => inside(root, args[4])));
  assert.deepEqual(sealInputs(after.roots, args[4]), after, "Go readers changed a sealed input");
  const retained = verifyRetainedCandidateBodies(
    read("materialized-body-index.json"),
    after,
    spec.metadataOnlyImages,
    (digest) => {
      const file = path.join(capture, "input-blobs", digest),
        stat = fs.lstatSync(file);
      assert(stat.isFile() && !stat.isSymbolicLink() && stat.nlink === 1);
      return fs.readFileSync(file);
    },
  );
  // Durably retain the intended archive action before discarding only the two owned caches.
  const receipt = {
    from: capture,
    archive,
    rootsRetired: roots,
    postReaderSealSha256: after.sha256,
    absoluteMetadataPaths:
      "original observed paths preserved byte-exact; evidence directory moved afterward",
    completeInputBodiesRetained: true,
    retained,
    rawProfilesRetained: true,
    rawStreams,
    nativeLoadedFiles: "unobserved",
    osPageCache: "uncontrolled; no flush",
  };
  fs.writeFileSync(
    path.join(capture, "archive-request.json"),
    JSON.stringify(receipt, null, 2) + "\n",
    { flag: "wx" },
  );
  for (const root of roots) {
    assert(!inside(spec.project, root) && !inside(spec.provider, root));
    assert(!inside(root, spec.project) && !inside(root, spec.provider));
    assert(!inside(root, capture), "never remove the evidence namespace");
    assert.equal(fs.realpathSync(root), root);
    assert(fs.lstatSync(root).isDirectory() && !fs.lstatSync(root).isSymbolicLink());
  }
  for (const root of roots) fs.rmSync(root, { recursive: true });
  fs.renameSync(capture, archive);
  fs.writeFileSync(
    path.join(archive, "archive-completed.json"),
    JSON.stringify(receipt, null, 2) + "\n",
    { flag: "wx" },
  );
  return receipt;
}
