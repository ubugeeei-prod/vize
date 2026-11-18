<script lang="ts">
import { defineComponent } from 'vue'
import Field from './Field.vue'
export default defineComponent({
  components: { Field },
  data() { return { text: 'a', num: 1, total: 0 } },
  methods: { onItem(id: number) { return id }, onBadItem(id: string) { return id } },
})
</script>

<template>
  <Field v-model="text" :max-length="5" v-model:count="total" @item-click="onItem" />
  <Field v-model="num" />
  <Field v-model="text" :max-length="'5'" />
  <Field v-model="text" @item-click="onBadItem" />
  <Field v-model="text" v-bind="{ maxLength: 'x' }" />
  <component :is="Field" v-model="text" :max-length="'x'" />
</template>
