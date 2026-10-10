// Integration-test identity: no example-spec substitution and no binary probe.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { hash7502 } from "./native-attribute-values-7502-inputs.ts";

export const root7502 = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)));
export const testName7502 = "native_attribute_values_7502";
export const sourcePath7502 = `crates/vize_atelier_sfc/tests/${testName7502}.rs`;
const scopes = [
  "Cargo.lock",
  "Cargo.toml",
  ".cargo",
  "rust-toolchain",
  "rust-toolchain.toml",
  "crates",
  "davinci",
  "vendor",
  "tests",
  "tools",
  ".github",
  "package.json",
  "npm/ui/package.json",
  "npm/plugin-sdk",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
];
function git(args: string[]) {
  const result = spawnSync("git", ["--no-replace-objects", ...args], {
    cwd: root7502,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trimEnd();
}
export function source7502() {
  assert.equal(
    git(["diff", "--name-only", "HEAD", "--", ...scopes]),
    "",
    "committed capture inputs required",
  );
  assert.equal(
    git(["ls-files", "--others", "--exclude-standard", "--", ...scopes]),
    "",
    "untracked source inputs cannot enter an observation",
  );
  assert.equal(git(["ls-files", "--error-unmatch", "--", sourcePath7502]), sourcePath7502);
  const input = "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502";
  const sha = (file: string) => hash7502(readFileSync(path.join(root7502, file)));
  return {
    sourceRevision: git(["rev-parse", "HEAD"]),
    sourceTree: git(["rev-parse", "HEAD^{tree}"]),
    productSourceTree: git(["rev-parse", "HEAD:crates/vize_atelier_sfc/src"]),
    observerSourceBlob: git(["rev-parse", `HEAD:${sourcePath7502}`]),
    fixtureSourceTree: git(["rev-parse", `HEAD:${input}`]),
    sourcePath: sourcePath7502,
    cargoLockSha256: sha("Cargo.lock"),
    observerSourceSha256: sha(sourcePath7502),
    inputPackSha256: sha(`${input}/original_inputs.json`),
    originalReviewedOutputSha256: sha(`${input}/reviewed_output.json`),
    successorReviewedOutputSha256: sha(`${input}/reviewed_output_v2.json`),
    runtimePackageSha256: sha("npm/ui/package.json"),
    runtimeLockSha256: sha("pnpm-lock.yaml"),
  };
}
export function workspacePackage7502(packageId: string) {
  assert(packageId.startsWith("path+file:"), "actual workspace package is mandatory");
  const url = new URL(packageId.slice(5));
  assert.match(url.hash, /^#(?:vize_atelier_sfc@)?\d+\.\d+\.\d+(?:[-+][\w.+-]+)?$/);
  url.hash = "";
  assert.equal(
    realpathSync(fileURLToPath(url)),
    realpathSync(path.join(root7502, "crates/vize_atelier_sfc")),
  );
}
