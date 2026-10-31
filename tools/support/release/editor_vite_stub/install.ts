import assert from "node:assert/strict";
import { copyFileSync, existsSync, mkdirSync, readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
  type JsonRecord,
} from "../../json-value.ts";
import { npmArgv, Recorder, sha256, success } from "../../config/windows_consumer/common.ts";
import { writeFixture } from "./filesystem.ts";

export function install(
  root: string,
  tools: string,
  version: string,
  authority: JsonRecord,
  record: Recorder,
) {
  mkdirSync(root, { recursive: false });
  mkdirSync(tools, { recursive: false });
  // Both originals are checked-in reported inputs; no reconstruction from expected output.
  const fixture = join(
    process.cwd(),
    "tests/_fixtures/differential/typechecker/editor-vite-stub-11145/project.json",
  );
  const original = readJsonRecord(fixture);
  assert.equal(original.issue, 11145);
  assert.equal(
    sha256(readFileSync(fixture)),
    "8e13ee81f8a0086e34e44f2ec49f7dc6ecde19d55694a75bf3415b212d120f5a",
  );
  writeFixture(root, original.files, (name) => !name.startsWith("node_modules/"));
  record.json("original-fixture.json", original);
  const bootstrap = record.run(
    "install-pnpm",
    npmArgv(
      "install",
      "--prefix",
      tools,
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--save-exact",
      "pnpm@12.1.0",
    ),
    tools,
  );
  success(bootstrap);
  const pnpm = [process.execPath, join(tools, "node_modules/pnpm/bin/pnpm.cjs")];
  const pmVersion = record.run("pnpm-version", [...pnpm, "--version"], root);
  success(pmVersion);
  assert.equal(pmVersion.stdout.trim(), "12.1.0");
  success(record.run("original-git-init", ["git", "init", "-q"], root));
  success(
    record.run(
      "public-install",
      [
        ...pnpm,
        "add",
        "-D",
        "--save-exact",
        "vize@" + version,
        "@vizejs/vite-plugin@" + version,
        "vite-plus@0.3.0",
        "vue@3.5.43",
        "typescript@6.0.3",
      ],
      root,
      process.env,
      600_000,
    ),
  );
  copyFileSync(join(root, "pnpm-lock.yaml"), join(record.root, "installed-pnpm-lock.yaml"));
  copyFileSync(join(root, "package.json"), join(record.root, "installed-package.json"));
  assert.equal(
    existsSync(join(root, "node_modules/vite")),
    false,
    "original isolated topology is required; never remove an installed runtime Vite to manufacture it",
  );
  const output = join(record.root, "installed-identity.json");
  success(
    record.run(
      "installed-identity",
      [
        process.execPath,
        fileURLToPath(new URL("./identity-probe.ts", import.meta.url)),
        root,
        output,
      ],
      root,
    ),
  );
  const identity = readJsonRecord(output);
  assert.equal(identity.node, "v26.6.0");
  for (const key of ["cliVersion", "nativeVersion", "platformVersion", "pluginVersion"])
    assert.equal(identity[key], version, key);
  assert.deepEqual(identity.hostVersions, {
    "vite-plus": "0.3.0",
    vue: "3.5.43",
    typescript: "6.0.3",
  });
  const binary = asRecord(identity.binary),
    native = asRecord(identity.identity);
  let matched = 0;
  for (const item of asArray(authority.receipts).map((value) => asRecord(value))) {
    const origin = asRecord(item.origin);
    assert.ok(asNumber(origin.runId) > 0 && asNumber(origin.artifactId) > 0);
    assert.ok(asString(origin.member));
    const raw = Buffer.from(asString(item.rawBase64), "base64");
    assert.equal(sha256(raw), item.sha256);
    const receipt = asRecord(parseJson(raw.toString("utf8")));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, version);
    assert.equal(receipt.source, authority.tagSourceSha);
    const provider = asRecord(receipt.provider);
    if (provider.targetPlatform !== identity.targetPlatform) continue;
    assert.equal(provider.source, authority.tagSourceSha);
    assert.equal(provider.releaseVersion, version);
    const files = asArray(provider.files)
      .map((value) => asRecord(value))
      .filter((value) => value.path === provider.nodeFile);
    assert.equal(files.length, 1);
    assert.equal(binary.sha256, files[0].sha256);
    assert.equal(binary.bytes, files[0].bytes);
    assert.equal(native.token, provider.nativeBuildToken);
    matched++;
  }
  assert.equal(matched, 1, "one genuine producer image must bind this host");
  const lock = readFileSync(join(root, "pnpm-lock.yaml"), "utf8");
  for (const name of [
    "vize",
    "@vizejs/vite-plugin",
    "@vizejs/native",
    asString(identity.platformPackage),
  ]) {
    const expected = asRecord(asRecord(authority.npmSRI)[name]);
    const origin = new URL(asString(expected.metadataOrigin));
    assert.equal(origin.protocol, "https:");
    assert.equal(origin.hostname, "registry.npmjs.org");
    const response = record.run(
      "registry-" + name.replaceAll(/[@/]/g, "-"),
      npmArgv(
        "view",
        name + "@" + version,
        "dist",
        "--json",
        "--registry=https://registry.npmjs.org",
      ),
      root,
    );
    success(response);
    const dist = asRecord(parseJson(response.stdout));
    assert.equal(dist.integrity, expected.integrity);
    assert.equal(dist.tarball, expected.tarball);
    assert.ok(lock.includes(asString(expected.integrity)), "actual public lock SRI: " + name);
  }
  for (const [name, pinned] of Object.entries(asRecord(identity.hostVersions))) {
    const response = record.run(
      "host-registry-" + name,
      npmArgv(
        "view",
        name + "@" + asString(pinned),
        "dist",
        "--json",
        "--registry=https://registry.npmjs.org",
      ),
      root,
    );
    success(response);
    const dist = asRecord(parseJson(response.stdout));
    assert.ok(lock.includes(asString(dist.integrity)), "actual pinned host SRI: " + name);
    assert.equal(new URL(asString(dist.tarball)).hostname, "registry.npmjs.org");
  }
  assert.equal(realpathSync(root), root);
  return { pnpm, identity, original };
}
