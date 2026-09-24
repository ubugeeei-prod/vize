import assert from "node:assert/strict";
import { test } from "node:test";

import { useMachine } from "./use-machine.ts";

function createFetcher(log: string[] = []) {
  return useMachine({
    initial: "idle",
    context: { retries: 0 },
    states: {
      idle: { on: { FETCH: "loading" } },
      loading: {
        entry: () => log.push("enter loading"),
        exit: () => log.push("exit loading"),
        on: { RESOLVE: "success", REJECT: "failure" },
      },
      failure: {
        on: {
          RETRY: [
            {
              target: "loading",
              guard: (context) => context.retries < 2,
              action: (context) => ({ retries: context.retries + 1 }),
            },
            { target: "gaveUp" },
          ],
        },
      },
      success: {},
      gaveUp: {},
    },
  });
}

void test("transitions, hooks, and guarded fallbacks", () => {
  const log: string[] = [];
  const machine = createFetcher(log);
  assert.equal(machine.state.value, "idle");
  assert.equal(machine.send("RESOLVE"), false);
  assert.equal(machine.send("FETCH"), true);
  machine.send("REJECT");
  assert.deepEqual(log, ["enter loading", "exit loading"]);
  machine.send("RETRY");
  machine.send("REJECT");
  machine.send("RETRY");
  machine.send("REJECT");
  assert.equal(machine.context.value.retries, 2);
  assert.equal(machine.send("RETRY"), true);
  assert.equal(machine.state.value, "gaveUp");
  assert.equal(machine.matches("gaveUp", "success"), true);
});

void test("can and nextEvents reflect guards", () => {
  const machine = createFetcher();
  assert.deepEqual(machine.nextEvents.value, ["FETCH"]);
  machine.send("FETCH");
  assert.deepEqual(machine.nextEvents.value, ["RESOLVE", "REJECT"]);
  assert.equal(machine.can("FETCH"), false);
  machine.send("RESOLVE");
  assert.deepEqual(machine.nextEvents.value, []);
  machine.reset();
  assert.equal(machine.state.value, "idle");
  assert.deepEqual(machine.context.value, { retries: 0 });
});

void test("self transitions run exit then entry", () => {
  const log: string[] = [];
  const toggle = useMachine({
    initial: "on",
    context: undefined,
    states: {
      on: {
        entry: () => log.push("entry"),
        exit: () => log.push("exit"),
        on: { PING: "on", FLIP: "off" },
      },
      off: { on: { FLIP: "on" } },
    },
  });
  toggle.send("PING");
  assert.deepEqual(log, ["exit", "entry"]);
});
