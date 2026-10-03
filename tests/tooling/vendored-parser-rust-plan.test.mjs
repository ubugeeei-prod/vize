import assert from "node:assert/strict";
import { test } from "node:test";
import { rustCommand } from "../../tools/support/compat/github/run-affected-rust.mjs";

void test("vendored parser retains runtime tests without joining first-party Clippy", () => {
  const packages = ["vize_l1", "vize_oxc_parser"];
  const plan = {
    schemaVersion: 1,
    scope: "workspace",
    packages,
    cargoArgs: packages.flatMap((name) => ["--package", name]),
  };
  assert.deepEqual(rustCommand(plan, ["cargo", "clippy", "@packages@"]), [
    "cargo",
    "clippy",
    "--package",
    "vize_l1",
  ]);
  assert.deepEqual(rustCommand(plan, ["cargo", "test", "@packages@"]), [
    "cargo",
    "test",
    ...plan.cargoArgs,
  ]);
});
