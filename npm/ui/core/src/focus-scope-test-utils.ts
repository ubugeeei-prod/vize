import { ref } from "vue";
import type { Ref } from "vue";

import { createFocusScope } from "./focus-scope.ts";
import type { FocusScopeController, FocusScopeOptions } from "./focus-scope.ts";

export interface FocusScopeHarness {
  readonly controller: FocusScopeController;
  readonly first: HTMLButtonElement;
  readonly last: HTMLButtonElement;
  readonly outside: HTMLButtonElement;
  readonly programmatic: HTMLDivElement;
  readonly root: HTMLDivElement;
  readonly rootRef: Ref<Element | null>;
  readonly trigger: HTMLButtonElement;
  readonly unmount: () => void;
}

export function mountFocusScope(
  options: Omit<FocusScopeOptions, "root"> = {},
  activate = true,
): FocusScopeHarness {
  const trigger = document.createElement("button");
  trigger.textContent = "trigger";
  const outside = document.createElement("button");
  outside.textContent = "outside";
  const root = document.createElement("div");
  const first = document.createElement("button");
  first.textContent = "first";
  const programmatic = document.createElement("div");
  programmatic.tabIndex = -1;
  programmatic.textContent = "programmatic";
  const disabled = document.createElement("button");
  disabled.disabled = true;
  const last = document.createElement("button");
  last.textContent = "last";
  root.append(first, programmatic, disabled, last);
  document.body.append(trigger, root, outside);
  trigger.focus();
  const rootRef = ref<Element | null>(root);
  const controller = createFocusScope({ root: rootRef, ...options });
  if (activate) controller.activate();
  return {
    controller,
    first,
    last,
    outside,
    programmatic,
    root,
    rootRef,
    trigger,
    unmount: () => {
      controller.dispose();
      trigger.remove();
      root.remove();
      outside.remove();
    },
  };
}

export function tab(target: Element, shiftKey = false): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "Tab",
    shiftKey,
  });
  target.dispatchEvent(event);
  return event;
}
