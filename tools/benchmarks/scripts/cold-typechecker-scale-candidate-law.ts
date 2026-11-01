/** Finite output/count relations only. Source pins and semantic qualification are separate. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { strictDebugParts } from "./cold-typechecker-scale-strict-pair.ts";

type Packet = { stdout: Buffer; stderr: Buffer };
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");

/** Reconstruct the ENTIRE expected after packet; retain the original buffers unchanged. */
export function verifyComponentChecksTransition(
  before: Packet,
  after: Packet,
  cwd: string,
  assets: { before: Buffer; after: Buffer },
) {
  assert(path.isAbsolute(cwd));
  assert(before.stdout.equals(after.stdout), "whole authored JSON/code/map/opaque bytes changed");
  assert(!assets.before.equals(assets.after), "source asset transition must be real");
  const moduleHeader = Buffer.from('declare module "vize:virtual-ts" {\n');
  for (const asset of [assets.before, assets.after]) {
    assert(asset.subarray(0, moduleHeader.length).equals(moduleHeader));
    assert(asset.subarray(-2).equals(Buffer.from("}\n")));
  }
  const value: unknown = JSON.parse(before.stdout.toString("utf8"));
  const left = strictDebugParts(before.stderr, value, cwd);
  const right = strictDebugParts(after.stderr, value, cwd);
  assert(left.header.equals(right.header));
  assert(left.suffix.equals(right.suffix), "whole generated suffix/order/geometry changed");
  const offset = left.helper.indexOf(assets.before);
  assert(offset >= 0, "target whole source asset is not applied to this program");
  assert.equal(left.helper.indexOf(assets.before, offset + 1), -1, "ambiguous source asset");
  const expectedHelper = Buffer.concat([
    left.helper.subarray(0, offset),
    assets.after,
    left.helper.subarray(offset + assets.before.length),
  ]);
  assert(expectedHelper.equals(right.helper), "unclassified helper prefix or suffix change");
  const expectedWhole = Buffer.concat([left.header, expectedHelper, left.suffix]);
  assert(expectedWhole.equals(after.stderr), "whole after debug bytes changed");
  return {
    sourceAssetOccurrences: 1,
    beforeAssetBytes: assets.before.length,
    afterAssetBytes: assets.after.length,
    beforeAssetSha256: hash(assets.before),
    afterAssetSha256: hash(assets.after),
    beforeWholeDebugSha256: hash(before.stderr),
    afterWholeDebugSha256: hash(after.stderr),
    scope:
      "one source-authenticated complete component-checks asset; all other raw bytes equal; no compiler-instantiation or semantic-authority claim",
  };
}

/** Interpret a complete counter's separately validated counts; never invent outer Vec capacity. */
export function classifyRoutePopulation(counts: {
  bindings: number;
  positiveBindings: number;
  negativeBindings: number;
  bindingInlineInitializedBytes: number;
}) {
  for (const value of Object.values(counts))
    assert(Number.isSafeInteger(value) && value >= 0, "unsafe or negative source count");
  assert.equal(counts.bindings, counts.positiveBindings + counts.negativeBindings);
  assert.equal(counts.bindings === 0, counts.bindingInlineInitializedBytes === 0);
  return {
    ...counts,
    population:
      counts.bindings === 0
        ? "empty-binding-vector"
        : counts.positiveBindings === 0
          ? "nonempty-negative-bindings"
          : "positive-route-payload",
    initializedBindingCloneWorkPresent: counts.bindings > 0,
    outerBindingVectorCapacity: "unobserved",
    scope:
      "initialized binding population only; structural capacities are not actual clone allocations, retained memory or RSS saved",
  };
}
