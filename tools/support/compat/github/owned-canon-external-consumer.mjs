import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = fs.realpathSync(process.cwd());
const temporary = process.env.RUNNER_TEMP;
assert.ok(temporary, "use the existing hosted native qualification artifact root");
assert.equal(process.platform, "linux", "this consumer belongs to the Linux native qualification");
const capture = path.join(temporary, "native-source-recipe-owned-canon-consumer");
assert.equal(fs.existsSync(capture), false, "do not replace prior consumer evidence");
fs.mkdirSync(path.join(capture, "src"), { recursive: true });
const sha256 = (body) => createHash("sha256").update(body).digest("hex");
const sourcePath = "tests/_fixtures/ci/owned-canon-external-consumer/lib.rs";
const source = fs.readFileSync(path.join(root, sourcePath));
const manifest = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const profileMarker = "[profile.release]\n";
assert.equal(manifest.split(profileMarker).length, 2, "one whole source profile section");
const profiles = profileMarker + manifest.split(profileMarker)[1];
assert.ok(
  profiles.includes('[profile.ci-opt]\ninherits = "release"\nlto = "thin"\ncodegen-units = 16'),
);
const consumerManifest =
  '[package]\nname = "owned-canon-external-consumer"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n' +
  "[workspace]\n[dependencies]\nvize_canon = { path = " +
  JSON.stringify(path.join(root, "crates/vize_canon")) +
  " }\n\n" +
  profiles;
assert.equal(consumerManifest.includes("[patch"), false);
fs.writeFileSync(path.join(capture, "Cargo.toml"), consumerManifest);
fs.writeFileSync(path.join(capture, "src/lib.rs"), source);

function command(name, arguments_) {
  const result = spawnSync(name, arguments_, {
    cwd: capture,
    env: process.env,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 30 * 60 * 1000,
  });
  const label = name === "cargo" ? arguments_[0] : name;
  fs.writeFileSync(path.join(capture, `${label}.stdout`), result.stdout ?? "");
  fs.writeFileSync(path.join(capture, `${label}.stderr`), result.stderr ?? "");
  fs.writeFileSync(
    path.join(capture, `${label}.outcome.json`),
    JSON.stringify(
      { status: result.status, signal: result.signal, error: result.error?.message ?? null },
      null,
      2,
    ) + "\n",
  );
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

const sourceIdentity = spawnSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
  cwd: root,
  encoding: "utf8",
});
assert.equal(sourceIdentity.status, 0, sourceIdentity.stderr);
const sourceSha = sourceIdentity.stdout.trim();
assert.equal(sourceSha, process.env.SOURCE_SHA, "exact selected native source");
fs.writeFileSync(path.join(capture, "rustc.txt"), command("rustc", ["-Vv"]));
const metadata = JSON.parse(
  command("cargo", [
    "metadata",
    "--manifest-path",
    path.join(capture, "Cargo.toml"),
    "--format-version",
    "1",
  ]),
);
const canonical = metadata.packages.find((entry) => entry.name === "vize_canon");
assert.ok(canonical);
const node = metadata.resolve.nodes.find((entry) => entry.id === canonical.id);
assert.ok(node.features.includes("native"), "Canon's default native feature actually compiled");
const own = metadata.packages.filter((entry) => entry.name.startsWith("vize_corsa"));
assert.deepEqual(own.map((entry) => entry.name).sort(), [
  "vize_corsa",
  "vize_corsa_client",
  "vize_corsa_core",
  "vize_corsa_jsonrpc",
  "vize_corsa_lsp",
  "vize_corsa_orchestrator",
]);
assert.ok(own.every((entry) => entry.source === null));
for (const entry of own) {
  assert.equal(entry.version, canonical.version);
  assert.equal(
    fs.realpathSync(entry.manifest_path),
    path.join(root, "vendor", entry.name.replace(/^vize_/u, ""), "Cargo.toml"),
    "the actual owned dependency body belongs to this source",
  );
}
assert.equal(
  metadata.packages.some((entry) =>
    /^(?:corsa|corsa_core|corsa_client|corsa_jsonrpc|corsa_lsp|corsa_orchestrator)$/u.test(
      entry.name,
    ),
  ),
  false,
);
command("cargo", [
  "build",
  "--locked",
  "--manifest-path",
  path.join(capture, "Cargo.toml"),
  "--profile",
  "ci-opt",
  "--lib",
]);
fs.writeFileSync(
  path.join(capture, "receipt.json"),
  JSON.stringify(
    {
      sourceSha,
      authority: "independent-default-native-Canon-source-consumer",
      fixture: { path: sourcePath, bytes: source.length, sha256: sha256(source) },
      rootManifestSha256: sha256(manifest),
      consumerManifestSha256: sha256(consumerManifest),
      completeMetadataSha256: sha256(fs.readFileSync(path.join(capture, "metadata.stdout"))),
      ownPackages: own.map((entry) => ({
        name: entry.name,
        version: entry.version,
        id: entry.id,
        manifest: entry.manifest_path,
      })),
      defaultNativeCompiled: true,
      consumerPatches: false,
      publicRegistryAcceptance: false,
      nativeRuntimeExecution: false,
    },
    null,
    2,
  ) + "\n",
);
