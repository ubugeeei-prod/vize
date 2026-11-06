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
  assert.equal(
    createHash("sha256").update(current).digest("hex"),
    "168b9494f95ef11a9ac310e248740ef8b22ef867b757c595ff5ad8e14712385d",
    "the complete musl smoke successor stays sealed",
  );
  assert.equal(current.split(muslJob).length, 2, "one musl smoke job");
  assert.equal(current.split(selectedProducer).length, 2, "one selected producer boundary");
  const muslStart = current.indexOf(muslJob);
  const producerStart = current.indexOf(selectedProducer, muslStart + muslJob.length);
  assert.ok(producerStart > muslStart, "musl smoke precedes the selected producer");
  let reversed = current.slice(0, muslStart) + current.slice(producerStart);
  assert.equal(reversed.split(currentMatrixStart).length, 2, "one current native matrix");
  reversed = reversed.replace(currentMatrixStart, predecessorMatrixStart);
  assert.equal(reversed, predecessor, "only the exact musl successor may extend native smoke");
}
