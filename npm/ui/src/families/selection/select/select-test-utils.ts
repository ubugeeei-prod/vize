import { h, nextTick } from "vue";

import SelectContent from "./select-content.vue";
import SelectItem from "./select-item.vue";
import SelectItemIndicator from "./select-item-indicator.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";

/** Fixture option used across Select tests. */
export interface Fruit {
  readonly id: number;
  readonly name: string;
  readonly disabled?: boolean;
}

export const fruits: readonly Fruit[] = Object.freeze([
  { id: 1, name: "Apple" },
  { id: 2, name: "Banana" },
  { id: 3, name: "Blueberry" },
  { id: 4, name: "Cherry", disabled: true },
  { id: 5, name: "Date" },
]);

/** Let queued open-focus work and post-flush watchers settle. */
export async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
  await nextTick();
}

/** Dispatch one keydown and return it for `defaultPrevented` assertions. */
export function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

/** Structural slice of the mounted-DOM harness used by these helpers. */
export interface RoleQueries {
  getByRole(role: string, filter?: { readonly name?: string | RegExp }): HTMLElement;
}

/** Mount options for a fruit Select with portal disabled so options live inside the root. */
export function fruitSelectOptions(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return {
    props: { by: "id", id: "fruit", itemText: (fruit: Fruit) => fruit.name, ...rootProps },
    record: ["update:modelValue", "change", "update:open", "open-change"],
    slots: {
      default: () => [
        h(SelectTrigger, { ariaLabel: "Fruit" }, () => h(SelectValue)),
        h(SelectContent, { portalDisabled: true, ...contentProps }, () =>
          fruits.map((fruit) =>
            h(
              SelectItem<Fruit>,
              { disabled: fruit.disabled === true, key: fruit.id, value: fruit },
              () => [fruit.name, h(SelectItemIndicator, null, () => "✓")],
            ),
          ),
        ),
      ],
    },
  };
}

/** The trigger element of a mounted fruit Select. */
export function trigger(handle: RoleQueries): HTMLButtonElement {
  const element = handle.getByRole("combobox", { name: "Fruit" });
  if (!(element instanceof HTMLButtonElement)) throw new Error("trigger must be a button");
  return element;
}

/** Id of the option whose text is `name`. */
export function optionId(handle: RoleQueries, name: string): string {
  return handle.getByRole("option", { name: new RegExp(`^${name}`) }).id;
}
