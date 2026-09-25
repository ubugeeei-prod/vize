<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { useShortcutRegistry } from "../../interaction/shortcut/shortcut.ts";
import { sidebarContext } from "./sidebar-context.ts";
import type {
  SidebarCollapsible,
  SidebarProviderExpose,
  SidebarSide,
  SidebarSlotState,
  SidebarState,
  SidebarStorage,
  SidebarVariant,
} from "./sidebar-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = true,
  openMobile = undefined,
  defaultOpenMobile = false,
  collapsible = "offcanvas",
  side = "left",
  variant = "sidebar",
  storage = undefined,
  storageKey = "vize-sidebar-open",
  keyboardShortcut = "Mod+B",
  mobileQuery = "(max-width: 767px)",
  width = "16rem",
  widthIcon = "3rem",
  widthMobile = "18rem",
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled desktop open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial desktop open state for uncontrolled use.
   *
   * @default true
   */
  readonly defaultOpen?: boolean;

  /**
   * Controlled mobile sheet open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly openMobile?: boolean;

  /**
   * Initial mobile sheet open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpenMobile?: boolean;

  /**
   * Desktop collapse mode: off-canvas, icon rail, or never.
   *
   * @default "offcanvas"
   */
  readonly collapsible?: SidebarCollapsible;

  /**
   * Screen edge the sidebar is attached to.
   *
   * @default "left"
   */
  readonly side?: SidebarSide;

  /**
   * Consumer styling variant mirrored to `data-variant`.
   *
   * @default "sidebar"
   */
  readonly variant?: SidebarVariant;

  /**
   * Adapter that persists the desktop open state. It is read only after mount,
   * so server and hydration markup always start from `open`/`defaultOpen`.
   *
   * @default undefined
   */
  readonly storage?: SidebarStorage;

  /**
   * Key used with `storage`.
   *
   * @default "vize-sidebar-open"
   */
  readonly storageKey?: string;

  /**
   * Document-level shortcut that toggles the sidebar. `null` disables it.
   *
   * @default "Mod+B"
   */
  readonly keyboardShortcut?: string | null;

  /**
   * Media query that switches to the mobile sheet. Evaluated only on the
   * client; server rendering assumes desktop. `null` disables the sheet.
   *
   * @default "(max-width: 767px)"
   */
  readonly mobileQuery?: string | null;

  /**
   * Expanded width published as `--vize-sidebar-width`.
   *
   * @default "16rem"
   */
  readonly width?: string;

  /**
   * Rail width published as `--vize-sidebar-width-icon`.
   *
   * @default "3rem"
   */
  readonly widthIcon?: string;

  /**
   * Mobile sheet width published as `--vize-sidebar-width-mobile`.
   *
   * @default "18rem"
   */
  readonly widthMobile?: string;
}>();

const emit = defineEmits<{
  /** Fired when the desktop open state requests a controlled value. */
  "update:open": [value: boolean];

  /** Fired when the mobile sheet requests a controlled value. */
  "update:openMobile": [value: boolean];

  /** Fired after any distinct desktop open request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Sidebar layout: SidebarRoot, SidebarInset, and triggers. */
  default(props: SidebarSlotState): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "sidebar" });
const sidebarId = computed(() => deriveDeterministicId(baseId.value, "panel"));
const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const openMobileState = useControllableState({
  value: () => openMobile,
  defaultValue: () => defaultOpenMobile,
});
const collapsibleState = computed(() => collapsible);
const sideState = computed(() => side);
const variantState = computed(() => variant);
const mobileMatches = shallowRef(false);
const isMobile = computed(() => mobileQuery !== null && mobileMatches.value);
const isOpen = computed(() => collapsible === "none" || openState.value.value);
const isOpenMobile = computed(() => openMobileState.value.value);
const state = computed<SidebarState>(() => (isOpen.value ? "expanded" : "collapsed"));
const shortcutTarget = shallowRef<Document | null>(null);
const shortcuts = useShortcutRegistry({ target: shortcutTarget });
const style = computed(() => ({
  "--vize-sidebar-width": width,
  "--vize-sidebar-width-icon": widthIcon,
  "--vize-sidebar-width-mobile": widthMobile,
}));
const slotState = computed<SidebarSlotState>(() => ({
  collapsible: collapsibleState.value,
  isMobile: isMobile.value,
  open: isOpen.value,
  openMobile: isOpenMobile.value,
  side: sideState.value,
  state: state.value,
  variant: variantState.value,
}));

function readOpen(): boolean {
  return isOpen.value;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  if (collapsible === "none") return false;
  const previous = readOpen();
  if (!openState.set(value)) return false;
  emit("update:open", value);
  emit("open-change", value, previous, nativeEvent);
  storage?.set(storageKey, String(value));
  return true;
}

function setOpenMobile(value: boolean): boolean {
  if (!openMobileState.set(value)) return false;
  emit("update:openMobile", value);
  return true;
}

function toggle(nativeEvent: Event | null = null): boolean {
  if (isMobile.value) return setOpenMobile(!isOpenMobile.value);
  return setOpen(!readOpen(), nativeEvent);
}

let releaseShortcut: (() => void) | null = null;
let releaseMedia: (() => void) | null = null;

function registerShortcut(): void {
  releaseShortcut?.();
  releaseShortcut = null;
  if (keyboardShortcut === null) return;
  releaseShortcut = shortcuts.register({
    description: "Toggle sidebar",
    handler: (match) => {
      toggle(match.originalEvent);
    },
    shortcut: keyboardShortcut,
  });
}

function watchMobileQuery(view: Window | null): void {
  releaseMedia?.();
  releaseMedia = null;
  if (mobileQuery === null || view === null || typeof view.matchMedia !== "function") return;
  const media = view.matchMedia(mobileQuery);
  mobileMatches.value = media.matches;
  const onChange = (event: MediaQueryListEvent) => {
    mobileMatches.value = event.matches;
  };
  media.addEventListener("change", onChange);
  releaseMedia = () => media.removeEventListener("change", onChange);
}

let mounted = false;

watch(
  () => keyboardShortcut,
  () => {
    if (mounted) registerShortcut();
  },
);
watch(
  () => mobileQuery,
  () => {
    if (!mounted) return;
    mobileMatches.value = false;
    watchMobileQuery(document.defaultView);
  },
);

onMounted(() => {
  mounted = true;
  const persisted = storage?.get(storageKey) ?? null;
  if (persisted === "true" || persisted === "false") setOpen(persisted === "true");
  shortcutTarget.value = document;
  registerShortcut();
  watchMobileQuery(document.defaultView);
});

onScopeDispose(() => {
  releaseShortcut?.();
  releaseMedia?.();
  releaseShortcut = null;
  releaseMedia = null;
});

sidebarContext.provide({
  collapsible: collapsibleState,
  isMobile,
  open: isOpen,
  openMobile: isOpenMobile,
  setOpen,
  setOpenMobile,
  side: sideState,
  sidebarId,
  state,
  toggle,
  variant: variantState,
});

type SidebarProviderSetupExpose = Omit<
  SidebarProviderExpose,
  keyof SidebarSlotState | "sidebarId"
> & {
  readonly collapsible: ComputedRef<SidebarCollapsible>;
  readonly isMobile: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly openMobile: ComputedRef<boolean>;
  readonly side: ComputedRef<SidebarSide>;
  readonly sidebarId: ComputedRef<string>;
  readonly state: ComputedRef<SidebarState>;
  readonly variant: ComputedRef<SidebarVariant>;
};

const exposed = {
  collapsible: collapsibleState,
  isMobile,
  open: isOpen,
  openMobile: isOpenMobile,
  setOpen,
  setOpenMobile,
  side: sideState,
  sidebarId,
  state,
  toggle,
  variant: variantState,
} satisfies SidebarProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="sidebar-provider"
    part="provider"
    :style
    :data-state="state"
    :data-collapsible="collapsible"
    :data-side="side"
    :data-mobile="isMobile ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
