/** Negative controls corrupt copies of real captured packets; no new native process. */
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createHash } from "node:crypto";
import { array, canonical, first, frames, object } from "./verify-codec.ts";
import { strictJson } from "./verify-json.ts";
import { snapshotIdentity, verifyMutations } from "./verify-mutations.ts";
import type { ObjectValue } from "./verify-codec.ts";

const directory = process.argv[2],
  version = process.argv[3];
if (directory === undefined || version === undefined)
  throw new Error("usage: verify-mutation-controls.ts DIRECTORY VERSION");
const clientBytes = readFileSync(join(directory, "client-to-native.wire.bin"));
const nativeBytes = readFileSync(join(directory, "native-to-client.wire.bin"));
const carrierValue = object(
  strictJson(readFileSync(join(directory, "carrier-receipt.json"), "utf8")),
);
const originalRequests = frames(clientBytes),
  originalResponses = frames(nativeBytes);
const modes = ["stale-initializer-target", "stale-request-snapshot", "type-origin-as-value-origin"];
const rows: unknown[] = [];
for (const mode of modes) {
  const requests = structuredClone(originalRequests);
  const responses = structuredClone(originalResponses);
  const carrier = structuredClone(carrierValue);
  const phases = array(carrier.semanticMutationPhases).map(object);
  const vue = phases.find((phase) => phase.label === "initializer-vue");
  const custom = phases.find((phase) => phase.label === "initializer-custom");
  assert.ok(vue && custom);
  const matching = (phase: ObjectValue, method: string) =>
    requests.filter(
      (request) =>
        request.method === method &&
        snapshotIdentity(object(request.params).snapshot) === snapshotIdentity(phase.snapshot),
    );
  const initializers = (phase: ObjectValue) =>
    matching(phase, "getSymbolAtPosition").filter((request) => {
      const params = object(request.params);
      return (
        params.file === phase.authoredPath && params.position === phase.initializerUtf16Position
      );
    });
  const byID = new Map(responses.map((response) => [canonical(response.id), response]));
  const response = (request: ObjectValue): ObjectValue => {
    const found = byID.get(canonical(request.id));
    assert.ok(found);
    return found;
  };
  const current = first(initializers(custom));
  if (mode === "stale-request-snapshot") {
    object(current.params).snapshot = vue.snapshot;
  } else {
    const substitute =
      mode === "stale-initializer-target"
        ? response(first(initializers(vue))).result
        : response(first(matching(custom, "getSymbolOfType"))).result;
    response(current).result = structuredClone(substitute);
  }
  assert.throws(() =>
    verifyMutations(carrier, requests, (request) => response(request).result, version),
  );
  rows.push({
    control: mode,
    rejected: true,
    scope: "synthetic corruption of captured full JSON packets",
  });
}
const sha = (data: Buffer) => createHash("sha256").update(data).digest("hex");
assert.equal(sha(readFileSync(join(directory, "client-to-native.wire.bin"))), sha(clientBytes));
assert.equal(sha(readFileSync(join(directory, "native-to-client.wire.bin"))), sha(nativeBytes));
const receipt = {
  schema: "vize.binding-origin.mutation-wire-controls.v1",
  version,
  controls: rows,
  originalClientSha256: sha(clientBytes),
  originalNativeSha256: sha(nativeBytes),
  nativeProcessesStarted: 0,
  originClassifierQualified: false,
  hostWire: false,
  passed: true,
};
writeFileSync(
  join(directory, "semantic-mutation-controls.json"),
  JSON.stringify(receipt, null, 2) + "\n",
  { flag: "wx" },
);
console.log(JSON.stringify(receipt));
