// Whole original outside arrays: only the authenticated lazy source is promoted.
import assert from "node:assert/strict";
import { object } from "./native-input-model-trim-boundary.ts";

export function assertCurrentModelOutside(value: unknown, inputs: unknown[], lazySource: unknown) {
  assert(typeof lazySource === "string");
  object(value);
  for (const target of ["dom", "ssr", "vapor"] as const) {
    const rows: unknown = value[target];
    assert(Array.isArray(rows));
    const expected = inputs.flatMap((input: unknown) => {
      object(input);
      return target === "dom"
        ? ["development", "production"].map((mode) => [input.id, input.source, mode])
        : [[input.id, input.source]];
    });
    assert.deepEqual(
      rows.map((row: unknown): unknown[] => {
        object(row);
        object(row.result);
        const promoted: boolean = row.id === "modifier" && row.source === lazySource;
        assert.equal(row.result.state, promoted ? "emitted" : "refused");
        if (promoted) assert(typeof row.result.code === "string");
        else assert(typeof row.result.error === "string");
        return target === "dom" ? [row.id, row.source, row.mode] : [row.id, row.source];
      }),
      expected,
    );
  }
}
