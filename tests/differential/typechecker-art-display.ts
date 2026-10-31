/** The two Art cases have a finite, separately recorded native display authority. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import type { Artifact, Diagnostic, TypecheckerCase } from "./typechecker.ts";

export const artDisplayPack = "art-variant-implicit-component-9854";
const sourceRevision = "c4ee353b79611126138a82fb68ae187dd171ef70";
const historical = {
  path: `${artDisplayPack}/cases.json`,
  sha256: "bdf1c89cf693d30a0c5778ae10f7fb207f1a867242b7c1f490d40eac4c692834",
};
const reference = {
  path: `${artDisplayPack}/native-display-current.json`,
  sha256: "5963f62b417b944dadb54d0d4ead587714e9bf2e4725b2773e611fe887eb5c94",
};
const inputHashes = [
  ["tsconfig.json.txt", "e36b280dce185f4be85cbff1838c0ee06d8d7a46b38b3e63df995c0decd7b13b"],
  ["src/MyCard.vue.txt", "455de116ba8d563f320c6b90d5a655a7c9cef4d8305e00e6705adf289420df35"],
  ["src/MyCard.art.vue.txt", "8244d93074da638cd1c9f7a93845de4e7cf146f4da11b7c170ac7eff39ae3eea"],
  [
    "src/MyCardExplicit.art.vue.txt",
    "224008e13f14210912b95eadfed63f36f18ffc1ee8108261f418d0f11bc9600a",
  ],
  [
    "src/MyCardValid.art.vue.txt",
    "766c26dee563a4eb211aded1d2f54a97d5dae4e0a40dfbd3bad655eb3d1bf834",
  ],
];
export type ArtDiagnosticAuthority = {
  kind: "finite-art-native-display";
  reference: Artifact;
  sourceRevision: string;
  capture: { runId: number; jobId: number };
};
type Case = { id: string; inputs: { file: string; source: string }[]; diagnostics: Diagnostic[] };
type Pack = { sourceRevision: string; cases: Case[] };

export function selectArtDiagnosticAuthority(
  root: string,
  fixture: TypecheckerCase,
): TypecheckerCase {
  assert.equal(fixture.pack, artDisplayPack, "this authority only selects the original Art pack");
  assert.equal(fixture.sourceRevision, "f07065df35d25bc76f355e5f106c80dbefa922b5");
  assert.deepEqual(fixture.packReference, historical);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8"));
  const authored = manifest.cases.filter((item: TypecheckerCase) => item.pack === artDisplayPack);
  assert.equal(
    sha256(Buffer.from(JSON.stringify(authored))),
    "58c3fa196b473259fdc5d0aed95897db45de542eebfae250b91815348639034d",
    "the complete two original Art manifest entries stay frozen",
  );
  const { diagnostics: _diagnostics, ...context } = fixture;
  assert.deepEqual(
    context,
    authored.find((item: TypecheckerCase) => item.case === fixture.case),
  );
  const before: Pack = JSON.parse(readPinnedArtifact(root, historical).toString("utf8"));
  for (const [name, digest] of inputHashes)
    readPinnedArtifact(root, { path: `${artDisplayPack}/${name}`, sha256: digest });
  const current: Pack = JSON.parse(readPinnedArtifact(root, reference).toString("utf8"));
  assert.equal(current.sourceRevision, sourceRevision);
  assert.deepEqual(
    current.cases.map((item) => item.id),
    before.cases.map((item) => item.id),
  );
  const expected = structuredClone(before);
  expected.sourceRevision = sourceRevision;
  assert.equal(current.cases[0].diagnostics.length, 4);
  for (const index of [1, 3]) {
    const old = before.cases[0].diagnostics[index],
      next = current.cases[0].diagnostics[index];
    assert.deepEqual({ ...next, message: old.message }, old);
    const suffix =
      "'.\nProperty 'title' is missing in type '{ elevated: true; }' but required in type '{ readonly title: string; readonly elevated?: boolean | undefined; }'.";
    assert.equal(old.message.slice(-suffix.length), suffix);
    assert.equal(next.message.slice(-suffix.length), suffix);
    expected.cases[0].diagnostics[index].message = next.message;
  }
  assert.deepEqual(current, expected, "only the two authentic parameter display strings change");
  const old = before.cases.find((item) => item.id === fixture.case),
    selected = current.cases.find((item) => item.id === fixture.case);
  assert.ok(old && selected, "only the two recorded Art cases select this authority");
  assert.deepEqual(fixture.diagnostics, old.diagnostics);
  assert.deepEqual(
    fixture.inputs.map((input) => [input.file, input.path]),
    old.inputs.map((input) => [input.file, `${artDisplayPack}/${input.source}`]),
  );
  return {
    ...fixture,
    diagnostics: selected.diagnostics,
    diagnosticAuthority: {
      kind: "finite-art-native-display",
      reference: { ...reference },
      sourceRevision,
      capture: { runId: 41129427590, jobId: 124316268671 },
    },
  };
}
