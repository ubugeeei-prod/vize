import assert from "node:assert/strict";
import {
  disposition7502,
  exactKeys7502,
  hash7502,
  loadInputs7502,
} from "./native-attribute-values-7502-inputs.ts";

function trace(value: any, hydrated: boolean, cloned = false) {
  exactKeys7502(value, [
    "initialTree",
    "tree",
    "diagnostics",
    "originalNodes",
    "retainedOriginalNodes",
    "retained",
    "unmounted",
    "renderIdentity",
    ...(cloned ? ["distinctNodes"] : []),
  ]);
  assert.equal(value.renderIdentity, true);
  assert.deepEqual(value.diagnostics, []);
  assert.deepEqual(value.unmounted, []);
  assert(Number.isSafeInteger(value.originalNodes) && value.originalNodes >= 0);
  assert.equal(value.retained.length, value.originalNodes);
  assert(value.retained.every((entry: any) => typeof entry === "boolean"));
  assert.equal(value.retainedOriginalNodes, value.retained.filter(Boolean).length);
  if (hydrated) {
    assert(value.originalNodes > 0);
    assert(value.retained.every(Boolean));
  } else {
    assert.equal(value.originalNodes, 0);
    assert.deepEqual(value.initialTree, []);
  }
  if (cloned) assert.equal(value.distinctNodes, true);
}
function rendered(value: any) {
  exactKeys7502(value, ["html", "tree", "diagnostics", "renderIdentity"]);
  assert.equal(typeof value.html, "string");
  assert(value.html.length > 0);
  assert.equal(value.renderIdentity, true);
  assert.deepEqual(value.diagnostics, []);
}
export function validateRuntime7502(output: any, packet: any, mode: string) {
  exactKeys7502(output, [
    "schema",
    "version",
    "mode",
    "capturedFromRust",
    "acceptance",
    "nativeCodeSource",
    "counts",
    "controls",
    "executions",
  ]);
  assert.equal(output.schema, "vize.native-attribute-values-7502.runtime");
  assert.equal(output.version, 2);
  assert.equal(output.mode, mode);
  assert.equal(output.capturedFromRust, true);
  assert.equal(output.acceptance, "unreviewed");
  assert.equal(output.nativeCodeSource, "fresh-source-built-capture");
  assert.deepEqual(output.counts, {
    originalControls: 14,
    knownIncorrectRc9Clients: 9,
    positiveNativeExecutions: 76,
    lowerRefusalOutcomes: 6,
    targetRefusalOutcomes: 2,
  });
  assert.equal(output.controls.length, 14);
  assert.equal(output.executions.length, 76);
  for (const [index, fixture] of loadInputs7502().fixtures.entries()) {
    const control = output.controls[index];
    exactKeys7502(control, [
      "id",
      "source",
      "existingExpectedTemplateHtml",
      "expectedTree",
      "directTemplate",
      "primary",
      "stockDomMount",
      "stockSsr",
      "stockVaporSsr",
      "stockVaporMount",
      "stockVaporHydration",
      "stockClientMatches",
      "authority",
    ]);
    for (const key of ["id", "source", "existingExpectedTemplateHtml"])
      assert.equal(control[key], fixture[key]);
    assert.equal(control.authority, "primary-only/no-native-credit");
    assert.equal(typeof control.stockClientMatches, "boolean");
    exactKeys7502(control.directTemplate, ["first", "clone", "distinct"]);
    assert.equal(control.directTemplate.distinct, true);
    assert.deepEqual([control.directTemplate.first], control.expectedTree);
    assert.deepEqual([control.directTemplate.clone], control.expectedTree);
    exactKeys7502(control.primary, ["versions", "dom", "ssr", "vapor"]);
    assert.deepEqual(control.primary.versions, {
      domCompiler: "3.5.35",
      plugin: "6.0.7",
      vaporCompiler: "3.6.0-rc.9",
    });
    for (const target of ["dom", "ssr", "vapor"]) {
      const primary = control.primary[target];
      exactKeys7502(primary, [
        "code",
        "map",
        "helperCode",
        ...(target === "vapor" ? ["serverCode", "serverMap"] : []),
      ]);
      for (const key of ["code", "helperCode", ...(target === "vapor" ? ["serverCode"] : [])]) {
        assert.equal(typeof primary[key], "string");
        assert(primary[key].length > 0);
      }
      assert(primary.map && typeof primary.map === "object");
      if (target === "vapor") assert(primary.serverMap && typeof primary.serverMap === "object");
    }
    trace(control.stockDomMount, false);
    trace(control.stockVaporMount, false);
    trace(control.stockVaporHydration, true);
    rendered(control.stockSsr);
    rendered(control.stockVaporSsr);
    for (const entry of [
      control.stockDomMount,
      control.stockSsr,
      control.stockVaporSsr,
      control.stockVaporHydration,
    ])
      assert.deepEqual(entry.tree, control.expectedTree);
  }
  assert.equal(output.controls.filter((entry: any) => !entry.stockClientMatches).length, 9);
  const positives = packet.rows.filter((row: any) => disposition7502(row) === "positive");
  for (const [index, row] of positives.entries()) {
    const execution = output.executions[index];
    exactKeys7502(execution, [
      "id",
      "target",
      "linkMode",
      "codeSha256",
      ...(row.target === "ssr" ? ["rendered", "repeated"] : ["mounted", "cloned", "hydrated"]),
    ]);
    for (const key of ["id", "target", "linkMode"]) assert.equal(execution[key], row[key]);
    assert.equal(execution.codeSha256, hash7502(row.result.code));
    const expected = output.controls.find((control: any) => control.id === row.id).expectedTree;
    if (row.target === "ssr") {
      rendered(execution.rendered);
      rendered(execution.repeated);
      assert.deepEqual(execution.rendered, execution.repeated);
      assert.deepEqual(execution.rendered.tree, expected);
    } else {
      trace(execution.mounted, false);
      trace(execution.cloned, false, true);
      trace(execution.hydrated, true);
      for (const entry of [execution.mounted, execution.cloned, execution.hydrated])
        assert.deepEqual(entry.tree, expected);
      assert.deepEqual(execution.hydrated.initialTree, expected);
    }
  }
  return output;
}
