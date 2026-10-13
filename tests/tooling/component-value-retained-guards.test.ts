import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runNoMutatingComponentJoinCaller } from "./support/no-mutating-component-join.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
void test("complete merged source retains both literal NoMutating and component caller authorities", async () => {
  await runNoMutatingComponentJoinCaller(
    root,
    "tests/tooling/component-value-retained-guards.test.ts",
  );
});
