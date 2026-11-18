<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { MyChild: Child, 'kebab-child': Child, lowerchild: Child },
})
</script>

<template>
  <MyChild :n="1" />
  <my-child :n="'a'" />
  <MyChild :n="'b'" />
  <kebab-child :n="'c'" />
  <KebabChild :n="'d'" />
  <lowerchild :n="'e'" />
  <Lowerchild :n="'f'" />
</template>
