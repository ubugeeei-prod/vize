<script lang="ts">
import type {} from "./shims";
import { defineComponent } from "vue";
import { NButton, NCard, NSelect, NTag } from "naive-ui";
import type { SelectOption } from "naive-ui";

interface LibraryOption extends SelectOption {
  label: string;
  value: string;
}

interface ReviewTask {
  id: string;
  title: string;
  owner: string;
  done: boolean;
}

const libraryOptions: LibraryOption[] = [
  { label: "Naive UI", value: "naive-ui" },
  { label: "PrimeVue", value: "primevue" },
  { label: "Element Plus", value: "element-plus" },
];

const reviewTasks: ReviewTask[] = [
  { id: "select", title: "Check v-model:value", owner: "typechecker", done: true },
  { id: "button", title: "Check button listener", owner: "linter", done: false },
  { id: "tag", title: "Check tag props", owner: "compiler", done: false },
];

export default defineComponent({
  name: "NaiveUiOptionsApi",
  components: {
    NButton,
    NCard,
    NSelect,
    NTag,
  },
  data() {
    return {
      selectedLibrary: "naive-ui",
      libraryOptions,
      reviewTasks,
      lastReviewed: "",
    };
  },
  computed: {
    selectedLabel(): string {
      const option = this.libraryOptions.find((item) => item.value === this.selectedLibrary);
      return option?.label ?? "Unknown library";
    },
    visibleTasks(): ReviewTask[] {
      return this.reviewTasks.filter((task) => task.owner !== "compiler" || !task.done);
    },
    completedCount(): number {
      return this.reviewTasks.filter((task) => task.done).length;
    },
    statusTagType(): "success" | "warning" {
      return this.completedCount === this.reviewTasks.length ? "success" : "warning";
    },
    statusMessage(): string {
      return `${this.completedCount}/${this.reviewTasks.length} reviewed for ${this.selectedLabel}`;
    },
  },
  methods: {
    markReviewed(value: string): void {
      this.lastReviewed = value;
    },
    toggleTask(taskId: string): void {
      const task = this.reviewTasks.find((item) => item.id === taskId);
      if (task) {
        task.done = !task.done;
      }
    },
  },
});
</script>

<template>
  <NCard title="Naive UI Options API" size="small">
    <NSelect v-model:value="selectedLibrary" :options="libraryOptions" filterable />
    <NButton type="primary" @click="markReviewed(selectedLibrary)">
      Review {{ selectedLabel }}
    </NButton>
    <NTag :type="statusTagType">{{ statusMessage }}</NTag>
    <ul>
      <li v-for="task in visibleTasks" :key="task.id">
        <span>{{ task.title }} by {{ task.owner }}</span>
        <NButton quaternary size="small" @click="toggleTask(task.id)">
          {{ task.done ? "Reopen" : "Done" }}
        </NButton>
      </li>
    </ul>
    <p v-if="lastReviewed">Last reviewed: {{ lastReviewed }}</p>
  </NCard>
</template>

<style scoped>
ul {
  display: grid;
  gap: 6px;
  padding-left: 18px;
}

li {
  align-items: center;
  display: flex;
  gap: 8px;
}
</style>
