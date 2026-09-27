<template>
  <div>{{ shadowedSetter }}</div>
</template>
<script lang="ts">
// oxlint-disable-next-line no-shadow-restricted-names -- Intentional callable module binding shadows builtin undefined.
const undefined = (_value: string) => {}
export default {
  computed: {
    shadowedSetter: { get: () => 's', set: undefined },
  },
}
</script>
