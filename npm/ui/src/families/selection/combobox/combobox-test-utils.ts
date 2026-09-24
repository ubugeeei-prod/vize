import { h, nextTick } from "vue";
import type { VNode } from "vue";

import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import ComboboxAnchor from "./combobox-anchor.vue";
import ComboboxChip from "./combobox-chip.vue";
import ComboboxChipRemove from "./combobox-chip-remove.vue";
import ComboboxCreateItem from "./combobox-create-item.vue";
import ComboboxEmpty from "./combobox-empty.vue";
import ComboboxInput from "./combobox-input.vue";
import ComboboxLoading from "./combobox-loading.vue";
import ComboboxTrigger from "./combobox-trigger.vue";
import type { ComboboxSlotState } from "./combobox-types.ts";

/** Fixture option used across Combobox tests. */
export interface City {
  readonly id: number;
  readonly name: string;
}

export const cities: readonly City[] = Object.freeze([
  { id: 1, name: "Berlin" },
  { id: 2, name: "Bogotá" },
  { id: 3, name: "Boston" },
  { id: 4, name: "Kyoto" },
  { id: 5, name: "Zürich" },
]);

/** Let queued open-focus work and post-flush watchers settle. */
export async function settle(): Promise<void> {
  for (let tick = 0; tick < 4; tick++) await nextTick();
}

/** Dispatch one keydown and return it. */
export function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

/** Type text into the input as one `input` event. */
export async function type(
  input: HTMLInputElement,
  text: string,
  inputType = "insertText",
): Promise<void> {
  input.value = text;
  input.dispatchEvent(new InputEvent("input", { bubbles: true, data: text, inputType }));
  await settle();
}

/** Options accepted by {@link mountCombobox}. */
export interface MountComboboxOptions {
  /** Render options from the root `filteredItems` slot state instead of a static list. */
  readonly itemsMode?: boolean;
}

function renderOptions(state: ComboboxSlotState<City>, itemsMode: boolean): VNode[] {
  const source = itemsMode ? state.filteredItems : cities;
  return source.map((city) =>
    h(SelectItem<City>, { key: city.id, textValue: city.name, value: city }, () => city.name),
  );
}

/** Structural slice of the mounted-DOM harness used by these helpers. */
export interface ComboboxQueries {
  getByRole(role: string, filter?: { readonly name?: string | RegExp }): HTMLElement;
  root(): HTMLElement;
}

/** Mount options for a City combobox with portal disabled so the popup lives inside the root. */
export function comboboxOptions(
  rootProps: Record<string, unknown> = {},
  options: MountComboboxOptions = {},
) {
  return {
    props: { by: "id", id: "city", itemText: (city: City) => city.name, ...rootProps },
    record: ["update:modelValue", "change", "update:open", "update:inputValue", "create"],
    slots: {
      default: (state: ComboboxSlotState<City>) => [
        h(ComboboxAnchor, null, () => [
          ...state.selected.map((city) =>
            h(ComboboxChip<City>, { key: city.id, value: city }, () => [
              city.name,
              h(ComboboxChipRemove, null, () => "×"),
            ]),
          ),
          h(ComboboxInput, { ariaLabel: "City", placeholder: "Search cities" }),
          h(ComboboxTrigger, null, () => "▾"),
        ]),
        h(SelectContent, { portalDisabled: true }, () => [
          h(ComboboxLoading, null, () => "Loading…"),
          ...renderOptions(state, options.itemsMode === true || "items" in rootProps),
          h(ComboboxCreateItem, null, {
            default: ({ query }: { readonly query: string }) => `Create "${query}"`,
          }),
          h(ComboboxEmpty, null, () => "No cities"),
        ]),
      ],
    },
  };
}

/** The input of a mounted combobox. */
export function comboboxInput(handle: ComboboxQueries): HTMLInputElement {
  const element = handle.getByRole("combobox", { name: "City" });
  if (!(element instanceof HTMLInputElement)) throw new Error("combobox must be an input");
  return element;
}

/** Visible option labels in DOM order. */
export function visibleOptions(handle: ComboboxQueries): string[] {
  return [...handle.root().querySelectorAll("[role='option']:not([hidden])")].map(
    (option) => option.textContent ?? "",
  );
}

/** Text of the option referenced by `aria-activedescendant`. */
export function activeOption(handle: ComboboxQueries): string {
  const id = comboboxInput(handle).getAttribute("aria-activedescendant");
  if (id === null) return "";
  return handle.root().querySelector(`[id="${id}"]`)?.textContent ?? "";
}
