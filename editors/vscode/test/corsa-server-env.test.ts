import assert from "node:assert/strict";
import test from "node:test";

import { serverProcessEnv } from "../src/corsa-runtime.ts";

void test("the editor leaves native runtime selection to the project resolver", () => {
  const env = Object.freeze({ PATH: "/usr/bin" });
  assert.deepEqual(serverProcessEnv(env, false), env);
  assert.deepEqual(serverProcessEnv(env, true), { PATH: "/usr/bin", RUST_BACKTRACE: "1" });
  assert.deepEqual(env, { PATH: "/usr/bin" });
});

void test("all explicit native runtime overrides survive normal and debug startup unchanged", () => {
  for (const key of [
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "TSRS_PATH",
    "TSRS_EXECUTABLE",
  ]) {
    const env = Object.freeze({ [key]: "/workspace/日本語/native compiler", PATH: "/usr/bin" });
    assert.deepEqual(serverProcessEnv(env, false), env);
    assert.deepEqual(serverProcessEnv(env, true), { ...env, RUST_BACKTRACE: "1" });
    if (key !== "CORSA_PATH") assert.equal(serverProcessEnv(env, false).CORSA_PATH, undefined);
  }
});

void test("empty overrides remain empty and are not replaced by an extension cache", () => {
  const env = Object.freeze({ CORSA_PATH: "", TSGO_PATH: "", TSRS_PATH: "" });
  assert.deepEqual(serverProcessEnv(env, false), env);
  assert.deepEqual(serverProcessEnv(env, true), { ...env, RUST_BACKTRACE: "1" });
});
