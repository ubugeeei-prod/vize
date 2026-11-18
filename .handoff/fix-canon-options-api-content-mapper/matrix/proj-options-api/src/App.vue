<script lang="ts">
import { defineComponent } from "vue";
import Counter from "./Counter.vue";
import { greetingMixin } from "./greetingMixin";
import { baseComponent } from "./baseComponent";

// Plain object-literal default export (canon wraps it in defineComponent),
// exercising `components` registration plus `mixins` and `extends`.
export default defineComponent({
  name: "App",
  components: { Counter },
  mixins: [greetingMixin],
  extends: baseComponent,
  data() {
    return {
      title: "Options API parity",
    };
  },
  computed: {
    message(): string {
      // `greet` from the mixin and `describe` from the extended base are both
      // resolvable on `this`.
      return `${this.greet(this.title)} (${this.describe()})`;
    },
  },
});
</script>

<template>
  <main>
    <h1>{{ message }}</h1>
    <Counter label="Clicks" :step="2" />
  </main>
</template>
