import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { authenticateRuntimeApiSourceFamily } from "./support/runtime-props-api-source-custody.mjs";
import {
  authenticateSetupExportMembership,
  setupExportMembership,
} from "./support/setup-export-preset-source-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-suppression-8090");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));

await test("whole suppression routes retain unchanged companion producers and every initial failure", () => {
  assert.equal(
    sha256(read("default-route-authority.json")),
    "0576b6b7a57c8d3a8bf076adfbea1611599b7f31e5bd792dfba2dba3663cc7c7",
  );
  const authority = json("default-route-authority.json");
  for (const entry of authority.files) {
    assert.equal(sha256(read(entry.path)), entry.sha256, entry.path);
    assert.equal(read(entry.path).length, entry.bytes);
  }
  assert.equal(authority.summary.productionChanged, false);
  const corpus = json("cases.json"),
    successor = json("default-route-successors.json");
  const receipt = json("default-route-public-before.json");
  assert.equal(successor.cases.length, 24);
  assert.equal(receipt.runs.length, 168);
  assert.deepEqual(receipt.baseline, corpus.beforeBaseline);
  assert.equal(sha256(read("cases.json")), successor.initialCorpus.sha256);
  for (const entry of successor.producers) {
    if (entry.path === setupExportMembership) {
      const membership = authenticateSetupExportMembership(
        root,
        authenticateRuntimeApiSourceFamily(root),
      );
      assert.deepEqual(entry, { path: membership.path, sha256: membership.beforeSha256 });
    } else assert.equal(sha256(fs.readFileSync(path.join(root, entry.path))), entry.sha256);
  }
  for (const entry of receipt.custody) assert.equal(sha256(read(entry.path)), entry.sha256);
  const routes = [
    "default",
    "recommended",
    "essential",
    "ecosystem",
    "incremental",
    "explicit-off",
    "n8n-projection",
  ];
  corpus.cases.forEach((entry, index) => {
    const whole = successor.cases[index];
    for (const key of ["id", "filename", "sourceSha256"]) assert.equal(whole[key], entry[key]);
    assert.deepEqual(whole.defaultDiagnostics, [
      ...whole.companionDiagnostics,
      ...entry.diagnostics,
    ]);
    assert.deepEqual(whole.beforeDefaultDiagnostics, [
      ...whole.companionDiagnostics,
      ...entry.beforeDiagnostics,
    ]);
    assert.deepEqual(whole.essentialDiagnostics, entry.diagnostics);
    assert.deepEqual(whole.beforeEssentialDiagnostics, entry.beforeDiagnostics);
    assert.deepEqual(whole.offDiagnostics, whole.companionDiagnostics);
    assert.deepEqual(whole.incrementalDiagnostics, []);
    if (entry.id !== "style-block-cannot-leak") {
      assert.deepEqual(whole.companionDiagnostics, []);
      assert.deepEqual(whole.projectionDiagnostics, []);
    } else {
      assert.equal(whole.companionDiagnostics.length, 2);
      assert.deepEqual(whole.projectionDiagnostics, [
        {
          ...whole.companionDiagnostics[1],
          severity: "error",
          help: "Recommended order: <script> -> <template> -> <style>",
        },
      ]);
    }
    const packets = receipt.runs.slice(index * 7, index * 7 + 7);
    assert.deepEqual(
      packets.map(({ route }) => route),
      routes,
    );
    for (const run of packets) {
      const findings =
        run.route === "essential"
          ? whole.beforeEssentialDiagnostics
          : run.route === "incremental"
            ? whole.incrementalDiagnostics
            : run.route === "explicit-off"
              ? whole.offDiagnostics
              : run.route === "n8n-projection"
                ? whole.projectionDiagnostics
                : whole.beforeDefaultDiagnostics;
      assert.equal(run.id, entry.id);
      assert.deepEqual(run.input, { text: entry.source, sha256: entry.sourceSha256 });
      assert.deepEqual(run.expected, wholeJson(entry, findings));
      assert.deepEqual(run.actual, run.expected);
      assert.deepEqual(JSON.parse(run.stdout), run.expected);
      assert.equal(run.status, findings.some((d) => d.severity === "error") ? 1 : 0);
      assert.equal(run.expectedStatus, run.status);
      assert.equal(run.stderr, "");
      assert.equal(run.signal, null);
      assert.equal(run.error, null);
    }
  });
  const failure = json("initial-source-failure.json");
  assert.equal(failure.workflowRun, 38156254146);
  assert.equal(failure.sourceCommit, "d20d205ee3252ab9067882db881539c68f057973");
  assert.equal(failure.packets.length, 3);
  for (const entry of failure.packets) {
    const raw = gunzipSync(Buffer.from(entry.packet, "base64"));
    assert.equal(sha256(raw), entry.sha256);
    assert.equal(raw.length, entry.rawBytes);
    if (entry.kind !== "complete-cli-artifact") continue;
    const packet = JSON.parse(raw);
    assert.equal(packet.runs.length, 134);
    const last = packet.runs.at(-1);
    assert.equal(last.id, "style-block-cannot-leak");
    assert.equal(last.expected[0].warningCount, 0);
    assert.equal(last.actual[0].warningCount, 2);
    assert.deepEqual(JSON.parse(last.stdout), last.actual);
  }
  for (const [name, count, failedRoute] of [
    ["initial-default-route-before.json.gz", 136, "essential"],
    ["second-default-route-before.json.gz", 140, "n8n-projection"],
  ]) {
    const initial = JSON.parse(gunzipSync(read(name)));
    assert.equal(initial.packet.runs.length, count);
    const last = initial.packet.runs.at(-1);
    assert.equal(last.id, "style-block-cannot-leak");
    assert.equal(last.route, failedRoute);
    assert.deepEqual(JSON.parse(last.stdout), last.actual);
    assert.notDeepEqual(last.expected, last.actual);
    const helper = initial.packet.custody.find(
      ({ path }) => path === "observe-default-route-before.mjs",
    );
    assert.equal(sha256(Buffer.from(initial.observer)), helper.sha256);
  }
});
