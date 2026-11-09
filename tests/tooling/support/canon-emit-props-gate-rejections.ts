import assert from "node:assert/strict";

export const emitGateRejections = [
  ["tests/_fixtures/tooling/emit-props-gate-11697.json", /immutable emit gate oracle context/u],
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
    /generic specialization image/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
    /generic specialization image/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/scope/component_events/generic_inference.rs",
    /integrated static listener source/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/scope/component_events/emit_props_gate.rs",
    /generic specialization image/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
    /selected emit gate producer: crates\/vize_canon\/src\/virtual_ts\/helpers\/declarations\/core-types\.d\.ts\.txt/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
    /canon leftovers source identity/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt",
    /canon leftovers source identity/u,
  ],
  [
    "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
    /canon leftovers source identity/u,
  ],
  ["crates/vize_canon/src/virtual_ts/scope/component_events.rs", /listener relation source/u],
] as const;

export function assertEmitGateRejectionPaths(sourceHashes: Record<string, string>): void {
  assert.deepEqual(
    emitGateRejections.map(([path]) => path),
    ["tests/_fixtures/tooling/emit-props-gate-11697.json", ...Object.keys(sourceHashes)],
  );
}
