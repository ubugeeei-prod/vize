import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  authenticateNoMutatingTestCorrection,
  composeNoMutatingTestCorrectionHelper,
  noMutatingTestCorrectionDirectory,
  noMutatingTestCorrectionManifestSHA256,
  reviewedNoMutatingTestCorrectionReadView,
} from "./support/no-mutating-props-test-correction-source-custody.mjs";
import { authenticateNoMutatingPropsSourceFamily } from "./support/no-mutating-props-source-custody.mjs";
import { renderConsumerMigrationShards } from "../../tools/support/compat/davinci/lib/consumer-migration-render.mjs";
import { scanConsumerMigrationSurfaces } from "../../tools/support/compat/davinci/lib/consumer-migration-scan.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (name, ...args) => fs.readFileSync(path.join(root, name), ...args);
const hash = (body) => createHash("sha256").update(body).digest("hex");
const guardPath = "tests/tooling/support/no-mutating-props-test-correction-source-custody.mjs";
const guardSHA = "53a1af6753ffdab235406c70752cb871c57cf6a9c9971204b42083500a6af7a2";
const manifestPath = `${noMutatingTestCorrectionDirectory}/successor.json`;
const ledgerPath = `${noMutatingTestCorrectionDirectory}/generated-ledger.json`;
const ledgerSHA = "2eb8718288557d569bd5d7354f36e07cbd0f9fd6665eb7924f4419cd30d92f36";
const current = () => authenticateNoMutatingTestCorrection(root, read);
const identity = (body, expected) => {
  assert.equal(body.length, expected.bytes);
  assert.equal(hash(body), expected.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    expected.gitBlob,
  );
};

await test("the complete 23-body correction retains all nine tests and four historical laws", () => {
  assert.equal(hash(read(manifestPath)), noMutatingTestCorrectionManifestSHA256);
  // This external law seals the whole executing helper; it does not claim self-authentication.
  assert.equal(hash(read(guardPath)), guardSHA);
  const receipt = current();
  assert.equal(receipt.paths.length, 23);
  assert.equal(new Set(receipt.paths).size, 23);
  assert.deepEqual(
    [receipt.sourceOnly, receipt.compiled, receipt.native, receipt.performance],
    [true, false, false, false],
  );
  const { manifest } = receipt;
  const before = read(manifest.test.before.archive).toString();
  const after = read(manifest.test.path).toString();
  assert.equal(before.split("#[test]").length - 1, 9);
  assert.equal(after.split("#[test]").length - 1, 9);
  assert.equal(after.split("crate::builtin_script_rules()").length, 2);
  assert.equal(
    after.replace("crate::builtin_script_rules()", "crate::linter::builtin_script_rules()"),
    before,
  );
  const oldHelper = read(manifest.originalHelper.archive);
  assert.deepEqual(
    read(manifest.originalHelper.path),
    composeNoMutatingTestCorrectionHelper(oldHelper),
  );
  const helper = read(manifest.originalHelper.path).toString();
  const addedImport =
    'import { reviewedNoMutatingTestCorrectionReadView } from "./no-mutating-props-test-correction-source-custody.mjs";\n';
  const addedEntry = "  read = reviewedNoMutatingTestCorrectionReadView(root, read);\n";
  assert.equal(helper.split(addedImport).length, 2);
  assert.equal(helper.split(addedEntry).length, 2);
  assert.equal(helper.replace(addedImport, "").replace(addedEntry, ""), oldHelper.toString());
  assert.deepEqual(read(manifest.originalFourLaws.path), read(manifest.originalFourLaws.archive));
  assert.equal(read(manifest.originalFourLaws.path).toString().split("await test(").length - 1, 4);
  assert.equal(authenticateNoMutatingPropsSourceFamily(root, read).registeredRuleBodies.length, 11);
});

await test("every complete-family or declaration corruption and omission is refused", () => {
  const observed = new Set();
  authenticateNoMutatingTestCorrection(root, (name) => {
    observed.add(name);
    return read(name);
  });
  for (const name of observed) {
    const foreign = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateNoMutatingTestCorrection(root, foreign), name);
    const omitted = (requested) => {
      assert.notEqual(requested, name, `missing ${name}`);
      return read(requested);
    };
    assert.throws(() => authenticateNoMutatingTestCorrection(root, omitted), name);
    assert.equal(current().paths.length, 23);
  }
  const { manifest } = current();
  const old = (name, ...args) =>
    read(name === manifest.test.path ? manifest.test.before.archive : name, ...args);
  assert.throws(() => authenticateNoMutatingTestCorrection(root, old));
  assert.equal(reviewedNoMutatingTestCorrectionReadView(root, old), old);
  // This is the unchanged predecessor receipt, with no new correction context.
  assert.equal(authenticateNoMutatingPropsSourceFamily(root, old).registeredRuleBodies.length, 11);
  assert.equal(current().paths.length, 23);
});

await test("matching callbacks cannot hide physical mutations, links or missing family members", () => {
  const observed = new Set();
  const receipt = authenticateNoMutatingTestCorrection(root, (name) => {
    observed.add(name);
    return read(name);
  });
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-test-correction-"));
  try {
    for (const name of observed) {
      const target = path.join(owned, name);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, read(name), { mode: 0o644 });
    }
    assert.equal(authenticateNoMutatingTestCorrection(owned, read).paths.length, 23);
    for (const name of receipt.paths) {
      const target = path.join(owned, name);
      const body = read(name);
      fs.writeFileSync(target, Buffer.concat([body, Buffer.from(" ")]));
      assert.throws(() => authenticateNoMutatingTestCorrection(owned, read), undefined, name);
      fs.writeFileSync(target, body);
      fs.chmodSync(target, 0o755);
      assert.throws(() => authenticateNoMutatingTestCorrection(owned, read), undefined, name);
      fs.chmodSync(target, 0o644);
      fs.unlinkSync(target);
      assert.throws(() => authenticateNoMutatingTestCorrection(owned, read), undefined, name);
      fs.symlinkSync(path.join(root, name), target);
      assert.throws(() => authenticateNoMutatingTestCorrection(owned, read), undefined, name);
      fs.unlinkSync(target);
      fs.mkdirSync(target);
      assert.throws(() => authenticateNoMutatingTestCorrection(owned, read), undefined, name);
      fs.rmdirSync(target);
      fs.writeFileSync(target, body, { mode: 0o644 });
      assert.equal(authenticateNoMutatingTestCorrection(owned, read).paths.length, 23);
    }
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});

await test("the sole correction read view preserves binary, UTF8 and unrelated arguments", () => {
  const { manifest } = current();
  const calls = [];
  const tracked = (name, ...args) => {
    calls.push([name, ...args]);
    return read(name, ...args);
  };
  const view = reviewedNoMutatingTestCorrectionReadView(root, tracked);
  calls.length = 0;
  assert.deepEqual(view(manifest.test.path), read(manifest.test.before.archive));
  assert.deepEqual(calls, [[manifest.test.before.archive]]);
  calls.length = 0;
  assert.equal(view(manifest.test.path, "utf8"), read(manifest.test.before.archive, "utf8"));
  assert.deepEqual(calls, [[manifest.test.before.archive, "utf8"]]);
  calls.length = 0;
  const unrelated = manifest.originalFourLaws.path;
  assert.equal(view(unrelated, "utf8"), read(unrelated, "utf8"));
  assert.deepEqual(calls, [[unrelated, "utf8"]]);
  const token = { unchanged: true };
  let forwarded;
  const custom = (name, ...args) => {
    if (name === "outside") {
      forwarded = [name, ...args];
      return token;
    }
    return read(name, ...args);
  };
  const selected = reviewedNoMutatingTestCorrectionReadView(root, custom);
  assert.equal(selected("outside", "literal", 42), token);
  assert.deepEqual(forwarded, ["outside", "literal", 42]);
});

await test("the one generated ledger and unchanged full generator closure remain reproducible", () => {
  assert.equal(hash(read(ledgerPath)), ledgerSHA);
  const ledger = JSON.parse(read(ledgerPath));
  assert.equal(ledger.generatorSources.length, 11);
  assert.equal(new Set(ledger.generatorSources.map(({ path: name }) => name)).size, 11);
  for (const source of ledger.generatorSources) {
    const status = fs.lstatSync(path.join(root, source.path));
    assert(status.isFile() && !status.isSymbolicLink());
    identity(read(source.path), source);
  }
  const before = read(ledger.ledger.before.archive);
  const after = read(ledger.ledger.after.archive);
  identity(before, ledger.ledger.before);
  identity(after, ledger.ledger.after);
  assert.deepEqual(read(ledger.ledger.path), after);
  assert.equal(before.toString().split("\n").length, 1304);
  assert.equal(after.toString().split("\n").length, 1315);
  const a = new Set(before.toString().trimEnd().split("\n"));
  const b = new Set(after.toString().trimEnd().split("\n"));
  assert.equal([...a].filter((line) => !b.has(line)).length, 3);
  assert.equal([...b].filter((line) => !a.has(line)).length, 14);
  const rendered = renderConsumerMigrationShards(scanConsumerMigrationSurfaces());
  assert.equal(rendered.filter(({ relPath }) => relPath === ledger.ledger.path).length, 1);
  assert.deepEqual(
    Buffer.from(rendered.find(({ relPath }) => relPath === ledger.ledger.path).text),
    after,
  );
  // Original wrapper delegates to this same JavaScript generator; no Rust is executed here.
  const wrapper = read("tools/commands/davinci/consumer-migration-surfaces.rs", "utf8");
  assert(
    wrapper.includes(
      "artifact_command::run_node_generator(env::args().nth(1).as_deref(), LEGACY_GENERATOR, USAGE)",
    ),
  );
});
