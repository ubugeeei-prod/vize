<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'
import DefaultsChild from './DefaultsChild.vue'
import RuntimeChild from './RuntimeChild.vue'
import ArrayChild from './ArrayChild.vue'
import OptionsChild from './OptionsChild.vue'
import InterfaceChild from './InterfaceChild.vue'

const t = ref('a')
const n = ref(1)
const nm = ref('')
function onChange(value: string) {
  console.log(value)
}
</script>

<template>
  <Child :title="t" @change="onChange" v-model:count="n"><template #item="{ row }">{{ row }}</template></Child>
  <Child title="x" />
  <DefaultsChild title="d" @change="onChange" />
  <RuntimeChild title="r" @change="onChange" />
  <ArrayChild title="a" @change="onChange" />
  <OptionsChild title="o" @change="onChange"><template #item="{ row }">{{ row }}</template></OptionsChild>
  <InterfaceChild title="i" :item-count="n" active v-model:name="nm" @change="onChange"><template v-slot:item="{ row }">{{ row }}</template></InterfaceChild>
</template>
