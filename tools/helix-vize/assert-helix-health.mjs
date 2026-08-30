import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const helixBinary = path.resolve(process.argv[2] ?? "hx");
const serverPath = path.resolve(
  process.env.VIZE_SERVER_PATH ?? path.join(root, "target", "ci", "vize"),
);
const configHome = process.env.XDG_CONFIG_HOME;
const runtime = process.env.HELIX_RUNTIME;

assert.ok(fs.existsSync(helixBinary), `Helix binary does not exist: ${helixBinary}`);
assert.ok(fs.existsSync(serverPath), `Vize server does not exist: ${serverPath}`);
assert.ok(configHome, "XDG_CONFIG_HOME must point at the isolated Helix config");
assert.ok(runtime, "HELIX_RUNTIME must point at the pinned Helix runtime");
assert.ok(fs.existsSync(runtime), `Helix runtime does not exist: ${runtime}`);

const installedConfig = path.join(configHome, "helix", "languages.toml");
const packagedConfig = path.join(root, "editors", "helix", "languages.toml");
assert.equal(
  fs.readFileSync(installedConfig, "utf8"),
  fs.readFileSync(packagedConfig, "utf8"),
  "Helix must inspect the exact packaged languages.toml",
);

const expectedServer = `✓ vize: ${serverPath}`;
for (const language of ["vue", "art-vue"]) {
  const result = spawnSync(helixBinary, ["--health", language], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${path.dirname(serverPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    },
  });
  const output = stripAnsi(`${result.stdout ?? ""}${result.stderr ?? ""}`);
  assert.equal(result.status, 0, `hx --health ${language} failed:\n${output}`);

  const configuredServers = linesBetween(
    output,
    "Configured language servers:",
    "Configured debug adapter:",
  );
  assert.deepEqual(
    configuredServers,
    [expectedServer],
    `hx --health ${language} did not resolve exactly the packaged Vize server`,
  );
}

console.log("helix package health passed for vue and art-vue");

function stripAnsi(value) {
  const sequence = new RegExp(`${String.fromCodePoint(27)}\\[[0-?]*[ -/]*[@-~]`, "g");
  return value.replace(sequence, "");
}

function linesBetween(value, start, end) {
  const lines = value.split(/\r?\n/).map((line) => line.trim());
  const startAt = lines.indexOf(start);
  const endAt = lines.findIndex((line) => line.startsWith(end));
  assert.ok(startAt >= 0, `missing ${JSON.stringify(start)} in Helix health output`);
  assert.ok(endAt > startAt, `missing ${JSON.stringify(end)} in Helix health output`);
  return lines.slice(startAt + 1, endAt).filter(Boolean);
}
