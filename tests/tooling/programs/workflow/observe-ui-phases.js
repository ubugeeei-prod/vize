import fs from "node:fs";
import assert from "node:assert/strict";

const fault =
  await import("../../../../.github/actions/test-js-packages-with-history/native-fault-evidence.ts");
const retention =
  await import("../../../../.github/actions/test-js-packages-with-history/native-fault-retention.ts");
fault.observers.privateGuest = () => ({ privateFirecrackerGuest: false });
fault.observers.sourceSnapshot = () => ({ synthetic: true });
let sharedSeen,
  sharedProviders,
  sequence = 0;
fault.observers.observeDescendants = (pid, seen, providers, output, _proc, phase) => {
  if (sharedSeen) {
    assert.equal(seen, sharedSeen);
    assert.equal(providers, sharedProviders);
  }
  sharedSeen = seen;
  sharedProviders = providers;
  const attempt = retention.beginProcessObservation(seen, phase);
  for (let index = 0; index < 80; index++) {
    const next = sequence++;
    retention.retainProcessPacket(
      seen,
      String(next),
      { pid, startTicks: "123", observedAt: next, tasks: [{ tid: next }] },
      output,
      phase,
    );
  }
  retention.finishProcessObservation(attempt);
};
const mode = process.argv[2];
if (mode === "missing-command") fs.unlinkSync(process.env.PATH + "/vp");
if (mode === "non-executable-command") fs.chmodSync(process.env.PATH + "/vp", 0o644);
if (mode === "occupied-directory")
  fs.mkdirSync(process.env.RUNNER_TEMP + "/js-native-fault-evidence-phases-1-ui");
if (mode === "unwritable-outcome") {
  const write = fs.writeFileSync;
  fs.writeFileSync = (filename, ...args) => {
    if (String(filename).endsWith("/outcome.json"))
      throw new Error("synthetic evidence I/O failure");
    return write(filename, ...args);
  };
}
process.exitCode = await fault.main(["--ui-conformance"], "linux");
