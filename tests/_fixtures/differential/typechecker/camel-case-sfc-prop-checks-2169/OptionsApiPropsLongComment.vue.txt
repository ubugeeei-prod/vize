<script lang="ts">
import { defineComponent, PropType } from 'vue'

interface Item { id: number }

// long enough comment line one xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line two xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line three xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line four xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line five xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

export default defineComponent({
  props: {
    items: {
      type: Array as PropType<Item[]>,
      required: false,
    },
    size: {
      type: Number,
      default: 0,
    },
  },
})
</script>
