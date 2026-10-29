import assert from "node:assert/strict";
import { projectionCoordinateProof } from "./typechecker-template-emit-projection-history-coordinates.ts";
import { qualifyFrozenProjectionBaseline } from "./typechecker-template-emit-projection-history-baseline.ts";
import { loadRetainedProjectionFixture } from "./typechecker-template-emit-projection-history-fixture.ts";
import type { RetainedProjectionFixture } from "./typechecker-template-emit-projection-history-model.ts";
import {
  applyInverseEdits,
  retainedArtifactCustody,
  retainedFamilies,
  sha256,
} from "./typechecker-template-emit-projection-history-shared.ts";

export { projectionCoordinateProof } from "./typechecker-template-emit-projection-history-coordinates.ts";
export { qualifyFrozenProjectionBaseline } from "./typechecker-template-emit-projection-history-baseline.ts";
export {
  buildRetainedProjectionFixture,
  loadRetainedProjectionFixture,
} from "./typechecker-template-emit-projection-history-fixture.ts";
export { qualifyLiveProjectionCapture } from "./typechecker-template-emit-projection-history-live.ts";
export {
  applyInverseEdits,
  diffGeneratedBytes,
  historicalInterval,
  historicalOffset,
  retainedArtifactCustody,
  retainedFamilies,
  retainedManifestSha256,
  retainedManifestUrl,
  sha256,
} from "./typechecker-template-emit-projection-history-shared.ts";

export function qualifyRetainedProjectionHistory(
  fixture: RetainedProjectionFixture = loadRetainedProjectionFixture(),
) {
  const { manifest, packets } = fixture;
  assert.deepEqual(manifest.artifacts, retainedArtifactCustody);
  assert.deepEqual(manifest.families, retainedFamilies);
  const expectedIds = retainedFamilies.flatMap((family) =>
    ["canon", "content-mapper", "maestro"].map((surface) => `${family}/${surface}`),
  );
  assert.deepEqual(
    manifest.editScripts.map(({ id }) => id),
    expectedIds,
  );
  const editProofs = [];
  for (const script of manifest.editScripts) {
    assert.equal(
      script.path,
      `${script.family}/${script.surface === "maestro" ? "maestro-document-0" : script.surface}/text.ts`,
    );
    const before = packets.baseline.get(script.path);
    const after = packets.incident.get(script.path);
    assert(before && after, `missing edit-script input: ${script.path}`);
    assert.equal(before.length, script.beforeBytes);
    assert.equal(after.length, script.afterBytes);
    assert.equal(sha256(before), script.beforeSha256);
    assert.equal(sha256(after), script.afterSha256);
    assert(script.edits.length > 0, `empty edit script: ${script.id}`);
    let previousBeforeEnd = 0;
    let previousAfterEnd = 0;
    for (const edit of script.edits) {
      assert(edit.beforeStart >= previousBeforeEnd && edit.afterStart >= previousAfterEnd);
      assert(edit.beforeEnd >= edit.beforeStart && edit.afterEnd >= edit.afterStart);
      assert(edit.beforeEnd <= before.length && edit.afterEnd <= after.length);
      assert(
        edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd,
        `empty edit in ${script.id}`,
      );
      const beforeSlice = before.subarray(edit.beforeStart, edit.beforeEnd);
      const afterSlice = after.subarray(edit.afterStart, edit.afterEnd);
      assert(!beforeSlice.equals(afterSlice), `no-op edit in ${script.id}`);
      if (beforeSlice.length > 0 && afterSlice.length > 0) {
        assert.notEqual(beforeSlice[0], afterSlice[0], `untrimmed edit prefix in ${script.id}`);
        assert.notEqual(
          beforeSlice[beforeSlice.length - 1],
          afterSlice[afterSlice.length - 1],
          `untrimmed edit suffix in ${script.id}`,
        );
      }
      previousBeforeEnd = edit.beforeEnd;
      previousAfterEnd = edit.afterEnd;
    }
    assert.deepEqual(applyInverseEdits(after, before, script.edits), before);
    editProofs.push({
      id: script.id,
      edits: script.edits.length,
      beforeBytes: before.length,
      afterBytes: after.length,
      restoredSha256: sha256(before),
    });
  }
  const coordinateProof = projectionCoordinateProof(manifest, packets);
  assert.deepEqual(coordinateProof, manifest.coordinateProof);
  const frozenBaseline = qualifyFrozenProjectionBaseline(packets);
  return {
    artifacts: manifest.artifacts,
    packetFiles: manifest.packets.baseline.files.length,
    editProofs,
    coordinateProof,
    frozenBaseline,
  };
}
