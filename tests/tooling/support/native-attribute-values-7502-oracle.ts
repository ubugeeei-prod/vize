import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  exactKeys7502,
  disposition7502,
  fixtureSha7502,
  hash7502,
  ledgerSha7502,
  loadInputs7502,
  modes7502,
  outputUrl7502,
  requireReviewed7502,
  targets7502,
  transition7502,
  unchangedRows7502,
} from "./native-attribute-values-7502-inputs.ts";
import { custody7502 } from "./native-attribute-values-7502-custody.ts";

export function validateCapture7502(packet: any) {
  const pack = loadInputs7502();
  exactKeys7502(packet, [
    "schema",
    "version",
    "transition",
    "custody",
    "fixtureSha256",
    "ledgerSha256",
    "original",
    "options",
    "rows",
    "summary",
  ]);
  assert.equal(packet.schema, "vize.native-attribute-values-7502.capture");
  assert.equal(packet.version, 2);
  assert.deepEqual(packet.transition, transition7502);
  assert.equal(packet.custody, "once-selected-original");
  assert.equal(packet.fixtureSha256, fixtureSha7502);
  assert.equal(packet.ledgerSha256, ledgerSha7502);
  assert.deepEqual(packet.original, {
    fix: pack.originalFix,
    parent: pack.originalParent,
    testBlob: pack.testBlob,
    reporterBlob: pack.reporterBlob,
  });
  assert.deepEqual(packet.options, {
    filename: "AttributeValues7502.vue",
    carrier: "<template>{source}</template>",
    descriptorPolicy: "unchanged target Default::default()",
    originalCompileOptions: "compile_vapor Default::default()",
    targets: targets7502,
    linkModes: modes7502,
  });
  assert.equal(
    packet.rows.length,
    84,
    "complete original fourteen by three targets by two link modes",
  );
  let index = 0;
  for (const fixture of pack.fixtures)
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
        ]);
        for (const key of [
          "id",
          "source",
          "sourceSha256",
          "existingExpectedTemplateHtml",
          "expectedTemplateHtmlSha256",
          "existingOptions",
          "disposition",
        ])
          assert.equal(row[key], fixture[key]);
        for (const [key, value] of Object.entries({ ...target, ...mode }))
          assert.equal(row[key], value);
        assert.equal(row.filename, "AttributeValues7502.vue");
        assert.equal(row.nativeSource, `<template>${fixture.source}</template>`);
        assert.equal(row.nativeSourceSha256, hash7502(row.nativeSource));
        custody7502(row);
        const result = row.result;
        exactKeys7502(result, [
          "classification",
          "publicError",
          "code",
          "mapText",
          "map",
          "mapError",
          "links",
        ]);
        assert(Array.isArray(result.links));
        const disposition = disposition7502(row);
        if (disposition === "lower-refusal") {
          assert.equal(result.classification, "lower-refusal");
          assert.equal(typeof result.publicError, "string");
          assert(result.publicError.length > 0);
          assert.equal(result.code, null);
          assert.equal(result.mapText, null);
          assert.equal(result.map, null);
          assert.equal(result.mapError, null);
          assert.deepEqual(result.links, []);
          assert(
            row.observation.selected && row.observation.file,
            "complete lower original observation required",
          );
          assert.equal(row.observation.file.complete, false);
          assert.equal(row.observation.sourceIssues.length, 1);
          assert.equal(row.observation.sourceIssues[0].templateIssueKind, "UnsupportedChild");
        } else if (disposition === "target-refusal") {
          assert.equal(result.classification, "target-refusal");
          assert.equal(typeof result.publicError, "string");
          assert(result.publicError.includes("AttributeSemantics"));
          for (const key of ["code", "mapText", "map", "mapError"]) assert.equal(result[key], null);
          assert.deepEqual(result.links, []);
          assert.equal(row.observation.admitted, true);
          assert.equal(row.observation.file.complete, true);
        } else {
          assert.equal(result.classification, "complete-original-sfc-module");
          assert.equal(result.publicError, null);
          assert.equal(result.mapError, null);
          assert.equal(typeof result.code, "string");
          assert(
            result.code.length > 0 && result.code.endsWith("\n"),
            "whole actual module required",
          );
          if (mode.sourceMap) {
            assert.equal(typeof result.mapText, "string");
            assert(result.mapText.length > 0, "raw complete map is mandatory");
            assert.deepEqual(JSON.parse(result.mapText), result.map);
            assert.equal(result.map.version, 3);
            assert.deepEqual(result.map.sources, [row.filename]);
            assert.deepEqual(result.map.sourcesContent, [row.nativeSource]);
            assert.equal(typeof result.map.mappings, "string");
            assert(result.map.mappings.length > 0 && result.links.length > 0);
          } else {
            assert.equal(result.mapText, null, "NoLinks is authentic None");
            assert.equal(result.map, null);
            assert.deepEqual(result.links, []);
          }
        }
        if (mode.linkMode === "NoLinks") {
          const recorded = packet.rows[index - 2];
          assert.equal(result.code, recorded.result.code, "whole Recorded/NoLinks module equality");
          assert.equal(result.publicError, recorded.result.publicError);
          assert.deepEqual(row.observation, recorded.observation);
          assert.deepEqual(row.l3, recorded.l3);
        }
      }
  assert.deepEqual(packet.summary, {
    fixtures: 14,
    outcomes: 84,
    positive: 76,
    lowerRefusals: 6,
    targetRefusals: 2,
  });
  unchangedRows7502(packet);
  return packet;
}

export function reviewedCapture7502(actual: any) {
  validateCapture7502(actual);
  requireReviewed7502(actual, JSON.parse(readFileSync(outputUrl7502, "utf8")));
  return actual;
}
