import { test } from "node:test";
void test("passes", () => {});
void test("fails", () => {
  throw new Error("no");
});
