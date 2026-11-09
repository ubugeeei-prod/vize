import assert from "node:assert/strict";
import { test } from "vitest";

import { createMouseRoutingState, routeAndPublishMouseInput } from "./mouseRouting.js";
import type { MouseEvent } from "./protocol.js";
import type { FrescoHostNode } from "./renderer.js";

const root: FrescoHostNode = {
  id: 1,
  type: "box",
  props: {},
  children: [],
  parent: null,
};

test("publishes a mouse packet after a routed handler throws", () => {
  const failure = new Error("handler failed");
  const event: MouseEvent = { type: "mouse", kind: "down", x: 2, y: 3 };
  const published: { value: MouseEvent | null } = { value: null };

  assert.throws(
    () =>
      routeAndPublishMouseInput(
        {
          hitTest: () => root,
          dispatchInput: () => {
            throw failure;
          },
        },
        root,
        event,
        [],
        published,
      ),
    failure,
  );
  assert.equal(published.value, event);
});

test("publishes a mouse packet when no root is mounted", () => {
  const event: MouseEvent = { type: "mouse", kind: "moved", x: 0, y: 0 };
  const published: { value: MouseEvent | null } = { value: null };

  routeAndPublishMouseInput(
    {
      hitTest: () => undefined,
      dispatchInput: () => undefined,
    },
    null,
    event,
    [],
    published,
  );

  assert.equal(published.value, event);
});

test("synthesizes enter and leave around routed pointer movement", () => {
  const left = { ...root, id: 2 };
  const right = { ...root, id: 3 };
  const targets: Array<FrescoHostNode | undefined> = [left, left, right, undefined];
  const dispatched: Array<[number, MouseEvent["kind"]]> = [];
  const published: Array<MouseEvent["kind"]> = [];
  const sink = {
    get value(): MouseEvent | null {
      return null;
    },
    set value(event: MouseEvent | null) {
      if (event) published.push(event.kind);
    },
  };
  const renderer = {
    hitTest: () => targets.shift(),
    dispatchInput: (targetId: number, event: MouseEvent) => {
      dispatched.push([targetId, event.kind]);
    },
  };

  for (const x of [1, 2, 3, 4]) {
    routeAndPublishMouseInput(renderer, root, { type: "mouse", kind: "moved", x, y: 0 }, [], sink);
  }

  assert.deepEqual(dispatched, [
    [left.id, "enter"],
    [left.id, "moved"],
    [left.id, "moved"],
    [left.id, "leave"],
    [right.id, "enter"],
    [right.id, "moved"],
    [right.id, "leave"],
  ]);
  assert.deepEqual(published, [
    "enter",
    "moved",
    "moved",
    "leave",
    "enter",
    "moved",
    "leave",
    "moved",
  ]);
});

test("does not change the hovered target for non-moving mouse packets", () => {
  const state = createMouseRoutingState();
  const dispatched: Array<MouseEvent["kind"]> = [];

  routeAndPublishMouseInput(
    {
      hitTest: () => root,
      dispatchInput: (_targetId, event) => {
        dispatched.push(event.kind);
      },
    },
    root,
    { type: "mouse", kind: "down", x: 1, y: 1 },
    [],
    { value: null },
    state,
  );

  assert.deepEqual(dispatched, ["down"]);
  assert.equal(state.hoveredTargetId, undefined);
});
