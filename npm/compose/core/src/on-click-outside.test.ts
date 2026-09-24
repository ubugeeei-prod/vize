import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { onClickOutside } from "./on-click-outside.ts";
import type { ClickOutsideHost } from "./on-click-outside.ts";
import { asElement, eventWith, FakeDocument, FakeElement } from "./testing/fake-dom.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

class ClickWindow extends EventTarget implements ClickOutsideHost {
  readonly document: { activeElement: Element | null } = { activeElement: null };
}

function clickOn(host: EventTarget, node: FakeElement | null): void {
  const path = node ? [node] : [];
  host.dispatchEvent(eventWith("pointerdown", { composedPath: () => path }));
  host.dispatchEvent(eventWith("click", { composedPath: () => path }));
}

void test("fires for outside clicks only and honors the ignore list", () => {
  const document = new FakeDocument();
  const target = document.createElement();
  const inner = document.createElement();
  target.append(inner);
  const toggle = document.createElement();
  const tagged = document.createElement();
  tagged.matching.add("[data-ignore]");
  const outside = document.createElement();
  const host = new ClickWindow();
  const calls: string[] = [];
  const scope = effectScope();
  scope.run(() =>
    onClickOutside(asElement(target), (event) => calls.push(event.type), {
      host,
      ignore: [asElement(toggle), "[data-ignore]"],
    }),
  );

  clickOn(host, inner);
  clickOn(host, toggle);
  clickOn(host, tagged);
  assert.deepEqual(calls, []);
  clickOn(host, outside);
  assert.deepEqual(calls, ["click"]);

  host.dispatchEvent(eventWith("pointerdown", { composedPath: () => [inner] }));
  host.dispatchEvent(eventWith("click", { composedPath: () => [outside] }));
  assert.deepEqual(calls, ["click"]);

  scope.stop();
  clickOn(host, outside);
  assert.deepEqual(calls, ["click"]);
});

void test("detects focus moving into an outside iframe", () => {
  const document = new FakeDocument();
  const target = document.createElement();
  const iframe = document.createElement();
  iframe.tagName = "IFRAME";
  const host = new ClickWindow();
  let queued: (() => void) | undefined;
  const scheduler: TimeoutScheduler = {
    setTimeout: (callback) => {
      queued = callback;
      return 1;
    },
    clearTimeout: () => {
      queued = undefined;
    },
  };
  let calls = 0;
  const stop = onClickOutside(asElement(target), () => (calls += 1), {
    host,
    detectIframe: true,
    scheduler,
  });
  host.document.activeElement = asElement(iframe);
  host.dispatchEvent(new Event("blur"));
  const run = queued;
  queued = undefined;
  run?.();
  assert.equal(calls, 1);
  stop();
  stop();
  host.dispatchEvent(new Event("blur"));
  assert.equal(queued, undefined);
});

void test("is inert without a window", () => {
  const stop = onClickOutside(null, () => assert.fail("never"), { host: () => undefined });
  stop();
});
