import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { authenticateDeepPropsSourceFamily } from "./support/component-value-retained-guards.ts";
import { authenticateRuntimeApiSourceFamily } from "./support/component-value-retained-guards.ts";
import { authenticateSetupExportPresetSourceFamily } from "./support/component-value-retained-guards.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/lint/props-deep-destructure-8090";
const read = (name) => fs.readFileSync(path.join(root, directory, name));
const json = (name) => JSON.parse(read(name));
const archive = (name) => JSON.parse(gunzipSync(read(name)));
const blob = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

await test("whole original eight roles and every initial refusal remain immutable", () => {
  assert.equal(
    sha256(read("original-eight.json")),
    "ec0a00983c97eacab181f8b707e0150e9535e8d66b970c05920c0a0bc77ad40e",
  );
  assert.equal(
    sha256(gunzipSync(read("original-eight-observation.json.gz"))),
    "af8bb17ac06f50db11f552a90b09c3361e5ce3c254a4b4e11bcd0c55dda9dbb6",
  );
  const manifest = json("archive-manifest.json");
  for (const entry of manifest.files) {
    const bytes = read(entry.path);
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(sha256(bytes), entry.sha256, entry.path);
    if (entry.rawSha256) {
      const raw = gunzipSync(bytes);
      assert.equal(raw.length, entry.rawBytes);
      assert.equal(sha256(raw), entry.rawSha256);
    }
  }
  const original = json("original-eight.json");
  const observed = archive("original-eight-observation.json.gz");
  assert.equal(original.cases.length, 8);
  assert.equal(observed.cases.length, 8);
  assert.equal(
    observed.native.binary.sha256,
    "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254",
  );
  for (const [index, entry] of original.cases.entries()) {
    const packet = observed.cases[index];
    assert.equal(packet.id, entry.id);
    assert.equal(packet.source, entry.source);
    assert.equal(sha256(Buffer.from(packet.source)), entry.sourceSha256);
    assert.deepEqual(packet.native.wholeJson, entry.sourceAuthoredCurrentCli);
    assert.equal(packet.native.status, entry.sourceAuthoredCurrentStatus);
    assert.equal(packet.native.stderrBase64, "");
    assert.equal(packet.native.signal, null);
    assert.equal(packet.native.error, null);
    assert.equal(
      sha256(Buffer.from(packet.native.stdoutBase64, "base64")),
      packet.native.stdoutSha256,
    );
    assert.deepEqual(
      JSON.parse(Buffer.from(packet.native.stdoutBase64, "base64").toString()),
      packet.native.wholeJson,
    );
    assert.equal(packet.finalSource, entry.source);
    if (entry.sourceInferredCompilerAccepted === null) {
      assert.equal(packet.compiler.applicable, false);
      assert.equal(packet.compiler.expectedAccepted, null);
    } else assert.equal(packet.compiler.accepted, entry.sourceInferredCompilerAccepted);
    assert.deepEqual(packet.relatedProvider.frozenExpected, entry.sourceAuthoredRelatedProvider);
    assert.equal(packet.relatedProvider.expectedMatch, false);
  }
});

await test("finite optional-before and scalar-after corrections preserve whole source proofs", () => {
  assert.equal(
    sha256(read("original-neighbors.json")),
    "ffc6df355e15eb06d128dc593652c508eeced0b78bc9bb423ac7a1199affa22f",
  );
  assert.equal(
    sha256(read("cases.json")),
    "d522ae6c3fc8a04403c43661c7d66584188a2168b1e01947093044f4239de8c8",
  );
  const original = json("original-neighbors.json");
  const corpus = json("cases.json");
  const law = json("before-authorship-correction.json");
  const unicode = json("source-custody/unicode-after-only-source-proof.json");
  assert.equal(
    sha256(read("source-custody/unicode-after-only-source-proof.json")),
    "5dffe4cbf3ab54ed830ab903133950f81d5587e1095984ad6f243a4236252852",
  );
  const unicodeExpected = structuredClone(unicode.before);
  unicodeExpected.afterCli[0].messages[0].column = 28;
  unicodeExpected.afterCli[0].messages[0].endColumn = 46;
  assert.deepEqual(unicode.after, unicodeExpected);
  for (const entry of unicode.producerSources) {
    const bytes = read("source-custody/" + path.basename(entry.archive));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(blob(bytes), entry.gitBlob);
  }
  const body = read(law.source.archive.path);
  assert.equal(body.length, law.source.archive.bytes);
  assert.equal(sha256(body), law.source.archive.sha256);
  assert.equal(blob(body), law.source.gitBlob);
  assert(body.toString().includes("let Expression::CallExpression(call) = expression else"));
  const proof = json("source-custody/optional-before-source-proof.json");
  for (const entry of [proof.parser, proof.ast]) {
    const source = read("source-custody/" + path.basename(entry.archive));
    assert.equal(source.length, entry.bytes);
    assert.equal(sha256(source), entry.sha256);
    if (entry.gitBlob) assert.equal(blob(source), entry.gitBlob);
  }
  assert.equal(original.cases.length, 31);
  assert.equal(corpus.cases.length, 33);
  const extra = json("reference/ts-wrapped-callee-authored-controls.json");
  assert.equal(
    sha256(read("reference/ts-wrapped-callee-authored-controls.json")),
    "4e6c693e881e8d547846b0a99d653d7c0eca784425b748b08c2288630fa71b4d",
  );
  const compiler = json("reference/ts-wrapped-callee-compiler-reference.json");
  assert.equal(
    sha256(read("reference/ts-wrapped-callee-compiler-reference.json")),
    "b13e3bb3eab24f9d2b7f5a74c8a40e94f6ac8126b8d03920324236484fecc544",
  );
  assert.equal(extra.cases.length, 2);
  for (const [index, packet] of compiler.cases.entries()) {
    assert.equal(packet.source, extra.cases[index].source);
    assert.equal(packet.accepted, true);
    assert.equal(packet.expectedAcceptanceMatch, true);
  }
  let beforeWarnings = 0,
    afterWarnings = 0;
  for (const [index, entry] of corpus.cases.entries()) {
    const { sourceFile, ...packet } = entry;
    const previous = index < 31 ? original.cases[index] : extra.cases[index - 31];
    const expected =
      previous.id === law.id
        ? law.after
        : previous.id === unicode.before.id
          ? unicode.after
          : previous;
    assert.deepEqual(packet, expected);
    assert.equal(read(sourceFile).toString(), entry.source);
    assert.equal(sha256(read(sourceFile)), entry.sourceSha256);
    assert.deepEqual(entry.beforeCli, wholeJson(entry, entry.beforeApi.diagnostics));
    assert.deepEqual(entry.afterCli, wholeJson(entry, entry.afterApi.diagnostics));
    beforeWarnings += entry.beforeApi.warning_count;
    afterWarnings += entry.afterApi.warning_count;
  }
  assert.equal(beforeWarnings, 21);
  assert.equal(afterWarnings, 10);
  assert.equal(law.id, "setup-optional-call");
  assert.deepEqual(law.after.afterApi, law.before.afterApi);
  assert.deepEqual(law.after.afterCli, law.before.afterCli);
});

await test("all thirty one public before packets preserve raw processes and source bytes", () => {
  const initial = archive("initial-public-before31-failure.json.gz");
  const before = archive("public-before31.json.gz");
  const cases = json("cases.json").cases;
  assert.deepEqual(initial.summary, { cases: 31, matched: 30, diagnostics: 21 });
  assert.deepEqual(before.summary, { cases: 31, matched: 31, diagnostics: 21 });
  for (const [index, packet] of before.runs.entries()) {
    const original = initial.runs[index];
    const { expected, matched, ...actual } = packet;
    const { expected: _previous, matched: _matched, ...oldActual } = original;
    assert.deepEqual(actual, oldActual);
    const entry = cases[index];
    assert.equal(packet.id, entry.id);
    assert.equal(packet.source, entry.source);
    assert.equal(packet.sourceSha256, entry.sourceSha256);
    assert.equal(packet.finalSource, entry.source);
    assert.equal(packet.finalSourceSha256, entry.sourceSha256);
    assert.deepEqual(expected, entry.beforeCli);
    assert.equal(packet.status, packet.expectedStatus);
    assert.equal(packet.signal, null);
    assert.equal(packet.error, null);
    assert.equal(packet.stderrBase64, "");
    assert.equal(sha256(Buffer.from(packet.stdoutBase64, "base64")), packet.stdoutSha256);
    assert.deepEqual(JSON.parse(Buffer.from(packet.stdoutBase64, "base64").toString()), expected);
    assert.deepEqual(packet.actual, expected);
    assert.equal(matched, true);
  }
});

await test("the related provider's eight whole path refusals change only physical filePath", () => {
  const law = json("provider-path-correction.json");
  assert.equal(
    sha256(read("provider-path-correction.json")),
    "98463973d2d6e19449dfa74982b8664d85b736267d6d0b76a9f26f2eb50a4263",
  );
  const original = json("original-eight.json");
  const observed = archive("original-eight-observation.json.gz");
  assert.equal(law.cases.length, 8);
  for (const [index, entry] of law.cases.entries()) {
    assert.equal(entry.id, original.cases[index].id);
    assert.deepEqual(entry.before, original.cases[index].sourceAuthoredRelatedProvider);
    const expected = structuredClone(entry.before);
    assert.equal(expected.length, 1);
    assert.equal(expected[0].filePath, law.beforePrefix + original.cases[index].filename);
    expected[0].filePath = law.afterPrefix + original.cases[index].filename;
    assert.deepEqual(entry.after, expected);
    assert.deepEqual(entry.after, observed.cases[index].relatedProvider.wholeResult);
  }
});

await test("the complete twelve-body depth successor refuses unknown and mixed source", () => {
  const readRoot = (name) => fs.readFileSync(path.join(root, name));
  const manifest = authenticateDeepPropsSourceFamily(root, readRoot);
  assert.equal(manifest.sources.length, 12);
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 3);
  const paths = [
    ...new Set([
      `${directory}/source-custody/successor.json`,
      manifest.incomingFamily.path,
      ...manifest.sources.flatMap((source) => [
        source.path,
        source.before.archive,
        source.after.archive,
      ]),
      ...[manifest.producer.before, manifest.producer.after].flatMap((source) => [
        source.path,
        source.archive,
      ]),
    ]),
  ];
  for (const name of paths) {
    const corrupt = (requested) =>
      requested === name
        ? Buffer.concat([readRoot(requested), Buffer.from(" ")])
        : readRoot(requested);
    assert.throws(() => authenticateDeepPropsSourceFamily(root, corrupt), name);
  }
  for (const source of manifest.sources.filter(({ changed }) => changed)) {
    const mixed = (name) =>
      name === source.path ? readRoot(source.before.archive) : readRoot(name);
    assert.throws(() => authenticateDeepPropsSourceFamily(root, mixed), source.path);
  }
});

await test("five additional public before packets preserve wrapped callees and explicit off", () => {
  const bytes = gunzipSync(read("public-additional-before5.json.gz"));
  assert.equal(sha256(bytes), "bd35b8d2ff32a84addf8d67b3d0849057c6c73fd86a8d98d48665ac7cee9faa1");
  const evidence = JSON.parse(bytes);
  assert.equal(evidence.runs.length, 5);
  assert.equal(
    evidence.public.binarySha256,
    "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254",
  );
  const cases = json("cases.json").cases;
  for (const [index, packet] of evidence.runs.entries()) {
    const entry = cases.find(({ id }) => id === packet.id);
    assert(entry);
    assert.equal(packet.route, index < 2 ? "scoped" : "explicit-off");
    assert.equal(packet.source, entry.source);
    assert.equal(packet.sourceSha256, entry.sourceSha256);
    assert.equal(packet.finalSource, entry.source);
    assert.equal(packet.finalSourceSha256, entry.sourceSha256);
    const expected =
      index < 2
        ? entry.beforeCli
        : [{ file: entry.filename, messages: [], errorCount: 0, warningCount: 0 }];
    assert.deepEqual(packet.expected, expected);
    assert.deepEqual(packet.actual, expected);
    assert.equal(packet.status, 0);
    assert.equal(packet.expectedStatus, 0);
    assert.equal(packet.signal, null);
    assert.equal(packet.error, null);
    assert.equal(packet.stderrBase64, "");
    assert.equal(packet.matched, true);
    assert.equal(sha256(Buffer.from(packet.stdoutBase64, "base64")), packet.stdoutSha256);
    assert.deepEqual(JSON.parse(Buffer.from(packet.stdoutBase64, "base64").toString()), expected);
    assert.equal(sha256(Buffer.from(packet.config)), packet.configSha256);
    if (index >= 2)
      assert.deepEqual(JSON.parse(packet.config), {
        linter: { preset: "incremental", rules: { "script/no-deep-destructure-in-props": "off" } },
      });
  }
});

await test("old source callers admit only the complete successor and retain whole helper history", () => {
  const readRoot = (name) => fs.readFileSync(path.join(root, name));
  const bytes = read("source-custody/helper-integration/manifest.json");
  assert.equal(sha256(bytes), "0a3c546ac610655d29d6f4d059a6950e195fa1ed380c77c1ebd56aca9105ba9d");
  const integration = JSON.parse(bytes);
  assert.equal(integration.helpers.length, 2);
  for (const helper of integration.helpers) {
    for (const arm of [helper.before, helper.after]) {
      const body = read("source-custody/helper-integration/" + arm.archive);
      assert.equal(body.length, arm.bytes);
      assert.equal(sha256(body), arm.sha256);
      assert.equal(blob(body), arm.gitBlob);
    }
    assert.equal(sha256(readRoot(helper.path)), helper.after.sha256);
  }
  const runtime = authenticateRuntimeApiSourceFamily(root, readRoot);
  const preset = authenticateSetupExportPresetSourceFamily(root, runtime, readRoot);
  const depth = authenticateDeepPropsSourceFamily(root, readRoot);
  assert.equal(runtime.sources.length, 12);
  assert.equal(preset.sources.length, 12);
  for (const source of depth.sources) {
    const unknown = (name) =>
      name === source.path ? Buffer.concat([readRoot(name), Buffer.from(" ")]) : readRoot(name);
    assert.throws(() => authenticateRuntimeApiSourceFamily(root, unknown), source.path);
    assert.throws(
      () => authenticateSetupExportPresetSourceFamily(root, runtime, unknown),
      source.path,
    );
  }
  for (const source of depth.sources.filter(({ changed }) => changed)) {
    const mixed = (name) =>
      name === source.path ? readRoot(source.before.archive) : readRoot(name);
    assert.throws(() => authenticateRuntimeApiSourceFamily(root, mixed), source.path);
    assert.throws(
      () => authenticateSetupExportPresetSourceFamily(root, runtime, mixed),
      source.path,
    );
  }
});
