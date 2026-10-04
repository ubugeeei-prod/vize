import assert from "node:assert/strict";
import type { Observation, Result, Trace, Warning } from "./vue1-pinned-getter.ts";

type Span = [number, number];
type ExpectedOutcome = {
  result: Result | null;
  trace: Trace[];
  warnings: Warning[];
  noop: boolean;
};
type Primary = {
  input: string;
  tokens: unknown;
  expression: string;
  getterBody: string;
  authoredPreparedSpan: Span;
};
export type Case = {
  id: string;
  source: string;
  blockSpan: Span;
  path: number[];
  callbackSpan: Span;
  callback: string;
  width: number;
  indentWidth: number;
  lineEnding: "Lf";
  expected: string;
  primary: {
    before: Primary;
    after: Primary;
    scope: string;
    outcomes: Record<"test" | "production", ExpectedOutcome>;
  };
};
export type Control = {
  id: string;
  scope: string;
  semanticCredit: false;
  input: string;
  tokens: unknown;
  expression: string | null;
  getterBody: string | null;
  outcomes: Record<"test" | "production", ExpectedOutcome>;
};
type Segment = { decoded: Span; authored: Span; kind: "Identity" | "Entity" };
type NativeCase = {
  id: string;
  source: string;
  blockSpan: Span;
  path: number[];
  callbackSpan: Span;
  callback: string;
  preparedText: string;
  preparedSpan: Span;
  preparedMap: Segment[] | null;
  width: number;
  indentWidth: number;
  lineEnding: "Lf";
  printed: string;
  idempotent: string;
  formattedPreparedText: string;
  formattedPreparedSpan: Span;
  formattedPreparedMap: Segment[] | null;
};
export type NativePacket = {
  schema: string;
  version: number;
  sourceHead: string | null;
  executionCommit: string | null;
  workflowRun: string | null;
  executablePath: string;
  processId: number;
  cases: NativeCase[];
};

export function verifyOutcome(actual: Observation, expected: ExpectedOutcome, label: string) {
  assert.deepEqual(
    { result: actual.result, trace: actual.trace, warnings: actual.warnings, noop: actual.noop },
    expected,
    label,
  );
  if (actual.result?.kind === "throw") {
    assert.ok(actual.error, `${label} retains the whole actual thrown error`);
    assert.deepEqual(
      { name: actual.error.name, message: actual.error.message },
      actual.result.error,
    );
    assert.ok(actual.error.ownProperties.length > 0);
    const stack = (
      actual.error.ownProperties as Array<{
        key: string;
        value: { kind: string; value?: unknown };
      }>
    ).find((entry) => entry.key === "stack");
    assert.equal(stack?.value.kind, "string");
    assert.ok(typeof stack?.value.value === "string" && stack.value.value.length > 0);
  } else {
    assert.equal(actual.error, null);
  }
}

export function verifyGetter(actual: Observation, body: string | null, label: string) {
  if (actual.expression === null) {
    assert.deepEqual(actual.expressionRecordKeys, []);
    assert.equal(actual.getterSource, null);
    assert.equal(actual.getterName, null);
    return;
  }
  assert.deepEqual(actual.expressionRecordKeys, ["exp", "get"], label);
  if (actual.noop) {
    assert.equal(actual.getterName, "noop");
    assert.equal(actual.getterSource, "function noop() {}");
  } else {
    assert.equal(actual.getterName, "anonymous");
    assert.equal(actual.getterSource, `function anonymous(scope\n) {\nreturn ${body};\n}`, label);
  }
}

export function utf8Slice(source: string, span: Span): string {
  const bytes = Buffer.from(source);
  assert.equal(span.length, 2);
  assert.ok(span.every((offset) => Number.isSafeInteger(offset) && offset >= 0));
  assert.ok(span[0] <= span[1] && span[1] <= bytes.length);
  const selected = bytes.subarray(span[0], span[1]);
  const text = selected.toString("utf8");
  assert.deepEqual(Buffer.from(text), selected, "both endpoints are UTF-8 boundaries");
  return text;
}

export function verifyMap(source: string, span: Span, prepared: string, map: Segment[] | null) {
  const raw = utf8Slice(source, span);
  if (map === null) {
    assert.equal(raw, prepared, "unmapped source is the exact original identity slice");
    return;
  }
  assert.ok(map.length > 0);
  let decoded = 0;
  let authored = span[0];
  for (const segment of map) {
    assert.deepEqual(Object.keys(segment).sort(), ["authored", "decoded", "kind"]);
    assert.equal(segment.decoded[0], decoded);
    assert.equal(segment.authored[0], authored);
    assert.ok(segment.decoded[1] > decoded && segment.authored[1] > authored);
    const cooked = utf8Slice(prepared, segment.decoded);
    const original = utf8Slice(source, segment.authored);
    if (segment.kind === "Identity") assert.equal(cooked, original);
    else {
      assert.equal(segment.kind, "Entity");
      assert.ok(original.startsWith("&") && original.endsWith(";"));
    }
    decoded = segment.decoded[1];
    authored = segment.authored[1];
  }
  assert.equal(decoded, Buffer.byteLength(prepared));
  assert.equal(authored, span[1]);
}

export function verifyNativeIdentity(
  native: NativePacket,
  executableSha256: string | null,
  captureSha256: string | null,
) {
  assert.equal(native.schema, "vize.native-vue1-text-document-capture");
  assert.equal(native.version, 1);
  assert.equal(native.sourceHead, process.env.VIZE_GLYPH_VUE1_TEXT_SOURCE_HEAD);
  assert.equal(native.executionCommit, process.env.GITHUB_SHA);
  assert.equal(native.workflowRun, process.env.GITHUB_RUN_ID);
  assert.match(native.sourceHead ?? "", /^[0-9a-f]{40}$/);
  assert.match(native.executionCommit ?? "", /^[0-9a-f]{40}$/);
  assert.match(native.workflowRun ?? "", /^[0-9]+$/);
  assert.ok(Number.isSafeInteger(native.processId) && native.processId > 0);
  assert.ok(native.executablePath.length > 0);
  assert.match(executableSha256 ?? "", /^[0-9a-f]{64}$/);
  assert.match(captureSha256 ?? "", /^[0-9a-f]{64}$/);
}
