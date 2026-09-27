<template>
  <div>{{ builtinUndefined }} {{ wrappedArrow }} {{ wrappedNull }} {{ wrappedString }} {{ wrappedUndefined }}</div>
</template>
<script lang="ts">
// This function's local name does not shadow the descriptor's module scope.
function unrelated() {
  // oxlint-disable-next-line no-shadow-restricted-names -- Intentional function-local binding does not shadow module undefined.
  const undefined = (_value: string) => {}
  return undefined
}
export default {
  computed: {
    builtinUndefined: { get: () => 'b', set: undefined },
    wrappedArrow: { get: () => 'a', set: (((_value: string) => {}) as (_value: string) => void) },
    wrappedNull: { get: () => 'n', set: ((null as unknown as (_value: string) => void)) },
    wrappedString: { get: () => 's', set: (('absent' as unknown as (_value: string) => void)) },
    wrappedUndefined: { get: () => 'u', set: ((undefined as unknown as (_value: string) => void)) },
  },
}
</script>
