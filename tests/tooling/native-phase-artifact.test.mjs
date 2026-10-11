import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { pack, reconstruct } from "../../tools/support/compat/github/native-phase-artifact.mjs";
import {
  manifestName,
  verifyReconstruction,
} from "../../tools/support/compat/github/native-phase-artifact-files.mjs";

const tarBinary = process.env.VIZE_NATIVE_ARTIFACT_TAR ?? "tar";
const sourceSha = "1".repeat(40);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const put = (path, bytes, mode = 0o644) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes, { mode });
};

function fixture(t) {
  const temporary = mkdtempSync(join(tmpdir(), "vize-native-envelope-"));
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  const root = join(temporary, "capture");
  const dependencies = join(temporary, "dependencies");
  mkdirSync(root);
  put(join(dependencies, "opaque.bin"), Buffer.from([0, 255, 128, 10, 13, 0, 47]));
  put(join(dependencies, ".hidden", "declaration.d.ts"), "export type 雪 = 'unchanged';\n");
  put(join(dependencies, "name\nwith newline.d.ts"), "export const line: 1;\n");
  put(join(dependencies, "run"), "#!/bin/sh\nexit 0\n", 0o755);
  const expected = new Map();
  for (let index = 0; index < 21; index++) {
    const prefix = `jsx-slot-parameter-types/case-${index}`;
    const project = join(root, prefix);
    mkdirSync(project, { recursive: true });
    symlinkSync(dependencies, join(project, "node_modules"), "dir");
    put(join(project, "stdout.raw"), `complete native packet ${index}\n`);
    put(join(project, "stderr.raw"), index === 0 ? "original failure\n" : "");
    expected.set(`${prefix}/stdout.raw`, readFileSync(join(project, "stdout.raw")));
    expected.set(`${prefix}/stderr.raw`, readFileSync(join(project, "stderr.raw")));
    for (const file of [
      "opaque.bin",
      ".hidden/declaration.d.ts",
      "name\nwith newline.d.ts",
      "run",
    ]) {
      expected.set(`${prefix}/node_modules/${file}`, readFileSync(join(dependencies, file)));
    }
  }
  put(join(root, "native-source-recipe.json"), '{"sourceSha":"original"}\n');
  put(join(root, "native-source-recipe.sh"), "#!/bin/sh\nfalse\n");
  expected.set("native-source-recipe.json", readFileSync(join(root, "native-source-recipe.json")));
  expected.set("native-source-recipe.sh", readFileSync(join(root, "native-source-recipe.sh")));
  // Match original Actions eligibility, including broken/cyclic links and hidden files.
  symlinkSync(join(temporary, "missing"), join(root, "jsx-slot-parameter-types", "broken"));
  symlinkSync(
    join(root, "jsx-slot-parameter-types"),
    join(root, "jsx-slot-parameter-types", "cycle"),
    "dir",
  );
  put(
    join(root, "typechecker-native-phases.json.samples", "work", "excluded.txt"),
    "not in original selection",
  );
  return { temporary, root, dependencies, expected, output: join(temporary, "envelope") };
}

function independentFiles(root) {
  const names = [];
  function visit(path, prefix = "") {
    for (const name of readdirSync(path)) {
      if (!prefix && name === manifestName) continue;
      const relative = prefix ? `${prefix}/${name}` : name;
      const stat = lstatSync(join(path, name));
      assert.equal(stat.isSymbolicLink(), false, "reconstruction is self-contained");
      if (stat.isDirectory()) visit(join(path, name), relative);
      else names.push(relative);
    }
  }
  visit(root);
  return names.sort((left, right) => left.localeCompare(right, "en"));
}

function rebuildArchive(f, restored, paths) {
  const archive = join(f.output, "native-phase-evidence.tar.zst");
  const list = join(f.temporary, "repacked.nul");
  writeFileSync(list, paths.map((path) => `${path}\0`).join(""));
  const result = spawnSync(
    tarBinary,
    [
      "--create",
      "--no-recursion",
      "--null",
      "--use-compress-program=zstd -T4 -3",
      "--file",
      archive,
      "--directory",
      restored,
      "--files-from",
      list,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  const receiptPath = join(f.output, "receipt.json");
  const receipt = JSON.parse(readFileSync(receiptPath, "utf8"));
  receipt.archiveSha256 = hash(readFileSync(archive));
  writeFileSync(receiptPath, JSON.stringify(receipt));
  return archive;
}

void test("21 actual shared-inode aliases reconstruct every original file and unchanged source bytes", (t) => {
  const f = fixture(t);
  const original = statSync(join(f.dependencies, "opaque.bin"), { bigint: true });
  for (let index = 0; index < 21; index++) {
    const alias = statSync(
      join(f.root, `jsx-slot-parameter-types/case-${index}/node_modules/opaque.bin`),
      { bigint: true },
    );
    assert.equal(alias.dev, original.dev);
    assert.equal(alias.ino, original.ino);
  }
  const receipt = pack(f.root, f.output, { sourceSha, tarBinary });
  const restored = join(f.temporary, "restored");
  reconstruct(f.output, restored, { tarBinary });
  assert.deepEqual(
    independentFiles(restored),
    [...f.expected.keys()].sort((left, right) => left.localeCompare(right, "en")),
  );
  const manifest = JSON.parse(readFileSync(join(restored, manifestName), "utf8"));
  const rows = new Map(manifest.files.map((row) => [row.path, row]));
  for (const [name, bytes] of f.expected) {
    const actual = readFileSync(join(restored, name));
    assert.deepEqual(actual, bytes, name);
    assert.equal(rows.get(name).sha256, hash(bytes), name);
    assert.equal(rows.get(name).bytes, bytes.length);
  }
  assert.equal(receipt.fileCount, f.expected.size);
  assert.equal(receipt.reconstructedFileCount, f.expected.size);
  assert(receipt.uniqueInodeBytes < receipt.rawBytes);
  const first = statSync(join(restored, "jsx-slot-parameter-types/case-0/node_modules/opaque.bin"));
  const last = statSync(join(restored, "jsx-slot-parameter-types/case-20/node_modules/opaque.bin"));
  assert.equal(first.ino, last.ino, "tar encoded shared input once with all alias paths");
  assert.equal(
    statSync(join(restored, "jsx-slot-parameter-types/case-0/node_modules/run")).mode & 0o777,
    0o755,
  );
  const after = statSync(join(f.dependencies, "opaque.bin"), { bigint: true });
  for (const field of ["dev", "ino", "nlink", "mode", "size", "mtimeNs", "ctimeNs"]) {
    assert.equal(after[field], original[field], `source ${field} changed`);
  }
  assert.deepEqual(readdirSync(f.output).sort(), ["native-phase-evidence.tar.zst", "receipt.json"]);
});

void test("damaged transport, missing raw files and changed reconstructed bytes fail closed", (t) => {
  const f = fixture(t);
  pack(f.root, f.output, { sourceSha, tarBinary });
  const restored = join(f.temporary, "restored");
  reconstruct(f.output, restored, { tarBinary });
  const expected = JSON.parse(readFileSync(join(restored, manifestName), "utf8"));
  const missing = "jsx-slot-parameter-types/case-20/stderr.raw";
  rmSync(join(restored, missing));
  assert.throws(() => verifyReconstruction(restored, expected), /reconstruction differs/);
  put(join(restored, missing), f.expected.get(missing));
  put(join(restored, "jsx-slot-parameter-types/case-1/stdout.raw"), "damaged packet");
  assert.throws(() => verifyReconstruction(restored, expected), /reconstruction differs/);
  put(
    join(restored, "jsx-slot-parameter-types/case-1/stdout.raw"),
    f.expected.get("jsx-slot-parameter-types/case-1/stdout.raw"),
  );

  // A recomputed outer transport hash cannot disguise an omitted raw payload.
  const archive = rebuildArchive(
    f,
    restored,
    [...f.expected.keys()].filter((path) => path !== missing).concat(manifestName),
  );
  assert.throws(
    () => reconstruct(f.output, join(f.temporary, "incomplete"), { tarBinary }),
    /reconstruction differs/,
  );
  const damaged = readFileSync(archive);
  damaged[0] ^= 0xff;
  writeFileSync(archive, damaged);
  assert.throws(
    () => reconstruct(f.output, join(f.temporary, "damaged"), { tarBinary }),
    /damaged native evidence archive/,
  );
});

void test("external raw file, directory and manifest symlinks fail with a rebuilt transport hash", (t) => {
  for (const kind of ["file", "directory", "manifest"]) {
    const f = fixture(t);
    pack(f.root, f.output, { sourceSha, tarBinary });
    const restored = join(f.temporary, "restored");
    reconstruct(f.output, restored, { tarBinary });
    let paths = [...f.expected.keys()].concat(manifestName);
    if (kind === "file") {
      const name = "native-source-recipe.json";
      rmSync(join(restored, name));
      symlinkSync(join(f.root, name), join(restored, name));
    } else if (kind === "directory") {
      const prefix = "jsx-slot-parameter-types/case-20/node_modules";
      rmSync(join(restored, prefix), { recursive: true });
      symlinkSync(f.dependencies, join(restored, prefix), "dir");
      paths = paths.filter((path) => !path.startsWith(`${prefix}/`)).concat(prefix);
    } else {
      const external = join(f.temporary, "external-manifest.json");
      writeFileSync(external, readFileSync(join(restored, manifestName)));
      rmSync(join(restored, manifestName));
      symlinkSync(external, join(restored, manifestName));
    }
    rebuildArchive(f, restored, paths);
    assert.throws(
      () => reconstruct(f.output, join(f.temporary, "substituted"), { tarBinary }),
      /symbolic link in reconstructed native evidence|reconstructed manifest must be a regular file/,
      `${kind} symlink must not borrow raw bytes from outside the reconstructed tree`,
    );
  }
});
