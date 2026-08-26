/** Compile-only assertions for the public shortcut contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createShortcutRegistry,
  formatShortcut,
  getShortcutKeycaps,
  parseShortcut,
  type ShortcutMatch,
  type ShortcutProps,
  type ShortcutSequence,
} from "./shortcut.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const registry = createShortcutRegistry({
  target: ref<HTMLElement | null>(null),
  platform: "apple",
  sequenceTimeout: () => 750,
  isDisabled: ref(false),
});

export const release = registry.register({
  shortcut: "Mod+K",
  scope: "palette",
  when: () => true,
  description: "Open palette",
  handler(match: ShortcutMatch) {
    const scope: string = match.scope;
    void scope;
  },
});

type _PendingIsReadonly = Expect<
  Equal<typeof registry.pendingSequence, Readonly<ShallowRef<ShortcutSequence>>>
>;
type _ScopesAreReadonly = Expect<
  Equal<typeof registry.activeScopes, Readonly<ShallowRef<readonly string[]>>>
>;
type _PropsAreExact = Expect<Equal<typeof registry.shortcutProps, Readonly<ShortcutProps>>>;
type _RegisterReturnsReleaser = Expect<Equal<typeof release, () => void>>;
type _InputReportsCompletion = Expect<Equal<ReturnType<typeof registry.input>, boolean>>;
type _KeycapsAreNested = Expect<
  Equal<ReturnType<typeof getShortcutKeycaps>, readonly (readonly string[])[]>
>;

export const vueAttributes: HTMLAttributes = registry.shortcutProps;
export const sequence: ShortcutSequence = parseShortcut("Mod+K Mod+S", { platform: "standard" });
export const formatted: string = formatShortcut(sequence, { platform: "apple", style: "symbol" });

// @ts-expect-error consumers cannot mutate readonly pending state.
registry.pendingSequence.value = [];
// @ts-expect-error chords in a sequence are immutable.
sequence[0].key = "j";
// @ts-expect-error platform is a closed union.
parseShortcut("Mod+K", { platform: "windows" });
// @ts-expect-error style is a closed union.
formatShortcut("Mod+K", { style: "verbose" });
// @ts-expect-error handler is required.
registry.register({ shortcut: "Mod+K" });
// @ts-expect-error sequenceTimeout must resolve to a number.
createShortcutRegistry({ sequenceTimeout: "750" });
