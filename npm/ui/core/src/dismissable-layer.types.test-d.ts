/** Compile-only assertions for the public dismissable-layer contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createDismissableLayer,
  type DismissableLayerController,
  type DismissableLayerDismissReason,
  type DismissableLayerInteractOutsideEvent,
  type DismissableLayerProps,
} from "./dismissable-layer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const root = ref<Element | null>(null);
const controller: DismissableLayerController = createDismissableLayer({
  root,
  branches: () => [],
  enabled: ref(true),
  onDismiss(event) {
    const reason: DismissableLayerDismissReason = event.reason;
    void reason;
  },
  onInteractOutside(event: DismissableLayerInteractOutsideEvent) {
    event.preventDefault();
  },
});
const props: Readonly<DismissableLayerProps> = controller.layerProps;
const release: () => void = controller.registerBranch(document.body);
release();
void props;

type _ActiveIsReadonly = Expect<Equal<typeof controller.isActive, Readonly<ShallowRef<boolean>>>>;
type _TopIsReadonly = Expect<Equal<typeof controller.isTopLayer, Readonly<ShallowRef<boolean>>>>;

// @ts-expect-error branches retain DOM element type safety.
createDismissableLayer({ root, branches: [window] });
// @ts-expect-error enablement must resolve to boolean.
createDismissableLayer({ root, enabled: ref("yes") });
// @ts-expect-error Escape routing must resolve to boolean.
createDismissableLayer({ root, escapeKey: "true" });
// @ts-expect-error callback evidence is immutable.
createDismissableLayer({ root, onDismiss: (event) => (event.reason = "escape-key") });
