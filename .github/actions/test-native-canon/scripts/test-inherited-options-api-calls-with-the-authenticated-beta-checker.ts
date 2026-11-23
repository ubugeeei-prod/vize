import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
const program = fs.realpathSync(process.argv[2]);
const directory = process.argv[3];
const manifestPath = path.join(path.dirname(path.dirname(program)), "package.json");
const manifestBytes = fs.readFileSync(manifestPath);
const manifest = JSON.parse(manifestBytes.toString("utf8"));
const sha256 = createHash("sha256").update(fs.readFileSync(program)).digest("hex");
const save = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n", {
    flag: "wx",
  });
save("identity-preflight.json", {
  program,
  sha256,
  providerRole: process.env.VIZE_TEST_INHERITED_PROVIDER_ROLE,
  expectedDiagnosticExitCode: 2,
  manifestPath,
  manifestBytes: [...manifestBytes],
  scope: "Authenticated supplemental source 7.1 control; reporter binary identity is unknown",
});
assert.equal(process.env.VIZE_TEST_INHERITED_PROVIDER_ROLE, "a183-ts7.1");
assert.equal(manifest.name, "@typescript/typescript-linux-x64");
assert.equal(manifest.version, "7.1.0-beta");
assert.equal(sha256, "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70");
const result = spawnSync(program, ["--version"]);
fs.writeFileSync(path.join(directory, "version.stdout"), result.stdout ?? Buffer.alloc(0), {
  flag: "wx",
});
fs.writeFileSync(path.join(directory, "version.stderr"), result.stderr ?? Buffer.alloc(0), {
  flag: "wx",
});
save("version-result.json", {
  program,
  sha256,
  args: ["--version"],
  status: result.status,
  signal: result.signal,
  pid: result.pid ?? null,
  error: result.error ? String(result.error) : null,
  companionOnly: true,
});
assert.equal(result.error, undefined);
assert.equal(result.status, 0);
assert.equal(result.signal, null);
assert.equal(result.stdout.toString(), "Version 7.1.0-beta\n");
assert.equal(result.stderr.length, 0);
