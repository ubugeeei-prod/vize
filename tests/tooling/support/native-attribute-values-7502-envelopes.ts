import assert from "node:assert/strict";
import { custody7502 } from "./native-attribute-values-7502-custody.ts";
import {
  exactKeys7502,
  fixtureSha7502,
  hash7502,
  ledgerSha7502,
  loadInputs7502,
  modes7502,
  targets7502,
} from "./native-attribute-values-7502-inputs.ts";

export const envelopes7502 = [
  { id: "ordinary", suffix: "<script></script>", issue: "Script(Ordinary)" },
  { id: "setup", suffix: "<script setup></script>", issue: "Script(Setup)" },
  { id: "style", suffix: "<style></style>", issue: "Style" },
  { id: "scoped-style", suffix: "<style scoped>.x{color:red}</style>", issue: "Style" },
];
export function validateEnvelopes7502(packet: any) {
  exactKeys7502(packet, [
    "schema",
    "version",
    "custody",
    "fixtureSha256",
    "ledgerSha256",
    "authority",
    "carrier",
    "summary",
    "rows",
  ]);
  assert.equal(packet.schema, "vize.native-attribute-values-7502.envelopes");
  assert.equal(packet.version, 1);
  assert.equal(packet.custody, "once-selected-original");
  assert.equal(packet.fixtureSha256, fixtureSha7502);
  assert.equal(packet.ledgerSha256, ledgerSha7502);
  assert.equal(
    packet.authority,
    "independently authored script/style carrier refusal controls; no styled runtime credit",
  );
  assert.equal(packet.carrier, "<template>{source}</template>{envelopeSuffix}");
  assert.deepEqual(packet.summary, { fixtures: 14, envelopes: 4, outcomes: 336 });
  assert.equal(packet.rows.length, 336);
  let index = 0;
  for (const fixture of loadInputs7502().fixtures)
    for (const envelope of envelopes7502)
      for (const target of targets7502)
        for (const mode of modes7502) {
          const row = packet.rows[index++];
          exactKeys7502(row, [
            "id",
            "target",
            "linkMode",
            "sourceMap",
            "runtimeVersion",
            "filename",
            "source",
            "sourceSha256",
            "nativeSource",
            "nativeSourceSha256",
            "existingExpectedTemplateHtml",
            "expectedTemplateHtmlSha256",
            "existingOptions",
            "disposition",
            "observation",
            "l3",
            "result",
            "envelopeId",
            "envelopeSuffix",
            "expectedIssue",
            "envelopeIssueMatches",
          ]);
          for (const [key, value] of Object.entries(fixture)) assert.equal(row[key], value);
          for (const [key, value] of Object.entries({ ...target, ...mode }))
            assert.equal(row[key], value);
          assert.equal(row.filename, "AttributeValues7502.vue");
          assert.equal(row.envelopeId, envelope.id);
          assert.equal(row.envelopeSuffix, envelope.suffix);
          assert.equal(row.expectedIssue, envelope.issue);
          assert.equal(row.envelopeIssueMatches, true);
          assert.equal(
            row.nativeSource,
            `<template>${fixture.source}</template>${envelope.suffix}`,
          );
          assert.equal(row.nativeSourceSha256, hash7502(row.nativeSource));
          custody7502({ ...row, disposition: "lower-refusal" });
          const original = row.observation;
          assert.equal(original.selected, null);
          assert.equal(original.file, null);
          assert.equal(original.rejectedCreationDebug, null);
          assert.equal(original.interpolationFailureDebug, null);
          assert.deepEqual(original.descriptorErrors, []);
          assert.deepEqual(original.descriptorIssues, []);
          assert.equal(original.sourceIssues.length, 1);
          assert.equal(original.sourceIssues[0].kind, envelope.issue);
          assert.equal(original.sourceIssues[0].templateIssueKind, null);
          exactKeys7502(row.result, [
            "classification",
            "publicError",
            "code",
            "mapText",
            "map",
            "mapError",
            "links",
          ]);
          assert.equal(row.result.classification, "lower-refusal");
          assert.equal(typeof row.result.publicError, "string");
          assert(row.result.publicError.length > 0);
          for (const key of ["code", "mapText", "map", "mapError"])
            assert.equal(row.result[key], null);
          assert.deepEqual(row.result.links, []);
          if (!mode.sourceMap) {
            const recorded = packet.rows[index - 2];
            assert.deepEqual(row.observation, recorded.observation);
            assert.deepEqual(row.l3, recorded.l3);
            assert.deepEqual(row.result, recorded.result);
          }
        }
  return packet;
}
