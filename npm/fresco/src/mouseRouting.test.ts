import assert from "node:assert/strict";
import { test } from "vitest";

import { routeAndPublishMouseInput } from "./mouseRouting.js";
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
  const event: MouseEvent = { type: "mouse", kind: "move", x: 0, y: 0 };
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
