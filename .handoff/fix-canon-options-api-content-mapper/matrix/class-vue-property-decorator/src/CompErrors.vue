<script lang="ts">
import { Options, Vue } from 'vue-class-component'
import { Prop } from 'vue-property-decorator'
@Options({})
export default class CompErrors extends Vue {
  @Prop({ required: true }) readonly msg!: string
  count = 0
  inc(by: number) { this.count += by }
}
</script>

<template>
  <div>
    {{ msg.toFixed() }} {{ count.toUpperCase() }} {{ nope }}
    <button @click="inc('x')">+</button>
  </div>
</template>
