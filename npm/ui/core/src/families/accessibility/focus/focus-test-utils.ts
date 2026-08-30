import { createFocus } from "./focus.ts";
import type { FocusController, FocusEvent, FocusOptions, FocusProps } from "./focus.ts";

export const focusEventNames: Readonly<Record<keyof FocusProps, string>> = {
  onBlur: "blur",
  onFocus: "focus",
  onFocusin: "focusin",
  onFocusout: "focusout",
};

export interface FocusHarness {
  readonly controller: FocusController;
  readonly events: FocusEvent[];
  readonly host: HTMLElement;
  readonly transitions: boolean[];
  readonly unmount: () => void;
}

export function mountFocus(options: FocusOptions = {}, tag = "button"): FocusHarness {
  const host = document.createElement(tag);
  if (host instanceof HTMLButtonElement) host.type = "button";
  else host.tabIndex = -1;
  document.body.append(host);
  const events: FocusEvent[] = [];
  const transitions: boolean[] = [];
  const controller = createFocus({
    ...options,
    onBlur(event) {
      events.push(event);
      options.onBlur?.(event);
    },
    onFocus(event) {
      events.push(event);
      options.onFocus?.(event);
    },
    onFocusChange(value, event) {
      transitions.push(value);
      options.onFocusChange?.(value, event);
    },
  });
  for (const [property, type] of Object.entries(focusEventNames) as Array<
    [keyof FocusProps, string]
  >) {
    const listener = controller.focusProps[property];
    if (listener) host.addEventListener(type, listener as EventListener);
  }
  return {
    controller,
    events,
    host,
    transitions,
    unmount() {
      try {
        controller.dispose();
      } finally {
        host.remove();
      }
    },
  };
}

export function forceModalityFallback(element: Element): () => void {
  const ownDescriptor = Object.getOwnPropertyDescriptor(element, "matches");
  Object.defineProperty(element, "matches", {
    configurable: true,
    value: (selector: string) => {
      if (selector === ":focus-visible") throw new DOMException("unsupported", "SyntaxError");
      return Element.prototype.matches.call(element, selector);
    },
  });
  return () => {
    if (ownDescriptor) Object.defineProperty(element, "matches", ownDescriptor);
    else delete (element as { matches?: Element["matches"] }).matches;
  };
}
