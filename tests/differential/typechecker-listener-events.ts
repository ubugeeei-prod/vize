/** A finite authority for the exact undeclared-event listener regression. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import type { Artifact, Diagnostic, TypecheckerCase } from "./typechecker.ts";
export const listenerEventPack = "component-listener-prop-duplicate-10397";
const sourceRevision = "90800904bcfbf3ad073ba95349852ee71ffb37e2";
const historical = {
  path: `${listenerEventPack}/cases.json`,
  sha256: "7b2175e25db8e5924133ff31d4ade064341d10bff923eeb002256d63277c210c",
};
const reference = {
  path: `${listenerEventPack}/native-event-current.json`,
  sha256: "6ba0345ffe593207a8e114c8834fd7fc8da97758d13a7c7068203713b65bf7df",
};
const inputs = [
  ["tsconfig.json.txt", "20607bc6729ba818893549d0567ca341f57cdf1eb56e34e176d2b2205e7002e4"],
  ["src/Child.vue.txt", "c06aaefcc168971c448e95ac6354ea792a7e5e7227de5f52d7372c9f354821fb"],
  ["src/App.vue.txt", "f74d671621c4509482b2d9dfcf499be3b193b2ff840dd5a254e16a8629a0b2b7"],
];
export type ListenerDiagnosticAuthority = {
  kind: "finite-open-component-event";
  reference: Artifact;
  sourceRevision: string;
  capture: { runId: number; jobId: number };
};
type Case = { id: string; inputs: { file: string; source: string }[]; diagnostics: Diagnostic[] };
type Pack = { sourceRevision: string; cases: Case[] };
export function selectListenerEventAuthority(
  root: string,
  fixture: TypecheckerCase,
): TypecheckerCase {
  assert.equal(fixture.pack, listenerEventPack);
  assert.equal(fixture.sourceRevision, "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943");
  assert.deepEqual(fixture.packReference, historical);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8"));
  const authored = manifest.cases.filter(
    (item: TypecheckerCase) => item.pack === listenerEventPack,
  );
  assert.equal(
    sha256(Buffer.from(JSON.stringify(authored))),
    "1fe7f5d2f1748a70db0b9e442731897f4ed1e297dd2e68bb780185ae2f02fc32",
    "the complete original listener manifest entry stays frozen",
  );
  const before: Pack = JSON.parse(readPinnedArtifact(root, historical).toString("utf8"));
  assert.deepEqual(fixture, {
    ...authored.find((item: TypecheckerCase) => item.case === fixture.case),
    diagnostics: before.cases[0].diagnostics,
  });
  for (const [name, digest] of inputs)
    readPinnedArtifact(root, { path: `${listenerEventPack}/${name}`, sha256: digest });
  const current: Pack = JSON.parse(readPinnedArtifact(root, reference).toString("utf8"));
  assert.equal(current.sourceRevision, sourceRevision);
  assert.deepEqual(
    current.cases.map((item) => item.id),
    ["required-boolean-with-listener"],
  );
  const expected = structuredClone(before);
  expected.sourceRevision = sourceRevision;
  assert.equal(expected.cases[0].diagnostics.length, 4);
  expected.cases[0].diagnostics = expected.cases[0].diagnostics.slice(2);
  assert.deepEqual(
    current,
    expected,
    "only the exact two frozen no-emits Event diagnostics become obsolete",
  );
  assert.equal(fixture.case, before.cases[0].id);
  assert.deepEqual(fixture.diagnostics, before.cases[0].diagnostics);
  assert.deepEqual(
    fixture.inputs.map((input) => [input.file, input.path]),
    before.cases[0].inputs.map((input) => [input.file, `${listenerEventPack}/${input.source}`]),
  );
  return {
    ...fixture,
    diagnostics: current.cases[0].diagnostics,
    diagnosticAuthority: {
      kind: "finite-open-component-event",
      reference: { ...reference },
      sourceRevision,
      capture: { runId: 41137319424, jobId: 124342873996 },
    },
  };
}
