import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";

const [capture] = process.argv.slice(2);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const expected = {
  base: "30ef8eb44fefa108dbdbcaadeef5d514c882d003",
  current: "78719553cf550bcffbf1d0fb891fda642f42de3a",
};
const rows = [];
for (const [role, sourceSha] of Object.entries(expected)) {
  const directory = join(capture, role);
  const names = fs.readdirSync(directory);
  const observations = names
    .filter((name) => /^observer\.\d+\.json$/.test(name))
    .map((name) => JSON.parse(fs.readFileSync(join(directory, name), "utf8")));
  const active = observations.filter((observation) => observation.rows.length > 0);
  assert.equal(active.length, 1, "Exactly one filtered test child must observe this role");
  const observation = active[0];
  assert.equal(observation.sourceSha, sourceSha);
  assert.equal(observation.role, role);
  assert.equal(observation.node, "v24.14.0");
  assert.deepEqual(observation.observationErrors, []);
  const status = Number(fs.readFileSync(join(directory, "exit-status.txt"), "utf8").trim());
  const stdout = fs.readFileSync(join(directory, "stdout.txt"), "utf8");
  assert.match(stdout, /current fail-closed Rust collector qualifies finite routes while retaining complete original kernel paths and bytes/);
  const traceNames = names.filter((name) => /^syscalls\.\d+$/.test(name));
  assert.ok(traceNames.length > 0, "Missing actual syscall trace");
  const traces = traceNames.map((name) => fs.readFileSync(join(directory, name), "utf8"));
  const refusals = observation.rows.filter((row) => row.error?.code === "ELOOP");
  assert.ok(refusals.length > 0, "No kernel boundary observed");
  const evidence = refusals.map((row) => {
    const suffix = row.path.split("/jellyfin-vue/")[1];
    const roots = suffix.match(/packaging\/deb\/root/g)?.length ?? 0;
    const copy = suffix.endsWith("/copy");
    const matchingTrace = traces.some((trace) =>
      trace.split("\n").some((line) => line.includes(row.path) && /ELOOP/.test(line)),
    );
    assert.ok(matchingTrace, "JS refusal lacks identical-path syscall ELOOP evidence");
    return { ...row, completeFixtureFollows: roots + Number(copy) };
  });
  const expectedDiagnostic = /Committed kernel boundary paths changed/.test(stdout);
  assert.ok(status === 0 || (status === 1 && expectedDiagnostic), "Unexpected test failure; preserve it as unqualified evidence");
  rows.push({
    role,
    sourceSha,
    status,
    classification: status === 0 ? "not-reproduced-in-this-one-observation" : "strict-boundary-refusal-reproduced",
    observationPid: observation.pid,
    observations: observation.rows.length,
    refusals: evidence,
    stdoutSha256: sha256(stdout),
    traceFiles: traceNames,
  });
}
fs.writeFileSync(
  join(capture, "classification.json"),
  JSON.stringify({
    diagnosticOnly: true,
    historicalKernelCause: "unknown: the original failed historical recapture was not retained",
    noSourceOrOracleChange: true,
    noRetry: true,
    rows,
  }, null, 2) + "\n",
);
console.log(JSON.stringify(rows.map(({ role, sourceSha, status, classification, observations, refusals }) => ({
  role, sourceSha, status, classification, observations,
  refusals: refusals.map(({ path, completeFixtureFollows, error }) => ({ path, completeFixtureFollows, error })),
}))));
