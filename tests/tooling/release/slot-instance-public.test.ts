import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { asArray, asRecord, readJsonRecord } from "../../../tools/support/json-value.ts";
import {
  assertCustody,
  fileImage,
  type DeclarationGraph,
} from "../../../tools/support/release/computed_event_custody.ts";
import {
  assertLockedPackage,
  validateCapsule,
  validateNative,
} from "../../../tools/support/release/slot_instance_public/authority.ts";
import {
  assertGraph,
  assertProcess,
  assertRows,
  expected,
  modes,
  profiles,
} from "../../../tools/support/release/slot_instance_public/contract.ts";
import {
  inputs,
  preservationRepair,
  replaceOnce,
  targetRepair,
} from "../../../tools/support/release/slot_instance_public/observe.ts";
import { providerManifest } from "../../../tools/support/release/slot_instance_public/install.ts";
import { assertJournalPid } from "../../../tools/support/release/slot_instance_public/process.ts";
import { mergeImages } from "../../../tools/support/release/slot_instance_public/libraries.ts";
import { writeTarGz, type TarEntry } from "../../../tools/support/archive/tar.ts";
import { Recorder } from "../../../tools/support/config/windows_consumer/common.ts";
import {
  assertPackedManifest,
  captureInstalledManifests,
  packedManifest,
} from "../../../tools/support/release/slot_instance_public/packed-manifest.ts";

const sha = (raw: Buffer) => createHash("sha256").update(raw).digest("hex");
const edgeFile = fileURLToPath(
  new URL(
    "../../../tools/support/release/slot_instance_public/declaration-edges.json.txt",
    import.meta.url,
  ),
);
const edges = readJsonRecord(edgeFile);
test("packed dependency transforms require exact capsule-SRI bytes and every other source field", () => {
  const source = {
    name: "vize",
    version: "0.620.0",
    bin: { vize: "bin/vize" },
    exports: { "./virtual-ts": "./virtual-ts.d.ts" },
    typescript: { contentMapper: { exec: ["node", "./bin/vize", "content-mapper"] } },
    files: ["virtual-ts.d.ts"],
    dependencies: { "@vizejs/native": "workspace:*" },
    devDependencies: { typescript: "catalog:default" },
    peerDependencies: { vite: "catalog:default" },
  };
  const packed = {
    ...source,
    dependencies: { "@vizejs/native": "0.620.0" },
    devDependencies: { typescript: "6.0.3" },
    peerDependencies: { vite: "^8.0.0" },
    optionalDependencies: { "@typescript/typescript-linux-x64": "7.0.2" },
  };
  const raw = Buffer.from(JSON.stringify(packed, null, 2) + "\n");
  const members: TarEntry[] = [
    { name: "package/package.json", data: raw },
    { name: "package/virtual-ts.d.ts", data: Buffer.from("export type Proof = true;\n") },
  ];
  const payload = writeTarGz(members),
    sri = (bytes: Buffer) => "sha512-" + createHash("sha512").update(bytes).digest("base64");
  const authenticated = packedManifest(payload, sri(payload));
  assert.deepEqual(authenticated.manifest, raw);
  assert.deepEqual(assertPackedManifest(raw, authenticated.manifest, source), packed);
  assert.throws(() => packedManifest(payload, sri(Buffer.from("different archive"))));
  assert.throws(() =>
    assertPackedManifest(
      Buffer.from(JSON.stringify({ ...packed, dependencies: { arbitrary: "1.0.0" } })),
      authenticated.manifest,
      source,
    ),
  );
  for (const change of [
    { exports: { ".": "./foreign.js" } },
    { typescript: { contentMapper: { exec: ["foreign"] } } },
    { files: [] },
    { addedNonDependency: true },
  ]) {
    const changed = Buffer.from(JSON.stringify({ ...packed, ...change }));
    const archive = writeTarGz([{ name: "package/package.json", data: changed }]);
    const exact = packedManifest(archive, sri(archive));
    assert.throws(() => assertPackedManifest(changed, exact.manifest, source));
  }
  for (const extra of [
    members[0],
    { name: "package/../escape", data: raw },
    { name: "foreign/package.json", data: raw },
    { name: "package/link", typeflag: "2", linkname: "package.json" },
    { name: "package/hardlink", typeflag: "1", linkname: "package.json" },
  ]) {
    const invalid = writeTarGz([...members, extra]);
    assert.throws(() => packedManifest(invalid, sri(invalid)));
  }
  const absent = writeTarGz(members.slice(1));
  assert.throws(() => packedManifest(absent, sri(absent)));
});
test("all three actual manifest buffers survive a validation failure before native loading", () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "slot-packed-custody-")));
  try {
    const modules = join(root, "node_modules"),
      record = new Recorder(join(root, "evidence"));
    const names = ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"];
    for (const name of names) {
      const directory = join(modules, name);
      mkdirSync(directory, { recursive: true });
      writeFileSync(join(directory, "package.json"), "invalid-json-" + name);
    }
    const captured = captureInstalledManifests(modules, record);
    assert.equal(captured.size, 3);
    assert.throws(() => JSON.parse(captured.get("vize")!.raw.toString("utf8")));
    for (const [name, label] of [
      [names[0], "cli"],
      [names[1], "native"],
      [names[2], "platform"],
    ])
      assert.equal(
        readFileSync(join(record.root, "installed-" + label + "-package.raw.json"), "utf8"),
        "invalid-json-" + name,
      );
    assert.equal(
      readJsonRecord(join(record.root, "installation-phase.json")).phase,
      "capture-installed-manifests",
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
test("overlapping library and program-input snapshots retain every unique file and reject changes", () => {
  const first = { path: "/owned/typescript/lib/typescript.js", bytes: 10, sha256: "a".repeat(64) };
  const second = { path: "/owned/vize/virtual-ts.d.ts", bytes: 20, sha256: "b".repeat(64) };
  assert.deepEqual(mergeImages([first, second, { ...first }, { ...second }]), [first, second]);
  assert.throws(() => mergeImages([first, { ...first, bytes: 11 }]));
  assert.throws(() => mergeImages([first, { ...first, sha256: "c".repeat(64) }]));
  const actual = fileImage(fileURLToPath(import.meta.url));
  assert.throws(
    () => assertCustody([actual, actual]),
    "the original duplicate-path failure is real",
  );
  assertCustody(mergeImages([actual, { ...actual }]));
});
test("C12 nine and C13 ten remain exact separate source/version/edge roles", () => {
  for (const role of ["before", "head"] as const) {
    const profile = profiles[role],
      root = "/owned/node_modules/vize";
    const graph: DeclarationGraph = {
      files: asArray(expected(role, "esm").sdk).map((value) => {
        const row = asRecord(value);
        return {
          path: root + "/" + row.file,
          bytes: Number(row.bytes),
          sha256: String(row.sha256),
        };
      }),
      edges: asArray(edges[role]).map((value) => {
        const row = asRecord(value);
        return {
          ...row,
          from: root + "/" + row.from,
          target: root + "/" + row.target,
        } as DeclarationGraph["edges"][number];
      }),
    };
    assertGraph(role, root, graph);
    assert.throws(() => assertGraph(role === "head" ? "before" : "head", root, graph));
    for (const mutation of [
      (g: DeclarationGraph) => g.files.pop(),
      (g: DeclarationGraph) => g.files.push(g.files[0]),
      (g: DeclarationGraph) => {
        g.files[0].sha256 = "0".repeat(64);
      },
      (g: DeclarationGraph) => {
        g.files[0].path = "/foreign/virtual-ts.d.ts";
      },
      (g: DeclarationGraph) => g.edges.reverse(),
      (g: DeclarationGraph) => {
        g.edges[0].specifier += ".extra";
      },
      (g: DeclarationGraph) => {
        g.edges[0].attributes = {};
      },
      (g: DeclarationGraph) => {
        g.edges[0].target = root + "/foreign.d.ts";
      },
    ]) {
      const altered = structuredClone(graph);
      mutation(altered);
      assert.throws(() => assertGraph(role, root, altered));
    }
    const capsule = {
      version: profile.version,
      tagSourceSha: profile.candidate,
      producerSource: profile.source,
      npmSRI: {},
      receipts: [{}],
    };
    validateCapsule(role, capsule);
    for (const field of ["version", "tagSourceSha", "producerSource"] as const)
      assert.throws(() => validateCapsule(role, { ...capsule, [field]: "wrong" }));
    assert.equal(providerManifest(role).dependencies.vize, profile.version);
    assert.equal(providerManifest(role).dependencies["@vizejs/native"], profile.version);
  }
});
test("genuine receipt origin, native identity and registry SRI all bind the exact role", () => {
  const role = "head",
    profile = profiles[role],
    binary = { bytes: 123, sha256: "a".repeat(64) },
    token = "owned-native-token";
  const receipt = {
    schema: "vize.platformVsix.v1",
    version: profile.version,
    source: String(profile.candidate),
    provider: {
      source: String(profile.candidate),
      releaseVersion: profile.version,
      targetPlatform: "linux-x64",
      nativeBuildToken: token,
      nodeFile: "native.node",
      files: [{ path: "native.node", ...binary }],
    },
  };
  const item = (value: typeof receipt) => {
    const raw = Buffer.from(JSON.stringify(value));
    return {
      origin: { runId: String(profile.run), artifactId: 1, member: "receipt.json" },
      rawBase64: raw.toString("base64"),
      sha256: sha(raw),
    };
  };
  validateNative(role, { receipts: [item(receipt)] }, binary, token);
  for (const change of [
    (r: typeof receipt) => {
      r.source = "b".repeat(40);
    },
    (r: typeof receipt) => {
      r.provider.source = "b".repeat(40);
    },
    (r: typeof receipt) => {
      r.provider.nativeBuildToken += "other";
    },
    (r: typeof receipt) => {
      r.provider.files[0].sha256 = "b".repeat(64);
    },
    (r: typeof receipt) => {
      r.provider.files[0].bytes++;
    },
  ]) {
    const r = structuredClone(receipt);
    change(r);
    assert.throws(() => validateNative(role, { receipts: [item(r)] }, binary, token));
  }
  assert.throws(() =>
    validateNative(role, { receipts: [item(receipt), item(receipt)] }, binary, token),
  );
  assert.throws(() => validateNative(role, { receipts: [] }, binary, token));
  const origin = item(receipt);
  origin.origin.runId = "1";
  assert.throws(() => validateNative(role, { receipts: [origin] }, binary, token));
  const sri = {
    version: profile.version,
    integrity: "sha512-YWJjZA==",
    resolved: "https://registry.npmjs.org/vize/-/vize-0.621.0.tgz",
  };
  const capsule = {
    npmSRI: {
      vize: {
        metadataOrigin: "https://registry.npmjs.org/vize/0.621.0",
        integrity: sri.integrity,
        tarball: sri.resolved,
      },
    },
  };
  const lock = { packages: { "node_modules/vize": sri } };
  assertLockedPackage(lock, capsule, "vize", profile.version);
  for (const change of [
    { version: "0.620.0" },
    { integrity: "sha512-Zm9v" },
    { resolved: "https://foreign.invalid/vize.tgz" },
    { link: true },
  ])
    assert.throws(() =>
      assertLockedPackage(
        { packages: { "node_modules/vize": { ...sri, ...change } } },
        capsule,
        "vize",
        profile.version,
      ),
    );
});
test("all40/60 observations and75 implicit/explicit exports refuse semantic mutations", () => {
  for (const role of ["before", "head"] as const)
    for (const mode of modes) {
      const cell = expected(role, mode),
        query = { rows: asArray(cell.rows), exports: asArray(cell.exports) };
      assertRows(role, mode, query, false);
      assertRows(role, mode, { rows: asArray(cell.preservationRows), exports: [] }, true);
      for (const change of [
        (q: typeof query) => q.rows.pop(),
        (q: typeof query) => {
          asRecord(q.rows[0]).flags = 1;
        },
        (q: typeof query) => {
          asRecord(q.rows[0]).intrinsic = "any";
        },
        (q: typeof query) => q.rows.reverse(),
        (q: typeof query) => q.exports.pop(),
        (q: typeof query) => {
          const row = asRecord(q.exports[0]);
          row.value = !row.value;
        },
        (q: typeof query) => {
          asRecord(q.exports[0]).declarations = [];
        },
      ]) {
        const altered = structuredClone(query);
        change(altered);
        assert.throws(() => assertRows(role, mode, altered, false));
      }
    }
});
test("DefaultZero and preservation repairs own exactly one input site; full raw outcomes stay exact", () => {
  assertJournalPid('{"pid":123}\n{"pid":123}\n', 123);
  assert.throws(() => assertJournalPid('{"pid":123}\n{"pid":456}\n', 123));
  assert.throws(() => assertJournalPid('{"pid":123}', 123));
  for (const mode of modes) {
    const source = inputs(mode);
    assert.notEqual(targetRepair(source.target), source.target);
    assert.notEqual(preservationRepair(source.preservation), source.preservation);
    assert.throws(() => targetRepair(source.target + source.target));
    assert.throws(() => preservationRepair(source.target));
  }
  assert.throws(() => replaceOnce("one", "missing", "fixed"));
  const stdout =
    "proof.mts(55,36): error TS2322: Type 'number' is not assignable to type 'string'.\n";
  const packet = { status: 2, signal: null, error: null, stdout, stderr: "" };
  assertProcess(packet, stdout);
  for (const change of [
    { status: 0 },
    { stdout: stdout + "extra\n" },
    { stdout: stdout.trimEnd() },
    { stderr: "warning" },
    { signal: "SIGTERM" },
    { error: "timeout" },
  ])
    assert.throws(() => assertProcess({ ...packet, ...change }, stdout));
  assert.ok(readFileSync(edgeFile).length > 0);
});
