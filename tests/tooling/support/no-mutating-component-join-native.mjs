import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  authenticateNoMutatingComponentJoin,
  authenticateNoMutatingComponentJoinCommitted,
} from "./no-mutating-component-join.mjs";

/** Preserve old source lineage separately from the actual current native build source. */
export async function currentJoinDeepPropsNativeSource(root) {
  const joined = authenticateNoMutatingComponentJoin(root);
  const currentHead = authenticateNoMutatingComponentJoinCommitted(root);
  const historical = joined.frames.find(
    (context) => context.frame.source === "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  );
  assert(historical);
  const old = await import(
    pathToFileURL(
      path.join(historical.root, "tests/tooling/support/component-value-retained-guards.ts"),
    ).href
  );
  const family = old.authenticateDeepPropsSourceFamily(historical.root);
  const deepPropsFamily = {
    ...family,
    historicalSource: historical.frame.source,
    historicalProjection: true,
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  };
  return {
    deepPropsFamily,
    receipt: {
      authority: "complete-current-no-mutating-component-source-join",
      sourceRevision: currentHead,
      completeCurrentUnionBodies: joined.completeCurrentUnionBodies,
      completeCurrentClosedBodies: joined.completeCurrentClosedBodies,
      historicalFamilySource: historical.frame.source,
      runtimeCredit: false,
      performanceCredit: false,
      acceptanceCredit: "none",
    },
    assertBuildSource(expectedRevision) {
      assert.equal(
        expectedRevision,
        currentHead,
        "actual native build must use the joined current source",
      );
      assert.equal(authenticateNoMutatingComponentJoinCommitted(root), currentHead);
    },
  };
}
