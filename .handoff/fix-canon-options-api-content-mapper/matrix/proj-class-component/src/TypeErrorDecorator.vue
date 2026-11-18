<script lang="ts">
import { Vue } from "vue-property-decorator";

// Class component with an INTENTIONAL type error: `Number.prototype.toFixed`
// takes an optional digit count (number), not a boolean.
export default class TypeErrorDecorator extends Vue {
  count = 0;

  get formatted(): string {
    return this.count.toFixed(true);
  }
}
</script>

<template>
  <span>{{ formatted }}</span>
</template>
