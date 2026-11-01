import assert from "node:assert/strict";
import type { NativeEvent } from "./model.ts";
export function nativeActors(events: NativeEvent[], commandId: string): NativeEvent[] {
  const actors = new Map<number, { state: "ready" | "attempt" | "exit"; base: NativeEvent }>(),
    returns: NativeEvent[] = [];
  for (const event of events) {
    assert(
      ["initialized", "attempt", "returned", "failed", "exit"].includes(event.event),
      "unknown native event",
    );
    assert.equal(event.commandId, commandId);
    assert(Number.isSafeInteger(event.pid) && event.pid > 0);
    assert(Number.isSafeInteger(event.parentPid) && event.parentPid > 0);
    assert(Array.isArray(event.argv) && event.argv.every((v) => typeof v === "string"));
    if (event.event === "initialized") {
      assert(!actors.has(event.pid), "duplicate initialization");
      actors.set(event.pid, { state: "ready", base: event });
      continue;
    }
    const actor = actors.get(event.pid);
    assert(actor, "event before initialization");
    assert.notEqual(actor.state, "exit", "event after exit");
    for (const key of [
      "commandId",
      "parentPid",
      "argv",
      "nativePath",
      "sha256",
      "ancestry",
    ] as const)
      assert.deepEqual(event[key], actor.base[key]);
    if (event.event === "attempt") {
      assert.equal(actor.state, "ready");
      actor.state = "attempt";
    } else if (event.event === "returned") {
      assert.equal(actor.state, "attempt", "unpaired/duplicate return");
      actor.state = "ready";
      returns.push(event);
    } else if (event.event === "exit") {
      assert.equal(actor.state, "ready", "exit during native load");
      actor.state = "exit";
    } else throw new Error("failed native load");
  }
  for (const actor of actors.values())
    assert.notEqual(actor.state, "attempt", "missing native load result");
  return returns;
}
