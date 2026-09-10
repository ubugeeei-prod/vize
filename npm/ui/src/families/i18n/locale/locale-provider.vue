<script setup lang="ts">
import { reactive, watchEffect } from "vue";

import {
  localeContext,
  resolveCalendar,
  resolveDirection,
  resolveLocale,
  resolveNumberingSystem,
  resolveTimeZone,
  resolveTimeZoneDisambiguation,
  type DirectionPreference,
  type LocaleTimeZoneDisambiguation,
  type TextDirection,
} from "./locale-runtime.ts";

const {
  locale = "en-US",
  direction = "ltr",
  numberingSystem,
  calendar,
  timeZone,
  timeZoneDisambiguation = "compatible",
} = defineProps<{
  /**
   * BCP 47 locale for the subtree.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Writing direction. `auto` resolves from the locale when possible.
   *
   * @default "ltr"
   */
  readonly direction?: DirectionPreference;

  /**
   * Intl numbering system for formatter composables in the subtree.
   *
   * @default undefined
   */
  readonly numberingSystem?: string;

  /**
   * Intl calendar system for date-time formatter composables in the subtree.
   *
   * @default undefined
   */
  readonly calendar?: string;

  /**
   * IANA time-zone identifier for date-time and Temporal adapters.
   *
   * @default "UTC"
   */
  readonly timeZone?: string;

  /**
   * Temporal-compatible policy for DST gaps and overlaps.
   *
   * @default "compatible"
   */
  readonly timeZoneDisambiguation?: LocaleTimeZoneDisambiguation;
}>();

defineSlots<{
  /** Localized subtree. Receives the resolved locale, direction, and formatter defaults. */
  default(props: {
    readonly locale: string;
    readonly direction: TextDirection;
    readonly numberingSystem: string | undefined;
    readonly calendar: string | undefined;
    readonly timeZone: string;
    readonly timeZoneDisambiguation: LocaleTimeZoneDisambiguation;
  }): unknown;
}>();

const value = reactive({
  locale: "en-US",
  direction: "ltr" as TextDirection,
  numberingSystem: undefined as string | undefined,
  calendar: undefined as string | undefined,
  timeZone: "UTC",
  timeZoneDisambiguation: "compatible" as LocaleTimeZoneDisambiguation,
});

watchEffect(() => {
  value.locale = resolveLocale(locale);
  value.direction = resolveDirection(direction, value.locale);
  value.numberingSystem = resolveNumberingSystem(numberingSystem, value.locale);
  value.calendar = resolveCalendar(calendar, value.locale);
  value.timeZone = resolveTimeZone(timeZone);
  value.timeZoneDisambiguation = resolveTimeZoneDisambiguation(timeZoneDisambiguation);
});

localeContext.provide(value);
</script>

<template>
  <div
    data-vize-ui="locale"
    :data-vize-ui-numbering-system="value.numberingSystem"
    :data-vize-ui-calendar="value.calendar"
    :data-vize-ui-time-zone="value.timeZone"
    :data-vize-ui-time-zone-disambiguation="value.timeZoneDisambiguation"
    :lang="value.locale"
    :dir="value.direction"
  >
    <slot
      :locale="value.locale"
      :direction="value.direction"
      :numbering-system="value.numberingSystem"
      :calendar="value.calendar"
      :time-zone="value.timeZone"
      :time-zone-disambiguation="value.timeZoneDisambiguation"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
