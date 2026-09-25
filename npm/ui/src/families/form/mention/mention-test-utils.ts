import { h, nextTick } from "vue";

import MentionContent from "./mention-content.vue";
import MentionEmpty from "./mention-empty.vue";
import MentionInput from "./mention-input.vue";
import MentionItem from "./mention-item.vue";
import type { MentionSlotState } from "./mention-types.ts";

/** Fixture item used across Mention tests. */
export interface Person {
  readonly handle: string;
  readonly name: string;
}

export const people: readonly Person[] = Object.freeze([
  { handle: "ada", name: "Ada Lovelace" },
  { handle: "alan", name: "Alan Turing" },
  { handle: "grace", name: "Grace Hopper" },
]);

/** Let post-flush watchers settle. */
export async function settle(): Promise<void> {
  for (let tick = 0; tick < 4; tick++) await nextTick();
}

/** Dispatch one keydown and return it. */
export function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

/** Replace the field text, place the caret, and fire `input`. */
export async function typeInto(
  field: HTMLTextAreaElement | HTMLInputElement,
  text: string,
  caret = text.length,
): Promise<void> {
  field.value = text;
  field.setSelectionRange(caret, caret);
  field.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
  await settle();
}

/** Structural slice of the mounted harness used by these helpers. */
export interface MentionQueries {
  root(): HTMLElement;
}

/** Mount options for a people mention textarea with the popup rendered in place. */
export function mentionOptions(
  rootProps: Record<string, unknown> = {},
  inputProps: Record<string, unknown> = {},
) {
  return {
    props: {
      id: "composer",
      itemText: (person: Person) => person.handle,
      items: people,
      ...rootProps,
    },
    record: ["update:modelValue", "select", "update:query", "query-change", "update:open"],
    slots: {
      default: (state: MentionSlotState<Person>) => [
        h(MentionInput, { ariaLabel: "Message", ...inputProps }),
        h(MentionContent, { ariaLabel: "People", portalDisabled: true }, () => [
          ...state.filteredItems.map((person) =>
            h(MentionItem<Person>, { key: person.handle, value: person }, () => person.name),
          ),
          h(MentionEmpty, null, () => "No people"),
        ]),
      ],
    },
  };
}

/** The mounted field. */
export function field(handle: MentionQueries): HTMLTextAreaElement | HTMLInputElement {
  const element = handle.root().querySelector("[data-vize-ui='mention-input']");
  if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) return element;
  throw new Error("mention field missing");
}

/** Visible option labels in DOM order. */
export function options(handle: MentionQueries): string[] {
  return [...handle.root().querySelectorAll("[role='option']")].map(
    (option) => option.textContent ?? "",
  );
}

/** Text of the highlighted option. */
export function activeOption(handle: MentionQueries): string {
  const id = field(handle).getAttribute("aria-activedescendant");
  if (id === null) return "";
  return handle.root().querySelector(`[id="${id}"]`)?.textContent ?? "";
}
