import type { AttachedOwner } from "./beta-attached-route.ts";
import type { WatcherContext } from "./stable-watcher-boundary.ts";
import { verifyAfterLogicalRevision } from "./logical-revision.ts";
import { verifyBetaSemanticRoute } from "./semantic-route.ts";
import { verifyLogicalBatchSuffix } from "./logical-batch.ts";

/** Current logical revisions admit no physical-project membership method or URI alias. */
export function verifyBetaLogicalRoute(
  owners: readonly AttachedOwner[],
  context: WatcherContext,
  expectedSource: string,
  attachedRoot: string,
  sourceRows: readonly unknown[],
  attachedValue: unknown,
) {
  const revision = verifyAfterLogicalRevision(owners, context, expectedSource, attachedValue);
  return verifyBetaSemanticRoute(
    owners,
    context,
    expectedSource,
    attachedRoot,
    sourceRows,
    revision,
    revision.logicalRoot,
    () => [],
    (owner, packets) => verifyLogicalBatchSuffix(owner, packets, attachedValue),
  );
}
