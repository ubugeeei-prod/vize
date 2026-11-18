<script lang="ts">
import { Options, Vue } from 'vue-class-component'
import { Prop, Emit, Watch, Model, PropSync } from 'vue-property-decorator'
@Options({})
export default class Comp extends Vue {
  @Prop({ required: true }) readonly msg!: string
  @Prop({ type: Number, default: 0 }) readonly size!: number
  @Model('modelValue', { type: String }) readonly value!: string
  @PropSync('open', { type: Boolean }) syncedOpen!: boolean
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
    {{ value }} {{ syncedOpen }}
    <button @click="inc(1)">+</button>
    <button @click="pick(2)">pick</button>
    <button @click="syncedOpen = !syncedOpen">t</button>
  </div>
</template>
