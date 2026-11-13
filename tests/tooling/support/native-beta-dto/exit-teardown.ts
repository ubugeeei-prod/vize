import assert from "node:assert/strict";
import type { nativeOwnerTerminalState } from "./native-io.ts";
import { micros } from "./stable-watcher-timing.ts";

type TerminalState = Pick<
  ReturnType<typeof nativeOwnerTerminalState>,
  "members" | "threadExits" | "exitGroups" | "unfinishedOperations"
>;

/**
 * Once a sibling enters exit_group, the kernel can stop another thread on its
 * way into a syscall before strace decodes the number; strace then leaves a
 * `???(` entry unfinished until that thread's exit. This is teardown, not I/O:
 * it carries no fd or bytes, so it is admitted only inside that window.
 */
export function requireExitTeardownUnfinished(owner: TerminalState) {
  if (owner.unfinishedOperations.length === 0) return;
  assert.equal(owner.exitGroups.length, 1, "teardown requires one exit_group entry");
  const group = owner.exitGroups[0]!;
  const members = new Set(owner.members.map((row) => row.tid));
  for (const row of owner.unfinishedOperations) {
    assert.ok(members.has(row.tid), "unfinished operation belongs to the owner");
    assert.equal(row.file, `syscalls.${row.tid}`);
    assert.equal(
      row.raw,
      `${row.at} ???( `,
      "only an undecoded syscall entry may remain unfinished",
    );
    const exits = owner.threadExits.filter((exit) => exit.tid === row.tid);
    assert.equal(exits.length, 1, "teardown entry requires its thread exit");
    assert.ok(
      micros(group.at) < micros(row.at) && micros(row.at) < micros(exits[0]!.at),
      "undecoded entry lies between exit_group and its thread exit",
    );
  }
}
