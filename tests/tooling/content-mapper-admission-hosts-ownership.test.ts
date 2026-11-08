import assert from "node:assert/strict";
import test from "node:test";

void test("only exact pre-spawn protected identities can be excluded from ownership errors", async () => {
  const { observationDisposition, requiresOwnerMarker } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
  const identity = {
    pid: 42,
    ppid: 1,
    group: 1,
    startTicks: "100",
    state: "S",
    ownerMarker: false,
  };
  const baseline = new Set(["42:100"]),
    owned = new Set<string>();
  assert.equal(requiresOwnerMarker(identity, owned), true);
  assert.equal(requiresOwnerMarker(identity, new Set(["42:100"])), false);
  assert.equal(
    requiresOwnerMarker({ ...identity, startTicks: "101" }, new Set(["42:100"])),
    true,
    "PID reuse must require a fresh owner marker",
  );
  assert.deepEqual(observationDisposition(identity, baseline, owned, "EACCES"), {
    scope: "preexisting-unrelated",
    excluded: true,
  });
  assert.equal(
    observationDisposition({ ...identity, startTicks: "101" }, baseline, owned, "EACCES").excluded,
    false,
    "PID reuse must not inherit baseline authority",
  );
  assert.equal(
    observationDisposition({ ...identity, pid: 43 }, baseline, owned, "EACCES").excluded,
    false,
    "post-spawn identity is unknown",
  );
  assert.deepEqual(observationDisposition(identity, baseline, new Set(["42:100"]), "EACCES"), {
    scope: "known-owned",
    excluded: false,
  });
  assert.deepEqual(
    observationDisposition({ ...identity, ownerMarker: true }, baseline, owned, "EACCES"),
    { scope: "known-owned", excluded: false },
  );
  assert.equal(
    observationDisposition(undefined, baseline, owned, "EACCES").excluded,
    false,
    "missing kernel identity cannot establish preexistence",
  );
  assert.equal(
    observationDisposition(identity, baseline, owned, "EIO").excluded,
    false,
    "non-permission observation failures remain fatal",
  );
});

void test("terminal permission races require matching kernel identities and an ended owned process", async () => {
  const { observationDisposition } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
  const before = { pid: 42, ppid: 1, group: 1, startTicks: "100", state: "S", ownerMarker: true };
  const baseline = new Set<string>(),
    owned = new Set(["42:100"]);
  for (const state of ["Z", "X"]) {
    const after = { ...before, state };
    assert.deepEqual(observationDisposition(after, baseline, owned, "EACCES", before, after), {
      scope: "known-terminal",
      excluded: true,
    });
    assert.equal(
      observationDisposition(after, baseline, owned, "EACCES").excluded,
      false,
      "terminal state alone is insufficient",
    );
    assert.equal(
      observationDisposition(
        after,
        baseline,
        owned,
        "EACCES",
        { ...before, startTicks: "99" },
        after,
      ).excluded,
      false,
      "PID reuse cannot inherit an old termination",
    );
    assert.equal(
      observationDisposition(after, baseline, owned, "EACCES", before, { ...after, pid: 43 })
        .excluded,
      false,
      "kernel PID must match on both sides",
    );
    assert.equal(
      observationDisposition(after, baseline, owned, "EIO", before, after).excluded,
      false,
      "non-permission failures stay fatal",
    );
    assert.equal(
      observationDisposition(
        { ...after, ownerMarker: false },
        baseline,
        new Set(),
        "EACCES",
        { pid: before.pid, startTicks: before.startTicks, state: before.state },
        after,
      ).excluded,
      false,
      "unknown ownership is not excused",
    );
  }
  for (const state of ["R", "S"]) {
    const after = { ...before, state };
    assert.deepEqual(observationDisposition(after, baseline, owned, "EACCES", before, after), {
      scope: "known-owned",
      excluded: false,
    });
  }
});
