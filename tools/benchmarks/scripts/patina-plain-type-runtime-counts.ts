import assert from "node:assert/strict";

export interface PlainTypeRuntimePacket {
  readonly case: string;
  readonly actual: {
    readonly error_count: number;
    readonly warning_count: number;
    readonly diagnostics: readonly { readonly severity: string; readonly rule_name: string }[];
  };
}

const cases = new Set([
  "active",
  "javascript",
  "declaration-ts",
  "declaration-mts",
  "declaration-cts",
  "default",
  "host-disabled",
  "rule-disabled",
  "unavailable",
  "no-query",
]);

/** The two requested-but-unavailable cells fail; ordinary findings stay warnings. */
export function assertPlainTypeRuntimeCounts(packet: PlainTypeRuntimePacket): void {
  assert(cases.has(packet.case), "unregistered conditional plain-type packet case");
  const failed = packet.case === "unavailable";
  const expectedErrors = failed ? 1 : 0;
  assert.equal(packet.actual.error_count, expectedErrors);
  if (failed) {
    assert.equal(packet.actual.diagnostics.length, 1);
    assert.equal(packet.actual.diagnostics[0]?.rule_name, "type/corsa-runtime");
  }
  assert.equal(packet.actual.warning_count, packet.actual.diagnostics.length - expectedErrors);
  for (const diagnostic of packet.actual.diagnostics) {
    assert.equal(diagnostic.severity, failed ? "error" : "warning");
  }
}
