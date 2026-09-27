<template>
  <div>{{ hoistedSetter }}</div>
</template>
<script lang="ts">
// oxlint-disable-next-line no-constant-condition -- Execute the nested var-hoisting control.
if (true) {
  // oxlint-disable-next-line no-shadow-restricted-names -- Intentional module var shadows builtin undefined.
  var undefined = (_value: string) => {}
}
export default {
  computed: {
    hoistedSetter: { get: () => 's', set: undefined },
  },
}
</script>
