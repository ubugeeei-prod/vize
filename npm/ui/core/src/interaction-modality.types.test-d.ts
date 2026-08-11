/** Compile-only assertions for the public interaction-modality contract. */

import { ref } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import {
  createInteractionModalityTracker,
  isElementFocusVisible,
  type InteractionModality,
  type InteractionModalityChange,
} from "./interaction-modality.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const keyboardModality: InteractionModality = "keyboard";
export const pointerModality: InteractionModality = "pointer";
export const touchModality: InteractionModality = "touch";
export const virtualModality: InteractionModality = "virtual";

// @ts-expect-error the public modality contract is a closed union.
export const speechModality: InteractionModality = "speech";

const reactiveDocument = ref<Document | null>(null);
export const tracker = createInteractionModalityTracker({
  document: reactiveDocument,
  initialModality: "keyboard",
  onChange(change: InteractionModalityChange) {
    const current: InteractionModality | null = change.modality;
    const event: Event | null = change.originalEvent;
    void current;
    void event;
  },
});

type _DocumentIsReadonlyShallowRef = Expect<
  Equal<typeof tracker.document, Readonly<ShallowRef<Document | null>>>
>;
type _VisibilityIsComputed = Expect<Equal<typeof tracker.isFocusVisible, ComputedRef<boolean>>>;

// @ts-expect-error consumers cannot mutate readonly reactive state directly.
tracker.modality.value = "touch";
// @ts-expect-error invalid modalities cannot enter the synchronized state.
tracker.setModality("mouse");
// @ts-expect-error Window is not a Document.
tracker.attach(window);

export const focusVisible: boolean = isElementFocusVisible(document.body, tracker.modality.value);
