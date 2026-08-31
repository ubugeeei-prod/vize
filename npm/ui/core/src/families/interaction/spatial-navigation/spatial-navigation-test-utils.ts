import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import { createSpatialNavigation } from "./spatial-navigation.ts";
import type {
  SpatialNavigationController,
  SpatialNavigationOptions,
  SpatialNavigationRect,
} from "./spatial-navigation.ts";

export interface SpatialValue {
  readonly label: string;
}

export interface SpatialHarness {
  readonly container: HTMLDivElement;
  readonly controller: SpatialNavigationController<string>;
  readonly elements: ReadonlyMap<string, HTMLButtonElement>;
  readonly rects: Map<string, SpatialNavigationRect>;
  readonly registry: CollectionRegistry<string, SpatialValue>;
  readonly unmount: () => void;
}

export function rect(left: number, top: number, width = 100, height = 100): SpatialNavigationRect {
  return { left, top, width, height, right: left + width, bottom: top + height };
}

export function keyboard(
  key: string,
  target: Element,
  init: KeyboardEventInit = {},
): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

export function mountSpatial(
  options: Omit<SpatialNavigationOptions<string, SpatialValue>, "registry"> = {},
): SpatialHarness {
  const container = document.createElement("div");
  const elements = new Map<string, HTMLButtonElement>();
  const rects = new Map<string, SpatialNavigationRect>([
    ["alpha", rect(0, 0)],
    ["bravo", rect(120, 0)],
    ["charlie", rect(0, 120)],
    ["delta", rect(120, 120)],
    ["blocked", rect(240, 0)],
  ]);
  const registry = createCollectionRegistry<string, SpatialValue>();
  for (const [key, disabled] of [
    ["alpha", false],
    ["bravo", false],
    ["charlie", false],
    ["delta", false],
    ["blocked", true],
  ] as const) {
    const element = document.createElement("button");
    element.textContent = key;
    element.getBoundingClientRect = () => rects.get(key)! as DOMRect;
    container.append(element);
    elements.set(key, element);
    registry.register({ key, value: { label: key }, textValue: key, disabled, element });
  }
  document.body.append(container);
  const controller = createSpatialNavigation({ registry, ...options });
  container.addEventListener("keydown", controller.spatialNavigationProps.onKeydown);
  return {
    container,
    controller,
    elements,
    rects,
    registry,
    unmount: () => {
      controller.dispose();
      registry.dispose();
      container.remove();
    },
  };
}
