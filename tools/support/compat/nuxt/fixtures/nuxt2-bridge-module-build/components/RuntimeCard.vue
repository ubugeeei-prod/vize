<template>
  <article class="runtime-card" data-test="runtime-card">
    <strong>{{ title }}</strong>
    <img :src="icon" alt="Fixture asset" data-test="asset" />
    <slot />
  </article>
</template>

<script>
import icon from "~/assets/runtime.svg";
export default {
  props: { title: { type: String, required: true } },
  data: () => ({ icon }),
};
</script>

<style scoped lang="scss">
@import "../assets/palette";
.runtime-card {
  color: $runtime-color;
  strong {
    font-weight: 700;
  }
}
</style>
