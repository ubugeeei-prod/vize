import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  archiveCandidateNativeBlock,
  verifyRetainedCandidateBodies,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-native.ts";
import { sealInputs } from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-phase.ts";

const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function fixture(t: { after: (fn: () => void) => void }) {
  // Synthetic file/receipt inputs test the archive seam; no native or Go process is executed.
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "candidate-archive-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const capture = path.join(root, "capture"),
    archive = path.join(root, "base");
  const project = path.join(root, "project"),
    provider = path.join(root, "provider");
  const xdg = path.join(root, "xdg-cache"),
    tmp = path.join(root, "tmp");
  for (const dir of [capture, project, provider, xdg, tmp]) fs.mkdirSync(dir);
  for (const dir of [
    "native-profile-data",
    "native-profile-report",
    "native-memory-report",
    "input-blobs",
  ])
    fs.mkdirSync(path.join(capture, dir));
  fs.writeFileSync(path.join(project, "App.vue"), "<template/>\n");
  fs.writeFileSync(path.join(provider, "package.json"), "{}\n");
  fs.writeFileSync(path.join(xdg, "App.vue.ts"), "export {};\n");
  const write = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(capture, name), JSON.stringify(value));
  const roots = [project, provider, xdg, tmp],
    snapshot = sealInputs(roots, null);
  const rows = snapshot.rows
    .filter((row) => row.kind === "file")
    .map((row) => {
      const body = fs.readFileSync(row.path as string);
      fs.writeFileSync(path.join(capture, "input-blobs", hash(body)), body);
      return { ...row, body: "retained-utf8-text", blob: hash(body) };
    });
  write("materialized-body-index.json", { rows, textBytes: snapshot.bytes });
  write("materialized-before.json", snapshot);
  write("native-profile.input-after.json", snapshot);
  write("request.json", {
    spec: { out: capture, project, provider, runtimeRoots: [xdg, tmp], metadataOnlyImages: [] },
  });
  const profiles = ["cpu", "mem"].map((kind) => {
    const file = path.join(capture, "native-profile-data", `104-${kind}profile.pb.gz`);
    const body = Buffer.from("synthetic already-read " + kind);
    fs.writeFileSync(file, body);
    return { path: file, sha256: hash(body) };
  });
  write("native-profile.files.json", profiles);
  write("native-profile.validation.json", { passed: true, profileFiles: profiles });
  write("native-profile-report/reader-and-denominator.json", {
    goFullRawMatchesSampleDenominator: true,
    profileSha256: profiles[0].sha256,
  });
  write("native-memory-report/reader-and-denominators.json", {
    goFullRawOrderedSamplesEqual: true,
    allSamplesCountedOnce: true,
    profileSha256: profiles[1].sha256,
  });
  for (const [index, name] of ["phase-vize", "vize", "native-replay", "native-profile"].entries())
    write(name + ".status.json", {
      error: null,
      signal: null,
      status: index < 2 ? 0 : 2,
      invokedPid: 101 + index,
    });
  for (const role of ["phase-vize", "vize", "native-initial", "native-replay", "native-profile"])
    for (const stream of ["stdout", "stderr"])
      fs.writeFileSync(
        path.join(capture, `${role}.${stream}.bin`),
        "synthetic whole " + role + stream,
      );
  const config = path.join(xdg, "tsconfig.json"),
    info = path.join(xdg, "info.tsbuildinfo");
  write("native-profile.request.json", {
    args: [
      "--checkers",
      "1",
      "--incremental",
      "--tsBuildInfoFile",
      info,
      "--pretty",
      "false",
      "--project",
      config,
      "--pprofDir",
      path.join(capture, "native-profile-data"),
    ],
  });
  return { root, capture, archive, xdg, tmp, project, provider, snapshot, rows, write };
}

test("native archive retires only sealed runtime after full body/profile joins, preserving historical raw paths", (t) => {
  const f = fixture(t),
    raw = fs.readFileSync(path.join(f.capture, "request.json"));
  const result = archiveCandidateNativeBlock(f.capture, f.archive);
  assert.equal(result.completeInputBodiesRetained, true);
  assert(!fs.existsSync(f.xdg) && !fs.existsSync(f.tmp) && !fs.existsSync(f.capture));
  assert(fs.readFileSync(path.join(f.archive, "request.json")).equals(raw));
  assert(fs.existsSync(path.join(f.project, "App.vue")));
  assert(fs.existsSync(path.join(f.archive, "native-profile-data/104-memprofile.pb.gz")));
});

test("native archive refuses missing decode, mutated inputs, missing bodies and existing target before retirement", (t) => {
  for (const alter of [
    (f: ReturnType<typeof fixture>) =>
      f.write("native-memory-report/reader-and-denominators.json", {
        goFullRawOrderedSamplesEqual: false,
      }),
    (f: ReturnType<typeof fixture>) => fs.writeFileSync(path.join(f.xdg, "App.vue.ts"), "tampered"),
    (f: ReturnType<typeof fixture>) =>
      fs.unlinkSync(path.join(f.capture, "input-blobs", String(f.rows[0].blob))),
    (f: ReturnType<typeof fixture>) => fs.mkdirSync(f.archive),
    (f: ReturnType<typeof fixture>) =>
      fs.unlinkSync(path.join(f.capture, "native-replay.stdout.bin")),
  ]) {
    const f = fixture(t);
    alter(f);
    assert.throws(() => archiveCandidateNativeBlock(f.capture, f.archive));
    assert(fs.existsSync(f.xdg) && fs.existsSync(f.tmp) && fs.existsSync(f.capture));
  }
});

test("body retention refuses omissions, wrong byte identity, invented image role and opaque source drift", (t) => {
  const f = fixture(t);
  const index = { rows: f.rows, textBytes: f.snapshot.bytes };
  const read = (digest: string) => fs.readFileSync(path.join(f.capture, "input-blobs", digest));
  assert.doesNotThrow(() => verifyRetainedCandidateBodies(index, f.snapshot, [], read));
  assert.throws(() =>
    verifyRetainedCandidateBodies({ ...index, rows: f.rows.slice(1) }, f.snapshot, [], read),
  );
  assert.throws(() =>
    verifyRetainedCandidateBodies(index, f.snapshot, [], () => Buffer.from("wrong")),
  );
  const altered = structuredClone(index);
  altered.rows[0].body = "metadata-only-native-image";
  assert.throws(() => verifyRetainedCandidateBodies(altered, f.snapshot, [], read));
  const opaque = structuredClone(f.snapshot);
  opaque.rows.find((row) => row.kind === "file")!.opaque = true;
  assert.throws(() => verifyRetainedCandidateBodies(index, opaque, [], read));
});
