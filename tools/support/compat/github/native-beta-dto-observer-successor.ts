import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  NATIVE_BETA_DTO_PROTOCOL_ID,
  nativeBetaDtoObserverBoundary,
  nativeBetaDtoProducerCustody,
} from "./native-beta-dto-fixtures.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
const object = (value: unknown): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** A current test observer is not a new ABI input or a native qualification. */
export function currentNativeBetaDtoObserver(
  paths: readonly string[],
  previous: unknown,
  current: unknown,
  read: ReadFile,
) {
  const boundary = nativeBetaDtoObserverBoundary();
  const changed = boundary.observers.filter((path) => paths.includes(path));
  const empty = { sources: new Set<string>(), evidence: null };
  if (changed.length === 0) return empty;
  assert.ok(previous === undefined || Array.isArray(previous));
  assert.ok(current === undefined || Array.isArray(current));
  const select = (rows: unknown[] | undefined) =>
    (rows ?? []).map(object).filter((row) => row.id === NATIVE_BETA_DTO_PROTOCOL_ID);
  const before = select(previous as unknown[] | undefined),
    after = select(current as unknown[] | undefined);
  // Initial ABI registration still belongs exclusively to the original path.
  if (before.length === 0 || after.length === 0) return empty;
  assert.equal(before.length, 1);
  assert.equal(after.length, 1);
  // Any real ABI source change keeps the existing new-registration requirement.
  for (const path of boundary.production) {
    const base = read("base", path),
      head = read("head", path);
    if (!base || !head || !base.equals(head)) return empty;
  }
  assert.deepEqual(after, before, "historical 10731 registration must remain unchanged");
  const row = after[0]!;
  assert.deepEqual(Object.keys(row).sort(), [
    "adapter",
    "id",
    "inputs",
    "state",
    "targets",
    "tests",
  ]);
  assert.equal(row.state, "active");
  assert.equal(row.adapter, "native-beta-diagnostic-wire-v1");
  assert.deepEqual(row.targets, ["native-diagnostic-dto"]);
  assert.ok(Array.isArray(row.inputs));
  assert.equal(row.inputs.length, boundary.inputs.length);
  const inputs = row.inputs.map((input, index) => {
    const value = object(input),
      path = boundary.inputs[index]!;
    assert.deepEqual(Object.keys(value).sort(), ["path", "sha256"]);
    assert.equal(path, "tests/_fixtures/differential/typechecker/" + value.path);
    const base = read("base", path),
      head = read("head", path);
    assert.ok(base && head && base.equals(head), "historical ABI bytes must remain unchanged");
    assert.equal(hash(head), value.sha256, "actual historical ABI input hash required");
    return { path, sha256: hash(head) };
  });
  const custody = nativeBetaDtoProducerCustody(read);
  for (const item of custody) {
    const base = read("base", item.archivedPath),
      head = read("head", item.archivedPath);
    assert.ok(base && head && base.equals(head), "historical producer archive changed");
  }
  const currentObserver = custody.filter((entry) => entry.currentObserverIdentity !== null);
  assert.equal(currentObserver.length, 2, "both preparation and audit must bind the observer");
  const identity = currentObserver[0]!.currentObserverIdentity!;
  const dependency = currentObserver[0]!.currentStorageObserverDependency!;
  for (const entry of currentObserver) {
    assert.deepEqual(entry.currentObserverIdentity, identity);
    assert.deepEqual(entry.currentStorageObserverDependency, dependency);
    assert.equal(entry.currentStorageRegistration, "after-source-selected-logical-editor-revision");
  }
  assert.deepEqual(boundary.observers, [identity.path, dependency.path]);
  assert.ok(Array.isArray(row.tests) && row.tests.length === 5);
  const historicalObserver = object(row.tests[3]);
  assert.equal(historicalObserver.source, identity.path);
  assert.equal(historicalObserver.sha256, identity.historicalSha256);
  return {
    sources: new Set(changed),
    evidence: {
      registration: "after-source-selected-logical-editor-revision" as const,
      sources: changed,
      historicalProtocol: NATIVE_BETA_DTO_PROTOCOL_ID,
      observer: identity,
      dependency,
      abiProductionUnchanged: boundary.production,
      historicalInputsUnchanged: inputs,
      nativeQualification: false as const,
    },
  };
}
