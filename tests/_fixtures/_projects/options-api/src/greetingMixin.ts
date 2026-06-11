import { defineComponent } from "vue";

// A mixin that contributes a method, consumed via the `mixins` option.
export const greetingMixin = defineComponent({
  methods: {
    greet(name: string): string {
      return `Hello, ${name}!`;
    },
  },
});
