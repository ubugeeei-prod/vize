import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  linkSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import type { TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import {
  MANIFEST,
  PAYLOAD,
  RECEIPT,
  pack,
  restore,
} from "../../tools/support/compat/github/artifact-payload.ts";
import {
  PayloadError,
  digestFile,
  jsonBytes,
  parseManifest,
  verifyTree,
} from "../../tools/support/compat/github/artifact-payload-files.ts";
import type { JsonObject } from "../../tools/support/compat/github/artifact-payload-files.ts";
import {
  ZipWriter,
  readZip,
  readZipEntry,
} from "../../tools/support/compat/github/artifact-payload-zip.ts";

const SOURCE = "1".repeat(40);
const DRIVER = "2".repeat(40);
const ARTIFACT = 42;
const filesModule = fileURLToPath(
  new URL("../../tools/support/compat/github/artifact-payload-files.ts", import.meta.url),
);

interface Fixture {
  readonly root: string;
  readonly original: string;
  readonly envelope: string;
  readonly expected: Map<string, { body: Buffer; mode: number }>;
  readonly metadata: JsonObject;
}

const repeat = (bytes: readonly number[], count: number): Buffer =>
  Buffer.concat(Array.from({ length: count }, () => Buffer.from(bytes)));

async function fixture(t: TestContext): Promise<Fixture> {
  const root = mkdtempSync(join(tmpdir(), "vize-payload-controls-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const original = join(root, "original.zip");
  const expected = new Map<string, { body: Buffer; mode: number }>();
  const archive = new ZipWriter(original);
  const add = async (name: string, body: Buffer, mode = 0o644): Promise<void> => {
    await archive.add(name, body, { externalAttr: ((0o100000 | mode) << 16) >>> 0 });
    expected.set(name, { body, mode });
  };
  const native = Buffer.concat([Buffer.from("unchanged native body"), Buffer.from([0, 0xff])]);
  for (let index = 0; index < 90; index++) {
    await add(`complete/case-${index}/native-tsc`, repeat([...native], 512), 0o755);
    await add(`complete/case-${index}/stdout.raw`, Buffer.from([index, 0, 255, 13, 10]));
  }
  // Distinct historical binaries must survive, regardless of similar names.
  for (let index = 0; index < 3; index++) {
    await add(`historical-${index}/source-native.node`, Buffer.alloc(2048, index));
  }
  await add(
    "雪/.hidden/name\nnewline",
    Buffer.from([...Buffer.from("opaque"), 0, 0x80, 0xff, 13, 10]),
    0o600,
  );
  await add("stderr.raw", Buffer.alloc(0));
  // Different equal-size bodies with the same CRC32 must remain distinct.
  await add("crc-collision-a.bin", Buffer.from("plumless"));
  await add("crc-collision-b.bin", Buffer.from("buckeroo"));
  await archive.add("empty/", Buffer.alloc(0), {
    method: 0,
    externalAttr: ((0o040750 << 16) | 0x10) >>> 0,
  });
  archive.close();
  const metadata: JsonObject = {
    id: ARTIFACT,
    name: "complete-original",
    size_in_bytes: statSync(original).size,
    digest: `sha256:${await digestFile(original)}`,
    workflow_run: { head_sha: SOURCE, id: 123 },
  };
  return { root, original, envelope: join(root, "envelope"), expected, metadata };
}

const packed = (f: Fixture): Promise<JsonObject> =>
  pack(f.original, f.envelope, f.metadata, SOURCE, DRIVER);

async function restored(f: Fixture, name = "restored"): Promise<string> {
  const destination = join(f.root, name);
  await restore(f.envelope, destination, SOURCE, DRIVER, ARTIFACT);
  return destination;
}

const readJson = (path: string): unknown => JSON.parse(readFileSync(path, "utf8"));
const manifestOf = (f: Fixture) => parseManifest(readJson(join(f.envelope, MANIFEST)));

async function repackObjects(
  f: Fixture,
  change: { missing?: string; changed?: string },
): Promise<void> {
  const original = join(f.envelope, PAYLOAD);
  const replaced = join(f.root, "replaced.zip");
  const target = new ZipWriter(replaced);
  for (const info of readZip(original)) {
    if (info.name === change.missing) continue;
    const body =
      info.name === change.changed
        ? Buffer.from("different whole body")
        : await readZipEntry(original, info);
    await target.add(info.name, body, { method: info.method, externalAttr: info.externalAttr });
  }
  target.close();
  renameSync(replaced, original);
  const receipt = { ...(readJson(join(f.envelope, RECEIPT)) as JsonObject) };
  writeFileSync(
    join(f.envelope, RECEIPT),
    jsonBytes({ ...receipt, payloadZipSha256: await digestFile(original) }),
  );
}

const refusal =
  (message: string): ((error: unknown) => boolean) =>
  (error) =>
    error instanceof PayloadError && error.message === message;

void test("complete original and independent mutation", async (t) => {
  const f = await fixture(t);
  const before = await digestFile(f.original);
  const receipt = await packed(f);
  const destination = await restored(f);
  const manifest = manifestOf(f);
  assert.equal(receipt.fileCount, f.expected.size);
  assert.equal(receipt.entryCount, f.expected.size + 1);
  assert.ok(Number(receipt.uniqueBodyBytes) < Number(receipt.rawBytes));
  const digests = (suffix: (path: string) => boolean): Set<string | undefined> =>
    new Set(manifest.entries.filter((row) => suffix(row.path)).map((row) => row.sha256));
  assert.equal(manifest.entries.filter((row) => row.path.endsWith("native-tsc")).length, 90);
  assert.equal(digests((path) => path.endsWith("native-tsc")).size, 1);
  assert.equal(digests((path) => path.endsWith("source-native.node")).size, 3);
  assert.equal(digests((path) => path.startsWith("crc-collision-")).size, 2);
  const infos = readZip(f.original);
  const [a, b] = ["a", "b"].map((name) =>
    infos.find((info) => info.name === `crc-collision-${name}.bin`),
  );
  assert.deepEqual([a?.crc, a?.size], [b?.crc, b?.size]);
  for (const [name, { body, mode }] of f.expected) {
    assert.deepEqual(readFileSync(join(destination, name)), body);
    assert.equal(statSync(join(destination, name)).mode & 0o7777, mode);
    assert.equal(statSync(join(destination, name)).nlink, 1);
  }
  assert.equal(statSync(join(destination, "empty")).mode & 0o7777, 0o750);
  const first = join(destination, "complete/case-0/native-tsc");
  const last = join(destination, "complete/case-89/native-tsc");
  assert.notEqual(statSync(first).ino, statSync(last).ino);
  writeFileSync(first, "one independent mutation");
  assert.deepEqual(readFileSync(last), f.expected.get("complete/case-89/native-tsc")?.body);
  await assert.rejects(verifyTree(destination, manifest), refusal("restored file body differs"));
  assert.equal(await digestFile(f.original), before);
});

void test("missing, changed, extra, modes and external aliases", async (t) => {
  const f = await fixture(t);
  await packed(f);
  const manifest = manifestOf(f);
  for (const control of ["missing", "extra", "mode", "symlink", "hardlink"]) {
    const destination = await restored(f, control);
    const first = join(destination, "complete/case-0/native-tsc");
    if (control === "missing") unlinkSync(first);
    else if (control === "extra") writeFileSync(join(destination, "extra"), "unexpected");
    else if (control === "mode") chmodSync(first, 0o644);
    else if (control === "symlink") {
      unlinkSync(first);
      symlinkSync(f.original, first);
    } else {
      const alias = join(destination, "complete/case-89/native-tsc");
      unlinkSync(alias);
      linkSync(first, alias);
    }
    await assert.rejects(verifyTree(destination, manifest), PayloadError, control);
  }
});

void test("an unreadable directory cannot hide extra inventory", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-payload-permission-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const directory = join(root, "permission-control");
  mkdirSync(directory, { mode: 0o755 });
  const closed = join(directory, "empty");
  mkdirSync(closed);
  writeFileSync(
    join(closed, "unexpected.raw"),
    Buffer.from([...Buffer.from("opaque unobserved bytes"), 0, 0xff]),
  );
  chmodSync(closed, 0);
  const manifest = {
    entries: [{ path: "empty", kind: "directory", mode: 0, externalAttr: 0, createSystem: 0 }],
  } as const;
  try {
    if (process.getuid?.() === 0) {
      // A root runner can read mode000: exercise the real refusal as nobody.
      chmodSync(root, 0o755);
      const program = `const files = await import(${JSON.stringify(filesModule)});
try { await files.verifyTree(${JSON.stringify(directory)}, ${JSON.stringify(manifest)}); process.exit(1); }
catch (error) { process.exit(error instanceof files.PayloadError ? 0 : 2); }`;
      const child = spawnSync(process.execPath, ["--input-type=module", "-e", program], {
        uid: 65534,
        gid: 65534,
        encoding: "utf8",
      });
      assert.equal(
        child.status,
        0,
        `unreadable inventory must refuse under an unprivileged identity\n${child.stderr}`,
      );
    } else {
      await assert.rejects(
        verifyTree(directory, manifest),
        refusal("cannot inspect complete restored inventory"),
      );
    }
  } finally {
    chmodSync(closed, 0o755);
  }
});

void test("corrupt archive and whole object", async (t) => {
  const f = await fixture(t);
  await packed(f);
  const name = readZip(join(f.envelope, PAYLOAD))[0]?.name;
  await repackObjects(f, { changed: name });
  await assert.rejects(restored(f, "whole-body-corruption"), refusal("damaged whole object body"));
  const archive = join(f.envelope, PAYLOAD);
  const bytes = readFileSync(archive);
  bytes.write("corrupt archive bytes", 0);
  writeFileSync(archive, bytes);
  await assert.rejects(restored(f, "transport-corruption"), refusal("damaged payload archive"));
});

void test("a missing object with a recomputed outer digest", async (t) => {
  const f = await fixture(t);
  await packed(f);
  const name = readZip(join(f.envelope, PAYLOAD))[0]?.name;
  await repackObjects(f, { missing: name });
  await assert.rejects(restored(f), refusal("missing or unexpected payload object"));
});

void test("source, driver and original digest are bound", async (t) => {
  const f = await fixture(t);
  await packed(f);
  for (const [source, driver, artifact] of [
    [DRIVER, DRIVER, ARTIFACT],
    [SOURCE, SOURCE, ARTIFACT],
    [SOURCE, DRIVER, ARTIFACT + 1],
  ] as const) {
    await assert.rejects(
      restore(f.envelope, join(f.root, "wrong-identity"), source, driver, artifact),
      refusal("transport source or driver identity differs"),
    );
  }
  const badMetadata = { ...f.metadata, digest: `sha256:${"0".repeat(64)}` };
  await assert.rejects(
    pack(f.original, join(f.root, "wrong-original"), badMetadata, SOURCE, DRIVER),
    refusal("original archive digest differs"),
  );
});

void test("manifest corruption and external envelope symlinks", async (t) => {
  const f = await fixture(t);
  await packed(f);
  const manifest = join(f.envelope, MANIFEST);
  const originalBytes = readFileSync(manifest);
  writeFileSync(manifest, Buffer.concat([originalBytes, Buffer.from(" ")]));
  await assert.rejects(restored(f, "wrong-manifest"), refusal("damaged payload manifest"));
  unlinkSync(manifest);
  const external = join(f.root, "external.json");
  writeFileSync(external, originalBytes);
  symlinkSync(external, manifest);
  await assert.rejects(restored(f, "external-manifest"), refusal("invalid envelope file"));
  const externalTree = join(f.root, "external-tree");
  mkdirSync(externalTree);
  const borrowed = join(f.root, "borrowed-root");
  symlinkSync(externalTree, borrowed, "dir");
  await assert.rejects(
    verifyTree(borrowed, { entries: [] }),
    refusal("restored root must be an independent directory"),
  );
});

void test("unsafe, duplicate and symlink original entries fail closed", async (t) => {
  const f = await fixture(t);
  for (const control of ["../escape", "/absolute", "duplicate", "symlink"]) {
    const label = control.replaceAll("/", "_");
    const original = join(f.root, `control-${label}.zip`);
    const archive = new ZipWriter(original);
    const externalAttr = control === "symlink" ? (0o120777 << 16) >>> 0 : 0;
    await archive.add(control, Buffer.from("original"), { externalAttr });
    if (control === "duplicate") await archive.add(control, Buffer.from("second different body"));
    archive.close();
    const metadata = {
      ...f.metadata,
      size_in_bytes: statSync(original).size,
      digest: `sha256:${await digestFile(original)}`,
    };
    await assert.rejects(
      pack(original, join(f.root, `output-${label}`), metadata, SOURCE, DRIVER),
      PayloadError,
      control,
    );
  }
});
