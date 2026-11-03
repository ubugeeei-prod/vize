// One current phase contract; the historical beta ABI remains independently required.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const DIAGNOSTIC_PHASE_ID = "typechecker/protocol/native-diagnostic-phase-11626";
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const registrationSha256 = "e1aac5141f26dd1845353613d0d1ad0a742f5af06a8a3c9fb3dc36d741e0efb0";
const sources = [
  "crates/vize_canon/src/lsp_client/initial_diagnostics/native_diagnostics.rs",
  "crates/vize_canon/src/lsp_client/diagnostics/batch.rs",
  "crates/vize_canon/src/lsp_client/initial_diagnostics/native_diagnostics/tests.rs",
];
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const object = (value: unknown): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};
function canonical(value: unknown): string {
  return JSON.stringify(value, (_, item: unknown) =>
    item && typeof item === "object" && !Array.isArray(item)
      ? Object.fromEntries(
          Object.entries(item).sort(([left], [right]) => left.localeCompare(right)),
        )
      : item,
  );
}
function pinned(read: ReadFile, side: "base" | "head", path: unknown, expected: unknown) {
  assert.ok(typeof path === "string");
  assert.match(path, /^(?:[a-zA-Z0-9_.-]+\/)*[a-zA-Z0-9_.-]+$/);
  assert.ok(typeof expected === "string");
  assert.match(expected, /^[a-f0-9]{64}$/);
  const bytes = read(side, path);
  assert.ok(bytes, `missing ${side} phase contract input: ${path}`);
  assert.equal(hash(bytes), expected, `changed ${side} phase contract input: ${path}`);
  return bytes;
}
function consumer(read: ReadFile, value: unknown) {
  const test = object(value);
  const source = pinned(read, "head", test.source, test.sha256).toString("utf8");
  assert.equal(typeof test.selector, "string");
  const selector = String(test.selector);
  const name = selector.split("::").at(-1)!;
  if (String(test.source).endsWith(".rs")) {
    assert.match(source, new RegExp(`#\\[test\\][\\s\\S]*?\\bfn ${name}\\(`));
  } else {
    assert.ok(source.includes(`test(${JSON.stringify(selector)},`));
  }
  if (test.input !== null) {
    assert.ok(typeof test.input === "string");
    assert.ok(source.includes(test.input), "pinned input must have a real consumer");
  }
}

function legacyLaws(read: ReadFile, value: unknown) {
  const legacy = object(value);
  const rows = (["base", "head"] as const).map((side) => {
    const bytes = read(side, manifestPath);
    assert.ok(bytes, "both historical ABI registrations are required");
    const manifest = object(JSON.parse(bytes.toString("utf8")));
    assert.ok(Array.isArray(manifest.protocolCases));
    const selected = manifest.protocolCases.map(object).filter((row) => row.id === legacy.id);
    assert.equal(selected.length, 1, "the historical ABI has one immutable registration");
    assert.equal(hash(Buffer.from(canonical(selected[0]))), legacy.registrationSha256);
    return selected[0]!;
  });
  assert.deepEqual(rows[0], rows[1]);
  assert.ok(Array.isArray(legacy.inputs));
  for (const value of legacy.inputs) {
    const input = object(value);
    const before = pinned(read, "base", input.path, input.sha256);
    assert.ok(before.equals(pinned(read, "head", input.path, input.sha256)));
  }
  assert.ok(Array.isArray(legacy.tests));
  assert.ok(Array.isArray(rows[0].tests));
  assert.deepEqual(
    legacy.tests.map((value) => object(value).selector),
    rows[0].tests.map((value) => object(value).selector),
    "every historical ABI law remains a current source consumer",
  );
  legacy.tests.forEach((value) => consumer(read, value));
  const workflow = object(legacy.workflow);
  const before = pinned(read, "base", workflow.path, workflow.sha256);
  assert.ok(before.equals(pinned(read, "head", workflow.path, workflow.sha256)));
  const text = before.toString("utf8");
  assert.ok(text.includes("version: 7.0.2") && text.includes("version: 7.1.0-beta"));
  assert.ok(text.includes("run-source.ts") && text.includes("audit-wire.ts"));
}

/** Registration authenticates finite sources, never runtime or performance acceptance. */
export function newNativeDiagnosticPhaseCases(
  previous: unknown,
  current: unknown,
  read: ReadFile,
  paths: readonly string[] = sources,
) {
  const result = { ids: [] as string[], sources: new Set<string>() };
  if (!paths.some((path) => sources.includes(path))) return result;
  if (current === undefined) return result;
  assert.ok(previous === undefined || Array.isArray(previous));
  assert.ok(Array.isArray(current) && current.length <= 1);
  const old = new Set((previous ?? []).map((value: unknown) => object(value).id));
  for (const value of current) {
    const row = object(value);
    assert.equal(row.id, DIAGNOSTIC_PHASE_ID);
    if (old.has(row.id)) continue;
    assert.equal(
      hash(Buffer.from(canonical(row))),
      registrationSha256,
      "unregistered phase contract",
    );
    assert.equal(row.state, "active");
    assert.equal(row.adapter, "native-diagnostic-phase-current-v1");
    assert.deepEqual(row.targets, ["native-diagnostic-phase"]);
    assert.ok(Array.isArray(row.sources));
    assert.deepEqual(
      row.sources.map((value) => object(value).path),
      sources,
    );
    for (const value of row.sources) {
      const pair = object(value);
      pinned(read, "base", pair.path, pair.before);
      pinned(read, "head", pair.path, pair.after);
    }
    assert.ok(Array.isArray(row.inputs) && row.inputs.length === 5);
    for (const value of row.inputs) {
      const input = object(value);
      pinned(read, "head", input.path, input.sha256);
    }
    assert.ok(Array.isArray(row.tests) && row.tests.length === 15);
    row.tests.forEach((value) => consumer(read, value));
    legacyLaws(read, row.legacyProtocol);
    result.ids.push(DIAGNOSTIC_PHASE_ID);
    sources.forEach((path) => result.sources.add(path));
  }
  return result;
}
