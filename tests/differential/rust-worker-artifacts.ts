import assert from "node:assert/strict";

export type RustWorkerRun = { runId: string; runAttempt: number };

export type RustWorkerArtifact = { directory: string; shard: number; attempt: number };

// Download-artifact scopes these immutable names to the current workflow run.
// Select before reading packets: a broken newer packet cannot fall back to an
// older success. The report's unconditional current-needs gate remains required.
export function selectRustWorkerArtifacts(directories: string[], run: RustWorkerRun) {
  assert.match(run.runId, /^[1-9]\d*$/, "a trusted current run ID is required");
  assert(
    Number.isSafeInteger(run.runAttempt) && run.runAttempt > 0,
    "a trusted current attempt is required",
  );
  const latest = new Map<number, RustWorkerArtifact>();
  const identities = new Set<string>();
  for (const directory of directories) {
    assert.equal(typeof directory, "string", "artifact directory identity must be a string");
    const name = /^rust-test-shard-([1-4])-([1-9]\d*)-([1-9]\d*)$/.exec(directory);
    assert(name, `unexpected Rust worker artifact directory: ${directory}`);
    assert.equal(name[2], run.runId, "Rust worker belongs to another run");
    const shard = Number(name[1]);
    const attempt = Number(name[3]);
    assert(
      Number.isSafeInteger(attempt) && attempt <= run.runAttempt,
      "Rust worker belongs to a future or invalid attempt",
    );
    const identity = `${shard}:${attempt}`;
    assert(!identities.has(identity), "duplicate Rust worker artifact identity");
    identities.add(identity);
    if (attempt > (latest.get(shard)?.attempt ?? 0)) {
      latest.set(shard, { directory, shard, attempt });
    }
  }
  assert.equal(latest.size, 4, "all four complete worker artifacts are required");
  return [1, 2, 3, 4].map((shard) => {
    const artifact = latest.get(shard);
    assert(artifact);
    return artifact;
  });
}
