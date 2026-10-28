/** Applicability only: the original ordering observer and certificates stay strict. */
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { main, planFinalizationComparison } from "./cross-file-finalization-scope.ts";

export const HISTORICAL_ORDERING_PATHS = [
  "crates/vize_croquis_cf/src/analyzer/core/files.rs",
  "crates/vize_croquis_cf/src/graph.rs",
  "crates/vize_croquis_cf/src/graph/order.rs",
  "crates/vize_croquis_cf/src/graph/order/tests.rs",
] as const;

export function planFinalizationOrdering(changedPaths: readonly string[], event: string) {
  return planFinalizationComparison(changedPaths, event, HISTORICAL_ORDERING_PATHS);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main({
    expectedPaths: HISTORICAL_ORDERING_PATHS,
    outputDirectory: "target/vize-tests/cross-file-finalization-order-scope",
    summaryTitle: "Historical cross-file ordering comparison",
  });
}
