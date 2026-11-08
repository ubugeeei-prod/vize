import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { packedSmokeTimingSuccessor } from "./native-smoke-timing-successor.ts";

const muslJob = "\n  fresh-install-musl-smoke:\n";
const selectedProducer = "\n  selected-host-native-producer:\n";
const currentMatrixStart = '        node-version: ["22", "24"]\n' + "        platform:\n";
const predecessorMatrixStart =
  '        node-version: ["22", "24"]\n' +
  "        # Linux musl targets are intentionally absent here: Ubuntu runners\n" +
  "        # provide GNU hosts, not native Alpine/musl hosts. Keep the\n" +
  "        # known-limitation policy in docs/content/stability.md in sync when\n" +
  "        # containerized musl tarball staging is added.\n" +
  "        platform:\n";

/** Admit the exact #11505 musl job and named release packages while retaining the earlier workflow. */
export function assertMuslFreshInstallSuccessor(historical: string, current: string): void {
  const predecessor = packedSmokeTimingSuccessor(historical);
  const relocations = [
    ["npm/linter/oxlint", "npm/oxlint"],
    ["npm/builder/babel", "npm/babel"],
  ] as const;
  const recovery = "      - uses: ./.github/actions/setup-wild-http-recovery\n";
  const upstream =
    "      - uses: wild-linker/action@0bbbfa5df4380cab8e63cb8505a1ce65e1d10203 # v0.9.0\n";
  assert.equal(current.split(recovery).length, 3, "two reviewed Wild recovery adapters");
  let sealed = current.replaceAll(recovery, upstream);
  for (const [relocated, historicalPath] of relocations) {
    assert.equal(sealed.split(relocated).length, 2, `one relocated ${relocated} package path`);
    sealed = sealed.replace(relocated, historicalPath);
  }
  for (const packagePath of ["npm/compose/router", "npm/compose/data"] as const) {
    assert.equal(sealed.split(packagePath).length, 2, `one admitted ${packagePath} package path`);
    sealed = sealed.replace(` ${packagePath}`, "");
  }
  const nodeCommand = "node tools/commands/release/npm/smoke-release-install.ts";
  const rustCommand = "rust-script tools/commands/release/npm/smoke-release-install.rs";
  const forcedRustCommand =
    "rust-script --force tools/commands/release/npm/smoke-release-install.rs";
  assert.equal(sealed.split(`${nodeCommand} --prepare-manifests npm/native`).length, 2);
  sealed = sealed.replace(
    `${nodeCommand} --prepare-manifests npm/native`,
    `${rustCommand} --prepare-manifests npm/native`,
  );
  assert.equal(sealed.split(`${nodeCommand} --prepare-manifests --runtime-checks`).length, 3);
  sealed = sealed.replaceAll(
    `${nodeCommand} --prepare-manifests --runtime-checks`,
    `${forcedRustCommand} --prepare-manifests --runtime-checks`,
  );
  assert.equal(
    createHash("sha256").update(sealed).digest("hex"),
    "d38f2a343f8c0bda93ef60b9e4b2318697e4f12925b79857feb1bab5143bd3aa",
    "the complete musl smoke successor stays sealed outside the package relocations",
  );
  assert.equal(sealed.split(muslJob).length, 2, "one musl smoke job");
  assert.equal(sealed.split(selectedProducer).length, 2, "one selected producer boundary");
  const muslStart = sealed.indexOf(muslJob);
  const producerStart = sealed.indexOf(selectedProducer, muslStart + muslJob.length);
  assert.ok(producerStart > muslStart, "musl smoke precedes the selected producer");
  let reversed = sealed.slice(0, muslStart) + sealed.slice(producerStart);
  assert.equal(reversed.split(currentMatrixStart).length, 2, "one current native matrix");
  reversed = reversed.replace(currentMatrixStart, predecessorMatrixStart);
  assert.equal(reversed, predecessor, "only the exact musl successor may extend native smoke");
}
