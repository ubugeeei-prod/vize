import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { effectScope } from "vue";

import { useMutationObserver } from "./mutation-observer.ts";
import {
  asElement,
  FakeDocument,
  latestObserver,
  mutationObserverHost,
  resetObservers,
  trigger,
} from "./testing/fake-dom.ts";

beforeEach(resetObservers);

void test("observes with exactly the provided init and disconnects with the scope", () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const scope = effectScope();
  let records = 0;
  const controls = scope.run(() =>
    useMutationObserver(element, (list) => (records += list.length), {
      host: mutationObserverHost(),
      attributes: true,
      attributeFilter: ["class"],
      subtree: true,
    }),
  );
  const observer = latestObserver();
  assert.equal(controls?.isSupported.value, true);
  assert.deepEqual(observer.observed.get(element), {
    attributes: true,
    subtree: true,
    attributeFilter: ["class"],
  });
  trigger(observer, [{}, {}]);
  assert.equal(records, 2);

  observer.pending = [{ type: "attributes" }];
  assert.deepEqual(controls?.takeRecords(), [{ type: "attributes" }]);
  scope.stop();
  assert.equal(observer.disconnected, true);
  assert.deepEqual(controls?.takeRecords(), []);
});

void test("does nothing without a capability", () => {
  const document = new FakeDocument();
  const controls = useMutationObserver(asElement(document.createElement()), () => undefined, {
    host: () => undefined,
    childList: true,
  });
  assert.equal(controls.isSupported.value, false);
  assert.deepEqual(controls.takeRecords(), []);
  controls.stop();
  controls.stop();
});
