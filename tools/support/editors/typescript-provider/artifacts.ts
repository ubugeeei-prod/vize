import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const targets: Record<string, { platform: string; node: string }> = {
  "x86_64-apple-darwin": { platform: "darwin-x64", node: "vize-vitrine.darwin-x64.node" },
  "aarch64-apple-darwin": { platform: "darwin-arm64", node: "vize-vitrine.darwin-arm64.node" },
  "x86_64-pc-windows-msvc": { platform: "win32-x64", node: "vize-vitrine.win32-x64-msvc.node" },
  "aarch64-pc-windows-msvc": {
    platform: "win32-arm64",
    node: "vize-vitrine.win32-arm64-msvc.node",
  },
  "x86_64-unknown-linux-gnu": { platform: "linux-x64", node: "vize-vitrine.linux-x64-gnu.node" },
  "aarch64-unknown-linux-gnu": {
    platform: "linux-arm64",
    node: "vize-vitrine.linux-arm64-gnu.node",
  },
  "x86_64-unknown-linux-musl": { platform: "alpine-x64", node: "vize-vitrine.linux-x64-musl.node" },
  "aarch64-unknown-linux-musl": {
    platform: "alpine-arm64",
    node: "vize-vitrine.linux-arm64-musl.node",
  },
};
export const providerFiles = [
  "package.json",
  "index.js",
  "index.d.ts",
  "native-binding.js",
  "native-binding-identity.js",
  "native-targets.js",
  "language-projection.js",
  "language-projection.d.ts",
] as const;
export const providerPrefix = "extension/native/typescript-projection/";
export const providerSchema = "vize.vscodeNativeProvider.v1";
export interface BuildReceipt {
  schema: "vize.nativeBuildReceipt.v1";
  source: string;
  releaseVersion: string;
  rustTarget: string;
  nodeFile: string;
  bytes: number;
  sha256: string;
  nativeBuildToken: string;
}
export interface ProviderReceipt {
  schema: typeof providerSchema;
  source: string;
  releaseVersion: string;
  rustTarget: string;
  targetPlatform: string;
  nativeBuildToken: string;
  nodeFile: string;
  files: Array<{ path: string; bytes: number; sha256: string }>;
}
export const compareNames = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);
export const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export const readJson = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
export function assertPackageOutput(root: string, output: string): void {
  const relative = path.relative(path.join(root, "editors/vscode"), path.resolve(output));
  assert(
    relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative),
    "Write platform VSIX outputs outside editors/vscode: VSCE prepublish rebuilds dist, and prior target archives must not enter the next package",
  );
}
export function regularBytes(file: string, maximum = 100_000_000): Buffer {
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && !stat.isSymbolicLink(), `Expected a physical regular file: ${file}`);
  assert(stat.size <= maximum, `File exceeds its bounded size policy: ${file}`);
  return fs.readFileSync(file);
}
export function checkBuildReceipt(
  value: BuildReceipt,
  bytes: Buffer,
  source: string,
  version: string,
  target: string,
): void {
  assert(Object.hasOwn(targets, target), `Unknown native Rust target: ${target}`);
  assert.equal(value.schema, "vize.nativeBuildReceipt.v1", "Unknown native build receipt schema");
  assert.equal(value.source, source, "Native provider was not built from this exact checkout");
  assert.match(value.source, /^[0-9a-f]{40}$/);
  assert.equal(value.releaseVersion, version, "Native provider release differs from the adapter");
  assert.equal(value.rustTarget, target, "Native provider Rust target mismatch");
  assert.equal(value.nodeFile, targets[target].node, "Unexpected native image basename");
  assert.equal(value.bytes, bytes.length, "Native image byte length changed");
  assert.equal(value.sha256, digest(bytes), "Native image SHA256 changed");
  assert.match(value.nativeBuildToken, /^[0-9a-f]{32}$/);
  assert(
    bytes.includes(Buffer.from(value.nativeBuildToken)),
    "Native image does not contain its producer token",
  );
}
export function stageProvider(
  root: string,
  artifactDir: string,
  destination: string,
  source: string,
  target: string,
): ProviderReceipt {
  assert(Object.hasOwn(targets, target), `Unknown native Rust target: ${target}`);
  const knownArtifacts = new Set(
    Object.entries(targets).flatMap(([rust, spec]) => [spec.node, `build-receipt.${rust}.json`]),
  );
  for (const entry of fs.readdirSync(artifactDir))
    assert(knownArtifacts.has(entry), `Unexpected native artifact: ${entry}`);
  const version = readJson(path.join(root, "npm/typescript-plugin/package.json")).version;
  const nativeVersion = readJson(path.join(root, "npm/native/package.json")).version;
  assert.equal(version, nativeVersion, "Adapter and native package versions differ");
  assert.equal(
    readJson(path.join(root, "editors/vscode/package.json")).version,
    version,
    "Extension and adapter versions differ",
  );
  const build = readJson(path.join(artifactDir, `build-receipt.${target}.json`)) as BuildReceipt;
  const image = regularBytes(path.join(artifactDir, targets[target].node));
  checkBuildReceipt(build, image, source, version, target);
  const fileBytes: Array<readonly [string, Buffer]> = providerFiles.map((file) => [
    file,
    regularBytes(path.join(root, "npm/native", file), 2_000_000),
  ]);
  assert(
    fileBytes.reduce((total, [, bytes]) => total + bytes.length, 0) <= 2_000_000,
    "Native JavaScript wrapper closure exceeds 2MB",
  );
  fileBytes.push(
    [targets[target].node, image],
    ["build.json", Buffer.from(`${JSON.stringify(build, null, 2)}\n`)],
  );
  const receipt: ProviderReceipt = {
    schema: providerSchema,
    source,
    releaseVersion: version,
    rustTarget: target,
    targetPlatform: targets[target].platform,
    nativeBuildToken: build.nativeBuildToken,
    nodeFile: build.nodeFile,
    files: fileBytes.map(([file, bytes]) => ({
      path: file,
      bytes: bytes.length,
      sha256: digest(bytes),
    })),
  };
  fs.mkdirSync(destination, { recursive: true });
  for (const [file, bytes] of fileBytes) fs.writeFileSync(path.join(destination, file), bytes);
  fs.writeFileSync(
    path.join(destination, "provider.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  return receipt;
}
