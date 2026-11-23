import { describe, test } from "node:test";
test("passes", () => {});
test("fails", () => {
  throw new Error("no");
});
test("skips", { skip: true }, () => {});
test("todo", { todo: true }, () => {});
describe("group", () => {
  test("inner", () => {});
});
