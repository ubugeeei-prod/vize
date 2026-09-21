<script setup lang="ts">
import { computed } from "vue";
import type { TimelineStep } from "./ladder";
import { describeWalk, type TimelineWalk } from "./fusion";
import { formatNanos } from "./format";
import StepPill from "./StepPill.vue";

const props = defineProps<{
  steps: TimelineStep[];
  /** The S2 transform plan's walks, from its plan page. */
  walks: TimelineWalk[];
  /** Key of the page on screen, to mark its step. */
  current: string | null;
}>();

const emit = defineEmits<{
  select: [TimelineStep];
}>();

/** A lone step, or the passes of one walk (they are contiguous in run order). */
type Entry =
  | { key: string; step: TimelineStep; walk?: undefined }
  | { key: string; walk: TimelineWalk; steps: TimelineStep[] };

const entries = computed<Entry[]>(() => {
  const out: Entry[] = [];
  for (const step of props.steps) {
    const walk = step.walk === null ? undefined : props.walks.find((w) => w.index === step.walk);
    const last = out[out.length - 1];
    if (!walk) out.push({ key: step.key, step });
    else if (last && "steps" in last && last.walk === walk) last.steps.push(step);
    else out.push({ key: `walk-${walk.index}`, walk, steps: [step] });
  }
  return out;
});

const passCount = computed(() => props.steps.filter((step) => !step.producer).length);
const changedPasses = computed(
  () => props.steps.filter((step) => !step.producer && step.changed).length,
);
const measured = computed(() => props.steps.filter((step) => step.nanos !== null));
const totalNanos = computed(() => measured.value.reduce((sum, step) => sum + (step.nanos ?? 0), 0));

function share(step: TimelineStep): string {
  const total = totalNanos.value;
  return total > 0 ? `${((step.nanos ?? 0) / total) * 100}%` : "0%";
}

function describe(step: TimelineStep): string {
  const rung = step.rung.toUpperCase();
  const time = step.nanos === null ? "" : `, ${formatNanos(step.nanos)}`;
  if (step.producer) return `${rung} ${step.pass}: produces a new artifact${time}`;
  return step.changed
    ? `${step.pass} changed the ${rung} folio${time}`
    : `${step.pass} left the ${rung} folio unchanged (its product is facts)${time}`;
}

function walkTitle(walk: TimelineWalk): string {
  const time = walk.nanos === null ? "" : `, ${formatNanos(walk.nanos)}`;
  return `Walk ${walk.index + 1} over the S2 tree (${describeWalk(walk)}): ${walk.passes.join(", ")}${time}`;
}
</script>

<template>
  <div class="davinci-timeline">
    <div class="davinci-timeline-row">
      <div
        v-if="totalNanos > 0"
        class="davinci-time-strip"
        role="img"
        :aria-label="`Measured compiler work: ${formatNanos(totalNanos)} in total`"
      >
        <span
          v-for="step in measured"
          :key="step.key"
          :class="['davinci-time-segment', `rung-${step.rung}`, { current: current === step.key }]"
          :style="{ width: share(step) }"
          :title="describe(step)"
        ></span>
      </div>
      <p class="davinci-timeline-summary">
        {{ changedPasses }} of {{ passCount }} {{ passCount === 1 ? "pass" : "passes" }} changed the
        folio<template v-if="walks.length > 0">
          in {{ walks.length }} {{ walks.length === 1 ? "walk" : "walks" }}</template
        ><template v-if="totalNanos > 0">, {{ formatNanos(totalNanos) }} total</template>
      </p>
    </div>
    <ol class="davinci-steps" aria-label="Pipeline steps in run order">
      <template v-for="entry in entries" :key="entry.key">
        <li v-if="entry.walk" :class="['davinci-walk', { fusable: entry.walk.fusable }]">
          <ol class="davinci-walk-steps" :aria-label="walkTitle(entry.walk)">
            <li v-for="step in entry.steps" :key="step.key">
              <StepPill
                :step
                :current="current === step.key"
                :title="describe(step)"
                @select="emit('select', step)"
              />
            </li>
          </ol>
          <span class="davinci-walk-label" :title="walkTitle(entry.walk)" aria-hidden="true">
            <span>walk {{ entry.walk.index + 1 }}</span>
            <span>{{ describeWalk(entry.walk) }}</span>
            <span v-if="entry.walk.nanos !== null" class="davinci-walk-time">{{
              formatNanos(entry.walk.nanos)
            }}</span>
          </span>
        </li>
        <li v-else>
          <StepPill
            :step="entry.step"
            :current="current === entry.step.key"
            :title="describe(entry.step)"
            @select="emit('select', entry.step)"
          />
        </li>
      </template>
    </ol>
  </div>
</template>
