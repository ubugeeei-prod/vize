import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";

export const noMutatingModuleLayoutDirectory =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/module-layout-correction";
export const noMutatingModuleLayoutManifestSHA256 =
  "e53d557df855bf8a365e56cdf3cd64772ea33b8032690df34eaed0c6ed444c5e";
const carrier = "crates/vize_patina/src/linter/script_rules/sfc.rs";
const priorCarrierSHA = "80bc625aefac361bb6390fedba9641d6bbff127d78d5190fe6e695e4f41c0dff";
const wrapperPath = "tests/tooling/support/no-mutating-props-source-custody.mjs";
const entry =
  "export function authenticateNoMutatingPropsSourceFamily(root, read = reader(root)) {\n";
const imported =
  'import { reviewedNoMutatingModuleLayoutReadView } from "./no-mutating-props-module-layout-source-custody.mjs";\n';
const selected =
  "  const moduleLayoutAuthority = reviewedNoMutatingModuleLayoutReadView(root, read);\n  root = moduleLayoutAuthority.root;\n  read = moduleLayoutAuthority.read;\n";
const digest = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1")
    .update("blob " + body.length + "\0")
    .update(body)
    .digest("hex");
const reader =
  (root) =>
  (name, ...args) =>
    fs.readFileSync(path.join(root, name), ...args);

export function composeNoMutatingModuleLayoutHelper(before) {
  const source = before.toString();
  assert.equal(source.split(entry).length, 2);
  return Buffer.from(imported + source.replace(entry, entry + selected));
}

export function composeNoMutatingModuleLayoutLaw(law) {
  return Buffer.from(
    'import path from "node:path";\nimport { fileURLToPath } from "node:url";\n' +
      'import { runNoMutatingModuleLayoutHistoricalLaw } from "./support/no-mutating-props-module-layout-source-custody.mjs";\n' +
      'const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");\n' +
      "await runNoMutatingModuleLayoutHistoricalLaw(\n  root,\n  " +
      JSON.stringify(law) +
      ",\n);\n",
  );
}

function regular(root, name) {
  assert(
    name &&
      !path.isAbsolute(name) &&
      path.normalize(name) === name &&
      !name.split("/").includes(".."),
    name,
  );
  const target = path.join(root, name);
  const status = fs.lstatSync(target);
  assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111), name);
  assert.equal(fs.realpathSync(target), path.join(fs.realpathSync(root), name), name);
}
function identity(body, expected, name) {
  assert.equal(body.length, expected.bytes, name);
  assert.equal(digest(body), expected.sha256, name);
  assert.equal(blob(body), expected.gitBlob, name);
}
function absent(root, name) {
  try {
    fs.lstatSync(path.join(root, name));
  } catch (error) {
    assert.equal(error.code, "ENOENT", name);
    return;
  }
  assert.fail("old callback path must be physically absent: " + name);
}

/** A new whole frame; none of the two original manifests or predicates are rebaselined. */
export function authenticateNoMutatingModuleLayoutSourceFamily(root, read = reader(root)) {
  const check = (name, expected) => {
    regular(root, name);
    const supplied = read(name);
    identity(supplied, expected, name);
    assert.deepEqual(fs.readFileSync(path.join(root, name)), supplied, name);
    return supplied;
  };
  const manifestPath = noMutatingModuleLayoutDirectory + "/successor.json";
  regular(root, manifestPath);
  const bytes = read(manifestPath);
  assert.equal(digest(bytes), noMutatingModuleLayoutManifestSHA256);
  assert.deepEqual(fs.readFileSync(path.join(root, manifestPath)), bytes);
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.no-mutating-props-module-layout-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.source, "c989a36c4b12076bdf14da2e918ca038b00de162");
  assert.deepEqual(manifest.familyCounts, [12, 11]);
  const original = JSON.parse(check(manifest.originalFamily.path, manifest.originalFamily));
  const namespace = JSON.parse(check(manifest.namespace.path, manifest.namespace));
  assert.equal(
    manifest.originalFamily.sha256,
    "052f284b2d11f94e6d81dc68096ae74e1334c8ab4323ccaa01951e94e20a7fd1",
  );
  assert.equal(
    manifest.namespace.sha256,
    "6b5aa9402ad8927e9eb68d4dcfaca99597bfa0065718bb23d7e06dbd3ebc412f",
  );
  const family = [...original.sources, ...original.registeredRuleBodies];
  assert.equal(family.length, 23);
  assert.equal(manifest.pairs.length, 23);
  assert.equal(new Set(manifest.pairs.map(({ path: name }) => name)).size, 23);
  assert.equal(new Set(manifest.pairs.map(({ beforePath: name }) => name)).size, 23);
  absent(root, manifest.relocation.before);
  for (const [index, source] of family.entries()) {
    const pair = manifest.pairs[index];
    const before = source.path === namespace.test.path ? namespace.test.after : source.after;
    assert.equal(pair.beforePath, source.path);
    assert.deepEqual(pair.before, before);
    const prior = check(before.archive, before);
    const after = check(pair.after.archive, pair.after);
    const actual = check(pair.path, pair.after);
    assert.deepEqual(actual, after);
    if (source.path === carrier) {
      assert.equal(pair.path, carrier);
      const attribute = '#[path = "registered_sfc.rs"]\n';
      assert.equal(prior.toString().split(attribute).length, 2);
      assert.deepEqual(after, Buffer.from(prior.toString().replace(attribute, "")));
    } else {
      assert.deepEqual(after, prior);
      assert.equal(
        pair.path,
        source.path === manifest.relocation.before ? manifest.relocation.after : source.path,
      );
    }
    for (const side of [source.before, source.after]) {
      if (side.archive) check(side.archive, side);
    }
  }
  const relocated = manifest.pairs.find(
    ({ beforePath: name }) => name === manifest.relocation.before,
  );
  assert(relocated);
  assert.equal(relocated.path, manifest.relocation.after);
  for (const expected of manifest.controls) check(expected.path, expected);
  const priorHelper = check(manifest.sourceHelper.archive, manifest.sourceHelper);
  regular(root, wrapperPath);
  assert.deepEqual(read(wrapperPath), composeNoMutatingModuleLayoutHelper(priorHelper));
  assert.deepEqual(fs.readFileSync(path.join(root, wrapperPath)), read(wrapperPath));
  for (const law of manifest.historicalLaws) {
    check(law.archive, law);
    regular(root, law.path);
    assert.deepEqual(read(law.path), composeNoMutatingModuleLayoutLaw(law.path));
    assert.deepEqual(fs.readFileSync(path.join(root, law.path)), read(law.path));
  }
  const previous = check(manifest.ledger.before.archive, manifest.ledger.before).toString();
  const afterLedger = check(manifest.ledger.after.archive, manifest.ledger.after).toString();
  check(manifest.ledger.path, manifest.ledger.after);
  const a = previous.trimEnd().split("\n");
  const b = afterLedger
    .replaceAll(manifest.relocation.after, manifest.relocation.before)
    .trimEnd()
    .split("\n");
  assert.equal(new Set(a).size, a.length);
  assert.equal(new Set(b).size, b.length);
  assert.deepEqual(a.toSorted(), b.toSorted());
  assert.equal(previous.split(manifest.relocation.before).length, 3);
  return {
    manifest,
    paths: manifest.pairs.map(({ path: name }) => name),
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  };
}

function snapshot(root, manifest) {
  regular(root, manifest.snapshot.path);
  const body = fs.readFileSync(path.join(root, manifest.snapshot.path));
  identity(body, manifest.snapshot, manifest.snapshot.path);
  const packet = JSON.parse(gunzipSync(body));
  assert.equal(packet.schema, "vize.lint.no-mutating-props-ancestor-source-snapshot");
  assert.equal(packet.version, 1);
  assert.equal(packet.source, "2690dafc8af3e7a78095f287eaaf494eb33d0eaf");
  assert.deepEqual(packet.credit, { compiled: false, native: false, performance: false });
  assert.equal(new Set(packet.files.map(({ path: name }) => name)).size, packet.files.length);
  const bodies = new Map();
  for (const expected of packet.files) {
    assert.equal(expected.mode, "100644", expected.path);
    assert(
      expected.path &&
        !path.isAbsolute(expected.path) &&
        path.normalize(expected.path) === expected.path &&
        !expected.path.split("/").includes(".."),
    );
    const bytes = Buffer.from(expected.base64, "base64");
    assert.equal(bytes.toString("base64"), expected.base64);
    identity(bytes, expected, expected.path);
    bodies.set(expected.path, bytes);
  }
  for (const pair of manifest.pairs)
    identity(bodies.get(pair.beforePath), pair.before, pair.beforePath);
  for (const law of manifest.historicalLaws)
    assert.deepEqual(bodies.get(law.path), fs.readFileSync(path.join(root, law.archive)));
  assert.deepEqual(
    bodies.get(wrapperPath),
    fs.readFileSync(path.join(root, manifest.sourceHelper.archive)),
  );
  return bodies;
}
function materialize(bodies) {
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-ancestor-"));
  for (const [name, body] of bodies) {
    const target = path.join(owned, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, body, { mode: 0o644, flag: "wx" });
  }
  return owned;
}
const materialized = new Map();
process.once("exit", () => {
  for (const owned of materialized.values()) fs.rmSync(owned, { recursive: true, force: true });
});

/** All current bytes and old-path absence are rechecked before each exact selection. */
export function reviewedNoMutatingModuleLayoutReadView(root, read = reader(root)) {
  // A literal ancestor receives only its original predicate's authority.
  if (digest(read(carrier)) === priorCarrierSHA) return { root, read };
  const { manifest } = authenticateNoMutatingModuleLayoutSourceFamily(root, read);
  const bodies = snapshot(root, manifest);
  const key = manifest.snapshot.sha256;
  if (!materialized.has(key)) materialized.set(key, materialize(bodies));
  const historicalRoot = materialized.get(key);
  const projection = new Map(
    manifest.historicalProjection.map(({ path: name, archive }) => [name, archive]),
  );
  return {
    root: historicalRoot,
    read: (name, ...args) => read(projection.get(name) ?? name, ...args),
  };
}

/** Execute either original law module whole, with its genuine ancestor physical roots. */
export async function runNoMutatingModuleLayoutHistoricalLaw(root, law) {
  const { manifest } = authenticateNoMutatingModuleLayoutSourceFamily(root);
  assert(manifest.historicalLaws.some(({ path: name }) => name === law));
  const bodies = snapshot(root, manifest);
  const owned = materialize(bodies);
  try {
    const declared = manifest.historicalLaws.find(({ path: name }) => name === law);
    assert.deepEqual(
      fs.readFileSync(path.join(owned, law)),
      fs.readFileSync(path.join(root, declared.archive)),
    );
    await import(pathToFileURL(path.join(owned, law)).href);
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
}
