import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { assertReservedApiOrderSource } from "./support/runtime-props-api-source-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/bare-reserved-8090");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));

await test("whole default-route companions retain independent before proof and failed packet", () => {
  const frozen = {
    "default-route-companions.json":
      "9099e764f108551270406eae4794746754aba43b4aff33128e24f070004b9be9",
    "default-route-public-before.json":
      "7c97b01a6194fdece86e3dd3115ee4eb10a4ba3870584fe733fba3e6c47e7e65",
    "default-route-initial-failure.json":
      "ed16e8e2b3313c6621da5a9b5caa31b5269fb5e3e33c02f023f4fac1c54a84cf",
    "observe-default-route-before.mjs":
      "8c650adcdf3b150a9639533219e38a4968be7386e98ea257542f6376b0941f30",
  };
  for (const [name, digest] of Object.entries(frozen))
    assert.equal(sha256(read(name)), digest, name);
  const corpus = json("cases.json");
  const companions = json("default-route-companions.json");
  const receipt = json("default-route-public-before.json");
  const failure = json("default-route-initial-failure.json");
  const apiOrder = json("api-route-order-successors.json");
  assert.equal(
    sha256(read("api-route-order-successors.json")),
    "79ecb574f8f5cf5de955ff98cbf22faa7387e30cc70b5c4115cffecc3e1582a7",
  );
  assert.equal(apiOrder.schema, "vize.lint.api-route-order-successors");
  assert.equal(apiOrder.cases.length, 16);
  for (const source of apiOrder.producer) {
    if (source.path === "crates/vize_patina/src/linter/script_rules/registry/rules.rs")
      assertReservedApiOrderSource(root, { ...source, bytes: 21_919 });
    else assert.equal(sha256(fs.readFileSync(path.join(root, source.path))), source.sha256);
  }
  const apiFailure = gunzipSync(Buffer.from(apiOrder.initialFailure.log.packet, "base64"));
  assert.equal(sha256(apiFailure), apiOrder.initialFailure.log.sha256);
  assert.equal(apiFailure.length, apiOrder.initialFailure.log.rawBytes);
  assert.equal(apiOrder.initialFailure.workflowRun, 38153184272);
  assert.equal(apiOrder.initialFailure.job, 114511898269);
  assert.equal(companions.schema, "vize.lint.default-route-companions");
  assert.equal(companions.version, 1);
  assert.equal(companions.rule, "script/no-reserved-props");
  assert.equal(companions.cases.length, 16);
  assert.equal(
    companions.cases.reduce((n, c) => n + c.diagnostics.length, 0),
    2,
  );
  assert.equal(
    companions.cases.reduce((n, c) => n + c.combinedApiDiagnostics.length, 0),
    19,
  );
  assert.equal(
    companions.cases.reduce((n, c) => n + c.beforeDefaultDiagnostics.length, 0),
    139,
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(root, companions.producer.path))),
    companions.producer.sha256,
  );
  assert.deepEqual(companions.producer.reservedNames, ["key", "ref", "ref_for", "ref_key", "is"]);
  assert.equal(companions.producer.dollarPrefix, true);
  assert.equal(companions.producer.targetOffPreservesCompanions, true);
  assert.deepEqual(companions.producer.incrementalAndN8n, []);
  assert.deepEqual(receipt.baseline, corpus.beforeBaseline);
  assert.equal(
    receipt.baseline.binarySha256,
    "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254",
  );
  for (const { path: name, sha256: digest } of receipt.custody) {
    assert.equal(sha256(read(name)), digest, name);
  }
  assert.equal(receipt.runs.length, 112);
  const routes = [
    "default",
    "recommended",
    "essential",
    "ecosystem",
    "incremental",
    "explicit-off",
    "n8n-projection",
  ];
  for (const [index, entry] of corpus.cases.entries()) {
    const companion = companions.cases[index];
    assert.equal(companion.id, entry.id);
    assert.equal(companion.sourceSha256, entry.sourceSha256);
    assert.equal(apiOrder.cases[index].id, entry.id);
    assert.equal(apiOrder.cases[index].sourceSha256, entry.sourceSha256);
    assert.deepEqual(apiOrder.cases[index].diagnostics, [
      ...entry.diagnostics,
      ...companion.diagnostics,
    ]);
    const combine = (findings) =>
      [...findings, ...companion.diagnostics].sort(
        (a, b) => a.start - b.start || a.end - b.end || a.rule_name.localeCompare(b.rule_name),
      );
    assert.deepEqual(companion.combinedApiDiagnostics, combine(entry.diagnostics));
    assert.deepEqual(companion.combinedCliDiagnostics, companion.combinedApiDiagnostics);
    assert.deepEqual(companion.beforeDefaultDiagnostics, combine(entry.beforeDiagnostics));
    const runs = receipt.runs.slice(index * 7, index * 7 + 7);
    assert.deepEqual(
      runs.map((r) => r.route),
      routes,
    );
    for (const run of runs) {
      const findings = routes.slice(0, 4).includes(run.route)
        ? companion.beforeDefaultDiagnostics
        : run.route === "explicit-off"
          ? companion.diagnostics
          : [];
      const expected = wholeJson(entry, findings);
      assert.equal(run.id, entry.id);
      assert.equal(run.input.text, entry.source);
      assert.equal(run.input.sha256, entry.sourceSha256);
      assert.equal(run.error, null);
      assert.equal(run.signal, null);
      assert.equal(run.stderr, "");
      assert.equal(run.status, findings.length ? 1 : 0);
      assert.equal(run.expectedStatus, run.status);
      assert.deepEqual(run.expected, expected);
      assert.deepEqual(run.actual, expected);
      assert.deepEqual(JSON.parse(run.stdout), expected);
      const configIndex = run.argv.indexOf("-c");
      if (run.route === "explicit-off") {
        assert.ok(configIndex > 0);
        assert.deepEqual(JSON.parse(run.config), { linter: { rules: { [corpus.rule]: "off" } } });
      } else if (run.route === "n8n-projection") {
        const projection = JSON.parse(
          fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
        );
        assert.deepEqual(JSON.parse(run.config), { linter: projection.linter });
      } else {
        assert.equal(run.config, null);
        assert.ok(run.argv.includes("--no-config"));
      }
    }
  }
  assert.equal(failure.workflowRun, 38151606314);
  assert.equal(failure.job, 114506946048);
  assert.equal(failure.head, "4dcf0e71ef61565302ea6df2fe64293514716f81");
  for (const packet of [failure.log, failure.completeFailedPacket]) {
    const bytes = gunzipSync(Buffer.from(packet.packet, "base64"));
    assert.equal(bytes.length, packet.rawBytes);
    assert.equal(sha256(bytes), packet.sha256);
  }
  const original = JSON.parse(
    gunzipSync(Buffer.from(failure.completeFailedPacket.packet, "base64")),
  );
  assert.equal(original.runs.length, 64);
  const failed = original.runs.at(-1);
  assert.equal(failed.id, "prefix-props-policy-control");
  assert.equal(failed.expected[0].errorCount, 3);
  assert.equal(failed.actual[0].errorCount, 5);
  assert.deepEqual(failed.expected, wholeJson(corpus.cases[9], corpus.cases[9].diagnostics));
  assert.deepEqual(
    failed.actual,
    wholeJson(corpus.cases[9], companions.cases[9].combinedCliDiagnostics),
  );
});
