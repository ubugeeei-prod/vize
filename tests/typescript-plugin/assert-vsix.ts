import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const vsix = path.join(root, "editors/vscode/dist/vize.vsix");
const source = path.join(root, "npm/typescript-plugin");
const files = fs
  .readdirSync(source)
  .filter((name) => /\.cjs$|\.d\.cts$|^README\.md$|^package\.json$/.test(name));
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const plugin = files.map((name) => {
  const actual = execFileSync(
    "unzip",
    ["-p", vsix, `extension/node_modules/@vizejs/typescript-vue-plugin/${name}`],
    { maxBuffer: 1_000_000 },
  );
  const expected = fs.readFileSync(path.join(source, name));
  assert.deepEqual(actual, expected, `VSIX ${name} differs from canonical npm source`);
  return { name, sha256: sha256(actual) };
});
const capture = process.env.VIZE_TYPESCRIPT_PLUGIN_CAPTURE;
assert.ok(capture, "VIZE_TYPESCRIPT_PLUGIN_CAPTURE names the capture directory");
const output = path.resolve(capture);
fs.writeFileSync(
  path.join(output, "vsix-receipt.json"),
  JSON.stringify(
    {
      source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
      vsixSha256: sha256(fs.readFileSync(vsix)),
      plugin,
      passed: true,
    },
    null,
    2,
  ),
);
