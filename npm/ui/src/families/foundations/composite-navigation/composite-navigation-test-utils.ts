import { createCollectionRegistry } from "../collection/collection.ts";
import type { CollectionRegistry } from "../collection/collection.ts";
import { createCompositeNavigation } from "./composite-navigation.ts";
import type {
  CompositeNavigationController,
  CompositeNavigationOptions,
} from "./composite-navigation.ts";

export interface CompositeValue {
  readonly label: string;
}

export interface CompositeHarness {
  readonly container: HTMLDivElement;
  readonly controller: CompositeNavigationController<string>;
  readonly elements: ReadonlyMap<string, HTMLButtonElement>;
  readonly registry: CollectionRegistry<string, CompositeValue>;
  readonly unmount: () => void;
}

export function keyboard(
  key: string,
  init: KeyboardEventInit = {},
  target?: Element,
): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key,
    ...init,
  });
  (target ?? document.body).dispatchEvent(event);
  return event;
}

export function mountComposite(
  options: Omit<CompositeNavigationOptions<string, CompositeValue>, "registry"> = {},
): CompositeHarness {
  const container = document.createElement("div");
  const elements = new Map<string, HTMLButtonElement>();
  const registry = createCollectionRegistry<string, CompositeValue>();
  for (const [key, label, disabled] of [
    ["alpha", "Alpha", false],
    ["blocked", "Blocked", true],
    ["bravo", "Bravo", false],
    ["charlie", "Charlie", false],
  ] as const) {
    const element = document.createElement("button");
    element.id = `item-${key}`;
    element.textContent = label;
    container.append(element);
    elements.set(key, element);
    registry.register({ key, value: { label }, textValue: label, disabled, element });
  }
  document.body.append(container);
  const controller = createCompositeNavigation({
    registry,
    ...options,
  } as CompositeNavigationOptions<string, CompositeValue>);
  const containerProps = controller.getContainerProps();
  container.addEventListener("focus", containerProps.onFocus);
  container.addEventListener("keydown", containerProps.onKeydown);
  for (const [key, element] of elements) {
    const props = controller.getItemProps(key);
    element.addEventListener("focus", props.onFocus);
    element.addEventListener("pointerdown", props.onPointerdown);
  }
  return {
    container,
    controller,
    elements,
    registry,
    unmount: () => {
      controller.dispose();
      registry.dispose();
      container.remove();
    },
  };
}

export function setDynamicProps(harness: CompositeHarness): void {
  const containerProps = harness.controller.getContainerProps();
  if (containerProps.tabindex === undefined) harness.container.removeAttribute("tabindex");
  else harness.container.tabIndex = containerProps.tabindex;
  const activeDescendant = containerProps["aria-activedescendant"];
  if (activeDescendant === undefined) harness.container.removeAttribute("aria-activedescendant");
  else harness.container.setAttribute("aria-activedescendant", activeDescendant);
  for (const [key, element] of harness.elements) {
    const props = harness.controller.getItemProps(key);
    if (props.tabindex === undefined) element.removeAttribute("tabindex");
    else element.tabIndex = props.tabindex;
    if (props.id !== undefined) element.id = props.id;
  }
}
