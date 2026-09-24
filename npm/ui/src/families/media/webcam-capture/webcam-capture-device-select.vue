<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type {
  WebcamCaptureDevice,
  WebcamCaptureDeviceSelectExpose,
} from "./webcam-capture-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to `messages.deviceSelect`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Option content per camera. Defaults to the device label. */
  option?(props: { readonly device: WebcamCaptureDevice; readonly selected: boolean }): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLSelectElement>("element");
const devices = computed(() => context.slotState.value.devices);
const selected = computed(() => context.slotState.value.deviceId ?? "");
const label = computed(() => ariaLabel ?? context.messages.value.deviceSelect);
const disabled = computed(() => context.slotState.value.external || devices.value.length === 0);

function onChange(event: Event): void {
  if (!(event.target instanceof HTMLSelectElement)) return;
  void context.selectDevice(event.target.value === "" ? undefined : event.target.value);
}

type SetupExpose = Omit<WebcamCaptureDeviceSelectExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <select
    ref="element"
    :value="selected"
    :disabled
    :aria-label="label"
    data-vize-ui="webcam-capture-device-select"
    part="device-select"
    :data-count="devices.length"
    @change="onChange"
  >
    <option
      v-for="device in devices"
      :key="device.deviceId"
      :value="device.deviceId"
      :selected="device.deviceId === selected"
    >
      <slot name="option" :device :selected="device.deviceId === selected">{{ device.label }}</slot>
    </option>
  </select>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
