<script setup lang="ts">
import { computed } from "vue";
import { formatArg, summarizeRemarks, type SpolveroRemark } from "./remarks";

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
      <p class="davinci-remarks-title">No optimization remarks for this source</p>
      <p class="davinci-hint">
        Remarks explain why a pass did or did not transform something, with the source site it
        looked at. The passes that ran here had nothing to explain.
      </p>
    </div>
    <template v-else>
      <p class="davinci-remarks-title">
        {{ summary.applied }} applied, {{ summary.missed }} missed<template
          v-if="summary.analysis > 0"
          >, {{ summary.analysis }} analysis</template
        >
      </p>
      <ul class="davinci-remark-list">
        <li
          v-for="remark in remarks"
          :key="`${remark.stage}:${remark.pass}:${remark.kind}:${remark.name}:${remark.span.start}:${remark.span.end}`"
          :class="['davinci-remark', remark.kind]"
        >
          <span class="davinci-remark-outcome">{{ remark.kind }}</span>
          <span class="davinci-remark-pass">{{ remark.pass }}</span>
          <span class="davinci-remark-message"
            ><strong>{{ remark.name }}</strong>
            <span v-for="arg in remark.args" :key="arg.key" class="davinci-remark-arg">{{
              formatArg(arg)
            }}</span></span
          >
          <button type="button" class="davinci-ghost" @click="() => emit('locate', remark)">
            Show source
          </button>
        </li>
      </ul>
    </template>
  </div>
</template>
