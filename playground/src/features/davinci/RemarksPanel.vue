<script setup lang="ts">
import { computed } from "vue";
import { summarizeRemarks, type SpolveroRemark } from "./remarks";

const props = defineProps<{
  remarks: SpolveroRemark[];
}>();

const emit = defineEmits<{
  /** Show a remark's authored site in the source. */
  locate: [SpolveroRemark];
}>();

const summary = computed(() => summarizeRemarks(props.remarks));
</script>

<template>
  <div class="davinci-remarks">
    <div v-if="remarks.length === 0" class="davinci-remarks-empty">
      <p class="davinci-remarks-title">No optimization remarks in this build</p>
      <p class="davinci-hint">
        Remarks explain why a pass did or did not transform something, with the source site it
        looked at. The pass manager already has the channel; this compiler build does not emit
        remarks into the feed yet, so nothing is shown rather than anything guessed.
      </p>
    </div>
    <template v-else>
      <p class="davinci-remarks-title">
        {{ summary.applied }} applied, {{ summary.missed }} missed
      </p>
      <ul class="davinci-remark-list">
        <li
          v-for="(remark, index) in remarks"
          :key="index"
          :class="['davinci-remark', remark.applied ? 'applied' : 'missed']"
        >
          <span class="davinci-remark-outcome">{{ remark.applied ? "applied" : "missed" }}</span>
          <span class="davinci-remark-pass">{{ remark.pass }}</span>
          <span class="davinci-remark-message">{{ remark.message }}</span>
          <button
            v-if="remark.span"
            type="button"
            class="davinci-ghost"
            @click="emit('locate', remark)"
          >
            Show source
          </button>
        </li>
      </ul>
    </template>
  </div>
</template>
