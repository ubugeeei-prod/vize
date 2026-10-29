import assert from "node:assert/strict";
import { test } from "node:test";
import {
  lspMarkerProductionPaths,
  recordLspMarkerProductionDiff,
  selectLspMarkerSources,
  verifyLspMarkerProductionReceipt,
} from "../../tools/support/compat/davinci/lsp-marker-scan-source.ts";

const oldBase = "a".repeat(40);
const actualBase = "b".repeat(40);
const merge = "c".repeat(40);
const candidate = "d".repeat(40);

test("actual checked-out merge parents supersede a stale PR event base", () => {
  const selected = selectLspMarkerSources({
    head: merge,
    parents: [actualBase, candidate],
    requestedBase: oldBase,
    expectedPrHead: candidate,
  });
  assert.equal(selected.selectedBase, actualBase);
  assert.equal(selected.eventBaseMatchesActualParent, false);
  assert.equal(selected.expectedPrHead, candidate);
  assert.equal(selected.policy, "actual checked-out merge first parent");
});

test("unrelated merge heads and non-merge PR checkouts cannot qualify", () => {
  assert.throws(
    () =>
      selectLspMarkerSources({
        head: merge,
        parents: [actualBase, oldBase],
        requestedBase: oldBase,
        expectedPrHead: candidate,
      }),
    /requested candidate head/u,
  );
  assert.throws(
    () =>
      selectLspMarkerSources({
        head: candidate,
        parents: [actualBase],
        requestedBase: oldBase,
        expectedPrHead: candidate,
      }),
    /two-parent merge/u,
  );
});

test("dispatch accepts a distinct explicit immutable baseline and rejects mutable or identical refs", () => {
  const selected = selectLspMarkerSources({
    head: merge,
    parents: [actualBase],
    requestedBase: oldBase,
  });
  assert.equal(selected.selectedBase, oldBase);
  assert.equal(selected.eventBaseMatchesActualParent, null);
  for (const requestedBase of ["main", "", merge]) {
    assert.throws(() => selectLspMarkerSources({ head: merge, parents: [], requestedBase }), {
      code: "ERR_ASSERTION",
    });
  }
});

test("production receipts bind the selected sources and exact marker paths", () => {
  const selected = selectLspMarkerSources({
    head: merge,
    parents: [actualBase, candidate],
    requestedBase: oldBase,
    expectedPrHead: candidate,
  });
  const changed = recordLspMarkerProductionDiff(selected, [
    `${lspMarkerProductionPaths[1]}linux.rs`,
  ]);
  assert.equal(changed.productionChanged, true);
  assert.deepEqual(changed.productionFiles, [
    "crates/vize_maestro/src/server/state/project_contexts/root_discovery/linux.rs",
  ]);
  assert.equal(verifyLspMarkerProductionReceipt(changed, actualBase, merge), changed);
  const unchanged = recordLspMarkerProductionDiff(selected, []);
  assert.equal(unchanged.productionChanged, false);
  assert.throws(
    () => recordLspMarkerProductionDiff(selected, ["tools/unrelated.ts"]),
    /only marker discovery sources/u,
  );
  const topLevelTampered = structuredClone(unchanged);
  topLevelTampered.selectedBase = oldBase;
  assert.throws(
    () => verifyLspMarkerProductionReceipt(topLevelTampered, actualBase, merge),
    /wrong top-level selected baseline/u,
  );
  const tampered = structuredClone(unchanged);
  tampered.productionDiff.selectedBase = oldBase;
  assert.throws(
    () => verifyLspMarkerProductionReceipt(tampered, actualBase, merge),
    /wrong selected baseline/u,
  );
});
