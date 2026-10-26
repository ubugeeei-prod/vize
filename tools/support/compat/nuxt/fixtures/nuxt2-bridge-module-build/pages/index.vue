<template>
  <main>
    <h1>Nuxt 2 with Vize</h1>
    <RuntimeCard :title="label">
      <button type="button" data-test="counter" @click="clicks++">Clicks: {{ clicks }}</button>
      <p data-test="hmr">HMR original</p>
    </RuntimeCard>
    <nuxt-link to="/about">About</nuxt-link>
  </main>
</template>

<script>
import RuntimeCard from "~/components/RuntimeCard.vue";
import { runtimeLabel } from "@/lib/runtime-label";
export default {
  components: { RuntimeCard },
  data: () => ({ label: runtimeLabel, clicks: 0 }),
};
</script>
