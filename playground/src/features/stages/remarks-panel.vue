<script setup lang="ts">
import { computed } from "vue";
import { formatArg, summarizeRemarks, type StageRemark } from "./remarks";

const { remarks } = defineProps<{
  remarks: StageRemark[];
}>();

const emit = defineEmits<{
  /** Show a remark's authored site in the source. */
  locate: [StageRemark];
}>();

const summary = computed(() => summarizeRemarks(remarks));
</script>

<template>
  <div class="level-remarks">
    <div v-if="remarks.length === 0" class="level-remarks-empty">
      <p class="level-remarks-title">No optimization remarks for this source</p>
      <p class="level-hint">
        Remarks explain why a pass did or did not transform something, with the source site it
        looked at. The passes that ran here had nothing to explain.
      </p>
    </div>
    <template v-else>
      <p class="level-remarks-title">
        {{ summary.applied }} applied, {{ summary.missed }} missed<template
          v-if="summary.analysis > 0"
          >, {{ summary.analysis }} analysis</template
        >
      </p>
      <ul class="level-remark-list">
        <li
          v-for="remark in remarks"
          :key="`${remark.stage}:${remark.pass}:${remark.kind}:${remark.name}:${remark.span.start}:${remark.span.end}`"
          :class="['level-remark', remark.kind]"
        >
          <span class="level-remark-outcome">{{ remark.kind }}</span>
          <span class="level-remark-pass">{{ remark.pass }}</span>
          <span class="level-remark-message"
            ><strong>{{ remark.name }}</strong>
            <span v-for="arg in remark.args" :key="arg.key" class="level-remark-arg">{{
              formatArg(arg)
            }}</span></span
          >
          <button type="button" class="level-ghost" @click="() => emit('locate', remark)">
            Show source
          </button>
        </li>
      </ul>
    </template>
  </div>
</template>
