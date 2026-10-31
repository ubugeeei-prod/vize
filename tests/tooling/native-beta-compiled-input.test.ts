import assert from "node:assert/strict";
import { test } from "node:test";
import {
  compiledInputPath,
  compiledInputSha256,
  verifyCompiledInput,
  verifyObserverDependencies,
  commonObserverSha256,
  storageObserverPath,
  storageObserverSha256,
} from "./support/native-beta-dto/beta-attached-route.ts";

const input = () => [{ path: compiledInputPath, bytes: 1108, sha256: compiledInputSha256 }];
void test("actual compiled input receipt joins the separate prepared receipt without replacing the Rust helper", () => {
  verifyCompiledInput(input(), input());
  const dependencies = [{ path: storageObserverPath, sha256: storageObserverSha256 }];
  const observer = {
    path: storageObserverPath.replace("/native_beta/storage_authority.rs", "/native_beta.rs"),
    sha256: commonObserverSha256,
  };
  verifyObserverDependencies(dependencies, dependencies, commonObserverSha256, observer);
  assert.throws(() => verifyCompiledInput(dependencies, dependencies));
  assert.throws(() => verifyObserverDependencies(input(), input(), commonObserverSha256, observer));
});
void test("missing, extra, mismatched or consistently forged compiled input facts refuse", () => {
  for (const value of [
    undefined,
    null,
    [],
    [...input(), ...input()],
    [{ ...input()[0], path: storageObserverPath }],
    [{ ...input()[0], path: compiledInputPath.replace("input.json", "source-authority.json") }],
    [{ ...input()[0], bytes: 1107 }],
    [{ ...input()[0], sha256: "0".repeat(64) }],
    [{ ...input()[0], ignored: true }],
  ]) {
    assert.throws(() => verifyCompiledInput(value, value));
    assert.throws(() => verifyCompiledInput(input(), value));
    assert.throws(() => verifyCompiledInput(value, input()));
  }
});
