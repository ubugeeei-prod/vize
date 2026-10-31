import assert from "node:assert/strict";
import { test } from "node:test";
import { contract } from "./support/jsx-child-backend-profile-contract.ts";
import { verifyCurrentBackendOutputs } from "./support/jsx-current-backend-profile-contract.ts";
const rows = () =>
  contract.rows.map(({ source, ssr, iterationsPerSample, after }) => ({
    source,
    ssr,
    iterationsPerSample,
    output: structuredClone(after),
    samplesNs: [1, 2, 3, 4, 5, 6, 7, 8, 9],
  }));
test("current actual base and head retain all six frozen successor outputs", () =>
  verifyCurrentBackendOutputs(rows(), rows()));
test("current admission rejects original-before data, partial outputs and input/sample mutations", () => {
  for (let index = 0; index < 6; index++) {
    for (const role of ["base", "head"]) {
      const base = rows(),
        head = rows(),
        target = role === "base" ? base : head;
      target[index]!.output = ["changed"];
      assert.throws(() => verifyCurrentBackendOutputs(base, head));
    }
    for (const key of ["source", "ssr", "iterationsPerSample", "samplesNs"] as const) {
      const head = rows();
      Object.assign(head[index]!, { [key]: key === "samplesNs" ? [0] : null });
      assert.throws(() => verifyCurrentBackendOutputs(rows(), head));
    }
  }
  const old = rows();
  old[0]!.output = structuredClone(contract.rows[0]!.before);
  assert.throws(() => verifyCurrentBackendOutputs(old, rows()));
  assert.throws(() => verifyCurrentBackendOutputs(rows().slice(1), rows()));
});
