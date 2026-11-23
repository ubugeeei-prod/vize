const assert = require("node:assert/strict");
const { createRequire } = require("node:module");
const path = require("node:path");

const expectedPackage = process.argv[2];
const consumer = createRequire(path.join(process.cwd(), "package.json"));
const fromVize = createRequire(consumer.resolve("vize"));
const native = fromVize("@vizejs/native");
const fromNative = createRequire(fromVize.resolve("@vizejs/native"));
const selected = fromNative(expectedPackage);
assert.strictEqual(native, selected, expectedPackage + " was installed but not loaded");
const token = native.getNativeBuildToken();
assert.match(token, /^[0-9a-f]{32}$/u, "loaded native image has no build token");
assert.equal(selected.getNativeBuildToken(), token, "loaded native build token differs");
const identity = { package: expectedPackage, token };
if (process.argv[3] === "--identity-only") process.stdout.write(JSON.stringify(identity));
else console.log(JSON.stringify({ schema: "vize.loadedNativeBinding", version: 1, ...identity }));
