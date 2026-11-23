import { describe, test } from "node:test";
void test("passes", () => {});
void test("fails", () => {
  throw new Error("no");
});
void test("skips", { skip: true }, () => {});
void test("todo", { todo: true }, () => {});
void describe("group", () => {
  void test("inner", () => {});
});
