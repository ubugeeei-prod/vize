import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { authenticateRuntimeApiSourceFamily } from "./support/runtime-props-api-source-custody.mjs";
import {
  authenticateSetupExportMembership,
  authenticateSetupExportPresetSourceFamily,
} from "./support/setup-export-preset-source-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/lint/setup-export-default-8090";
const read = (name) => fs.readFileSync(path.join(root, name));
const fixture = (name) => read(`${directory}/${name}`);
const json = (name) => JSON.parse(fixture(name));
const archive = (name) => JSON.parse(gunzipSync(fixture(name)));
// These finite packets have distinct spans. The pinned CLI comparator sorts by
// start/end before its remaining tie breakers; preserve the API's rule order.
const cliOrder = (diagnostics) => {
  assert.equal(
    new Set(diagnostics.map(({ start, end }) => `${start}:${end}`)).size,
    diagnostics.length,
  );
  return diagnostics.toSorted((left, right) => left.start - right.start || left.end - right.end);
};

await test("eighteen whole pinned provider/compiler packets preserve their separate contracts", () => {
  assert.equal(
    sha256(fixture("cases.json")),
    "d4d9d15024a4df328fd7b32fca416fbf6d92a99952824e6b4405b74a6854ffcb",
  );
  assert.equal(
    sha256(fixture("default-route-controls.json")),
    "ef170231466cc8053a3dd7ee0a7b0742261ac4c74cc39032941085c8a031bfab",
  );
  const manifest = json("provider-archive-manifest.json");
  assert.equal(
    sha256(fixture("provider-archive-manifest.json")),
    "fc0e251c162b7c192af3847b17b209aa78f3f77e37b6ec89c852c34e316576c8",
  );
  for (const entry of manifest.files) {
    const bytes = fixture(entry.path);
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(bytes.length, entry.bytes);
    if (entry.rawSha256) {
      const raw = gunzipSync(bytes);
      assert.equal(sha256(raw), entry.rawSha256);
      assert.equal(raw.length, entry.rawBytes);
    }
  }
  const authored = archive("reference/authored-provider18.json.gz");
  const observed = archive("reference/official-provider18.json.gz");
  const compiler = json("compiler-reference.json");
  assert.equal(
    sha256(fixture("compiler-reference.json")),
    "2a1748dad2d77cf4542e70c35b43c2a422e4e991f369f8e5a46ea83b11d8c898",
  );
  assert.equal(
    sha256(fixture("authored-inputs.json")),
    "db5a69507f8b01c966c6549a0534b5807126629952334706ffe5371de03c3b97",
  );
  assert.equal(
    observed.authoredSha256,
    "0c1c4bb67848474d8a7fdf0dcc78a74d68668cbd67d6a8e873e381295e66eaaf",
  );
  assert.equal(authored.cases.length, 18);
  assert.deepEqual(observed.summary, { cases: 18, matched: 18, diagnostics: 8, failed: 0 });
  assert.deepEqual(observed.verifiedPackagesBefore, observed.verifiedPackagesAfter);
  assert.deepEqual(observed.verifiedAssetsBefore, observed.verifiedAssetsAfter);
  for (const [index, entry] of json("cases.json").cases.entries()) {
    assert.equal(fixture(entry.sourceFile).toString(), entry.source);
    assert.equal(sha256(Buffer.from(entry.source)), entry.sourceSha256);
    assert.equal(authored.cases[index].source, entry.source);
    assert.deepEqual(observed.runs[index].expected, authored.cases[index].expectedProvider);
    assert.deepEqual(observed.runs[index].actual, authored.cases[index].expectedProvider);
    assert.equal(compiler.cases[index].source, entry.source);
    assert.equal(compiler.cases[index].compiler.accepted, authored.cases[index].compilerAccepted);
  }
  const registry = JSON.parse(fixture("reference/compiler-sfc-registry.json.fixture"));
  const tar = fixture("reference/compiler-sfc-3.5.43.tgz");
  assert.equal(registry.name, "@vue/compiler-sfc");
  assert.equal(registry.version, "3.5.43");
  assert.equal(
    registry.dist.integrity,
    "sha512-" + createHash("sha512").update(tar).digest("base64"),
  );
  assert.equal(registry.dist.shasum, createHash("sha1").update(tar).digest("hex"));
  assert.equal(
    sha256(gunzipSync(fixture("reference/compiler-sfc.cjs.js.gz"))),
    compiler.provider.sha256,
  );
});

await test("finite authorship correction keeps initial refusal and all seventy whole before packets", () => {
  const law = json("authorship-correction.json");
  assert.equal(
    sha256(fixture("authorship-correction.json")),
    "f09afe8e353ceb151f2151253e80c9d28a1061b6ab995d2c2cb7326b3690d1d7",
  );
  for (const entry of [law.initialAuthored, law.initialFailure, law.sourceLaw])
    assert.equal(sha256(fixture(entry.path)), entry.sha256);
  for (const entry of json("source-law-bodies.json").sources) {
    const bytes = Buffer.from(entry.source);
    assert.equal(bytes.length, entry.bytes);
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(sha256(read(entry.path)), entry.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
      entry.gitBlob,
    );
  }
  const initial = archive("initial-authored-packets.json.gz");
  const corpus = json("cases.json");
  const controls = json("default-route-controls.json");
  for (const [index, entry] of corpus.cases.entries()) {
    const old = initial.corpus.cases[index];
    assert.deepEqual(
      { ...entry, beforeCliPacket: old.beforeCliPacket, afterCliPacket: old.afterCliPacket },
      old,
    );
    assert.deepEqual(entry.beforeCliPacket, wholeJson(entry));
    assert.deepEqual(entry.beforeCliPacket, entry.afterCliPacket);
  }
  const oldFailure = archive("initial-public-before-failure.json.gz");
  assert.equal(oldFailure.runs.length, 70);
  assert.equal(
    oldFailure.runs.filter((run) => JSON.stringify(run.actual) !== JSON.stringify(run.expected))
      .length,
    10,
  );
  const before = archive("original-public-before.json.gz");
  assert.equal(before.runs.length, 70);
  assert.equal(
    before.public.binarySha256,
    "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254",
  );
  for (const run of before.runs) {
    assert.equal(sha256(Buffer.from(run.source)), run.sourceSha256);
    assert.equal(run.status, run.expectedStatus);
    assert.equal(run.stderrBase64, "");
    assert.deepEqual(
      JSON.parse(Buffer.from(run.stdoutBase64, "base64").toString("utf8")),
      run.expected,
    );
    assert.deepEqual(run.actual, run.expected);
  }
  assert.equal(controls.cases.length, 4);
  for (const entry of controls.cases) {
    assert.equal(fixture(entry.sourceFile).toString(), entry.source);
    assert.equal(entry.routes.length, 13);
    for (const route of entry.routes) {
      const previous = before.runs.find((run) => run.id === entry.id && run.route === route.id);
      assert.deepEqual(previous.expected, route.beforeCliPacket);
      assert.deepEqual(
        route.afterCliPacket,
        wholeJson(entry, cliOrder(route.apiAfterPacket.diagnostics)),
      );
      assert.deepEqual(
        route.beforeCliPacket,
        wholeJson(entry, cliOrder(route.apiBeforePacket.diagnostics)),
      );
    }
  }
});

await test("registry-only complete twelve-body successor refuses mixed or unknown source families", () => {
  const incoming = authenticateRuntimeApiSourceFamily(root);
  const manifest = authenticateSetupExportPresetSourceFamily(root, incoming);
  assert.equal(manifest.sources.length, 12);
  const names = [
    ...new Set([
      `${directory}/source-custody/successor.json`,
      manifest.incomingFamily.path,
      ...manifest.sources.flatMap((source) => [
        source.path,
        source.before.archive,
        source.after.archive,
      ]),
    ]),
  ];
  for (const name of names) {
    const corrupt = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateRuntimeApiSourceFamily(root, corrupt), name);
  }
  assert.throws(() =>
    authenticateSetupExportPresetSourceFamily(root, { ...incoming, foreign: true }),
  );
  assert.throws(() =>
    authenticateSetupExportPresetSourceFamily(root, {
      ...incoming,
      sources: [...incoming.sources].reverse(),
    }),
  );
});

await test("five default memberships add only the existing setup-export rule in registry order", () => {
  const law = json("source-custody/membership-law.json");
  const old = fixture("source-custody/original-preset-membership.snap.fixture");
  assert.equal(sha256(old), law.beforeSha256);
  const current = read(law.path);
  assert.equal(sha256(current), law.afterSha256);
  const decode = (bytes) => JSON.parse(bytes.toString().split("---\n").at(-1));
  const expected = decode(old);
  assert.equal(law.changes.length, 5);
  for (const change of law.changes) {
    const values = expected[change.preset];
    assert.equal(values.length, change.beforeCount);
    assert.equal(values[change.index], change.next);
    values.splice(change.index, 0, "script/no-export-in-script-setup");
    assert.equal(values.length, change.afterCount);
  }
  assert.deepEqual(decode(current), expected);
  assert.deepEqual(expected.incremental, []);
});

await test("original bare caller admits only the complete registry preset successor", () => {
  const bytes = fixture("source-custody/bare-caller-integration.json");
  assert.equal(sha256(bytes), "a5adc749279647772817f7ccabb307252631267d52adb46235ded98ad4886453");
  const law = JSON.parse(bytes);
  const before = fixture(`source-custody/${law.before.archive}`);
  assert.equal(sha256(before), law.before.sha256);
  assert.equal(before.length, law.before.bytes);
  assert.equal(
    createHash("sha1").update(`blob ${before.length}\0`).update(before).digest("hex"),
    law.before.gitBlob,
  );
  let expected = before.toString();
  for (const transform of law.transforms) {
    assert.equal(expected.split(transform.before).length, 2);
    expected = expected.replace(transform.before, transform.after);
  }
  const after = read(law.path);
  assert.deepEqual(after, Buffer.from(expected));
  assert.equal(sha256(after), law.after.sha256);
  assert.equal(after.length, law.after.bytes);
  assert.equal(law.originalRegistryBytes, 21_919);
  const log = gunzipSync(fixture(`source-custody/${law.initialFailure.archive}`));
  assert.equal(sha256(log), law.initialFailure.rawSha256);
  assert.equal(log.length, law.initialFailure.rawBytes);
  assert.equal(law.workflowRun, 38196958922);
  assert.equal(law.job, 114642149488);
});

await test("whole membership authentication refuses changed law, original or current tables", () => {
  const incoming = authenticateRuntimeApiSourceFamily(root);
  const law = authenticateSetupExportMembership(root, incoming);
  for (const name of [
    `${directory}/source-custody/membership-law.json`,
    `${directory}/source-custody/original-preset-membership.snap.fixture`,
    law.path,
  ]) {
    const corrupt = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateSetupExportMembership(root, incoming, corrupt), name);
  }
});

await test("finite caller and metadata repairs preserve whole originals and measured-host refusal", () => {
  const bytes = fixture("source-custody/policy-caller-integration.json");
  assert.equal(sha256(bytes), "5b1065fdafafa9b4e499406a45063f31ffb1b513ec2e200daec75a8865722080");
  const law = JSON.parse(bytes);
  assert.equal(law.source, "46ee5aadd5139401b6ecb687d5ad77ed63a7dee9");
  assert.equal(law.workflowRun, 38196958922);
  assert.equal(law.changes.length, 4);
  for (const change of law.changes) {
    const before = fixture(`source-custody/${change.before.archive}`);
    assert.equal(before.length, change.before.bytes);
    assert.equal(sha256(before), change.before.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${before.length}\0`).update(before).digest("hex"),
      change.before.gitBlob,
    );
    let expected = before.toString();
    for (const transform of change.transforms) {
      assert.equal(expected.split(transform.before).length, 2);
      expected = expected.replace(transform.before, transform.after);
    }
    const after = read(change.path);
    assert.deepEqual(after, Buffer.from(expected));
    assert.equal(after.length, change.after.bytes);
    assert.equal(sha256(after), change.after.sha256);
    if (change.path === "tests/_fixtures/patina-eslint-vue-rule-map.json") {
      const entries = JSON.parse(before).entries;
      entries["vue/no-export-in-script-setup"].patinaPresets = [
        "ecosystem",
        "essential",
        "general-recommended",
        "nuxt",
        "opinionated",
      ];
      assert.deepEqual(JSON.parse(after).entries, entries);
    }
  }
  assert.equal(law.preservedMeasuredHost.length, 2);
  for (const source of law.preservedMeasuredHost) {
    const before = fixture(`source-custody/${source.archive}`);
    assert.equal(before.length, source.bytes);
    assert.equal(sha256(before), source.sha256);
    assert.deepEqual(read(source.path), before);
  }
  assert.equal(law.initialFailures.length, 4);
  for (const failure of law.initialFailures) {
    const raw = fixture(`source-custody/${failure.archive}`);
    assert.equal(raw.length, failure.bytes);
    assert.equal(sha256(raw), failure.sha256);
    const log = gunzipSync(raw);
    assert.equal(log.length, failure.rawBytes);
    assert.equal(sha256(log), failure.rawSha256);
  }
});
