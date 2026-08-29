import { mount } from "@vue/test-utils";
import type { ComponentMountingOptions, VueWrapper } from "@vue/test-utils";
import { nextTick } from "vue";
import type { Component } from "vue";

/**
 * Mounted-DOM interaction harness: components mount into the real (happy-dom)
 * document so tests observe rendered DOM, focus movement, and dispatched
 * events — never source text. Keyboard activation of native controls (click on
 * Enter keydown for buttons, click on Space keyup for buttons and checkboxes)
 * is synthesized exactly as browsers do, because DOM implementations do not
 * run those UA behaviors for scripted keyboard events.
 */

/** One component emit observed in dispatch order. */
export interface RecordedEmit {
  /** Emit name exactly as declared, for example `press` or `update:modelValue`. */
  readonly event: string;

  /** Emit arguments in declaration order. */
  readonly payload: readonly unknown[];
}

/** Keys accepted by {@link InteractionHandle.press}. Tab is driven by {@link InteractionHandle.tab}. */
export type PressKey =
  | "Enter"
  | " "
  | "Escape"
  | "Home"
  | "End"
  | `Arrow${"Up" | "Down" | "Left" | "Right"}`;

/** Outcome of one full keydown → keyup keyboard press. */
export interface PressResult {
  /** Whether a listener canceled the keydown, as scroll suppression for Space must. */
  readonly keydownPrevented: boolean;

  /** Whether a listener canceled the keyup. */
  readonly keyupPrevented: boolean;

  /** Whether native activation synthesized a click on the target. */
  readonly activated: boolean;
}

/** Role and accessible-name filter for {@link InteractionHandle.getByRole}. */
export interface RoleFilter {
  /**
   * Required accessible name: exact string or matching pattern.
   *
   * @default undefined
   */
  readonly name?: string | RegExp;
}

/** Mount-time options for {@link mountInteraction}. */
export interface InteractionMountOptions {
  /**
   * Initial props.
   *
   * @default {}
   */
  readonly props?: Record<string, unknown>;

  /**
   * Fallthrough attributes.
   *
   * @default {}
   */
  readonly attrs?: Record<string, unknown>;

  /**
   * Slot content by name.
   *
   * @default {}
   */
  readonly slots?: Record<string, unknown>;

  /**
   * Emit names captured, interleaved, into {@link InteractionHandle.recorded}.
   *
   * @default []
   */
  readonly record?: readonly string[];
}

/** A mounted component plus the interaction and query surface of the harness. */
export interface InteractionHandle {
  /** Underlying test-utils wrapper for props updates and per-event emit queries. */
  readonly wrapper: VueWrapper;

  /** Root DOM element of the mounted component. */
  root(): HTMLElement;

  /** The component's `defineExpose` surface under a caller-supplied type. */
  exposes<Exposed>(): Exposed;

  /**
   * The unique element matching a computed ARIA role and accessible name.
   *
   * @throws {Error} When no element or more than one element matches.
   */
  getByRole(role: string, filter?: RoleFilter): HTMLElement;

  /** Like {@link InteractionHandle.getByRole} but `null` when absent; still throws on ambiguity. */
  queryByRole(role: string, filter?: RoleFilter): HTMLElement | null;

  /** The element that currently owns focus. */
  activeElement(): Element | null;

  /** Click a target through native activation semantics and settle reactivity. */
  click(target: Element): Promise<void>;

  /** Dispatch keydown and keyup, synthesizing native activation clicks browsers perform. */
  press(target: Element, key: PressKey): Promise<PressResult>;

  /**
   * Move focus like the browser's Tab key across this component's tab order.
   *
   * @returns The newly focused element, or `null` when focus left the component.
   */
  tab(options?: { readonly shift?: boolean }): Promise<Element | null>;

  /** Every recorded emit in dispatch order across all recorded names. */
  recorded(): readonly RecordedEmit[];

  /** Unmount and detach the component's container from the document. */
  unmount(): void;
}

/**
 * Mount a component into the document for behavior-level interaction tests.
 *
 * @param component Component or SFC default export to mount.
 * @param options Props, slots, attributes, and emit names to record.
 */
export function mountInteraction(
  component: Component,
  options: InteractionMountOptions = {},
): InteractionHandle {
  const container = document.createElement("div");
  document.body.append(container);
  const log: RecordedEmit[] = [];
  const wrapper = mount(component, {
    props: options.props ?? {},
    attrs: { ...options.attrs, ...createRecorders(options.record ?? [], log) },
    slots: (options.slots ?? {}) as ComponentMountingOptions<Component>["slots"],
    attachTo: container,
  }) as VueWrapper;

  const root = (): HTMLElement => {
    if (!(wrapper.element instanceof HTMLElement)) {
      throw new Error("mountInteraction requires a component with an HTML element root");
    }
    return wrapper.element;
  };

  const queryAll = (role: string, filter: RoleFilter): HTMLElement[] => {
    const rootElement = root();
    const candidates = [rootElement, ...rootElement.querySelectorAll("*")];
    return candidates.filter(
      (element): element is HTMLElement =>
        element instanceof HTMLElement &&
        computeRole(element) === role &&
        matchesName(element, filter.name),
    );
  };

  return {
    wrapper,
    root,
    exposes: <Exposed>() => wrapper.vm as unknown as Exposed,
    getByRole: (role, filter = {}) => {
      const matches = queryAll(role, filter);
      const described = describeQuery(role, filter);
      if (matches.length === 0) throw new Error(`No element found for ${described}`);
      if (matches.length > 1) throw new Error(`Found ${matches.length} elements for ${described}`);
      return matches[0] as HTMLElement;
    },
    queryByRole: (role, filter = {}) => {
      const matches = queryAll(role, filter);
      if (matches.length > 1) {
        throw new Error(`Found ${matches.length} elements for ${describeQuery(role, filter)}`);
      }
      return matches[0] ?? null;
    },
    activeElement: () => document.activeElement,
    click: async (target) => {
      if (target instanceof HTMLElement) target.click();
      else target.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
      await nextTick();
    },
    press: async (target, key) => {
      const result = pressKey(target, key);
      await nextTick();
      return result;
    },
    tab: async (tabOptions = {}) => {
      const next = moveFocusByTab(container, tabOptions.shift === true);
      await nextTick();
      return next;
    },
    recorded: () => [...log],
    unmount: () => {
      wrapper.unmount();
      container.remove();
    },
  };
}

function createRecorders(
  events: readonly string[],
  log: RecordedEmit[],
): Record<string, (...payload: unknown[]) => void> {
  const recorders: Record<string, (...payload: unknown[]) => void> = {};
  for (const event of events) {
    const key = `on${event.charAt(0).toUpperCase()}${event.slice(1)}`;
    recorders[key] = (...payload) => log.push({ event, payload });
  }
  return recorders;
}

function describeQuery(role: string, filter: RoleFilter): string {
  return filter.name === undefined ? `role "${role}"` : `role "${role}" named ${filter.name}`;
}

function computeRole(element: HTMLElement): string | null {
  const explicit = element.getAttribute("role");
  if (explicit !== null && explicit !== "") return explicit.trim().split(/\s+/)[0] ?? null;
  const tag = element.tagName.toLowerCase();
  if (tag === "button") return "button";
  if (tag === "a" && element.hasAttribute("href")) return "link";
  if (tag === "progress") return "progressbar";
  if (element instanceof HTMLInputElement) {
    if (element.type === "checkbox") return "checkbox";
    if (element.type === "radio") return "radio";
    if (element.type === "search") return "searchbox";
    return "textbox";
  }
  if (element instanceof HTMLTextAreaElement) return "textbox";
  return null;
}

function matchesName(element: HTMLElement, name: string | RegExp | undefined): boolean {
  if (name === undefined) return true;
  const accessible = computeAccessibleName(element);
  return typeof name === "string" ? accessible === name : name.test(accessible);
}

function computeAccessibleName(element: HTMLElement): string {
  const labelledBy = element.getAttribute("aria-labelledby");
  if (labelledBy !== null && labelledBy !== "") {
    const text = labelledBy
      .split(/\s+/)
      .map((id) => element.ownerDocument.getElementById(id)?.textContent ?? "")
      .join(" ");
    return collapseWhitespace(text);
  }
  const ariaLabel = element.getAttribute("aria-label");
  if (ariaLabel !== null && ariaLabel !== "") return collapseWhitespace(ariaLabel);
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
    const id = element.getAttribute("id");
    const forLabel = id === null ? null : element.ownerDocument.querySelector(`label[for="${id}"]`);
    const label = forLabel ?? element.closest("label");
    if (label !== null) return collapseWhitespace(label.textContent ?? "");
  }
  return collapseWhitespace(element.textContent ?? "");
}

function collapseWhitespace(text: string): string {
  return text.replaceAll(/\s+/g, " ").trim();
}

function pressKey(target: Element, key: PressKey): PressResult {
  const keydown = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  target.dispatchEvent(keydown);
  let activated = false;
  if (!keydown.defaultPrevented && key === "Enter" && activatesOnEnter(target)) {
    target.click();
    activated = true;
  }
  const keyup = new KeyboardEvent("keyup", { key, bubbles: true, cancelable: true });
  target.dispatchEvent(keyup);
  if (
    !keydown.defaultPrevented &&
    !keyup.defaultPrevented &&
    key === " " &&
    activatesOnSpace(target)
  ) {
    target.click();
    activated = true;
  }
  return {
    keydownPrevented: keydown.defaultPrevented,
    keyupPrevented: keyup.defaultPrevented,
    activated,
  };
}

function activatesOnEnter(target: Element): target is HTMLElement {
  if (target instanceof HTMLButtonElement) return !target.disabled;
  return target instanceof HTMLAnchorElement && target.hasAttribute("href");
}

function activatesOnSpace(target: Element): target is HTMLElement {
  if (target instanceof HTMLButtonElement) return !target.disabled;
  if (!(target instanceof HTMLInputElement)) return false;
  return (target.type === "checkbox" || target.type === "radio") && !target.disabled;
}

function moveFocusByTab(container: HTMLElement, shift: boolean): Element | null {
  const active = document.activeElement;
  if (active instanceof HTMLElement && active !== document.body) {
    const keydown = new KeyboardEvent("keydown", {
      key: "Tab",
      shiftKey: shift,
      bubbles: true,
      cancelable: true,
    });
    active.dispatchEvent(keydown);
    if (keydown.defaultPrevented) return active;
  }
  const order = tabOrder(container);
  const index = active instanceof HTMLElement ? order.indexOf(active) : -1;
  const fallback = shift ? order[order.length - 1] : order[0];
  const next = index === -1 ? fallback : order[shift ? index - 1 : index + 1];
  if (next === undefined) {
    if (active instanceof HTMLElement) active.blur();
    return null;
  }
  next.focus();
  next.dispatchEvent(new KeyboardEvent("keyup", { key: "Tab", shiftKey: shift, bubbles: true }));
  return next;
}

function tabOrder(container: HTMLElement): HTMLElement[] {
  const selector = "a[href], area[href], button, input, select, textarea, [tabindex]";
  const all = [...container.querySelectorAll(selector)].filter(
    (element): element is HTMLElement => element instanceof HTMLElement && isTabbable(element),
  );
  const positive = all
    .filter((element) => element.tabIndex > 0)
    .sort((left, right) => left.tabIndex - right.tabIndex);
  return [...positive, ...all.filter((element) => element.tabIndex === 0)];
}

function isTabbable(element: HTMLElement): boolean {
  if (element.tabIndex < 0) return false;
  if ("disabled" in element && element.disabled === true) return false;
  return !(element instanceof HTMLInputElement && element.type === "hidden");
}
