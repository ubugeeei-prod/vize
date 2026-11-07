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

/** Admit the exact #11505 musl job while retaining the complete earlier workflow. */
export function assertMuslFreshInstallSuccessor(historical: string, current: string): void {
  const predecessor = packedSmokeTimingSuccessor(historical);
  const relocations = [
    ["npm/linter/oxlint", "npm/oxlint"],
    ["npm/builder/babel", "npm/babel"],
  ] as const;
  let sealed = current;
  for (const [relocated, historicalPath] of relocations) {
    assert.equal(sealed.split(relocated).length, 2, `one relocated ${relocated} package path`);
    sealed = sealed.replace(relocated, historicalPath);
  }
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
