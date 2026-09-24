<script setup lang="ts" generic="Id extends string = string">
import { computed, onMounted, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useLiveRegion } from "../../accessibility/live-region/live-region.ts";
import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CommandInfo, CommandRouter } from "../../foundations/command/command.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { commandPaletteContext, commandPaletteDialogContext } from "./command-palette-context.ts";
import type {
  CommandPaletteContextValue,
  CommandPaletteItemEntry,
} from "./command-palette-context.ts";
import {
  defaultCommandPaletteFilter,
  defaultCommandPaletteResultsLabel,
} from "./command-palette-filter.ts";
import type {
  CommandPaletteFilter,
  CommandPaletteResultsLabel,
  CommandPaletteRootExpose,
  CommandPaletteSlotState,
  CommandPaletteState,
} from "./command-palette-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = true,
  search = undefined,
  defaultSearch = "",
  router = undefined,
  filter = defaultCommandPaletteFilter,
  shouldFilter = true,
  loading = false,
  loop = true,
  recent = undefined,
  defaultRecent = [],
  recentLimit = 5,
  resultsLabel = defaultCommandPaletteResultsLabel,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled visibility of the results list (`aria-expanded` on the input).
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial uncontrolled results visibility.
   *
   * @default true
   */
  readonly defaultOpen?: boolean;

  /**
   * Controlled search text. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly search?: string;

  /**
   * Initial uncontrolled search text.
   *
   * @default ""
   */
  readonly defaultSearch?: string;

  /**
   * Command router whose commands items can run and whose metadata the
   * default slot receives as `commands`.
   *
   * @default undefined
   */
  readonly router?: CommandRouter<Id>;

  /**
   * Item scorer. `0` hides an item. The default ranks exact, prefix,
   * word-prefix, substring, and subsequence matches.
   *
   * @default defaultCommandPaletteFilter
   */
  readonly filter?: CommandPaletteFilter;

  /**
   * Filter items locally. Set `false` when results are already filtered, for
   * example by a server search.
   *
   * @default true
   */
  readonly shouldFilter?: boolean;

  /**
   * Results are loading; the list reports `aria-busy`.
   *
   * @default false
   */
  readonly loading?: boolean;

  /**
   * Wrap arrow-key navigation at the first and last item.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Controlled most-recent-first command ids.
   *
   * @default undefined
   */
  readonly recent?: readonly Id[];

  /**
   * Initial uncontrolled recent command ids.
   *
   * @default []
   */
  readonly defaultRecent?: readonly Id[];

  /**
   * Maximum number of recent command ids kept.
   *
   * @default 5
   */
  readonly recentLimit?: number;

  /**
   * Live-region announcement for the visible result count.
   *
   * @default defaultCommandPaletteResultsLabel
   */
  readonly resultsLabel?: CommandPaletteResultsLabel;
}>();

const emit = defineEmits<{
  /** Fired when results visibility requests a new controlled value. */
  "update:open": [value: boolean];

  /** Fired when the search text requests a new controlled value. */
  "update:search": [value: string];

  /** Fired when running a router command updates the recent list. */
  "update:recent": [value: readonly Id[]];

  /** Fired after an item was selected; `commandId` is `null` for plain items. */
  select: [commandId: Id | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Palette parts. Receives search, result, and router command state. */
  default?(props: CommandPaletteSlotState<Id>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "command-palette" });
const listId = computed(() => deriveDeterministicId(baseId.value, "list"));
const inputId = computed(() => deriveDeterministicId(baseId.value, "input"));
const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const searchState = useControllableState({
  value: () => search,
  defaultValue: () => defaultSearch,
});
const recentState = useControllableState<readonly Id[]>({
  value: () => recent,
  defaultValue: () => defaultRecent,
});
const isOpen = openState.value;
const searchValue = searchState.value;
const loadingState = computed(() => loading);
const state = computed<CommandPaletteState>(() => (isOpen.value ? "open" : "closed"));
const inputElement = shallowRef<HTMLInputElement | null>(null);
const entries = shallowRef<ReadonlyMap<string, CommandPaletteItemEntry>>(new Map());
const scores = computed(() => {
  const next = new Map<string, number>();
  for (const [key, entry] of entries.value) {
    const visible = !shouldFilter || entry.forceMount();
    next.set(key, visible ? 1 : filter(entry.textValue(), searchValue.value, entry.keywords()));
  }
  return next;
});
const resultCount = computed(() => [...scores.value.values()].filter((score) => score > 0).length);
const registry = createCollectionRegistry<string, CommandPaletteItemEntry>({
  disabledBehavior: "skip",
});
const navigation = useCompositeNavigation<string, CommandPaletteItemEntry>({
  registry,
  focusStrategy: "active-descendant",
  getItemId: ({ key }) => key,
  loop: () => loop,
  orientation: "vertical",
  typeahead: false,
});
const activeItemId = computed(
  () => navigation.getContainerProps()["aria-activedescendant"] ?? null,
);
const commands = computed<readonly CommandInfo<Id>[]>(() => {
  const all = router?.commands.value ?? [];
  if (!shouldFilter) return all;
  return all.filter(
    (command) => filter(command.title ?? command.id, searchValue.value, command.keywords) > 0,
  );
});
const recentCommands = computed<readonly CommandInfo<Id>[]>(() => {
  const all = router?.commands.value ?? [];
  return recentState.value.value.flatMap((recentId) =>
    all.filter((command) => command.id === recentId),
  );
});
const slotState = computed<CommandPaletteSlotState<Id>>(() => ({
  commands: commands.value,
  loading: loadingState.value,
  open: isOpen.value,
  recentCommands: recentCommands.value,
  resultCount: resultCount.value,
  search: searchValue.value,
}));
const dialog = commandPaletteDialogContext.useOptional();
const liveRegion = useLiveRegion({ politeness: "polite" });
/** Visually hidden but announced; inline so the palette ships no stylesheet. */
const announcerStyle = {
  border: "0",
  clipPath: "inset(50%)",
  height: "1px",
  margin: "-1px",
  overflow: "hidden",
  padding: "0",
  position: "absolute",
  whiteSpace: "nowrap",
  width: "1px",
} as const;

function isVisible(key: string): boolean {
  return (scores.value.get(key) ?? 0) > 0;
}

function registerItem(entry: CommandPaletteItemEntry) {
  const next = new Map(entries.value);
  next.set(entry.id, entry);
  entries.value = next;
  const registration = registry.register({
    disabled: () => !isVisible(entry.id) || entry.disabled(),
    element: entry.element,
    key: entry.id,
    textValue: entry.textValue,
    value: entry,
  });
  return {
    key: registration.key,
    registered: registration.registered,
    unregister: () => {
      if (entries.value.get(entry.id) === entry) {
        const remaining = new Map(entries.value);
        remaining.delete(entry.id);
        entries.value = remaining;
      }
      return registration.unregister();
    },
  };
}

function setSearch(value: string): boolean {
  const changed = searchState.set(value);
  if (changed) emit("update:search", value);
  return changed;
}

function setOpen(value: boolean): boolean {
  const changed = openState.set(value);
  if (changed) emit("update:open", value);
  return changed;
}

function setActive(key: string): void {
  if (registry.navigableItems.value.some((item) => item.key === key)) registry.setActiveKey(key);
}

function activeEntry(): CommandPaletteItemEntry | undefined {
  return activeItemId.value === null ? undefined : registry.getItem(activeItemId.value)?.value;
}

function selectActive(nativeEvent: Event | null = null): boolean {
  return activeEntry()?.select(nativeEvent) ?? false;
}

function findCommand(commandId: string): CommandInfo<Id> | undefined {
  return router?.commands.value.find((command) => command.id === commandId);
}

function runCommand(commandId: string): boolean {
  const command = findCommand(commandId);
  if (!router || !command) return false;
  return router.execute(command.id, undefined, { source: "palette" }).status === "executed";
}

function didSelect(commandId: string | null, nativeEvent: Event | null): void {
  const command = commandId === null ? undefined : findCommand(commandId);
  if (command) {
    const next = [command.id, ...recentState.value.value.filter((value) => value !== command.id)];
    const limited = next.slice(0, Math.max(0, recentLimit));
    if (recentState.set(limited)) emit("update:recent", limited);
  }
  emit("select", command?.id ?? null, nativeEvent);
  dialog?.didSelect();
}

function focusInput(options?: FocusOptions): void {
  inputElement.value?.focus(options);
}

// A new search always activates the first visible item, like native listbox filtering.
watch(
  [searchValue, () => registry.navigableItems.value],
  ([, navigable], [previousSearch]) => {
    const activeKey = registry.activeKey.value;
    const stillNavigable = navigable.some((item) => item.key === activeKey);
    if (searchValue.value !== previousSearch || !stillNavigable) {
      registry.setActiveKey(navigable[0]?.key ?? null);
    }
  },
  { flush: "post" },
);

let mounted = false;
onMounted(() => {
  mounted = true;
});
watch([searchValue, resultCount], () => {
  if (mounted && searchValue.value !== "") {
    liveRegion.announce(resultsLabel(resultCount.value, searchValue.value));
  }
});

commandPaletteContext.provide({
  activeItemId,
  commands,
  didSelect,
  findCommand,
  id: baseId,
  inputElement,
  inputId,
  isVisible,
  listId,
  loading: loadingState,
  navigation,
  open: isOpen,
  registerItem,
  registry,
  resultCount,
  runCommand,
  search: searchValue,
  selectActive,
  setActive,
  setOpen,
  setSearch,
  state,
} satisfies CommandPaletteContextValue);

type CommandPaletteRootSetupExpose = Omit<
  CommandPaletteRootExpose,
  "activeItemId" | "id" | "listId" | "open" | "resultCount" | "search"
> & {
  readonly activeItemId: ComputedRef<string | null>;
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly resultCount: ComputedRef<number>;
  readonly search: ComputedRef<string>;
};

const exposed = {
  activeItemId,
  focusInput,
  id: baseId,
  listId,
  open: isOpen,
  resultCount,
  search: searchValue,
  selectActive,
  setOpen,
  setSearch,
} satisfies CommandPaletteRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="command-palette-root"
    part="root"
    :data-state="state"
    :data-loading="loading ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      data-vize-ui="command-palette-announcer"
      part="announcer"
      :style="announcerStyle"
    >
      {{ liveRegion.message.value }}
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
