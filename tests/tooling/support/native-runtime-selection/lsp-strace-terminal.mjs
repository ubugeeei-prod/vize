import assert from "node:assert/strict";

export function classifyTerminalSyscalls({
  owners,
  unfinished,
  exitEntries,
  readEntries,
  parents,
  leader,
  clock,
}) {
  const byPid = new Map(owners.map((owner) => [owner.process, owner]));
  const certify = (unknown) => {
    const process = leader(unknown.tid),
      owner = byPid.get(process);
    assert.ok(owner, "terminal unknown syscall belongs to an actual executed image group");
    const exits = owner.rows.filter((row) => /^exit_group\(\d+\)\s+=\s+\?(?:\s|$)/u.test(row.call));
    assert.equal(exits.length, 1, "one explicit exit_group owns terminal interruption");
    const exit = exits[0],
      code = Number(/^exit_group\((\d+)\)/u.exec(exit.call)[1]);
    assert.ok(
      BigInt(exit.interval.endNanoseconds) < clock(unknown.at).nanoseconds,
      "unknown interval must be strictly after the same-group explicit exit_group",
    );
    const members = new Set([
      process,
      ...[...parents.keys()].filter((tid) => leader(tid) === process),
    ]);
    for (const tid of members) {
      const terminals = owner.rows.filter(
        (row) => row.tid === tid && /^\+\+\+ (?:exited|killed)/u.test(row.call),
      );
      assert.equal(
        terminals.length,
        1,
        "every observed same-group thread has its own terminal proof",
      );
      assert.equal(
        terminals[0].call,
        `+++ exited with ${code} +++`,
        "same-code group terminal proof",
      );
    }
    const later = owner.rows.filter(
      (row) =>
        row.tid === unknown.tid && clock(row.at).nanoseconds >= clock(unknown.at).nanoseconds,
    );
    assert.equal(later.length, 1, "no hidden resumed/other syscall after terminal unknown");
    assert.equal(later[0].call, `+++ exited with ${code} +++`);
    return { owner, process, exit, code, members };
  };
  for (const unknown of unfinished) {
    const { owner, process, exit, members } = certify(unknown);
    (owner.terminalUnknownSyscalls ??= []).push({
      ...unknown,
      process,
      classification: "UNKNOWN_UNCAPTURED_TERMINAL_SYSCALL",
      exitGroup: exit,
      sameCodeThreadTerminals: [...members],
      syscallBytesCaptured: false,
    });
  }
  for (const entry of exitEntries) {
    const { owner, process, exit, code, members } = certify(entry);
    assert.equal(
      Number(/^exit_group\((\d+)\s*$/u.exec(entry.call)[1]),
      code,
      "nonreturning entry retains the proved same-group exit code",
    );
    (owner.terminalExitEntries ??= []).push({
      ...entry,
      process,
      classification: "UNKNOWN_UNCAPTURED_NONRETURNING_EXIT_ENTRY",
      exitGroup: exit,
      sameCodeThreadTerminals: [...members],
      syscallBytesCaptured: false,
    });
  }
  for (const entry of readEntries) {
    const { owner, process, exit, code, members } = certify(entry);
    assert.equal(code, 0, "terminal read observation requires the observed exit_group(0)");
    // The trace reports this entry after exit_group; it does not establish
    // when the kernel read began, returned bytes, or a zero return count.
    (owner.terminalReadEntries ??= []).push({
      ...entry,
      process,
      classification: "UNKNOWN_UNCAPTURED_TERMINAL_READ",
      exitGroup: exit,
      sameCodeThreadTerminals: [...members],
      syscallBytesCaptured: false,
      syscallReturnCountCaptured: false,
      actualKernelReadStartKnown: false,
    });
  }
  return {
    syscallsComplete:
      unfinished.length === 0 && exitEntries.length === 0 && readEntries.length === 0,
    terminalUnknownSyscalls: owners.flatMap((owner) => owner.terminalUnknownSyscalls ?? []),
    terminalExitEntries: owners.flatMap((owner) => owner.terminalExitEntries ?? []),
    terminalReadEntries: owners.flatMap((owner) => owner.terminalReadEntries ?? []),
  };
}
