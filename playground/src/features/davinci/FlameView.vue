<script setup lang="ts">
import "./FlameView.css";
import { computed, ref, shallowRef } from "vue";
import { LADDER_STEP_KEY, type ProfileExport } from "../../wasm/types/profile";
import { flameGraph, flameKeys, flameTrend, readProfileExport, type FlameFrame } from "./flame";
import { formatNanos } from "./format";

const props = defineProps<{
  /** This run's export (the ladder's step timings). */
  profile: ProfileExport | null;
  /** A pinned earlier run to compare against. */
  baseline: ProfileExport | null;
}>();

const emit = defineEmits<{
  /** A frame's attribution path, outermost first. */
  select: [string[]];
  pin: [];
  unpin: [];
}>();

const LEVEL_NAMES = ["Stage", "Pass", "Block"];

/** A `--profile-json` file opened in place of this run, and its span key. */
const opened = shallowRef<{ name: string; profile: ProfileExport } | null>(null);
const openError = ref<string | null>(null);
const key = ref(LADDER_STEP_KEY);

const shown = computed(() => opened.value?.profile ?? props.profile);
const compared = computed(() => (opened.value ? null : props.baseline));
const keys = computed(() => (opened.value ? flameKeys(opened.value.profile) : []));
const flame = computed(() =>
  shown.value
    ? flameGraph(shown.value, opened.value ? key.value : LADDER_STEP_KEY, compared.value)
    : null,
);
const rows = computed(() => {
  const frames = flame.value?.frames ?? [];
  return LEVEL_NAMES.map((name, depth) => ({
    name,
    frames: frames.filter((frame) => frame.depth === depth),
  })).filter((row) => row.frames.length > 0);
});

async function openExport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const read = readProfileExport(await file.text());
  input.value = "";
  if (!read.ok) {
    openError.value = `${file.name}: ${read.error}`;
    return;
  }
  openError.value = null;
  opened.value = { name: file.name, profile: read.profile };
  key.value = flameKeys(read.profile)[0] ?? LADDER_STEP_KEY;
}

function closeExport() {
  opened.value = null;
  openError.value = null;
}

/** A frame of this run opens the step it names; an opened file has no pages. */
function pick(frame: FlameFrame) {
  if (!opened.value) emit("select", frame.path);
}

function percent(nanos: number): string {
  const total = flame.value?.total ?? 0;
  return total > 0 ? `${(nanos / total) * 100}%` : "0%";
}

function describe(frame: FlameFrame): string {
  const calls = frame.count === 1 ? "1 call" : `${frame.count} calls`;
  const share = Math.round((frame.nanos / (flame.value?.total || 1)) * 100);
  const head = `${frame.path.join(" › ")}: ${formatNanos(frame.nanos)}, ${share}% of the run, ${calls}`;
  if (!compared.value) return head;
  return frame.baseline === null
    ? `${head}; did not run in the baseline`
    : `${head}; baseline ${formatNanos(frame.baseline)}`;
}
</script>

<template>
  <div class="davinci-flame">
    <header class="davinci-flame-bar">
      <p v-if="opened" class="davinci-hint">
        Showing <code>{{ opened.name }}</code> (tool {{ opened.profile.tool }}, command
        {{ opened.profile.command }})
      </p>
      <p v-else class="davinci-hint">
        Measured compiler work by stage, pass and block. Width is time; siblings sort by name.
      </p>
      <template v-if="opened">
        <label class="davinci-hint" for="davinci-flame-key">Span key</label>
        <select id="davinci-flame-key" v-model="key" class="davinci-select">
          <option v-for="name in keys" :key="name" :value="name">{{ name }}</option>
        </select>
        <button type="button" class="davinci-ghost" @click="closeExport">Back to this run</button>
      </template>
      <template v-else>
        <button type="button" class="davinci-ghost" :disabled="!profile" @click="emit('pin')">
          {{ baseline ? "Pin this run instead" : "Pin as baseline" }}
        </button>
        <button v-if="baseline" type="button" class="davinci-ghost" @click="emit('unpin')">
          Clear baseline
        </button>
      </template>
      <input
        id="davinci-flame-file"
        class="davinci-flame-file"
        type="file"
        accept=".json,application/json"
        @change="openExport"
      />
      <label class="davinci-ghost" for="davinci-flame-file">Open a --profile-json file</label>
    </header>
    <p v-if="openError" class="davinci-message error" role="alert">{{ openError }}</p>
    <p v-if="!flame || flame.total === 0" class="davinci-message">
      This run has no step timings to draw.
    </p>
    <div v-else class="davinci-flame-graph">
      <div v-for="row in rows" :key="row.name" class="davinci-flame-row">
        <span class="davinci-flame-level">{{ row.name }}</span>
        <div class="davinci-flame-lane">
          <button
            v-for="frame in row.frames"
            :key="frame.path.join(' › ')"
            type="button"
            :class="[
              'davinci-flame-frame',
              `rung-${frame.path[0]}`,
              compared ? `trend-${flameTrend(frame)}` : null,
            ]"
            :style="{ left: percent(frame.start), width: percent(frame.nanos) }"
            :title="describe(frame)"
            :aria-label="describe(frame)"
            @click="pick(frame)"
          >
            <span class="davinci-flame-name">{{ frame.path[frame.depth] }}</span>
            <span class="davinci-flame-time">{{ formatNanos(frame.nanos) }}</span>
          </button>
        </div>
      </div>
    </div>
    <p v-if="compared && flame && flame.total > 0" class="davinci-flame-legend">
      Against the pinned run: <span class="trend-slower">slower</span>
      <span class="trend-faster">faster</span> <span class="trend-same">within 10%</span>
      <span class="trend-new">new</span>
    </p>
  </div>
</template>
