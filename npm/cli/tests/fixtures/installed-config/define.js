import assert from "node:assert/strict";
import { defineConfig, normalizeGlobalTypes } from "vize/config";
import { shared } from "./shared.mjs";
assert.equal(defineConfig(shared), shared);
const array = [shared];
assert.equal(defineConfig(array), array);
const factory = () => shared;
assert.equal(defineConfig(factory), factory);
const asyncFactory = async () => shared;
assert.equal(defineConfig(asyncFactory), asyncFactory);
assert.deepEqual(normalizeGlobalTypes({ types: { UserId: "string" } }), {
  UserId: { type: "string" },
});
