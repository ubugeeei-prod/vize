import { ref } from "vue";
import type { Ref } from "vue";

import { createInertOutside } from "./inert-outside.ts";
import type { InertOutsideController, InertOutsideOptions } from "./inert-outside.ts";

export interface InertOutsideHarness {
  readonly after: HTMLDivElement;
  readonly afterInside: HTMLDivElement;
  readonly app: HTMLDivElement;
  readonly before: HTMLDivElement;
  readonly beforeInside: HTMLDivElement;
  readonly controller: InertOutsideController;
  readonly root: HTMLDivElement;
  readonly rootRef: Ref<Element | null>;
  readonly unmount: () => void;
}

export function mountInertOutside(
  options: Omit<InertOutsideOptions, "root"> = {},
  activate = true,
): InertOutsideHarness {
  const before = document.createElement("div");
  const app = document.createElement("div");
  const beforeInside = document.createElement("div");
  const root = document.createElement("div");
  const afterInside = document.createElement("div");
  const after = document.createElement("div");
  app.append(beforeInside, root, afterInside);
  document.body.append(before, app, after);
  const rootRef = ref<Element | null>(root);
  const controller = createInertOutside({ root: rootRef, ...options });
  if (activate) controller.activate();
  return {
    after,
    afterInside,
    app,
    before,
    beforeInside,
    controller,
    root,
    rootRef,
    unmount: () => {
      controller.dispose();
      before.remove();
      app.remove();
      after.remove();
    },
  };
}

export function isIsolated(element: Element): boolean {
  return element.getAttribute("aria-hidden") === "true" && element.hasAttribute("inert");
}
