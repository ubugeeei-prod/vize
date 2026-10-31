import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";
import {
  sourceSha,
  validateRecords,
  type Records,
} from "../../tools/support/compat/github/ui-startup-attribution.ts";
const expected = ["a", "b"].map((name) => resolve("/fixture-checkout/npm/ui/src", name + ".vue"));
function records(): Records {
  const inputs = [0, 1].map((index) => ({
    worker_pid: 100 + index,
    worker_index: index,
    worker_total: 2,
    input_ordinal: 0,
    caller_attempt_id: `caller-${100 + index}-0`,
    input_path: expected[index],
    input_sha256: "a".repeat(64),
    wire_request_id: null,
  }));
  return {
    parents: [
      {
        schema: "vize.ui.startup.process-samples.v1",
        provenance: { source_sha: sourceSha },
        outcome: "returned",
        expected_worker_census: "UNKNOWN_not_validated",
      },
    ],
    workers: inputs.map((input) => ({
      schema: "vize.ui.startup.worker.v1",
      provenance: { source_sha: sourceSha },
      worker_pid: input.worker_pid,
      inputs: [input],
      content_hashes_deferred_to_parent: true,
    })),
    constructors: [
      {
        schema: "vize.ui.startup.constructor.v1",
        caller: { ...inputs[0] },
        sdk_wire_request_id: null,
        config: { profiler_was_configured: false, profile_collection: "wrap_existing_sink_only" },
        elapsed_ns: "123",
        returned: true,
        events: [],
        dropped: 0,
      },
    ],
  };
}
test("synthetic absolute caller census joins constructor packets without RPC/image claims", () => {
  const result = validateRecords(records(), expected);
  assert.equal(result.callerCensusComplete, true);
  assert.equal(result.inputs, 2);
  assert.equal(result.observedConstructors, 1);
  assert.equal(result.constructorCensus, "UNKNOWN_not_validated");
  assert.equal(result.actualRpcWaitProven, false);
  assert.equal(result.physicalSameImageProven, false);
});
test("relative and foreign-root producer paths cannot match the authenticated absolute census", () => {
  for (const path of ["src/a.vue", resolve("/foreign-checkout/npm/ui/src/a.vue")]) {
    const value = records();
    (value.workers[0].inputs as Record<string, unknown>[])[0].input_path = path;
    assert.throws(() => validateRecords(value, expected));
  }
});
test("missing worker and fail-fast prefixes never certify the whole original input census", () => {
  const missing = records();
  missing.workers.pop();
  assert.equal(validateRecords(missing, expected).callerCensusComplete, false);
  const prefix = records();
  prefix.parents[0] = {
    ...prefix.parents[0],
    outcome: "threw",
    executable_hashes: [],
    build_manifests: [],
    hashes_omitted_reason: "siblings_may_still_be_active",
  };
  assert.equal(validateRecords(prefix, expected).callerCensusComplete, false);
  prefix.parents[0].executable_hashes = [{}];
  assert.throws(() => validateRecords(prefix, expected));
});
test("absent metadata, foreign caller/source, duplicate worker and newly enabled profiler are refused", () => {
  assert.throws(() => validateRecords({ parents: [], workers: [], constructors: [] }, expected));
  const foreign = records();
  foreign.constructors[0].caller = { caller_attempt_id: "foreign" };
  assert.throws(() => validateRecords(foreign, expected));
  const source = records();
  source.parents[0].provenance = { source_sha: "0".repeat(40) };
  assert.throws(() => validateRecords(source, expected));
  const duplicate = records();
  duplicate.workers[1] = duplicate.workers[0];
  assert.throws(() => validateRecords(duplicate, expected));
  const profiled = records();
  profiled.constructors[0].config = {
    profiler_was_configured: true,
    profile_collection: "wrap_existing_sink_only",
  };
  assert.throws(() => validateRecords(profiled, expected));
});
