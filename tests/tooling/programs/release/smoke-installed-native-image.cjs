const assert = require("node:assert/strict"),
  fs = require("node:fs"),
  path = require("node:path"),
  crypto = require("node:crypto");
const installed = process.argv[2];
const native = require(path.join(installed, "node_modules/@vizejs/native"));
const token = native.getNativeBuildToken();
assert.match(token, /^[0-9a-f]{32}$/);
assert.notEqual(token, process.env.VIZE_NATIVE_BUILD_TOKEN);
assert.equal(native.getNativeBuildToken(), token);
const images = Object.keys(require.cache).filter((name) => name.endsWith(".node"));
assert.equal(images.length, 1);
for (const image of images)
  assert.ok(fs.realpathSync(image).startsWith(fs.realpathSync(installed) + path.sep));
console.log(
  JSON.stringify({
    token,
    images: images.map((name) => {
      const body = fs.readFileSync(name);
      return { sha256: crypto.createHash("sha256").update(body).digest("hex"), bytes: body.length };
    }),
  }),
);
