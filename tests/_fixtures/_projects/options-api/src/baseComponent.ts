import { defineComponent } from "vue";

// A base component extended via the `extends` option.
export const baseComponent = defineComponent({
  data() {
    return {
      base: "base",
    };
  },
  methods: {
    describe(): string {
      return `base=${this.base}`;
    },
  },
});
