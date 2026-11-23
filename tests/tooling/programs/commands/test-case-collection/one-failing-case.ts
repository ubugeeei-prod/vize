import { test } from "node:test";
test("passes", () => {});
test("fails", () => {
  throw new Error("no");
});
