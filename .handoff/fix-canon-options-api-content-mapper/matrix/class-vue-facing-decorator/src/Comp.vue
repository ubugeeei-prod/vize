<script lang="ts">
import { Component, Prop, Vue, Emit, Watch } from 'vue-facing-decorator'
@Component
export default class Comp extends Vue {
  @Prop({ required: true }) readonly msg!: string
  @Prop({ default: 0 }) readonly size!: number
  count = 0
  get double(): number { return this.count * 2 }
  inc(by: number) { this.count += by }
  @Emit('pick') pick(n: number) { return n }
  @Watch('count') onCount(n: number) { return n }
}
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ size.toFixed() }} {{ count.toFixed() }} {{ double.toFixed() }}
    <button @click="inc(1)">+</button>
    <button @click="pick(2)">pick</button>
  </div>
</template>
