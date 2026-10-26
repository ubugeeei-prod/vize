import assert from "node:assert/strict";
import { test } from "node:test";
import { eligible, rewrite, run } from "../../tools/support/levels/rename-l3-vapor-counters.ts";

const oldPrefix = ["davinci", "s3_vapor", ""].join(".");

test("live sources use only the l3.vapor counter namespace", () => {
  assert.deepEqual(run("--check"), []);
});

test("the rename rewrites every old counter and keeps captured history", () => {
  const source = `"${oldPrefix}accepted" "${oldPrefix}legacy.options"`;
  assert.equal(rewrite(source), `"l3.vapor.accepted" "l3.vapor.legacy.options"`);
  assert.equal(rewrite(rewrite(source)), rewrite(source));
  assert.equal(eligible("crates/vize_atelier_vapor/src/l3.rs"), true);
  assert.equal(eligible("crates/vize_atelier_sfc/tests/fixtures/fix-history/a.json"), false);
  assert.equal(eligible("crates/vize_atelier_sfc/tests/fixtures/archived.rs"), false);
  assert.equal(eligible("docs/davinci/plan/phase-2-tasks.md"), false);
});
