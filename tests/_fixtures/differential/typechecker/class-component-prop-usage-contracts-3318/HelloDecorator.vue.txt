<script lang="ts">
import { Vue, Prop } from "vue-property-decorator";

// vue-class-component / vue-property-decorator class component exercising
// @Prop, a data field, a getter (computed) and a method.
export default class HelloDecorator extends Vue {
  @Prop({ type: String, required: true }) readonly name!: string;

  // data field
  count = 0;

  // getter -> computed
  get greeting(): string {
    return `Hello, ${this.name}! (${this.count})`;
  }

  // method
  increment(): void {
    this.count += 1;
  }
}
</script>

<template>
  <button type="button" @click="increment">{{ greeting }}</button>
</template>
