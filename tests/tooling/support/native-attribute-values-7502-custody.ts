import assert from "node:assert/strict";
import { disposition7502, exactKeys7502 } from "./native-attribute-values-7502-inputs.ts";

const span = (value: any) => {
  exactKeys7502(value, ["start", "end"]);
  assert(
    Number.isInteger(value.start) &&
      Number.isInteger(value.end) &&
      value.start >= 0 &&
      value.end >= value.start,
  );
};
function strings(value: any) {
  assert(Array.isArray(value) && value.every((entry) => typeof entry === "string"));
}
function nullableString(value: any) {
  assert(value === null || typeof value === "string");
}
function index(value: any) {
  assert(Number.isSafeInteger(value) && value >= 0);
}
function counts(value: any, actual: Record<string, number>) {
  exactKeys7502(value, Object.keys(actual));
  assert.deepEqual(value, actual);
}
export function value7502(value: any, source: string) {
  exactKeys7502(value, [
    "raw",
    "decoded",
    "nameSpan",
    "valueSpan",
    "fullValueSpan",
    "equalsSpan",
    "quoteSpans",
    "authoredRoot",
    "authoredRootSame",
    "sourceSpan",
    "decodeMap",
    "debugSource",
  ]);
  assert.equal(value.authoredRoot, source);
  assert.equal(value.authoredRootSame, true);
  for (const field of ["nameSpan", "valueSpan", "fullValueSpan", "equalsSpan", "sourceSpan"])
    span(value[field]);
  assert.deepEqual(value.sourceSpan, value.valueSpan);
  assert.equal(
    Buffer.from(source).subarray(value.valueSpan.start, value.valueSpan.end).toString("utf8"),
    value.raw,
  );
  assert.equal(typeof value.raw, "string");
  assert.equal(typeof value.decoded, "string");
  assert.equal(typeof value.debugSource, "string");
  if (value.quoteSpans !== null) {
    assert.equal(value.quoteSpans.length, 2);
    value.quoteSpans.forEach(span);
  }
  if (value.decodeMap !== null)
    for (const segment of value.decodeMap) {
      exactKeys7502(segment, ["decoded", "authored", "kind"]);
      span(segment.decoded);
      span(segment.authored);
      assert.equal(typeof segment.kind, "string");
    }
}
export function custody7502(row: any, disposition = disposition7502(row)) {
  const original = row.observation;
  exactKeys7502(original, [
    "debug",
    "descriptorDebug",
    "descriptorOptionsDebug",
    "descriptorSameSource",
    "descriptorIssues",
    "descriptorErrors",
    "sourceIssues",
    "admitted",
    "rejectedCreationDebug",
    "interpolationFailureDebug",
    "selected",
    "file",
    "counts",
  ]);
  for (const field of ["debug", "descriptorDebug", "descriptorOptionsDebug"])
    assert.equal(typeof original[field], "string");
  assert.equal(original.descriptorSameSource, true);
  strings(original.descriptorIssues);
  strings(original.descriptorErrors);
  assert(Array.isArray(original.sourceIssues));
  for (const issue of original.sourceIssues) {
    exactKeys7502(issue, ["containerIndex", "span", "kind", "debug", "templateIssueKind"]);
    span(issue.span);
    index(issue.containerIndex);
    nullableString(issue.templateIssueKind);
    assert.equal(typeof issue.kind, "string");
    assert.equal(typeof issue.debug, "string");
  }
  counts(original.counts, {
    descriptorIssues: original.descriptorIssues.length,
    descriptorErrors: original.descriptorErrors.length,
    sourceIssues: original.sourceIssues.length,
  });
  nullableString(original.rejectedCreationDebug);
  nullableString(original.interpolationFailureDebug);
  if (original.selected !== null) {
    const selected = original.selected;
    exactKeys7502(selected, [
      "debug",
      "index",
      "grammar",
      "blockSpan",
      "blockSource",
      "rootSource",
      "sameRootSource",
      "viewError",
      "rejectedFileDebug",
      "surfaceErrors",
      "surfaceUnsupported",
    ]);
    span(selected.blockSpan);
    index(selected.index);
    assert.equal(typeof selected.debug, "string");
    assert.equal(typeof selected.grammar, "string");
    nullableString(selected.viewError);
    nullableString(selected.rejectedFileDebug);
    assert.equal(selected.rootSource, row.nativeSource);
    assert.equal(selected.sameRootSource, true);
    assert.equal(selected.blockSource, row.source);
    assert.equal(
      Buffer.from(row.nativeSource)
        .subarray(selected.blockSpan.start, selected.blockSpan.end)
        .toString("utf8"),
      row.source,
    );
    strings(selected.surfaceErrors);
    strings(selected.surfaceUnsupported);
  }
  const file = original.file;
  if (file !== null) {
    exactKeys7502(file, [
      "source",
      "sameSource",
      "complete",
      "artifactDebug",
      "nodeCount",
      "rootDebug",
      "artifactScopesDebug",
      "provenance",
      "units",
      "scopes",
      "bindings",
      "references",
      "exports",
      "imports",
      "issues",
      "templateIssues",
      "templateInterruption",
      "interruptedPrograms",
      "rejectedHandlers",
      "unattachedHandlers",
      "unattachedForHeads",
      "rejectedForHeads",
      "interpolations",
      "interpolationFailures",
      "attributeValues",
      "diagnosticCounts",
    ]);
    assert.equal(file.source, row.nativeSource);
    assert.equal(file.sameSource, true);
    assert.equal(typeof file.complete, "boolean");
    index(file.nodeCount);
    for (const key of ["artifactDebug", "rootDebug", "artifactScopesDebug"])
      assert.equal(typeof file[key], "string");
    for (const entry of file.provenance) {
      exactKeys7502(entry, ["rule", "node", "before", "after", "span"]);
      span(entry.span);
      for (const key of ["rule", "before", "after"]) assert.equal(typeof entry[key], "string");
      if (entry.node !== null) index(entry.node);
    }
    for (const entry of file.bindings) {
      exactKeys7502(entry, ["id", "declarationDebug"]);
      index(entry.id);
      nullableString(entry.declarationDebug);
    }
    for (const field of [
      "units",
      "scopes",
      "references",
      "exports",
      "imports",
      "issues",
      "templateIssues",
      "interruptedPrograms",
      "rejectedHandlers",
      "unattachedHandlers",
      "unattachedForHeads",
      "rejectedForHeads",
    ])
      strings(file[field]);
    nullableString(file.templateInterruption);
    for (const entry of file.interpolations) {
      exactKeys7502(entry, ["state", "inputDebug"]);
      assert.equal(typeof entry.state, "string");
      assert.equal(typeof entry.inputDebug, "string");
    }
    for (const entry of file.interpolationFailures) {
      exactKeys7502(entry, ["span", "failureDebug"]);
      span(entry.span);
      assert.equal(typeof entry.failureDebug, "string");
    }
    counts(file.diagnosticCounts, {
      issues: file.issues.length,
      templateIssues: file.templateIssues.length,
      templateInterruption: Number(file.templateInterruption !== null),
      interruptedPrograms: file.interruptedPrograms.length,
      interpolationFailures: file.interpolationFailures.length,
    });
    for (const entry of file.attributeValues) {
      exactKeys7502(entry, ["index", "state", "observation", "failure"]);
      index(entry.index);
      assert.equal(typeof entry.state, "string");
      nullableString(entry.failure);
      if (entry.observation !== null) value7502(entry.observation, row.nativeSource);
    }
  }
  const l3 = row.l3;
  exactKeys7502(l3, [
    "state",
    "publicError",
    "ownerSame",
    "fileSame",
    "tablesDebug",
    "valueCount",
    "visitError",
    "values",
  ]);
  assert(Number.isInteger(l3.valueCount) && l3.valueCount >= 0);
  nullableString(l3.publicError);
  nullableString(l3.tablesDebug);
  nullableString(l3.visitError);
  assert.equal(l3.values.length, l3.valueCount);
  for (const entry of l3.values) {
    exactKeys7502(entry, [
      "index",
      "node",
      "slot",
      "attribute",
      "sameFile",
      "sameObservation",
      "sameDecodedValue",
      "observation",
    ]);
    index(entry.index);
    index(entry.node);
    index(entry.slot);
    assert.equal(entry.sameFile, true);
    assert.equal(entry.sameObservation, true);
    assert.equal(entry.sameDecodedValue, true);
    assert(entry.attribute && entry.observation, "real original value readback required");
    exactKeys7502(entry.attribute, ["name", "value", "span"]);
    span(entry.attribute.span);
    assert.equal(typeof entry.attribute.name, "string");
    assert.equal(typeof entry.attribute.value, "string");
    value7502(entry.observation, row.nativeSource);
    assert.equal(entry.attribute.value, entry.observation.decoded);
  }
  if (disposition !== "lower-refusal") {
    assert.equal(original.admitted, true);
    assert(file && file.complete);
    assert.deepEqual(original.sourceIssues, []);
    assert.deepEqual(original.descriptorIssues, []);
    assert.deepEqual(original.descriptorErrors, []);
    if (disposition === "target-refusal") {
      assert.equal(row.id, "original-regression-11");
      assert.equal(row.target, "vapor");
      assert.equal(l3.state, "analysis-refusal");
      assert(l3.publicError?.includes("AttributeSemantics"));
      assert.equal(l3.ownerSame, null);
      assert.equal(l3.fileSame, null);
      assert.equal(l3.valueCount, 0);
      assert.deepEqual(l3.values, []);
      assert.equal(l3.tablesDebug, null);
      assert.equal(l3.visitError, null);
    } else {
      assert.equal(l3.state, "complete");
      assert.equal(l3.ownerSame, true);
      assert.equal(l3.fileSame, true);
      assert.equal(l3.publicError, null);
      assert.equal(l3.visitError, null);
      assert.equal(l3.valueCount, file.attributeValues.length);
      assert.equal(l3.valueCount, 1);
    }
    assert.equal(original.rejectedCreationDebug, null);
    assert.equal(original.interpolationFailureDebug, null);
    assert(original.selected);
    assert.equal(original.selected.viewError, null);
    assert.equal(original.selected.rejectedFileDebug, null);
    assert.deepEqual(original.selected.surfaceErrors, []);
    assert.deepEqual(original.selected.surfaceUnsupported, []);
    for (const field of [
      "issues",
      "templateIssues",
      "interruptedPrograms",
      "interpolationFailures",
      "interpolations",
      "rejectedHandlers",
      "unattachedHandlers",
      "unattachedForHeads",
      "rejectedForHeads",
    ])
      assert.deepEqual(file[field], []);
    assert.equal(file.templateInterruption, null);
    assert.deepEqual(
      file.attributeValues.map((entry: any) => entry.index),
      [0],
    );
    assert.deepEqual(
      file.attributeValues.map((entry: any) => entry.failure),
      [null],
    );
    if (disposition !== "target-refusal") {
      assert.equal(
        l3.values[0].attribute.name,
        row.id === "original-regression-11" ? "class" : "title",
      );
      assert.deepEqual(l3.values[0].observation, file.attributeValues[0].observation);
    }
  } else {
    assert.equal(original.admitted, false);
    assert(original.sourceIssues.length > 0);
    assert.equal(l3.state, "lower-refusal");
    assert.equal(l3.ownerSame, null);
    assert.equal(l3.fileSame, null);
    assert.equal(l3.valueCount, 0);
    assert.equal(l3.tablesDebug, null);
    assert.equal(l3.publicError, null);
    assert.equal(l3.visitError, null);
    if (file) assert.equal(file.complete, false);
  }
}
