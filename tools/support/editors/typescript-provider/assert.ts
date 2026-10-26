import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  checkBuildReceipt,
  compareNames,
  digest,
  providerFiles,
  providerPrefix,
  providerSchema,
  readJson,
  targets,
  type BuildReceipt,
  type ProviderReceipt,
} from "./artifacts.ts";

export const zipBytes = (vsix: string, name: string, maximum = 100_000_000): Buffer =>
  execFileSync(
    "unzip",
    [
      "-p",
      vsix,
      Array.from(name, (character) =>
        "[]*?\\".includes(character) ? `\\${character}` : character,
      ).join(""),
    ],
    { maxBuffer: maximum },
  );
export function assertPlatformVsix(
  root: string,
  vsix: string,
  expected?: ProviderReceipt,
): ProviderReceipt {
  const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const version = readJson(path.join(root, "npm/typescript-plugin/package.json")).version;
  assert(
    fs.statSync(vsix).size <= 50_000_000,
    "Platform VSIX exceeds 50MB (measured released add-ons compress to 24–28MB)",
  );
  const entries = execFileSync("unzip", ["-Z1", vsix], { encoding: "utf8" }).trim().split(/\r?\n/);
  assert.equal(new Set(entries).size, entries.length, "VSIX contains duplicate archive entries");
  for (const entry of entries)
    assert(
      !entry.startsWith("/") && !entry.includes("\\") && !entry.split("/").includes(".."),
      `Unsafe archive path: ${entry}`,
    );
  const receipt = JSON.parse(
    zipBytes(vsix, `${providerPrefix}provider.json`, 1_000_000).toString(),
  ) as ProviderReceipt;
  assert.equal(receipt.schema, providerSchema);
  assert.equal(receipt.source, source, "VSIX provider source differs from this checkout");
  assert.equal(receipt.releaseVersion, version);
  assert(Object.hasOwn(targets, receipt.rustTarget));
  assert.equal(receipt.targetPlatform, targets[receipt.rustTarget].platform);
  assert.equal(receipt.nodeFile, targets[receipt.rustTarget].node);
  if (expected)
    assert.deepEqual(receipt, expected, "Packaged receipt differs from the validated producer");
  const names = [...providerFiles, receipt.nodeFile, "build.json"].sort(compareNames);
  assert.deepEqual(
    receipt.files.map((file) => file.path).sort(compareNames),
    names,
    "Provider file inventory differs from its canonical closure",
  );
  assert.deepEqual(
    entries.filter((entry) => entry.startsWith(providerPrefix)).sort(compareNames),
    [...names, "provider.json"].map((name) => `${providerPrefix}${name}`).sort(compareNames),
  );
  for (const file of receipt.files) {
    const bytes = zipBytes(vsix, `${providerPrefix}${file.path}`);
    assert.equal(bytes.length, file.bytes, `VSIX ${file.path} byte length mismatch`);
    assert.equal(digest(bytes), file.sha256, `VSIX ${file.path} hash mismatch`);
    if (providerFiles.includes(file.path as (typeof providerFiles)[number]))
      assert.deepEqual(
        bytes,
        fs.readFileSync(path.join(root, "npm/native", file.path)),
        `VSIX ${file.path} differs from canonical source`,
      );
  }
  const image = zipBytes(vsix, `${providerPrefix}${receipt.nodeFile}`);
  const build = JSON.parse(
    zipBytes(vsix, `${providerPrefix}build.json`, 1_000_000).toString(),
  ) as BuildReceipt;
  checkBuildReceipt(build, image, source, version, receipt.rustTarget);
  assert.equal(receipt.nativeBuildToken, build.nativeBuildToken);
  const manifest = zipBytes(vsix, "extension.vsixmanifest", 1_000_000).toString();
  assert(
    manifest.includes(`TargetPlatform="${receipt.targetPlatform}"`),
    "VSIX lacks its deterministic target platform",
  );
  assert(
    zipBytes(vsix, "[Content_Types].xml", 1_000_000).toString().includes('Extension=".node"'),
    "VSIX does not declare native image content types",
  );
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-platform-vsix-check-"));
  try {
    const base = path.join(temporary, "universal.vsix");
    fs.copyFileSync(vsix, base);
    execFileSync("zip", [
      "-q",
      "-d",
      base,
      ...entries.filter((entry) => entry.startsWith(providerPrefix)),
    ]);
    // The source-only extension still obeys the existing exact allowlist and
    // 5MB ceiling. The larger add-on has a separate bounded policy above.
    execFileSync(
      "rust-script",
      [path.join(root, "tools/commands/editors/vscode/assert-vsix-package.rs"), base],
      { cwd: root, stdio: "inherit" },
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
  return receipt;
}
