import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
function argument(name) {
  const index = args.indexOf(name);
  assert.notEqual(index, -1, `missing ${name}`);
  assert.ok(args[index + 1], `missing value for ${name}`);
  return args[index + 1];
}
const artifacts = path.resolve(argument("--artifacts"));
const output = path.resolve(argument("--output"));
const nodeVersion = argument("--node");
const typescriptVersion = argument("--typescript");
assert.equal(process.versions.node, nodeVersion);
const source = JSON.parse(fs.readFileSync(path.join(artifacts, "source.json"), "utf8"));
assert.equal(source.sha, process.env.SOURCE_SHA);
const nativeSource = JSON.parse(fs.readFileSync(path.join(artifacts, "consumer.json"), "utf8"));
const nativePackage = path.join(artifacts, "native-package");
assert.equal(nativeSource.nativeImages.length, 1);
const packageNames = ["eslint-plugin-vize", "oxlint-plugin-vize"];
const tarballs = packageNames.map((name) =>
  path.join(artifacts, `${name}-${nativeSource.packageVersion}.tgz`),
);
for (const tarball of tarballs) assert.equal(fs.existsSync(tarball), true);
const consumer = path.join(output, "project with spaces 雪");
fs.mkdirSync(consumer, { recursive: true });
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify({ private: true, type: "module" }, null, 2) + "\n",
);
const commands = [];
function run(command, parameters, env = process.env) {
  const result = spawnSync(command, parameters, {
    cwd: consumer,
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  commands.push({
    command,
    parameters,
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    error: result.error?.message,
  });
  fs.writeFileSync(path.join(output, "commands.json"), JSON.stringify(commands, null, 2) + "\n");
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return result;
}
run("npm", [
  "install",
  "--ignore-scripts",
  "--omit=optional",
  "--strict-peer-deps",
  "--save-exact",
  "--no-audit",
  "--no-fund",
  ...tarballs,
  "eslint@10.13.0",
  `typescript@${typescriptVersion}`,
]);
fs.mkdirSync(path.join(consumer, "node_modules/@vizejs"), { recursive: true });
fs.symlinkSync(nativePackage, path.join(consumer, "node_modules/@vizejs/native"), "dir");
fs.copyFileSync(
  new URL("./packed-consumer.mjs", import.meta.url),
  path.join(consumer, "consumer.mjs"),
);
const result = run(process.execPath, ["consumer.mjs"], {
  ...process.env,
  VIZE_PREFER_WORKSPACE_BINDING: "1",
  VIZE_ESLINT_EXPECTED_NODE: nodeVersion,
  VIZE_ESLINT_EXPECTED_TYPESCRIPT: typescriptVersion,
});
const packet = JSON.parse(result.stdout);
fs.writeFileSync(path.join(output, "consumer.json"), JSON.stringify(packet, null, 2) + "\n");
assert.equal(packet.nativeBuildToken, nativeSource.nativeBuildToken);
assert.deepEqual(
  packet.nativeImages.map(({ sha256 }) => sha256),
  nativeSource.nativeImages.map(({ sha256 }) => sha256),
);
assert.equal(packet.nodeVersion, nodeVersion);
assert.equal(packet.typescriptVersion, typescriptVersion);
assert.equal(packet.eslintVersion, "10.13.0");
assert.equal(packet.plugin.version, nativeSource.packageVersion);
for (const provider of Object.values(packet.providers)) {
  assert.equal(path.relative(consumer, provider.entry).split(path.sep)[0], "node_modules");
  assert.equal(path.relative(consumer, provider.packagePath).split(path.sep)[0], "node_modules");
}
fs.writeFileSync(
  path.join(output, "acceptance.json"),
  JSON.stringify(
    {
      sourceSha: source.sha,
      nodeVersion,
      typescriptVersion,
      nativeBuildToken: packet.nativeBuildToken,
      nativeImageSha256: packet.nativeImages[0].sha256,
      sourceNativeImageSha256: nativeSource.nativeImages[0].sha256,
      tarballs: tarballs.map((file) => ({
        file: path.basename(file),
        sha256: createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
      })),
      providers: packet.providers,
      supportedScriptKinds: packet.packets.map(({ kind }) => kind),
    },
    null,
    2,
  ) + "\n",
);
