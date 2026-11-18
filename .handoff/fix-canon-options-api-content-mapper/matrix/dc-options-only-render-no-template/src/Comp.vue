<script lang="ts">
import { defineComponent, h } from 'vue'
export default defineComponent({
  props: { a: { type: String, required: true } },
  render() { return h('div', this.a.toFixed()) },
})
</script>
