import { ref } from "vue";

import { createFocusGuards } from "./focus-guards.ts";
import type { FocusGuardRedirectEvent, FocusGuardsOptions } from "./focus-guards.ts";

export interface FocusGuardsHarness {
  readonly after: HTMLSpanElement;
  readonly before: HTMLSpanElement;
  readonly controller: ReturnType<typeof createFocusGuards>;
  readonly events: FocusGuardRedirectEvent[];
  readonly first: HTMLButtonElement;
  readonly last: HTMLButtonElement;
  readonly outsideAfter: HTMLButtonElement;
  readonly outsideBefore: HTMLButtonElement;
  readonly root: HTMLDivElement;
  readonly rootRef: ReturnType<typeof ref<Element | null>>;
  readonly unmount: () => void;
}

export function mountFocusGuards(
  options: Omit<FocusGuardsOptions, "root"> = {},
): FocusGuardsHarness {
  const outsideBefore = document.createElement("button");
  const before = document.createElement("span");
  const root = document.createElement("div");
  const first = document.createElement("button");
  const last = document.createElement("button");
  const after = document.createElement("span");
  const outsideAfter = document.createElement("button");
  first.textContent = "first";
  last.textContent = "last";
  root.append(first, last);
  document.body.append(outsideBefore, before, root, after, outsideAfter);
  const rootRef = ref<Element | null>(root);
  const events: FocusGuardRedirectEvent[] = [];
  const consumerRedirect = options.onRedirect;
  const controller = createFocusGuards({
    ...options,
    root: rootRef,
    onRedirect: (event) => {
      events.push(event);
      consumerRedirect?.(event);
    },
  });
  before.addEventListener("focus", controller.beforeProps.onFocus);
  after.addEventListener("focus", controller.afterProps.onFocus);
  controller.activate();
  before.tabIndex = controller.beforeProps.tabindex;
  after.tabIndex = controller.afterProps.tabindex;
  return {
    after,
    before,
    controller,
    events,
    first,
    last,
    outsideAfter,
    outsideBefore,
    root,
    rootRef,
    unmount: () => {
      controller.dispose();
      outsideBefore.remove();
      before.remove();
      root.remove();
      after.remove();
      outsideAfter.remove();
    },
  };
}

export function pressTab(backward = false, target: Document = document): void {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, key: "Tab", shiftKey: backward }),
  );
}
