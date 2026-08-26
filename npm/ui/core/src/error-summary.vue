<script setup lang="ts">
import { onMounted, toRef, useTemplateRef } from "vue";

import { useDeterministicId } from "./deterministic-id.ts";
import { useErrorSummary } from "./error-summary-runtime.ts";
import type { ErrorSummaryField } from "./error-summary-types.ts";

const {
  fields = [],
  heading = "",
  autoFocus = true,
} = defineProps<{
  /**
   * Invalid fields in document order. Field ids must be unique.
   *
   * @default []
   */
  readonly fields?: readonly ErrorSummaryField[];

  /**
   * Text for the heading that labels the summary region.
   *
   * @default ""
   */
  readonly heading?: string;

  /**
   * Move focus into the summary when invalid fields appear.
   *
   * @default true
   */
  readonly autoFocus?: boolean;
}>();

const emit = defineEmits<{
  /** Fires after a summary link moves focus to the named invalid control. */
  fieldFocus: [field: ErrorSummaryField];

  /** Fires after a repair settles focus, with the restored element or null. */
  restore: [target: HTMLElement | null];
}>();

defineSlots<{
  /** Replaces the heading prop text inside the labelling element. */
  heading?(props: { readonly fields: readonly ErrorSummaryField[] }): unknown;

  /** Replaces one link's content; the default is label-prefixed message text. */
  field?(props: { readonly field: ErrorSummaryField }): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const headingId = useDeterministicId({ hint: "error-summary-heading" });
const summary = useErrorSummary({
  fields: toRef(() => fields),
  element,
  autoFocus: toRef(() => autoFocus),
  onRestore: (target) => emit("restore", target),
});

onMounted(() => {
  if (autoFocus && summary.hasErrors.value) summary.focusSummary();
});

function linkText(field: ErrorSummaryField): string {
  return field.label === undefined ? field.message : `${field.label}: ${field.message}`;
}

function onFieldLink(field: ErrorSummaryField, event: MouseEvent): void {
  event.preventDefault();
  const control = summary.focusField(field.id);
  if (control !== null) emit("fieldFocus", field);
}

defineExpose({
  element,
  hasErrors: summary.hasErrors,
  focusSummary: summary.focusSummary,
  focusField: summary.focusField,
  restoreFocus: summary.restoreFocus,
});
</script>

<template>
  <div data-vize-ui="error-summary-host">
    <div
      v-if="summary.hasErrors.value"
      ref="element"
      data-vize-ui="error-summary"
      role="group"
      tabindex="-1"
      :aria-labelledby="headingId"
    >
      <div :id="headingId" data-vize-ui="error-summary-heading">
        <slot name="heading" :fields="summary.fields.value">{{ heading }}</slot>
      </div>
      <ul data-vize-ui="error-summary-list">
        <li v-for="field in summary.fields.value" :key="field.id" data-vize-ui="error-summary-item">
          <a
            data-vize-ui="error-summary-link"
            :href="'#' + field.id"
            @click="(event) => onFieldLink(field, event)"
          >
            <slot name="field" :field="field">{{ linkText(field) }}</slot>
          </a>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
