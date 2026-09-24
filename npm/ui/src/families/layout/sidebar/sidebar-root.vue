<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import {
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
} from "../../overlays/dialog/dialog.ts";
import { sidebarContext } from "./sidebar-context.ts";
import type { SidebarRootExpose, SidebarSlotState } from "./sidebar-types.ts";

const {
  ariaLabel = "Sidebar",
  to = "body",
  portalDisabled = false,
} = defineProps<{
  /**
   * Accessible name of the complementary landmark and the mobile sheet.
   *
   * @default "Sidebar"
   */
  readonly ariaLabel?: string;

  /**
   * CSS selector or element the mobile sheet is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render the mobile sheet in place instead of teleporting it.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;
}>();

defineSlots<{
  /** Sidebar contents. Receives the current Sidebar state. */
  default(props: SidebarSlotState): unknown;
}>();

const context = sidebarContext.use();
const element = useTemplateRef<HTMLElement>("element");
const offcanvasHidden = computed(
  () => context.collapsible.value === "offcanvas" && !context.open.value,
);
const slotState = computed<SidebarSlotState>(() => ({
  collapsible: context.collapsible.value,
  isMobile: context.isMobile.value,
  open: context.open.value,
  openMobile: context.openMobile.value,
  side: context.side.value,
  state: context.state.value,
  variant: context.variant.value,
}));

function onMobileOpenChange(value: boolean): void {
  context.setOpenMobile(value);
}

type SidebarRootSetupExpose = Omit<SidebarRootExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SidebarRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <DialogRoot
    v-if="context.isMobile.value"
    :open="context.openMobile.value"
    @update:open="onMobileOpenChange"
  >
    <DialogPortal :to :disabled="portalDisabled">
      <DialogOverlay />
      <DialogContent :aria-label :aria-labelledby="null" :aria-describedby="null">
        <div
          :id="context.sidebarId.value"
          data-vize-ui="sidebar-root"
          part="root"
          data-mobile="true"
          :data-state="context.openMobile.value ? 'expanded' : 'collapsed'"
          :data-side="context.side.value"
          :data-variant="context.variant.value"
        >
          <slot v-bind="slotState" />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
  <aside
    v-else
    :id="context.sidebarId.value"
    ref="element"
    :aria-label="ariaLabel"
    :inert="offcanvasHidden ? true : undefined"
    data-vize-ui="sidebar-root"
    part="root"
    :data-state="context.state.value"
    :data-collapsible="context.state.value === 'collapsed' ? context.collapsible.value : undefined"
    :data-side="context.side.value"
    :data-variant="context.variant.value"
  >
    <slot v-bind="slotState" />
  </aside>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
