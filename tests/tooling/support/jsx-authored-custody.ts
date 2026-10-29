import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export type PacketFacet = "components" | "diagnostics" | "module" | "moduleMap";
export interface ComponentPacket {
  code: string;
  map: string | null;
  mode: string;
  name: string;
  preamble: string;
  setup: string | null;
  style: string | null;
}
export interface ModulePacket {
  components: ComponentPacket[];
  diagnostics: string;
  module: string;
  moduleMap?: string | null;
  unexpected?: boolean;
}
export interface SuccessorRow {
  name: string;
  input: string;
  inputSha256: string;
  mode: string;
  historicalPacket: string;
  historicalSha256: string;
  successorPacket: string;
  successorSha256: string;
  allowedChangedFacets: string[];
  refusal: unknown;
  qualification: unknown;
}
export interface RefusalManifest {
  schemaVersion: number;
  pullRequest: string;
  historicalBaseline: unknown;
  capture: unknown;
  cases: SuccessorRow[];
}
export interface RuntimeBaseline {
  sourceHead: string;
  productionHead: string;
  nativeProbeSha256: string;
  cases: { input: string; inputSha256: string; output: string; outputSha256: string }[];
}
export interface AdversarialRow {
  observation: string;
  observationSha256: string;
  sourceSha256: string;
  facet: "summary" | "module";
  historical: string;
  historicalSha256: string;
  successor: string;
  successorSha256: string;
  runtime: boolean;
}
export interface AdversarialPacket {
  source: string;
  summary: string;
  module: string;
}
export interface AdversarialAuthority {
  schema: number;
  issue: number;
  pullRequest: number;
  capture: unknown;
  cases: AdversarialRow[];
}
export interface StaticAuthority {
  source: string;
  sourceSha256: string;
  historical: string;
  historicalSha256: string;
  successor: string;
  successorSha256: string;
  allowedChangedPointers: string[];
  qualification: { module: string; components: string; nativeHandled: number; runtime: boolean };
}
export interface LegacyRoles {
  fixedProducer: { entry: string };
  native: unknown;
}
export interface RoleManifest {
  schemaVersion: number;
  pullRequest: string;
  historical: LegacyRoles & { packet: string; sha256: string };
  capture: unknown;
  producerControl: unknown;
  runtimeControl: unknown;
}
export const hash = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");

const packetFields = ["components", "diagnostics", "module", "moduleMap"];

export function fields(value: unknown, expected: readonly string[]): void {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  const compare = (left: string, right: string) => left.localeCompare(right);
  assert.deepEqual(Object.keys(value).sort(compare), [...expected].sort(compare));
}

export function snapshotBody(bytes: Buffer): string {
  const text = bytes.toString("utf8");
  const boundary = text.indexOf("\n---\n", 4);
  assert.ok(text.startsWith("---\n") && boundary >= 0);
  return text.slice(boundary + 5);
}

export function verifyAdversarial(
  row: AdversarialRow,
  historical: string,
  successor: string,
  packet: AdversarialPacket,
): void {
  fields(packet, ["source", "summary", "module"]);
  assert.equal(hash(packet.source), row.sourceSha256);
  assert.equal(row.runtime, false);
  assert.equal(successor, packet[row.facet].replace(/\n*$/, "\n"));
  if (row.observation === "scoped_style_extraction.json") {
    assert.equal(row.facet, "summary");
    const scoped = "<p data-v-2216d773>";
    assert.equal(successor.split(scoped).length - 1, 2);
    assert.equal(successor.replaceAll(scoped, "<p>"), historical);
  } else if (row.facet === "summary") {
    const oldDiagnostic =
      '    diagnostics: [\n        DiagnosticSummary {\n            severity: "Error",\n            message: "Vapor/SSR authored module preservation is not supported for imports, exports or captured setup bindings; use VDOM output or consume the per-component renderer",\n            source: "export const DefaultMode = () => <main>default</main>;",\n            range: 1..55,\n        },\n    ],';
    assert.equal(historical.split(oldDiagnostic).length - 1, 1);
    assert.equal(historical.replace(oldDiagnostic, "    diagnostics: [],"), successor);
  } else {
    assert.equal(row.facet, "module");
    assert.equal(historical, "");
    assert.equal(packet.module.length, 1016);
  }
}

export function verifyStaticSuccessor(
  authority: StaticAuthority,
  previous: ModulePacket,
  current: ModulePacket,
): void {
  assert.deepEqual(authority.allowedChangedPointers, ["/components/3/code", "/components/4/code"]);
  assert.deepEqual(authority.qualification, {
    module: "refused",
    components: "inspectable-producer-only",
    nativeHandled: 0,
    runtime: false,
  });
  fields(previous, packetFields);
  fields(current, packetFields);
  assert.equal(current.module, "");
  assert.equal(current.moduleMap, null);
  assert.equal(current.components.length, 5);
  const restored = structuredClone(current);
  for (const index of [3, 4]) {
    assert.notEqual(current.components[index].code, previous.components[index].code);
    restored.components[index].code = previous.components[index].code;
  }
  assert.deepEqual(restored, previous, "only two exact component code pointers may change");
}
