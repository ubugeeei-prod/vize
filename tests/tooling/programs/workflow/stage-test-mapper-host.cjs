const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const root = path.join(process.env.RUNNER_TEMP, "mq-content-mapper-host");
const proof = path.join(process.env.RUNNER_TEMP, "mq-content-mapper-host-proof");
assert.deepEqual(process.argv.slice(2), [
  "tools/support/release/type-admission-mapper-host.ts",
  root,
  proof,
]);
for (const name of [
  "GH_TOKEN",
  "GITHUB_TOKEN",
  "NODE_AUTH_TOKEN",
  "NPM_TOKEN",
  "NODE_OPTIONS",
  "NODE_PATH",
]) {
  assert.equal(Object.hasOwn(process.env, name), false, name);
}
if (process.env.MOCK_MODE === "bootstrap-failure") process.exit(23);
fs.mkdirSync(proof, { recursive: true });
if (process.env.MOCK_MODE === "success") {
  const bin = path.join(root, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  fs.copyFileSync(process.env.MOCK_TSC, bin);
  fs.chmodSync(bin, 0o755);
}
